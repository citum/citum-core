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
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

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
    pub config: &'a Config,
    /// The active bibliography-only configuration.
    pub bibliography_config: Option<BibliographyConfig>,
    /// Pre-calculated hints for optimization.
    pub hints: &'a HashMap<String, ProcHints>,
    /// Shared state for citation numbers (used in numeric styles).
    pub citation_numbers: &'a RefCell<HashMap<String, usize>>,
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

pub(crate) use grouped_fallback::GroupRenderParams;
pub use grouped_fallback::TemplateRenderParams;
pub(super) use helpers::{
    find_grouping_component, has_contributor_component, leading_group_affix,
    strip_author_component, strip_leading_group_affixes,
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
}

#[derive(Default)]
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
        self.rendered_vars.contains(var_key) || self.substituted_bases.contains(&base)
    }

    fn mark_rendered(&mut self, var_key: Option<String>, substituted_key: Option<&str>) {
        if let Some(var_key) = var_key {
            self.rendered_vars.insert(var_key);
        }
        if let Some(substituted_key) = substituted_key {
            self.rendered_vars.insert(substituted_key.to_string());
            self.substituted_bases.insert(key_base(substituted_key));
        }
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
    pub config: &'a Config,
    /// The active bibliography-only configuration.
    pub bibliography_config: Option<BibliographyConfig>,
}

impl<'a> Renderer<'a> {
    /// Creates a new `Renderer` instance.
    pub fn new(
        resources: RendererResources<'a>,
        hints: &'a HashMap<String, ProcHints>,
        citation_numbers: &'a RefCell<HashMap<String, usize>>,
        compound: CompoundRenderData<'a>,
        show_semantics: bool,
        inject_ast_indices: bool,
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
        }
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
            c.integral
                .as_ref()
                .is_some_and(|i| i.template.is_some() || i.extends.is_some() || i.locales.is_some())
        })
    }

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

    fn affix_content<F>(
        &self,
        fmt: &F,
        content: String,
        prefix: Option<&str>,
        suffix: Option<&str>,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let prefix = prefix.unwrap_or("");
        let suffix = suffix.unwrap_or("");
        if prefix.is_empty() && suffix.is_empty() {
            content
        } else {
            fmt.affix(
                &Self::normalize_prefix_spacing(prefix),
                content,
                &Self::ensure_suffix_spacing(suffix),
            )
        }
    }

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
            Some((ids, self.affix_content(fmt, content, prefix, suffix)))
        }
    }

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
        }
    }

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

    fn citation_render_options<'b>(
        &'b self,
        mode: citum_schema::citation::CitationMode,
        suppress_author: bool,
        locator_raw: Option<&'b CitationLocator>,
        ref_type: Option<String>,
    ) -> RenderOptions<'b> {
        RenderOptions {
            config: self.config,
            bibliography_config: self.bibliography_config.clone(),
            locale: self.locale,
            context: RenderContext::Citation,
            mode,
            suppress_author,
            locator_raw,
            ref_type,
            show_semantics: self.show_semantics,
            current_template_index: None,
        }
    }

    /// Render author + citation number for numeric integral citations.
    ///
    /// This is used as a default for numeric styles in narrative mode (e.g., "Smith [1]").
    /// It renders the author's short name followed by the citation number in brackets.
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

        for item in items {
            let reference = self
                .bibliography
                .get(&item.id)
                .ok_or_else(|| ProcessorError::ReferenceNotFound(item.id.clone()))?;

            if use_author_number {
                // Numeric integral: render author + citation number
                let citation_number = self.get_or_assign_citation_number(&item.id);
                let item_str = self.render_author_number_for_numeric_integral_with_format::<F>(
                    reference,
                    item,
                    citation_number,
                );
                if let Some(chunk) = self.build_citation_chunk(
                    &fmt,
                    vec![item.id.clone()],
                    item_str,
                    item.prefix.as_deref(),
                    item.suffix.as_deref(),
                ) {
                    chunks.push(chunk);
                }
            } else {
                // Standard rendering: use template with citation number
                let item_language = crate::values::effective_item_language(reference);
                let default_template = spec.resolve_template_for_language(item_language.as_deref());

                let ref_type = reference.ref_type();
                let matched_type_template = spec.type_variants.as_ref().and_then(|type_variants| {
                    let mut matched_template = None;
                    for (selector, template) in type_variants {
                        if selector.matches(&ref_type) {
                            matched_template = template.clone().into_template();
                            break;
                        }
                    }
                    matched_template
                });

                let template = matched_type_template.or(default_template);
                let effective_template = template.as_deref().unwrap_or(&[]);
                let effective_delim = spec.delimiter.as_deref().unwrap_or(intra_delimiter);
                let request = self.citation_render_request(
                    item,
                    effective_template,
                    mode,
                    suppress_author,
                    position,
                    note_start_text_case,
                );
                if let Some(item_str) = self.render_item_from_template_with_format::<F>(
                    reference,
                    request,
                    effective_delim,
                ) && let Some(chunk) = self.build_citation_chunk(
                    &fmt,
                    vec![item.id.clone()],
                    item_str,
                    item.prefix.as_deref(),
                    item.suffix.as_deref(),
                ) {
                    chunks.push(chunk);
                }
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

fn key_base(key: &str) -> String {
    let mut parts = key.splitn(3, ':');
    match (parts.next(), parts.next()) {
        (Some(kind), Some(var)) => format!("{kind}:{var}"),
        _ => key.to_string(),
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

    fn context_suffix(rendering: &Rendering) -> String {
        match (&rendering.prefix, &rendering.suffix) {
            (Some(p), Some(s)) => format!(":{p}_{s}"),
            (Some(p), None) => format!(":{p}"),
            (None, Some(s)) => format!(":{s}"),
            (None, None) => String::new(),
        }
    }

    fn make_key(kind: &str, value: impl std::fmt::Debug, ctx: String) -> Option<String> {
        Some(format!("{kind}:{value:?}{ctx}"))
    }

    match component {
        TemplateComponent::Contributor(c) => {
            make_key("contributor", &c.contributor, context_suffix(&c.rendering))
        }
        TemplateComponent::Date(d) => make_key("date", &d.date, context_suffix(&d.rendering)),
        TemplateComponent::Variable(v) => {
            make_key("variable", &v.variable, context_suffix(&v.rendering))
        }
        TemplateComponent::Title(t) => {
            let form_part = t.form.as_ref().map_or(String::new(), |f| format!(":{f:?}"));
            Some(format!(
                "title:{:?}{form_part}{}",
                t.title,
                context_suffix(&t.rendering)
            ))
        }
        TemplateComponent::Number(n) => make_key("number", &n.number, context_suffix(&n.rendering)),
        TemplateComponent::Group(_) => None,
        _ => None,
    }
}
