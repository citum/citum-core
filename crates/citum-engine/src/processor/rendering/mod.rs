/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Rendering logic for citation and bibliography output.
//!
//! This module handles template-based rendering of citations and bibliographies,
//! including handling of localization, numbering, formatting, and special modes
//! like integral (narrative) citations for numeric and label styles.

use crate::error::ProcessorError;
use crate::reference::{Bibliography, Reference};
use crate::values::{ProcHints, RenderContext, RenderOptions};
use citum_schema::citation::CitationLocator;
use citum_schema::locale::Locale;
use citum_schema::options::{Config, bibliography::BibliographyConfig};
use citum_schema::template::TemplateComponent;
use grouped::component_predicates::{resolve_localized_type_variant, resolve_type_variant};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock, RwLock};

fn embedded_render_locales() -> &'static HashMap<String, Locale> {
    static LOCALES: OnceLock<HashMap<String, Locale>> = OnceLock::new();
    LOCALES.get_or_init(|| {
        let mut locales = HashMap::new();
        for id in citum_schema::embedded::EMBEDDED_LOCALE_IDS {
            let Some(bytes) = citum_schema::embedded::get_locale_bytes(id) else {
                continue;
            };
            let Ok(yaml) = std::str::from_utf8(bytes) else {
                continue;
            };
            let Ok(locale) = Locale::from_yaml_str(yaml) else {
                continue;
            };
            locales.insert(id.to_ascii_lowercase(), locale);
        }
        locales
    })
}

/// The renderer for citation and bibliography templates.
///
/// The `Renderer` is responsible for taking compiled templates and applying them
/// to bibliographic data, handling localization, numbering, and formatting.
pub struct Renderer<'a> {
    /// The style definition containing templates and options.
    pub style: &'a citum_schema::Style,
    /// The bibliography containing the reference data.
    pub bibliography: &'a Bibliography,
    /// The locale used for terms and formatting.
    pub locale: &'a Locale,
    /// The active configuration options.
    pub config: Arc<Config>,
    /// The active bibliography-only configuration.
    pub bibliography_config: Option<Arc<BibliographyConfig>>,
    /// Pre-calculated hints for optimization.
    pub hints: &'a HashMap<String, ProcHints>,
    /// Shared state for citation numbers (used in numeric styles).
    ///
    /// `RwLock`, not `RefCell`: bibliography entries render in parallel
    /// (behind the `parallel` feature) once above `PARALLEL_MIN_ENTRIES`,
    /// and each per-entry `Renderer` borrows this same run-scoped map.
    pub citation_numbers: &'a RwLock<HashMap<String, usize>>,
    /// Optional compound set membership indexed by reference id.
    pub compound_set_by_ref: &'a HashMap<String, String>,
    /// Optional 0-based member index within each compound set.
    pub compound_member_index: &'a HashMap<String, usize>,
    /// Compound sets keyed by set id.
    pub compound_sets: &'a IndexMap<String, Vec<String>>,
    /// Whether to output semantic markup (HTML spans, Djot attributes).
    pub show_semantics: bool,
    /// Whether to attach source template indices to rendered semantic wrappers.
    pub inject_ast_indices: bool,
    /// Mapping from filtered to original template indices (for grouped citations).
    pub filtered_to_original_index: RefCell<Option<Vec<usize>>>,
    /// Document-level abbreviation map for post-render substitution.
    pub abbreviation_map: Option<&'a crate::api::AbbreviationMap>,
    /// First note number per reference id (populated by normalize_note_context).
    pub first_note_by_id: Option<&'a RwLock<HashMap<String, u32>>>,
}

/// Borrowed compound-set context for rendering.
pub struct CompoundRenderData<'a> {
    /// Optional compound set membership indexed by reference id.
    pub set_by_ref: &'a HashMap<String, String>,
    /// Optional 0-based member index within each compound set.
    pub member_index: &'a HashMap<String, usize>,
    /// Compound sets keyed by set id.
    pub sets: &'a IndexMap<String, Vec<String>>,
}

mod collapse;
mod grouped;
mod grouped_fallback;
mod helpers;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests;

pub use grouped_fallback::GroupRenderParams;
pub use grouped_fallback::TemplateRenderParams;
pub(super) use helpers::{
    find_grouping_component, has_contributor_component, leading_group_affix,
    remove_first_contributor_with_role, strip_author_component, strip_leading_group_affixes,
};

/// Internal render request used to keep template-processing call sites compact.
pub struct TemplateRenderRequest<'a> {
    /// The template to render.
    pub template: &'a [TemplateComponent],
    /// The rendering context (Citation or Bibliography).
    pub context: RenderContext,
    /// The citation mode (Integral or `NonIntegral`).
    pub mode: citum_schema::citation::CitationMode,
    /// Whether to suppress the author in output.
    pub suppress_author: bool,
    /// The raw citation locator if present (for new rendering logic).
    pub locator_raw: Option<&'a CitationLocator>,
    /// The citation number for numeric styles.
    pub citation_number: usize,
    /// The citation position (e.g., Ibid).
    pub position: Option<citum_schema::citation::Position>,
    /// Optional note-start text-case policy for note-style repeated-note output.
    pub note_start_text_case: Option<citum_schema::NoteStartTextCase>,
    /// Integral name state for name formatting.
    pub integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    /// Org abbreviation state for org-name formatting.
    pub org_abbreviation_state: Option<citum_schema::citation::IntegralNameState>,
    /// First note number for this reference (note styles, subsequent position).
    pub first_reference_note_number: Option<u32>,
}

/// Per-item state resolved for rendering one ungrouped citation item.
struct UngroupedItemRenderState<'a> {
    reference: &'a Reference,
    template: Cow<'a, [TemplateComponent]>,
    delimiter: &'a str,
}

/// Shared, citation-wide parameters threaded into each ungrouped item render.
#[derive(Clone, Copy)]
struct UngroupedItemRenderParams<'a> {
    mode: &'a citum_schema::citation::CitationMode,
    suppress_author: bool,
    position: Option<&'a citum_schema::citation::Position>,
    note_start_text_case: Option<citum_schema::NoteStartTextCase>,
}

#[derive(Clone, Default)]
struct TemplateComponentTracker {
    rendered_vars: HashSet<String>,
    substituted_bases: HashSet<String>,
}

impl TemplateComponentTracker {
    fn should_skip(&self, var_key: Option<&str>) -> bool {
        let Some(var_key) = var_key else {
            return false;
        };
        let base = key_base(var_key);
        self.rendered_vars.contains(var_key) || self.substituted_bases.contains(base.as_ref())
    }

    fn mark_rendered(&mut self, var_key: Option<String>, substituted_key: Option<&str>) {
        if let Some(var_key) = var_key {
            self.rendered_vars.insert(var_key);
        }
        if let Some(substituted_key) = substituted_key {
            self.rendered_vars.insert(substituted_key.to_string());
            self.substituted_bases
                .insert(key_base(substituted_key).into_owned());
        }
    }

    fn merge_from(&mut self, other: Self) {
        self.rendered_vars.extend(other.rendered_vars);
        self.substituted_bases.extend(other.substituted_bases);
    }
}

/// Core style resources borrowed by every [`Renderer`] instance.
///
/// Bundles the four immutable resolution inputs so that [`Renderer::new`] stays
/// within clippy's argument-count limit.
pub struct RendererResources<'a> {
    /// The style definition containing templates and options.
    pub style: &'a citum_schema::Style,
    /// The bibliography containing the reference data.
    pub bibliography: &'a Bibliography,
    /// The locale used for terms and formatting.
    pub locale: &'a Locale,
    /// The active configuration options.
    pub config: Arc<Config>,
    /// The active bibliography-only configuration.
    pub bibliography_config: Option<Arc<BibliographyConfig>>,
    /// First note number per reference id (note styles; `None` for bibliography rendering).
    pub first_note_by_id: Option<&'a RwLock<HashMap<String, u32>>>,
}

impl<'a> Renderer<'a> {
    /// Creates a new `Renderer` instance.
    pub fn new(
        resources: RendererResources<'a>,
        hints: &'a HashMap<String, ProcHints>,
        citation_numbers: &'a RwLock<HashMap<String, usize>>,
        compound: CompoundRenderData<'a>,
        show_semantics: bool,
        inject_ast_indices: bool,
        abbreviation_map: Option<&'a crate::api::AbbreviationMap>,
    ) -> Self {
        Self {
            style: resources.style,
            bibliography: resources.bibliography,
            locale: resources.locale,
            config: resources.config,
            bibliography_config: resources.bibliography_config,
            hints,
            citation_numbers,
            compound_set_by_ref: compound.set_by_ref,
            compound_member_index: compound.member_index,
            compound_sets: compound.sets,
            show_semantics,
            inject_ast_indices,
            filtered_to_original_index: RefCell::new(None),
            abbreviation_map,
            first_note_by_id: resources.first_note_by_id,
        }
    }

    /// Select the embedded rendering locale for an explicitly matched localized layout.
    fn locale_for_reference(&self, reference: &Reference, context: RenderContext) -> &Locale {
        let language = crate::values::effective_item_language(reference);
        let selected = match context {
            RenderContext::Citation => self
                .style
                .citation
                .as_ref()
                .and_then(|spec| spec.resolve_localized_template(language.as_deref())),
            RenderContext::Bibliography => self
                .style
                .bibliography
                .as_ref()
                .and_then(|spec| spec.resolve_localized_template(language.as_deref())),
        };
        let Some(locale_id) = selected.and_then(|resolved| resolved.locale) else {
            return self.locale;
        };

        let locales = embedded_render_locales();
        let key = locale_id.to_ascii_lowercase();
        locales
            .get(&key)
            .or_else(|| {
                let primary = key.split(['-', '_']).next()?;
                locales.iter().find_map(|(candidate, locale)| {
                    candidate
                        .split(['-', '_'])
                        .next()
                        .is_some_and(|candidate_primary| candidate_primary == primary)
                        .then_some(locale)
                })
            })
            .unwrap_or(self.locale)
    }

    /// Resolve multilingual contributor names using the style's config.
    fn resolve_contributor_names(
        &self,
        contributor: &citum_schema::reference::contributor::Contributor,
    ) -> Vec<crate::reference::FlatName> {
        let ml = self.config.multilingual.as_ref();
        crate::values::resolve_multilingual_name(
            contributor,
            ml.and_then(|m| m.name_mode.as_ref()),
            ml.and_then(|m| m.preferred_transliteration.as_deref()),
            ml.and_then(|m| m.preferred_script.as_ref()),
            &self.locale.locale,
        )
    }

    /// Generate an alphabetic or numeric sub-label (e.g., "a", "1") for a
    /// reference member of a compound set.
    fn citation_sub_label_for_ref(&self, ref_id: &str) -> Option<String> {
        let compound = self
            .bibliography_config
            .as_ref()
            .and_then(|b| b.compound_numeric.as_ref())?;
        let set_id = self.compound_set_by_ref.get(ref_id)?;
        let members = self.compound_sets.get(set_id)?;
        if members.len() <= 1 {
            return None;
        }
        if !compound.subentry {
            return None;
        }
        let idx = *self.compound_member_index.get(ref_id)?;
        match compound.sub_label {
            citum_schema::options::bibliography::SubLabelStyle::Alphabetic => {
                crate::values::int_to_letter((idx + 1) as u32)
            }
            citum_schema::options::bibliography::SubLabelStyle::Numeric => {
                Some((idx + 1).to_string())
            }
        }
    }

    /// Determines if the processor should render author-plus-number text for a numeric style
    /// when in "integral" (narrative) citation mode.
    ///
    /// This happens when the style is numeric and the user requests a narrative
    /// citation (e.g., "Smith [1]"), but hasn't provided an explicit narrative template.
    fn should_render_author_number_for_numeric_integral(
        &self,
        mode: &citum_schema::citation::CitationMode,
    ) -> bool {
        matches!(mode, citum_schema::citation::CitationMode::Integral)
            && self.config.processing.as_ref().is_some_and(|processing| {
                matches!(processing, citum_schema::options::Processing::Numeric)
            })
            && !self.has_explicit_integral_template()
    }

    /// Whether the style provides an explicit integral (narrative) template.
    fn has_explicit_integral_template(&self) -> bool {
        self.style.citation.as_ref().is_some_and(|c| {
            c.integral.as_ref().is_some_and(|i| {
                i.template.is_some() || i.template_ref.is_some() || i.locales.is_some()
            })
        })
    }

    /// Determine if compound subentries should be collapsed for this citation.
    fn should_collapse_compound_subentries(
        &self,
        mode: &citum_schema::citation::CitationMode,
    ) -> bool {
        if !matches!(mode, citum_schema::citation::CitationMode::NonIntegral) {
            return false;
        }

        self.bibliography_config
            .as_ref()
            .and_then(|b| b.compound_numeric.as_ref())
            .is_some_and(|c| c.subentry && c.collapse_subentries)
    }

    /// Determine if citation numbers should be collapsed into ranges.
    fn should_collapse_citation_numbers(
        &self,
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
    ) -> bool {
        if !matches!(mode, citum_schema::citation::CitationMode::NonIntegral) {
            return false;
        }

        let is_numeric = self
            .config
            .processing
            .as_ref()
            .is_some_and(|p| matches!(p, citum_schema::options::Processing::Numeric));

        is_numeric
            && matches!(
                spec.collapse,
                Some(citum_schema::CitationCollapse::CitationNumber)
            )
    }

    /// Heuristic for ensuring proper spacing after a citation prefix.
    fn normalize_prefix_spacing(prefix: &str) -> String {
        if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
            format!("{prefix} ")
        } else {
            prefix.to_string()
        }
    }

    /// Ensure suffix has proper spacing (add space if suffix doesn't start with
    /// punctuation and isn't empty).
    fn ensure_suffix_spacing(suffix: &str) -> String {
        if suffix.is_empty() {
            String::new()
        } else if suffix.starts_with(char::is_whitespace)
            || suffix.starts_with(',')
            || suffix.starts_with(';')
            || suffix.starts_with('.')
        {
            // Already has leading space or punctuation
            suffix.to_string()
        } else {
            // Add space before suffix to separate from content
            format!(" {suffix}")
        }
    }

    /// Whether `options.multilingual.scripts.latin.punctuation: latin` applies to the
    /// reference behind `ref_id`.
    ///
    /// Citation-cluster-level `prefix`/`suffix`/`delimiter` (e.g. GB/T author-date's
    /// full-width `（ ）` wrap) are applied in [`Self::affix_content`], outside each
    /// component's own rendering — component-internal punctuation is already remapped
    /// by `render::component::wants_latin_punctuation`. This mirrors that check using
    /// the citation item's resolved reference.
    fn wants_latin_punctuation_for_id(&self, ref_id: &str) -> bool {
        let configured = self.config.multilingual.as_ref().is_some_and(|ml| {
            ml.scripts.get("latin").is_some_and(|script| {
                script.punctuation == Some(citum_schema::options::PunctuationStyle::Latin)
            })
        });

        configured
            && self.bibliography.get(ref_id).is_some_and(|reference| {
                crate::values::is_latin_script_language(
                    crate::values::effective_item_language(reference).as_deref(),
                )
            })
    }

    /// Apply prefix and suffix spacing heuristics to a rendered string.
    ///
    /// `ref_id` identifies the reference this content belongs to, so a
    /// script-aware punctuation remap (see [`Self::wants_latin_punctuation_for_id`])
    /// can be applied to affixes assembled outside component rendering. Pass
    /// `None` when no single reference applies (e.g. author-only content).
    fn affix_content<F>(
        &self,
        fmt: &F,
        content: String,
        prefix: Option<&str>,
        suffix: Option<&str>,
        ref_id: Option<&str>,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let prefix = prefix.unwrap_or("");
        let suffix = suffix.unwrap_or("");
        let affixed = if prefix.is_empty() && suffix.is_empty() {
            content
        } else {
            fmt.affix(
                &Self::normalize_prefix_spacing(prefix),
                content,
                &Self::ensure_suffix_spacing(suffix),
            )
        };

        if ref_id.is_some_and(|id| self.wants_latin_punctuation_for_id(id)) {
            crate::render::component::remap_to_latin_punctuation(affixed)
        } else {
            affixed
        }
    }

    /// Pair rendered content with associated reference IDs to form a semantic chunk.
    fn build_citation_chunk<F>(
        &self,
        fmt: &F,
        ids: Vec<String>,
        content: String,
        prefix: Option<&str>,
        suffix: Option<&str>,
    ) -> Option<(Vec<String>, String)>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        if content.is_empty() {
            None
        } else {
            let affixed = self.affix_content(
                fmt,
                content,
                prefix,
                suffix,
                ids.first().map(String::as_str),
            );
            Some((ids, affixed))
        }
    }

    /// Build a citation chunk for a single item from its rendered content.
    fn build_item_chunk<F>(
        &self,
        fmt: &F,
        item: &crate::reference::CitationItem,
        content: String,
    ) -> Option<(Vec<String>, String)>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.build_citation_chunk(
            fmt,
            vec![item.id.clone()],
            content,
            item.prefix.as_deref(),
            item.suffix.as_deref(),
        )
    }

    /// Create a template render request for a single citation item.
    fn citation_render_request<'b>(
        &self,
        item: &'b crate::reference::CitationItem,
        template: &'b [TemplateComponent],
        mode: &citum_schema::citation::CitationMode,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
        note_start_text_case: Option<citum_schema::NoteStartTextCase>,
    ) -> TemplateRenderRequest<'b> {
        TemplateRenderRequest {
            template,
            context: RenderContext::Citation,
            mode: mode.clone(),
            suppress_author,
            locator_raw: item.locator.as_ref(),
            citation_number: self.get_or_assign_citation_number(&item.id),
            position: position.cloned(),
            note_start_text_case,
            integral_name_state: item.integral_name_state,
            org_abbreviation_state: item.org_abbreviation_state,
            first_reference_note_number: self.first_note_by_id.as_ref().and_then(|m| {
                m.read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(&item.id)
                    .copied()
            }),
        }
    }

    /// Render a single item to a formatted string using a template.
    fn render_item_from_template_with_format<F>(
        &self,
        reference: &Reference,
        request: TemplateRenderRequest<'_>,
        delimiter: &str,
    ) -> Option<String>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.process_template_request_with_format::<F>(reference, request)
            .map(|proc| {
                crate::render::citation::citation_to_string_with_format::<F>(
                    &proc,
                    None,
                    None,
                    None,
                    Some(delimiter),
                )
            })
    }

    /// Resolve the reference, template, and delimiter needed to render one
    /// ungrouped citation item, applying type-variant and language fallbacks.
    fn resolve_ungrouped_item_render_state<'b>(
        &'b self,
        item: &'b crate::reference::CitationItem,
        spec: &'b citum_schema::CitationSpec,
        intra_delimiter: &'b str,
    ) -> Result<UngroupedItemRenderState<'b>, ProcessorError> {
        let reference = self
            .bibliography
            .get(&item.id)
            .ok_or_else(|| ProcessorError::ReferenceNotFound(item.id.clone()))?;
        let ref_type = reference.ref_type();
        let item_language = crate::values::effective_item_language(reference);
        let localized = spec.resolve_localized_template(item_language.as_deref());
        let template = localized
            .as_ref()
            .filter(|resolved| resolved.type_variants.is_some())
            .cloned()
            .map(|resolved| Cow::Owned(resolve_localized_type_variant(resolved, None, &ref_type)))
            .or_else(|| {
                resolve_type_variant(spec.type_variants.as_ref(), &ref_type).map(Cow::Borrowed)
            })
            .or_else(|| localized.map(|resolved| Cow::Owned(resolved.template)))
            .unwrap_or(Cow::Borrowed(&[] as &[TemplateComponent]));

        Ok(UngroupedItemRenderState {
            reference,
            template,
            delimiter: spec.delimiter.as_deref().unwrap_or(intra_delimiter),
        })
    }

    /// Initialize render options for a citation.
    fn citation_render_options<'b>(
        &'b self,
        reference: &Reference,
        mode: citum_schema::citation::CitationMode,
        suppress_author: bool,
        locator_raw: Option<&'b CitationLocator>,
        ref_type: Option<String>,
    ) -> RenderOptions<'b> {
        RenderOptions {
            config: self.config.clone(),
            bibliography_config: self.bibliography_config.clone(),
            locale: self.locale_for_reference(reference, RenderContext::Citation),
            context: RenderContext::Citation,
            mode,
            suppress_author,
            locator_raw,
            ref_type,
            show_semantics: self.show_semantics,
            current_template_index: None,
            abbreviation_map: self.abbreviation_map,
        }
    }

    /// Render author + citation number for numeric integral citations.
    ///
    /// Default implementation for narrative citations in numeric styles (e.g., "Smith [1]").
    fn render_author_number_for_numeric_integral_with_format<F>(
        &self,
        reference: &Reference,
        item: &crate::reference::CitationItem,
        citation_number: usize,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let options = self.citation_render_options(
            reference,
            citum_schema::citation::CitationMode::Integral,
            false,
            item.locator.as_ref(),
            Some(reference.ref_type()),
        );

        // Render author in short form
        let author_part = if let Some(authors) = reference.author() {
            let names_vec = self.resolve_contributor_names(&authors);
            fmt.text(&crate::values::format_contributors_short(
                &names_vec, &options,
            ))
        } else {
            String::new()
        };

        // Include compound sub-label (e.g. "a", "b") when applicable.
        let ref_id = reference.id().unwrap_or_default().to_string();
        let sub_label = self.citation_sub_label_for_ref(&ref_id).unwrap_or_default();

        // Format: "Author [Na]"
        if author_part.is_empty() {
            // Fallback: just citation number if no author
            format!("[{citation_number}{sub_label}]")
        } else {
            format!("{author_part} [{citation_number}{sub_label}]")
        }
    }

    /// Render one item as author + citation number for numeric integral cites.
    fn render_numeric_integral_item_chunk_with_format<F>(
        &self,
        fmt: &F,
        item: &crate::reference::CitationItem,
    ) -> Result<Option<(Vec<String>, String)>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let reference = self
            .bibliography
            .get(&item.id)
            .ok_or_else(|| ProcessorError::ReferenceNotFound(item.id.clone()))?;
        let citation_number = self.get_or_assign_citation_number(&item.id);
        let item_str = self.render_author_number_for_numeric_integral_with_format::<F>(
            reference,
            item,
            citation_number,
        );
        Ok(self.build_item_chunk(fmt, item, item_str))
    }

    /// Render one ungrouped item from its resolved template state.
    fn render_template_item_chunk_with_format<F>(
        &self,
        fmt: &F,
        item: &crate::reference::CitationItem,
        state: UngroupedItemRenderState<'_>,
        params: UngroupedItemRenderParams<'_>,
    ) -> Option<(Vec<String>, String)>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let request = self.citation_render_request(
            item,
            &state.template,
            params.mode,
            params.suppress_author,
            params.position,
            params.note_start_text_case,
        );
        self.render_item_from_template_with_format::<F>(state.reference, request, state.delimiter)
            .and_then(|item_str| self.build_item_chunk(fmt, item, item_str))
    }

    /// Render citation items without grouping, using plain text format.
    ///
    /// # Errors
    ///
    /// Returns an error when a referenced item is missing or item rendering
    /// fails.
    pub fn render_ungrouped_citation(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Vec<String>, ProcessorError> {
        self.render_ungrouped_citation_with_format::<crate::render::plain::PlainText>(
            items,
            spec,
            mode,
            intra_delimiter,
            suppress_author,
            position,
            spec.note_start_text_case,
        )
    }

    /// Render citation items without grouping, generic over the output format.
    ///
    /// This is the core logic for iterating over citation items, looking up references,
    /// and applying the appropriate template or fallback logic.
    ///
    /// # Errors
    ///
    /// Returns an error when a referenced item is missing or item rendering
    /// fails.
    #[allow(
        clippy::too_many_arguments,
        reason = "Ungrouped citation rendering now needs explicit note-start context."
    )]
    pub fn render_ungrouped_citation_with_format<F>(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
        note_start_text_case: Option<citum_schema::NoteStartTextCase>,
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let mut chunks: Vec<(Vec<String>, String)> = Vec::new();

        // For numeric styles with integral mode, render author + citation number instead.
        let use_author_number = self.should_render_author_number_for_numeric_integral(mode);
        let params = UngroupedItemRenderParams {
            mode,
            suppress_author,
            position,
            note_start_text_case,
        };

        for item in items {
            let chunk = if use_author_number {
                self.render_numeric_integral_item_chunk_with_format::<F>(&fmt, item)?
            } else {
                let state =
                    self.resolve_ungrouped_item_render_state(item, spec, intra_delimiter)?;
                self.render_template_item_chunk_with_format::<F>(&fmt, item, state, params)
            };

            if let Some(chunk) = chunk {
                chunks.push(chunk);
            }
        }

        if self.should_collapse_compound_subentries(mode) {
            chunks = self.collapse_compound_citation_chunks(chunks);
        }
        if self.should_collapse_citation_numbers(spec, mode) {
            chunks = self.collapse_numeric_citation_chunks(chunks);
        }

        Ok(chunks
            .into_iter()
            .map(|(ids, content)| fmt.citation(ids, content))
            .collect())
    }
}

fn key_base(key: &str) -> Cow<'_, str> {
    let mut parts = key.splitn(3, ':');
    match (parts.next(), parts.next()) {
        (Some(kind), Some(var)) => Cow::Owned(format!("{kind}:{var}")),
        _ => Cow::Borrowed(key),
    }
}

/// Get a unique key for a template component's variable.
///
/// The key includes rendering context (prefix/suffix) to allow the same variable
/// to render multiple times if it appears in semantically different contexts.
/// This enables styles like Chicago that require year after author AND after publisher.
#[must_use]
pub fn get_variable_key(component: &TemplateComponent) -> Option<String> {
    use citum_schema::template::Rendering;
    use std::fmt::Write;

    fn push_context_suffix(key: &mut String, rendering: &Rendering) {
        match (&rendering.prefix, &rendering.suffix) {
            (Some(prefix), Some(suffix)) => {
                key.push(':');
                key.push_str(prefix);
                key.push('_');
                key.push_str(suffix);
            }
            (Some(prefix), None) => {
                key.push(':');
                key.push_str(prefix);
            }
            (None, Some(suffix)) => {
                key.push(':');
                key.push_str(suffix);
            }
            (None, None) => {}
        }
    }

    fn make_key(kind: &str, value: impl std::fmt::Debug, rendering: &Rendering) -> Option<String> {
        let mut key = String::new();
        write!(&mut key, "{kind}:{value:?}").ok()?;
        push_context_suffix(&mut key, rendering);
        Some(key)
    }

    match component {
        TemplateComponent::Contributor(c) => c.contributor.as_single().map_or_else(
            || make_key("contributor", &c.contributor, &c.rendering),
            |role| make_key("contributor", role, &c.rendering),
        ),
        TemplateComponent::Date(d) => make_key("date", &d.date, &d.rendering),
        TemplateComponent::Variable(v) => make_key("variable", &v.variable, &v.rendering),
        TemplateComponent::Title(t) => {
            let mut key = format!("title:{:?}", t.title);
            if let Some(form) = &t.form {
                write!(&mut key, ":{form:?}").ok()?;
            }
            push_context_suffix(&mut key, &t.rendering);
            Some(key)
        }
        TemplateComponent::Number(n) => make_key("number", &n.number, &n.rendering),
        TemplateComponent::Identifier(i) => make_key("identifier", &i.identifier, &i.rendering),
        TemplateComponent::Group(_) => None,
        _ => None,
    }
}
