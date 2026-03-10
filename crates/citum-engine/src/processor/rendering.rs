//! Rendering logic for citation and bibliography output.
//!
//! This module handles template-based rendering of citations and bibliographies,
//! including handling of localization, numbering, formatting, and special modes
//! like integral (narrative) citations for numeric and label styles.

use crate::error::ProcessorError;
use crate::reference::{Bibliography, Reference};
use crate::render::{ProcTemplate, ProcTemplateComponent};
use crate::values::range::{ConsecutiveSegment, consecutive_segments};
use crate::values::{ComponentValues, ProcHints, RenderContext, RenderOptions};
use citum_schema::citation::{CitationLocator, LocatorSegment, LocatorType};
use citum_schema::locale::{Locale, TermForm};
use citum_schema::options::Config;
use citum_schema::template::ComponentOverride;
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

/// Collapse compound locator segments into a pre-labelled string.
///
/// Each segment is rendered as `"term value"` using the locale's short-form term,
/// then joined with `", "`. Falls back to the label name if no locale term exists.
fn collapse_compound_locator(segments: &[LocatorSegment], locale: &Locale) -> String {
    segments
        .iter()
        .map(|seg| {
            let plural = seg.value.is_plural();
            let term = locale
                .locator_term(&seg.label, plural, TermForm::Short)
                .or_else(|| locale.locator_term(&seg.label, plural, TermForm::Symbol))
                .map(|t| t.to_string())
                .unwrap_or_else(|| {
                    // Fall back to serde kebab-case name for user-facing output
                    serde_json::to_value(seg.label)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| format!("{:?}", seg.label))
                });
            format!("{} {}", term, seg.value.value_str())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Resolve a citation item's locator into a `(value, label)` pair for `RenderOptions`.
///
/// Compound locators are collapsed to a pre-labelled string with no separate label
/// (since labels are embedded per-segment). Flat locators pass through unchanged.
fn resolve_item_locator(
    item: &citum_schema::citation::CitationItem,
    locale: &Locale,
) -> (Option<String>, Option<LocatorType>) {
    match item.locator.as_ref() {
        Some(CitationLocator::Single(segment)) => (
            Some(segment.value.value_str().to_string()),
            Some(segment.label),
        ),
        Some(CitationLocator::Compound { segments }) => {
            (Some(collapse_compound_locator(segments, locale)), None)
        }
        None => (None, None),
    }
}

impl<'a> Renderer<'a> {
    /// Creates a new `Renderer` instance.
    pub fn new(
        style: &'a citum_schema::Style,
        bibliography: &'a Bibliography,
        locale: &'a Locale,
        config: &'a Config,
        hints: &'a HashMap<String, ProcHints>,
        citation_numbers: &'a RefCell<HashMap<String, usize>>,
        compound: CompoundRenderData<'a>,
    ) -> Self {
        Self {
            style,
            bibliography,
            locale,
            config,
            hints,
            citation_numbers,
            compound_set_by_ref: compound.set_by_ref,
            compound_member_index: compound.member_index,
            compound_sets: compound.sets,
        }
    }

    fn citation_sub_label_for_ref(&self, ref_id: &str) -> Option<String> {
        let compound = self
            .config
            .bibliography
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
        if !matches!(mode, citum_schema::citation::CitationMode::Integral) {
            return false;
        }

        let is_numeric = self
            .config
            .processing
            .as_ref()
            .map(|p| matches!(p, citum_schema::options::Processing::Numeric))
            .unwrap_or(false);

        if !is_numeric {
            return false;
        }

        // If the style provides an explicit integral template, use it instead of the hardcoded default.
        let has_explicit_integral = self
            .style
            .citation
            .as_ref()
            .map(|cs| cs.integral.is_some())
            .unwrap_or(false);

        !has_explicit_integral
    }

    /// Determines if the processor should render author text for a label style
    /// when in "integral" (narrative) citation mode.
    fn should_render_author_for_label_integral(
        &self,
        mode: &citum_schema::citation::CitationMode,
    ) -> bool {
        if !matches!(mode, citum_schema::citation::CitationMode::Integral) {
            return false;
        }

        let is_label = self
            .config
            .processing
            .as_ref()
            .map(|p| matches!(p, citum_schema::options::Processing::Label(_)))
            .unwrap_or(false);

        if !is_label {
            return false;
        }

        // If the style provides an explicit integral template, use it instead of the hardcoded default.
        let has_explicit_integral = self
            .style
            .citation
            .as_ref()
            .map(|cs| cs.integral.is_some())
            .unwrap_or(false);

        !has_explicit_integral
    }

    fn should_collapse_compound_subentries(
        &self,
        mode: &citum_schema::citation::CitationMode,
    ) -> bool {
        if !matches!(mode, citum_schema::citation::CitationMode::NonIntegral) {
            return false;
        }

        self.config
            .bibliography
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
            .map(|p| matches!(p, citum_schema::options::Processing::Numeric))
            .unwrap_or(false);

        is_numeric
            && matches!(
                spec.collapse,
                Some(citum_schema::CitationCollapse::CitationNumber)
            )
    }

    fn collapse_numeric_citation_chunks(
        &self,
        chunks: Vec<(Vec<String>, String)>,
    ) -> Vec<(Vec<String>, String)> {
        let citation_numbers = self.citation_numbers.borrow();
        let mut collapsed = Vec::new();
        let mut i = 0;

        while i < chunks.len() {
            let Some(ref_id) = chunks[i].0.first() else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            if chunks[i].0.len() != 1 {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }
            let Some(&citation_number) = citation_numbers.get(ref_id) else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            if chunks[i].1 != citation_number.to_string() {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }

            let mut j = i;
            let mut block_ids = Vec::new();
            let mut end_number = citation_number;

            while j < chunks.len() {
                let Some(candidate_id) = chunks[j].0.first() else {
                    break;
                };
                if chunks[j].0.len() != 1 {
                    break;
                }
                let Some(&candidate_number) = citation_numbers.get(candidate_id) else {
                    break;
                };
                if chunks[j].1 != candidate_number.to_string() {
                    break;
                }
                if !block_ids.is_empty() && candidate_number != end_number + 1 {
                    break;
                }

                block_ids.push(candidate_id.clone());
                end_number = candidate_number;
                j += 1;
            }

            if block_ids.len() < 2 {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }

            collapsed.push((block_ids, format!("{citation_number}–{end_number}")));
            i = j;
        }

        collapsed
    }

    fn collapse_compound_citation_chunks(
        &self,
        chunks: Vec<(Vec<String>, String)>,
    ) -> Vec<(Vec<String>, String)> {
        let Some(compound) = self
            .config
            .bibliography
            .as_ref()
            .and_then(|b| b.compound_numeric.as_ref())
        else {
            return chunks;
        };

        if !matches!(
            compound.sub_label,
            citum_schema::options::bibliography::SubLabelStyle::Alphabetic
        ) {
            return chunks;
        }

        let citation_numbers = self.citation_numbers.borrow();
        let mut collapsed = Vec::new();
        let mut i = 0;

        while i < chunks.len() {
            let Some(ref_id) = chunks[i].0.first() else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            let Some(group_id) = self.compound_set_by_ref.get(ref_id) else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            let Some(&citation_number) = citation_numbers.get(ref_id) else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };

            let mut j = i;
            let mut block_ids = Vec::new();
            let mut member_ordinals = Vec::new();

            while j < chunks.len() {
                let Some(candidate_id) = chunks[j].0.first() else {
                    break;
                };
                if chunks[j].0.len() != 1
                    || self.compound_set_by_ref.get(candidate_id) != Some(group_id)
                    || citation_numbers.get(candidate_id).copied() != Some(citation_number)
                {
                    break;
                }

                let Some(member_index) = self.compound_member_index.get(candidate_id).copied()
                else {
                    break;
                };
                let expected = format!(
                    "{}{}",
                    citation_number,
                    self.citation_sub_label_for_ref(candidate_id)
                        .unwrap_or_default()
                );
                if chunks[j].1 != expected {
                    break;
                }

                block_ids.push(candidate_id.clone());
                member_ordinals.push((member_index + 1) as u32);
                j += 1;
            }

            if block_ids.len() < 2 {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }

            let labels = consecutive_segments(&member_ordinals)
                .into_iter()
                .map(|segment| match segment {
                    ConsecutiveSegment::Single(value) => {
                        crate::values::int_to_letter(value).unwrap_or_default()
                    }
                    ConsecutiveSegment::Range { start, end } => {
                        let start_label = crate::values::int_to_letter(start).unwrap_or_default();
                        let end_label = crate::values::int_to_letter(end).unwrap_or_default();
                        format!("{start_label}-{end_label}")
                    }
                })
                .collect::<Vec<_>>()
                .join(",");

            collapsed.push((block_ids, format!("{citation_number}{labels}")));
            i = j;
        }

        collapsed
    }

    /// Ensure suffix has proper spacing (add space if suffix doesn't start with
    /// punctuation and isn't empty).
    fn ensure_suffix_spacing(&self, suffix: &str) -> String {
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
            format!(" {}", suffix)
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
        let (loc_value, loc_label) = resolve_item_locator(item, self.locale);
        let options = RenderOptions {
            config: self.config,
            locale: self.locale,
            context: RenderContext::Citation,
            mode: citum_schema::citation::CitationMode::Integral,
            suppress_author: false,
            locator: loc_value.as_deref(),
            locator_label: loc_label,
        };

        // Render author in short form
        let author_part = if let Some(authors) = reference.author() {
            let mode = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.name_mode.as_ref());
            let preferred_transliteration = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.preferred_transliteration.as_deref());
            let preferred_script = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.preferred_script.as_ref());
            let locale_str = &self.locale.locale;

            let names_vec = crate::values::resolve_multilingual_name(
                &authors,
                mode,
                preferred_transliteration,
                preferred_script,
                locale_str,
            );
            fmt.text(&crate::values::format_contributors_short(
                &names_vec, &options,
            ))
        } else {
            String::new()
        };

        // Include compound sub-label (e.g. "a", "b") when applicable.
        let ref_id = reference.id().unwrap_or_default();
        let sub_label = self.citation_sub_label_for_ref(&ref_id).unwrap_or_default();

        // Format: "Author [Na]"
        if !author_part.is_empty() {
            format!("{} [{}{}]", author_part, citation_number, sub_label)
        } else {
            // Fallback: just citation number if no author
            format!("[{}{}]", citation_number, sub_label)
        }
    }

    /// Render author-only text for label integral citations.
    ///
    /// Used as a default for label-based styles in narrative mode (e.g., "Smith [Smi20]").
    fn render_author_for_label_integral_with_format<F>(
        &self,
        reference: &Reference,
        item: &crate::reference::CitationItem,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let (loc_value, loc_label) = resolve_item_locator(item, self.locale);
        let options = RenderOptions {
            config: self.config,
            locale: self.locale,
            context: RenderContext::Citation,
            mode: citum_schema::citation::CitationMode::Integral,
            suppress_author: false,
            locator: loc_value.as_deref(),
            locator_label: loc_label,
        };

        if let Some(contributor) = reference.author().or_else(|| reference.editor()) {
            let mode = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.name_mode.as_ref());
            let preferred_transliteration = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.preferred_transliteration.as_deref());
            let preferred_script = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.preferred_script.as_ref());
            let locale_str = &self.locale.locale;

            let names_vec = crate::values::resolve_multilingual_name(
                &contributor,
                mode,
                preferred_transliteration,
                preferred_script,
                locale_str,
            );
            let author_part = fmt.text(&crate::values::format_contributors_short(
                &names_vec, &options,
            ));
            if !author_part.is_empty() {
                return author_part;
            }
        }

        reference
            .title()
            .map(|title| fmt.text(&title.to_string()))
            .unwrap_or_default()
    }

    /// Render citation items without grouping, using plain text format.
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
        )
    }

    /// Render citation items without grouping, generic over the output format.
    ///
    /// This is the core logic for iterating over citation items, looking up references,
    /// and applying the appropriate template or fallback logic.
    pub fn render_ungrouped_citation_with_format<F>(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let mut chunks: Vec<(Vec<String>, String)> = Vec::new();

        // For numeric styles with integral mode, render author + citation number instead.
        let use_author_number = self.should_render_author_number_for_numeric_integral(mode);
        // For label styles with integral mode, render narrative contributor text.
        let use_label_author = self.should_render_author_for_label_integral(mode);

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
                if !item_str.is_empty() {
                    let prefix = item.prefix.as_deref().unwrap_or("");
                    let suffix = item.suffix.as_deref().unwrap_or("");

                    let formatted_prefix =
                        if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                            format!("{} ", prefix)
                        } else {
                            prefix.to_string()
                        };

                    let content = if !prefix.is_empty() || !suffix.is_empty() {
                        let spaced_suffix = self.ensure_suffix_spacing(suffix);
                        fmt.affix(&formatted_prefix, item_str, &spaced_suffix)
                    } else {
                        item_str
                    };
                    chunks.push((vec![item.id.clone()], content));
                }
            } else if use_label_author {
                let item_str =
                    self.render_author_for_label_integral_with_format::<F>(reference, item);
                if !item_str.is_empty() {
                    let prefix = item.prefix.as_deref().unwrap_or("");
                    let suffix = item.suffix.as_deref().unwrap_or("");

                    let formatted_prefix =
                        if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                            format!("{} ", prefix)
                        } else {
                            prefix.to_string()
                        };

                    let content = if !prefix.is_empty() || !suffix.is_empty() {
                        let spaced_suffix = self.ensure_suffix_spacing(suffix);
                        fmt.affix(&formatted_prefix, item_str, &spaced_suffix)
                    } else {
                        item_str
                    };
                    chunks.push((vec![item.id.clone()], content));
                }
            } else {
                // Standard rendering: use template with citation number
                let citation_number = self.get_or_assign_citation_number(&item.id);
                let item_language = crate::values::effective_item_language(reference);
                let template = spec.resolve_template_for_language(item_language.as_deref());
                let effective_template = template.as_deref().unwrap_or(&[]);
                let effective_delim = spec.delimiter.as_deref().unwrap_or(intra_delimiter);

                let (loc_value, loc_label) = resolve_item_locator(item, self.locale);
                if let Some(proc) = self.process_template_with_number_with_format::<F>(
                    reference,
                    effective_template,
                    RenderContext::Citation,
                    mode.clone(),
                    suppress_author,
                    citation_number,
                    loc_value.as_deref(),
                    loc_label,
                    position,
                    item.integral_name_state,
                ) {
                    let item_str = crate::render::citation::citation_to_string_with_format::<F>(
                        &proc,
                        None,
                        None,
                        None,
                        Some(effective_delim),
                    );
                    if !item_str.is_empty() {
                        let prefix = item.prefix.as_deref().unwrap_or("");
                        let suffix = item.suffix.as_deref().unwrap_or("");

                        let formatted_prefix =
                            if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                                format!("{} ", prefix)
                            } else {
                                prefix.to_string()
                            };

                        let content = if !prefix.is_empty() || !suffix.is_empty() {
                            let spaced_suffix = self.ensure_suffix_spacing(suffix);
                            fmt.affix(&formatted_prefix, item_str, &spaced_suffix)
                        } else {
                            item_str
                        };
                        chunks.push((vec![item.id.clone()], content));
                    }
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

    /// Render citation items with author grouping for author-date styles.
    pub fn render_grouped_citation(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Vec<String>, ProcessorError> {
        self.render_grouped_citation_with_format::<crate::render::plain::PlainText>(
            items,
            spec,
            mode,
            intra_delimiter,
            suppress_author,
            position,
        )
    }

    /// Render a grouped citation into one formatted string per citation item.
    ///
    /// This preserves per-item output when grouping rules require items to stay
    /// separate, and otherwise applies the requested renderer format to the
    /// grouped citation output.
    pub fn render_grouped_citation_with_format<F>(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        use crate::reference::CitationItem;

        let preserve_individual_citations = items.iter().any(|item| {
            self.hints
                .get(&item.id)
                .is_some_and(|hints| hints.min_names_to_show.is_some() || hints.expand_given_names)
        });

        // Group adjacent items by author key (respecting substitution). When a cite
        // already depends on name expansion for disambiguation, keep each item
        // separate instead of collapsing by author.
        let mut groups: Vec<(String, Vec<&CitationItem>)> = Vec::new();

        for item in items {
            let reference = self.bibliography.get(&item.id);
            let author_key = if preserve_individual_citations {
                item.id.clone()
            } else {
                reference
                    .map(|r| self.get_author_grouping_key(r))
                    .unwrap_or_default()
            };

            // Check if this item has the same author as the previous group
            if !groups.is_empty()
                && groups.last().unwrap().0 == author_key
                && !author_key.is_empty()
            {
                groups.last_mut().unwrap().1.push(item);
            } else {
                groups.push((author_key, vec![item]));
            }
        }

        let mut rendered_groups = Vec::new();
        let fmt = F::default();

        for (_author_key, group) in groups {
            let first_item = group[0];
            let first_ref = self
                .bibliography
                .get(&first_item.id)
                .ok_or_else(|| ProcessorError::ReferenceNotFound(first_item.id.clone()))?;
            let first_language = crate::values::effective_item_language(first_ref);
            let first_template = spec.resolve_template_for_language(first_language.as_deref());
            let template = first_template.as_deref().unwrap_or(&[]);
            let has_explicit_integral_template = self
                .style
                .citation
                .as_ref()
                .and_then(|citation| citation.integral.as_ref())
                .is_some_and(|integral| integral.template.is_some() || integral.locales.is_some());

            // If we have an explicit integral template and we're in integral mode,
            // we should try to use it.
            if matches!(mode, citum_schema::citation::CitationMode::Integral)
                && has_explicit_integral_template
                && !template.is_empty()
            {
                // Narrative mode with explicit template (e.g., APA 7th)
                let citation_number = self.get_or_assign_citation_number(&first_item.id);
                let (loc_value, loc_label) = resolve_item_locator(first_item, self.locale);
                if let Some(proc) = self.process_template_with_number_with_format::<F>(
                    first_ref,
                    template,
                    RenderContext::Citation,
                    mode.clone(),
                    suppress_author,
                    citation_number,
                    loc_value.as_deref(),
                    loc_label,
                    position,
                    first_item.integral_name_state,
                ) {
                    // Use integral-specific delimiter, defaulting to space for narrative
                    let integral_delimiter = spec.delimiter.as_deref().unwrap_or(" ");
                    let item_str = crate::render::citation::citation_to_string_with_format::<F>(
                        &proc,
                        None,
                        None,
                        None,
                        Some(integral_delimiter),
                    );

                    let ids: Vec<String> = group.iter().map(|item| item.id.clone()).collect();
                    let prefix = first_item.prefix.as_deref().unwrap_or("");
                    let suffix = first_item.suffix.as_deref().unwrap_or("");

                    let formatted_prefix =
                        if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                            format!("{} ", prefix)
                        } else {
                            prefix.to_string()
                        };

                    let content = if !prefix.is_empty() || !suffix.is_empty() {
                        let spaced_suffix = self.ensure_suffix_spacing(suffix);
                        fmt.affix(&formatted_prefix, item_str, &spaced_suffix)
                    } else {
                        item_str
                    };

                    rendered_groups.push(fmt.citation(ids, content));
                    continue;
                }
            }

            // Non-integral legal cases and personal communications need full template
            // rendering; grouped author/year compression drops required content.
            if matches!(mode, citum_schema::citation::CitationMode::NonIntegral)
                && matches!(
                    first_ref.ref_type().as_str(),
                    "legal-case" | "personal-communication"
                )
            {
                for item in &group {
                    let reference = self
                        .bibliography
                        .get(&item.id)
                        .ok_or_else(|| ProcessorError::ReferenceNotFound(item.id.clone()))?;
                    let item_language = crate::values::effective_item_language(reference);
                    let template = spec.resolve_template_for_language(item_language.as_deref());
                    let effective_template = template.as_deref().unwrap_or(&[]);
                    let citation_number = self.get_or_assign_citation_number(&item.id);
                    let (loc_value, loc_label) = resolve_item_locator(item, self.locale);
                    if let Some(proc) = self.process_template_with_number_with_format::<F>(
                        reference,
                        effective_template,
                        RenderContext::Citation,
                        mode.clone(),
                        suppress_author,
                        citation_number,
                        loc_value.as_deref(),
                        loc_label,
                        position,
                        item.integral_name_state,
                    ) {
                        let item_str = crate::render::citation::citation_to_string_with_format::<F>(
                            &proc,
                            None,
                            None,
                            None,
                            Some(intra_delimiter),
                        );
                        if !item_str.is_empty() {
                            let prefix = item.prefix.as_deref().unwrap_or("");
                            let suffix = item.suffix.as_deref().unwrap_or("");
                            let formatted_prefix =
                                if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                                    format!("{} ", prefix)
                                } else {
                                    prefix.to_string()
                                };
                            let content = if !prefix.is_empty() || !suffix.is_empty() {
                                let spaced_suffix = self.ensure_suffix_spacing(suffix);
                                fmt.affix(&formatted_prefix, item_str, &spaced_suffix)
                            } else {
                                item_str
                            };
                            rendered_groups.push(fmt.citation(vec![item.id.clone()], content));
                        }
                    }
                }
                continue;
            }

            // Fallback to default hardcoded grouping (or if no integral template)
            let author_part = self.render_author_for_grouping_with_format::<F>(
                first_ref,
                first_item,
                template,
                mode,
                suppress_author,
                position,
            );

            let mut item_parts = Vec::new();
            let mut group_delimiter: Option<String> = None;
            for item in &group {
                let reference = self
                    .bibliography
                    .get(&item.id)
                    .ok_or_else(|| ProcessorError::ReferenceNotFound(item.id.clone()))?;
                let item_language = crate::values::effective_item_language(reference);
                let template = spec.resolve_template_for_language(item_language.as_deref());
                let effective_template = template.as_deref().unwrap_or(&[]);
                let (filtered_template, leading_affix) =
                    self.filter_author_from_template(effective_template);
                if group_delimiter.is_none() {
                    group_delimiter = leading_affix
                        .as_ref()
                        .filter(|value| !value.is_empty())
                        .cloned();
                }
                let item_delimiter = if leading_affix.is_some() {
                    ""
                } else {
                    intra_delimiter
                };

                let citation_number = self.get_or_assign_citation_number(&item.id);
                let (loc_value, loc_label) = resolve_item_locator(item, self.locale);
                if let Some(proc) = self.process_template_with_number_with_format::<F>(
                    reference,
                    &filtered_template,
                    RenderContext::Citation,
                    mode.clone(),
                    suppress_author,
                    citation_number,
                    loc_value.as_deref(),
                    loc_label,
                    position,
                    item.integral_name_state,
                ) {
                    let item_str = crate::render::citation::citation_to_string_with_format::<F>(
                        &proc,
                        None,
                        None,
                        None,
                        Some(item_delimiter),
                    );
                    if !item_str.is_empty() {
                        let suffix = item.suffix.as_deref().unwrap_or("");
                        if !suffix.is_empty() {
                            let spaced_suffix = self.ensure_suffix_spacing(suffix);
                            item_parts.push(fmt.affix("", item_str, &spaced_suffix));
                        } else {
                            item_parts.push(item_str);
                        }
                    }
                }
            }

            let prefix = first_item.prefix.as_deref().unwrap_or("");
            if !author_part.is_empty() && !item_parts.is_empty() {
                let author_item_delimiter = group_delimiter.as_deref().unwrap_or(intra_delimiter);
                let repeated_item_delimiter = if author_item_delimiter.trim().is_empty() {
                    ", "
                } else {
                    author_item_delimiter
                };
                let joined_items = item_parts.join(repeated_item_delimiter);
                // Format based on citation mode:
                // Integral: "Kuhn (1962a, 1962b)" - items in parentheses
                // NonIntegral: "Kuhn, 1962a, 1962b" - no inner parens (outer wrap adds them)
                let content = match mode {
                    citum_schema::citation::CitationMode::Integral => {
                        // Check for visibility overrides
                        if suppress_author {
                            // Should theoretically not happen in narrative mode, but handle gracefully
                            format!("({})", joined_items)
                        } else {
                            // Default narrative: Kuhn (1962)
                            format!("{} ({})", author_part, joined_items)
                        }
                    }
                    citum_schema::citation::CitationMode::NonIntegral => {
                        if suppress_author {
                            // Parenthetical SuppressAuthor: 1962
                            joined_items
                        } else {
                            // Default parenthetical: Kuhn, 1962
                            if self.config.punctuation_in_quote
                                && author_item_delimiter.starts_with(',')
                                && (author_part.ends_with('"') || author_part.ends_with('\u{201D}'))
                            {
                                let is_curly = author_part.ends_with('\u{201D}');
                                let mut fixed_author = author_part.clone();
                                fixed_author.pop();
                                format!(
                                    "{},{}{}{}",
                                    fixed_author,
                                    if is_curly { '\u{201D}' } else { '"' },
                                    &author_item_delimiter[1..],
                                    joined_items
                                )
                            } else {
                                format!("{}{}{}", author_part, author_item_delimiter, joined_items)
                            }
                        }
                    }
                };
                let ids: Vec<String> = group.iter().map(|item| item.id.clone()).collect();

                let formatted_prefix =
                    if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                        format!("{} ", prefix)
                    } else {
                        prefix.to_string()
                    };

                rendered_groups.push(fmt.citation(ids, fmt.affix(&formatted_prefix, content, "")));
            } else if !author_part.is_empty() {
                let ids: Vec<String> = group.iter().map(|item| item.id.clone()).collect();

                let formatted_prefix =
                    if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                        format!("{} ", prefix)
                    } else {
                        prefix.to_string()
                    };

                rendered_groups
                    .push(fmt.citation(ids, fmt.affix(&formatted_prefix, author_part, "")));
            } else if !item_parts.is_empty() {
                // Item-only case (SuppressAuthor)
                let content = item_parts.join(intra_delimiter);
                let ids: Vec<String> = group.iter().map(|item| item.id.clone()).collect();

                let formatted_prefix =
                    if !prefix.is_empty() && !prefix.ends_with(char::is_whitespace) {
                        format!("{} ", prefix)
                    } else {
                        prefix.to_string()
                    };

                rendered_groups.push(fmt.citation(ids, fmt.affix(&formatted_prefix, content, "")));
            }
        }

        Ok(rendered_groups)
    }

    /// Render just the author part for citation grouping.
    pub(crate) fn render_author_for_grouping_with_format<F>(
        &self,
        reference: &Reference,
        item: &crate::reference::CitationItem,
        template: &[TemplateComponent],
        mode: &citum_schema::citation::CitationMode,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let is_note_processing = self.config.processing.as_ref().is_some_and(|processing| {
            matches!(processing, citum_schema::options::Processing::Note)
        });
        if is_note_processing
            && matches!(
                position,
                Some(citum_schema::citation::Position::Ibid)
                    | Some(citum_schema::citation::Position::IbidWithLocator)
            )
        {
            return String::new();
        }

        let options = RenderOptions {
            config: self.config,
            locale: self.locale,
            context: RenderContext::Citation,
            mode: mode.clone(),
            suppress_author,
            locator: None,
            locator_label: None,
        };

        // Try to use the first semantically relevant component (including nested lists)
        // so disambiguation hints and component-specific formatting are preserved.
        // This ensures substitution, shortening, and mode-dependent conjunctions are respected.
        if let Some(comp) = template.first().and_then(find_grouping_component) {
            let base_hints = self
                .hints
                .get(&reference.id().unwrap_or_default())
                .cloned()
                .unwrap_or_default();
            // Inject citation position so subsequent et-al thresholds are applied.
            let hints = ProcHints {
                position: position.cloned(),
                integral_name_state: item.integral_name_state,
                ..base_hints
            };
            if let Some(vals) = comp.values::<F>(reference, &hints, &options)
                && !vals.value.is_empty()
            {
                return vals.value;
            }
        }

        // Fallback for cases where first component isn't suitable or returned empty
        if let Some(authors) = reference.author() {
            let mode = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.name_mode.as_ref());
            let preferred_transliteration = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.preferred_transliteration.as_deref());
            let preferred_script = self
                .config
                .multilingual
                .as_ref()
                .and_then(|m| m.preferred_script.as_ref());
            let locale_str = &self.locale.locale;

            let names_vec = crate::values::resolve_multilingual_name(
                &authors,
                mode,
                preferred_transliteration,
                preferred_script,
                locale_str,
            );
            F::default().text(&crate::values::format_contributors_short(
                &names_vec, &options,
            ))
        } else {
            String::new()
        }
    }

    /// Render the prose anchor for an integral citation without any trailing note text.
    pub(crate) fn render_integral_anchor_with_format<F>(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        inter_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<String, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        use crate::reference::CitationItem;

        let preserve_individual_citations = items.iter().any(|item| {
            self.hints
                .get(&item.id)
                .is_some_and(|hints| hints.min_names_to_show.is_some() || hints.expand_given_names)
        });

        let mut groups: Vec<(String, Vec<&CitationItem>)> = Vec::new();
        for item in items {
            let reference = self.bibliography.get(&item.id);
            let author_key = if preserve_individual_citations {
                item.id.clone()
            } else {
                reference
                    .map(|r| self.get_author_grouping_key(r))
                    .unwrap_or_default()
            };

            match groups.last_mut() {
                Some(group) if !author_key.is_empty() && group.0 == author_key => {
                    group.1.push(item);
                }
                _ => {
                    groups.push((author_key, vec![item]));
                }
            }
        }

        let mut rendered_groups = Vec::new();
        let fmt = F::default();
        for (_author_key, group) in groups {
            let first_item = group[0];
            let reference = self
                .bibliography
                .get(&first_item.id)
                .ok_or_else(|| ProcessorError::ReferenceNotFound(first_item.id.clone()))?;
            let item_language = crate::values::effective_item_language(reference);
            let template = spec.resolve_template_for_language(item_language.as_deref());
            let effective_template = template.as_deref().unwrap_or(&[]);
            let author_part = self.render_author_for_grouping_with_format::<F>(
                reference,
                first_item,
                effective_template,
                &citum_schema::citation::CitationMode::Integral,
                suppress_author,
                position,
            );
            if !author_part.is_empty() {
                rendered_groups.push(author_part);
            }
        }

        Ok(fmt.join(rendered_groups, inter_delimiter))
    }
    #[allow(dead_code)]
    fn render_author_for_grouping(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
        mode: &citum_schema::citation::CitationMode,
        position: Option<&citum_schema::citation::Position>,
    ) -> String {
        let item = crate::reference::CitationItem::default();
        self.render_author_for_grouping_with_format::<crate::render::plain::PlainText>(
            reference, &item, template, mode, false, position,
        )
    }

    /// Get a unique key for grouping citations by author.
    fn get_author_grouping_key(&self, reference: &Reference) -> String {
        if let Some(author) = reference.author() {
            author.to_string().to_lowercase()
        } else if let Some(editor) = reference.editor() {
            editor.to_string().to_lowercase()
        } else if let Some(title) = reference.title() {
            title.to_string().to_lowercase()
        } else {
            String::new()
        }
    }

    /// Filter out author components from a template.
    fn filter_author_from_template(
        &self,
        template: &[TemplateComponent],
    ) -> (Vec<TemplateComponent>, Option<String>) {
        let mut filtered: Vec<TemplateComponent> =
            template.iter().filter_map(strip_author_component).collect();
        let leading_affix = filtered.first().and_then(leading_group_affix);
        if let Some(first) = filtered.first_mut() {
            strip_leading_group_affixes(first);
        }
        (filtered, leading_affix)
    }

    /// Render just the year part (with suffix) for citation grouping.
    fn render_year_for_grouping_with_format<F>(&self, reference: &Reference) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let hints = self
            .hints
            .get(&reference.id().unwrap_or_default())
            .cloned()
            .unwrap_or_default();

        // Format year with disambiguation suffix
        if let Some(issued) = reference.issued() {
            let year = issued.year();
            let suffix = if hints.disamb_condition && hints.group_index > 0 {
                // Check if year suffix is enabled. Fall back to AuthorDate default
                // (year_suffix: true) when processing is not explicitly set, matching
                // the behavior in disambiguation.rs which uses unwrap_or_default().
                let use_suffix = self
                    .config
                    .processing
                    .as_ref()
                    .unwrap_or(&citum_schema::options::Processing::AuthorDate)
                    .config()
                    .disambiguate
                    .as_ref()
                    .map(|d| d.year_suffix)
                    .unwrap_or(false);

                if use_suffix {
                    crate::values::int_to_letter(hints.group_index as u32).unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            return fmt.text(&format!("{}{}", year, suffix));
        }
        String::new()
    }

    #[allow(dead_code)]
    fn render_year_for_grouping(&self, reference: &Reference) -> String {
        self.render_year_for_grouping_with_format::<crate::render::plain::PlainText>(reference)
    }

    /// Get the citation number for a reference, assigning one if not yet cited.
    fn get_or_assign_citation_number(&self, ref_id: &str) -> usize {
        let mut numbers = self.citation_numbers.borrow_mut();
        let next_num = numbers.len() + 1;
        *numbers.entry(ref_id.to_string()).or_insert(next_num)
    }

    /// Process a bibliography entry.
    pub fn process_bibliography_entry(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate> {
        self.process_bibliography_entry_with_format::<crate::render::plain::PlainText>(
            reference,
            entry_number,
        )
    }

    /// Process a bibliography entry with specific format.
    pub fn process_bibliography_entry_with_format<F>(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let bib_spec = self.style.bibliography.as_ref()?;

        // Resolve default template (handles preset vs explicit)
        let item_language = crate::values::effective_item_language(reference);
        let default_template = bib_spec.resolve_template_for_language(item_language.as_deref())?;

        // Determine effective template (override or default)
        let ref_type = reference.ref_type();
        let template = if let Some(type_templates) = &bib_spec.type_templates {
            let mut matched_template = None;
            for (selector, t) in type_templates {
                if selector.matches(&ref_type) {
                    matched_template = Some(t.clone());
                    break;
                }
            }
            matched_template.unwrap_or(default_template)
        } else {
            default_template
        };

        let template_ref = &template;

        let options = RenderOptions {
            config: self.config,
            locale: self.locale,
            context: RenderContext::Bibliography,
            mode: citum_schema::citation::CitationMode::NonIntegral,
            suppress_author: false,
            locator: None,
            locator_label: None,
        };

        self.process_template_with_number_internal_with_format::<F>(
            reference,
            template_ref,
            options,
            entry_number,
            None,
            None,
        )
    }

    /// Process a template for a reference with citation number.
    #[allow(clippy::too_many_arguments)]
    pub fn process_template_with_number(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
        context: RenderContext,
        mode: citum_schema::citation::CitationMode,
        suppress_author: bool,
        citation_number: usize,
        locator: Option<&str>,
        locator_label: Option<citum_schema::citation::LocatorType>,
        position: Option<&citum_schema::citation::Position>,
        integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    ) -> Option<ProcTemplate> {
        self.process_template_with_number_with_format::<crate::render::plain::PlainText>(
            reference,
            template,
            context,
            mode,
            suppress_author,
            citation_number,
            locator,
            locator_label,
            position,
            integral_name_state,
        )
    }

    /// Process a template for a reference with citation number and specific format.
    #[allow(clippy::too_many_arguments)]
    pub fn process_template_with_number_with_format<F>(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
        context: RenderContext,
        mode: citum_schema::citation::CitationMode,
        suppress_author: bool,
        citation_number: usize,
        locator: Option<&str>,
        locator_label: Option<citum_schema::citation::LocatorType>,
        position: Option<&citum_schema::citation::Position>,
        integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let options = RenderOptions {
            config: self.config,
            locale: self.locale,
            context,
            mode,
            suppress_author,
            locator,
            locator_label,
        };
        self.process_template_with_number_internal_with_format::<F>(
            reference,
            template,
            options,
            citation_number,
            position,
            integral_name_state,
        )
    }

    #[allow(dead_code)]
    fn process_template_with_number_internal(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
        options: RenderOptions<'_>,
        citation_number: usize,
        position: Option<&citum_schema::citation::Position>,
        integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    ) -> Option<ProcTemplate> {
        self.process_template_with_number_internal_with_format::<crate::render::plain::PlainText>(
            reference,
            template,
            options,
            citation_number,
            position,
            integral_name_state,
        )
    }

    fn process_template_with_number_internal_with_format<F>(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
        options: RenderOptions<'_>,
        citation_number: usize,
        position: Option<&citum_schema::citation::Position>,
        integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let default_hint = ProcHints::default();
        let base_hint = self
            .hints
            .get(&reference.id().unwrap_or_default())
            .unwrap_or(&default_hint);

        // Create a hint with citation number and position
        let hint = ProcHints {
            citation_number: if citation_number > 0 {
                Some(citation_number)
            } else {
                None
            },
            citation_sub_label: if options.context == RenderContext::Citation {
                reference
                    .id()
                    .as_deref()
                    .and_then(|id| self.citation_sub_label_for_ref(id))
            } else {
                None
            },
            position: position.cloned(),
            integral_name_state,
            ..base_hint.clone()
        };

        // Track rendered variables to prevent duplicates (CSL 1.0 spec:
        // "Substituted variables are suppressed in the rest of the output")
        let mut rendered_vars: HashSet<String> = HashSet::new();
        // Track base keys of substituted variables so they suppress all contextual
        // variants (for example "title:Primary" should suppress title with suffixes).
        let mut substituted_bases: HashSet<String> = HashSet::new();

        let key_base = |key: &str| -> String {
            let mut parts = key.splitn(3, ':');
            match (parts.next(), parts.next()) {
                (Some(kind), Some(var)) => format!("{kind}:{var}"),
                _ => key.to_string(),
            }
        };

        let components: Vec<ProcTemplateComponent> = template
            .iter()
            .filter_map(|component| {
                let ref_type = reference.ref_type().to_string();
                let resolved_component = resolve_component_for_ref_type(component, &ref_type);
                // Get unique key for this variable (e.g., "contributor:Author")
                let var_key = get_variable_key(&resolved_component);

                // Skip if this variable was already rendered
                if let Some(ref key) = var_key {
                    let base = key_base(key);
                    if rendered_vars.contains(key) || substituted_bases.contains(&base) {
                        return None;
                    }
                }

                // Extract value from reference using the requested format
                let mut values = resolved_component.values::<F>(reference, &hint, &options)?;
                if values.value.is_empty() {
                    return None;
                }
                if matches!(
                    resolved_component,
                    TemplateComponent::Date(citum_schema::template::TemplateDate {
                        date: citum_schema::template::DateVariable::Issued,
                        ..
                    })
                ) && reference.issued().is_none_or(|issued| issued.0.is_empty())
                    && self.preferred_no_date_term_form() == citum_schema::locale::TermForm::Long
                    && let Some(long) = options.locale.general_term(
                        &citum_schema::locale::GeneralTerm::NoDate,
                        citum_schema::locale::TermForm::Long,
                    )
                {
                    values.value = long.to_string();
                }
                let item_language =
                    crate::values::effective_component_language(reference, &resolved_component);

                // If whole-entry linking is enabled and this component doesn't have a URL,
                // try to resolve it from global config.
                if values.url.is_none()
                    && let Some(links) = &options.config.links
                {
                    use citum_schema::options::LinkAnchor;
                    if matches!(links.anchor, Some(LinkAnchor::Entry)) {
                        values.url = crate::values::resolve_url(links, reference);
                    }
                }

                // Mark variable as rendered for deduplication
                if let Some(key) = var_key {
                    rendered_vars.insert(key);
                }
                // Also mark substituted variable (e.g., title when it replaces author)
                if let Some(sub_key) = &values.substituted_key {
                    rendered_vars.insert(sub_key.clone());
                    substituted_bases.insert(key_base(sub_key));
                }

                Some(ProcTemplateComponent {
                    template_component: resolved_component,
                    value: values.value,
                    prefix: values.prefix,
                    suffix: values.suffix,
                    url: values.url,
                    ref_type: Some(ref_type),
                    config: Some(options.config.clone()),
                    item_language,
                    pre_formatted: values.pre_formatted,
                })
            })
            .collect();

        if components.is_empty() {
            None
        } else {
            Some(components)
        }
    }

    /// Apply the substitution string to the primary contributor component.
    pub fn apply_author_substitution(&self, proc: &mut ProcTemplate, substitute: &str) {
        self.apply_author_substitution_with_format::<crate::render::plain::PlainText>(
            proc, substitute,
        );
    }

    /// Apply the substitution string to the primary contributor component with specific format.
    pub fn apply_author_substitution_with_format<F>(
        &self,
        proc: &mut ProcTemplate,
        substitute: &str,
    ) where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        if let Some(component) = proc
            .iter_mut()
            .find(|c| matches!(c.template_component, TemplateComponent::Contributor(_)))
        {
            let fmt = F::default();
            component.value = fmt.text(substitute);
        }
    }

    fn preferred_no_date_term_form(&self) -> citum_schema::locale::TermForm {
        match self
            .style
            .info
            .source
            .as_ref()
            .map(|source| source.csl_id.as_str())
        {
            Some("http://www.zotero.org/styles/harvard-cite-them-right") => {
                citum_schema::locale::TermForm::Long
            }
            _ => citum_schema::locale::TermForm::Short,
        }
    }
}

/// Recursively removes author components from a template.
///
/// Filters out top-level author contributors and descends into lists,
/// returning `None` if the entire template becomes empty after filtering.
fn strip_author_component(component: &TemplateComponent) -> Option<TemplateComponent> {
    match component {
        TemplateComponent::Contributor(c)
            if c.contributor == citum_schema::template::ContributorRole::Author =>
        {
            None
        }
        TemplateComponent::List(list) => {
            let filtered_items: Vec<TemplateComponent> = list
                .items
                .iter()
                .filter_map(strip_author_component)
                .collect();

            if filtered_items.is_empty() {
                None
            } else {
                let mut filtered_list = list.clone();
                filtered_list.items = filtered_items;
                Some(TemplateComponent::List(filtered_list))
            }
        }
        _ => Some(component.clone()),
    }
}

/// Extract the leading affix used to separate grouped authors from item details.
fn leading_group_affix(component: &TemplateComponent) -> Option<String> {
    let own_affix = match component {
        TemplateComponent::Contributor(inner) => inner
            .rendering
            .prefix
            .clone()
            .or(inner.rendering.inner_prefix.clone()),
        TemplateComponent::Date(inner) => inner
            .rendering
            .prefix
            .clone()
            .or(inner.rendering.inner_prefix.clone()),
        TemplateComponent::Title(inner) => inner
            .rendering
            .prefix
            .clone()
            .or(inner.rendering.inner_prefix.clone()),
        TemplateComponent::Number(inner) => inner
            .rendering
            .prefix
            .clone()
            .or(inner.rendering.inner_prefix.clone()),
        TemplateComponent::Variable(inner) => inner
            .rendering
            .prefix
            .clone()
            .or(inner.rendering.inner_prefix.clone()),
        TemplateComponent::Term(inner) => inner
            .rendering
            .prefix
            .clone()
            .or(inner.rendering.inner_prefix.clone()),
        TemplateComponent::List(inner) => inner
            .rendering
            .prefix
            .clone()
            .or(inner.rendering.inner_prefix.clone())
            .or_else(|| inner.items.first().and_then(leading_group_affix)),
        _ => None,
    };

    own_affix.filter(|value| !value.is_empty())
}

/// Remove leading affixes from the first surviving grouped-citation component.
///
/// When the author component is stripped from an author-date template, the next
/// component often carries a prefix like `", "` that only makes sense when the
/// author is still present. Grouped citation assembly adds the author/date
/// delimiter separately, so the first surviving component must start "clean".
fn strip_leading_group_affixes(component: &mut TemplateComponent) {
    match component {
        TemplateComponent::Contributor(inner) => {
            inner.rendering.prefix = None;
            inner.rendering.inner_prefix = None;
        }
        TemplateComponent::Date(inner) => {
            inner.rendering.prefix = None;
            inner.rendering.inner_prefix = None;
        }
        TemplateComponent::Title(inner) => {
            inner.rendering.prefix = None;
            inner.rendering.inner_prefix = None;
        }
        TemplateComponent::Number(inner) => {
            inner.rendering.prefix = None;
            inner.rendering.inner_prefix = None;
        }
        TemplateComponent::Variable(inner) => {
            inner.rendering.prefix = None;
            inner.rendering.inner_prefix = None;
        }
        TemplateComponent::Term(inner) => {
            inner.rendering.prefix = None;
            inner.rendering.inner_prefix = None;
        }
        TemplateComponent::List(inner) => {
            inner.rendering.prefix = None;
            inner.rendering.inner_prefix = None;
            if let Some(first) = inner.items.first_mut() {
                strip_leading_group_affixes(first);
            }
        }
        _ => {}
    }
}

/// Finds a grouping component (contributor or title) within a template.
///
/// Descends into lists to find the first semantically relevant component
/// for grouping citations by author or title.
fn find_grouping_component(component: &TemplateComponent) -> Option<&TemplateComponent> {
    match component {
        TemplateComponent::Contributor(_) | TemplateComponent::Title(_) => Some(component),
        TemplateComponent::List(list) => list.items.iter().find_map(find_grouping_component),
        _ => None,
    }
}

/// Get a unique key for a template component's variable.
///
/// The key includes rendering context (prefix/suffix) to allow the same variable
/// to render multiple times if it appears in semantically different contexts.
/// This enables styles like Chicago that require year after author AND after publisher.
pub fn get_variable_key(component: &TemplateComponent) -> Option<String> {
    use citum_schema::template::*;

    // Helper to create context suffix from rendering options
    let context_suffix = |rendering: &Rendering| -> String {
        match (&rendering.prefix, &rendering.suffix) {
            (Some(p), Some(s)) => format!(":{}_{}", p, s),
            (Some(p), None) => format!(":{}", p),
            (None, Some(s)) => format!(":{}", s),
            (None, None) => String::new(),
        }
    };

    match component {
        TemplateComponent::Contributor(c) => {
            let ctx = context_suffix(&c.rendering);
            Some(format!("contributor:{:?}{}", c.contributor, ctx))
        }
        TemplateComponent::Date(d) => {
            let ctx = context_suffix(&d.rendering);
            Some(format!("date:{:?}{}", d.date, ctx))
        }
        TemplateComponent::Variable(v) => {
            let ctx = context_suffix(&v.rendering);
            Some(format!("variable:{:?}{}", v.variable, ctx))
        }
        TemplateComponent::Title(t) => {
            let ctx = context_suffix(&t.rendering);
            Some(format!("title:{:?}{}", t.title, ctx))
        }
        TemplateComponent::Number(n) => {
            let ctx = context_suffix(&n.rendering);
            Some(format!("number:{:?}{}", n.number, ctx))
        }
        TemplateComponent::List(_) => None, // Lists contain multiple variables, not deduplicated
        _ => None,                          // Future component types
    }
}

/// Resolves a template component by applying type-specific overrides.
///
/// Checks the component's overrides for the given reference type and returns
/// either the override (if matched) or the original component.
fn resolve_component_for_ref_type(
    component: &TemplateComponent,
    ref_type: &str,
) -> TemplateComponent {
    let Some(overrides) = component.overrides() else {
        return component.clone();
    };

    let mut replacement: Option<TemplateComponent> = None;
    let mut matched = false;

    for (selector, ov) in overrides {
        if selector.matches(ref_type) {
            matched = true;
            if let ComponentOverride::Component(c) = ov {
                replacement = Some((**c).clone());
            }
        }
    }

    if !matched {
        for (selector, ov) in overrides {
            if selector.matches("default")
                && let ComponentOverride::Component(c) = ov
            {
                replacement = Some((**c).clone());
            }
        }
    }

    replacement.unwrap_or_else(|| component.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::citation::{CitationLocator, LocatorValue};
    use citum_schema::template::*;

    #[test]
    fn test_variable_key_includes_context() {
        // Date with no prefix/suffix
        let date1 = TemplateComponent::Date(TemplateDate {
            date: DateVariable::Issued,
            form: DateForm::Year,
            rendering: Rendering::default(),
            fallback: None,
            links: None,
            overrides: None,
            custom: None,
        });

        // Same date with prefix
        let date2 = TemplateComponent::Date(TemplateDate {
            date: DateVariable::Issued,
            form: DateForm::Year,
            rendering: Rendering {
                prefix: Some(", ".to_string()),
                ..Default::default()
            },
            fallback: None,
            links: None,
            overrides: None,
            custom: None,
        });

        // Same date with suffix
        let date3 = TemplateComponent::Date(TemplateDate {
            date: DateVariable::Issued,
            form: DateForm::Year,
            rendering: Rendering {
                suffix: Some(".".to_string()),
                ..Default::default()
            },
            fallback: None,
            links: None,
            overrides: None,
            custom: None,
        });

        let key1 = get_variable_key(&date1);
        let key2 = get_variable_key(&date2);
        let key3 = get_variable_key(&date3);

        // All three should have different keys due to different contexts
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);

        // Verify the keys include context markers
        assert_eq!(key1, Some("date:Issued".to_string()));
        assert_eq!(key2, Some("date:Issued:, ".to_string()));
        assert_eq!(key3, Some("date:Issued:.".to_string()));
    }

    #[test]
    fn test_strip_author_component_nested_list() {
        let nested = TemplateComponent::List(TemplateList {
            items: vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    and: None,
                    shorten: None,
                    label: None,
                    name_order: None,
                    delimiter: None,
                    sort_separator: None,
                    links: None,
                    rendering: Rendering::default(),
                    overrides: None,
                    custom: None,
                }),
                TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    fallback: None,
                    links: None,
                    overrides: None,
                    custom: None,
                }),
            ],
            delimiter: Some(DelimiterPunctuation::Space),
            rendering: Rendering::default(),
            overrides: None,
            custom: None,
        });

        let filtered = strip_author_component(&nested).expect("list should remain");
        let TemplateComponent::List(filtered_list) = filtered else {
            panic!("expected list");
        };

        assert_eq!(filtered_list.items.len(), 1);
        assert!(matches!(filtered_list.items[0], TemplateComponent::Date(_)));
    }

    #[test]
    fn compound_locator_joins_segments_with_separator() {
        let locale = Locale::default();
        let segments = vec![
            LocatorSegment {
                label: LocatorType::Chapter,
                value: LocatorValue::from("3"),
            },
            LocatorSegment {
                label: LocatorType::Section,
                value: LocatorValue::from("42"),
            },
        ];
        let rendered = collapse_compound_locator(&segments, &locale);
        assert!(rendered.contains("3"), "should contain first value");
        assert!(rendered.contains("42"), "should contain second value");
        assert!(rendered.contains(", "), "should join with comma-space");
    }

    #[test]
    fn compound_locator_plural_detection() {
        let locale = Locale::default();
        // Range with en-dash should trigger plural
        let segments = vec![LocatorSegment {
            label: LocatorType::Page,
            value: LocatorValue::from("10\u{2013}12"),
        }];
        let rendered = collapse_compound_locator(&segments, &locale);
        assert!(rendered.contains("10\u{2013}12"));
    }

    #[test]
    fn compound_locator_fallback_uses_kebab_case() {
        let locale = Locale::default();
        let segments = vec![LocatorSegment {
            label: LocatorType::SubVerbo,
            value: LocatorValue::from("test"),
        }];
        let rendered = collapse_compound_locator(&segments, &locale);
        // Should use kebab-case "sub-verbo", not PascalCase "SubVerbo"
        assert!(
            rendered.contains("sub-verbo"),
            "expected kebab-case fallback, got: {rendered}"
        );
    }

    #[test]
    fn resolve_item_locator_prefers_compound() {
        let locale = Locale::default();
        let item = citum_schema::citation::CitationItem {
            id: "test".to_string(),
            locator: Some(
                CitationLocator::compound(vec![
                    LocatorSegment::new(LocatorType::Chapter, "5"),
                    LocatorSegment::new(LocatorType::Section, "42"),
                ])
                .unwrap(),
            ),
            ..Default::default()
        };
        let (value, label) = resolve_item_locator(&item, &locale);
        assert!(value.unwrap().contains("5"));
        assert!(
            label.is_none(),
            "compound locators embed labels per-segment"
        );
    }
}
