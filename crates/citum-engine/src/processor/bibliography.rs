/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Bibliography processing and rendering.
//!
//! This module owns bibliography entry generation, grouped rendering,
//! subsequent-author substitution, and the document-facing facade methods used
//! by the document processor.

use super::disambiguation::Disambiguator;
use super::matching::Matcher;
use super::rendering::{CompoundRenderData, Renderer};
use super::{ProcessedReferences, Processor};
use crate::grouping::{GroupSorter, SelectorEvaluator};
use crate::reference::{Bibliography, Reference};
use crate::render::bibliography::render_entry_body_with_format;
use crate::render::component::ProcTemplateComponent;
use crate::render::format::{OutputFormat, ProcEntryMetadata};
use crate::render::{ProcEntry, ProcTemplate};
use crate::values::{ProcHints, RenderContext, RenderOptions, format_contributors_short};
use citum_schema::grouping::{BibliographyGroup, DisambiguationScope, GroupHeading};
use citum_schema::template::{NumberVariable, TemplateComponent};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Rendered bibliography block data for document integration.
#[derive(Debug, Clone, Default)]
pub(crate) struct RenderedBibliographyGroup {
    /// The resolved group heading, if one exists.
    pub(crate) heading: Option<String>,
    /// The rendered bibliography body without any document-level heading wrapper.
    pub(crate) body: String,
}

impl Processor {
    fn compound_numeric_config(
        &self,
    ) -> Option<citum_schema::options::bibliography::CompoundNumericConfig> {
        self.get_config()
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.compound_numeric.as_ref())
            .cloned()
    }

    fn is_citation_number_label(component: &ProcTemplateComponent) -> bool {
        matches!(
            &component.template_component,
            TemplateComponent::Number(number) if number.number == NumberVariable::CitationNumber
        )
    }

    fn build_compound_group_lookup(
        compound_groups: &IndexMap<usize, Vec<String>>,
    ) -> HashMap<String, usize> {
        let mut ref_to_group = HashMap::new();
        for (&group_number, ids) in compound_groups {
            if ids.len() > 1 {
                for id in ids {
                    ref_to_group.insert(id.clone(), group_number);
                }
            }
        }
        ref_to_group
    }

    fn render_compound_entry_bodies<F>(entries: &[ProcEntry]) -> HashMap<String, String>
    where
        F: OutputFormat<Output = String>,
    {
        entries
            .iter()
            .map(|entry| {
                let content_entry = ProcEntry {
                    id: entry.id.clone(),
                    template: entry
                        .template
                        .iter()
                        .filter(|component| !Self::is_citation_number_label(component))
                        .cloned()
                        .collect(),
                    metadata: entry.metadata.clone(),
                };
                (
                    entry.id.clone(),
                    render_entry_body_with_format::<F>(&content_entry)
                        .trim()
                        .to_string(),
                )
            })
            .collect()
    }

    fn build_present_group_members(
        entries: &[ProcEntry],
        ref_to_group: &HashMap<String, usize>,
    ) -> HashMap<usize, Vec<String>> {
        let mut group_members_present = HashMap::new();
        for entry in entries {
            if let Some(&group_number) = ref_to_group.get(&entry.id) {
                group_members_present
                    .entry(group_number)
                    .or_insert_with(Vec::new)
                    .push(entry.id.clone());
            }
        }
        group_members_present
    }

    fn build_merged_compound_entry(
        &self,
        entry: ProcEntry,
        group_ids: &[String],
        entries_by_id: &HashMap<String, ProcEntry>,
        rendered_strings: &HashMap<String, String>,
        compound_config: &citum_schema::options::bibliography::CompoundNumericConfig,
    ) -> ProcEntry {
        let mut parts = Vec::new();

        for (index, id) in group_ids.iter().enumerate() {
            if !entries_by_id.contains_key(id) {
                continue;
            }

            let sub_label = match compound_config.sub_label {
                citum_schema::options::bibliography::SubLabelStyle::Alphabetic => {
                    format!(
                        "{}{}",
                        crate::values::int_to_letter((index + 1) as u32)
                            .unwrap_or_else(|| "a".to_string()),
                        compound_config.sub_label_suffix
                    )
                }
                citum_schema::options::bibliography::SubLabelStyle::Numeric => {
                    format!("{}{}", index + 1, compound_config.sub_label_suffix)
                }
            };

            if let Some(rendered) = rendered_strings.get(id) {
                parts.push(format!("{} {}", sub_label, rendered));
            }
        }

        let mut merged_template: Vec<_> = entry
            .template
            .iter()
            .filter(|component| Self::is_citation_number_label(component))
            .cloned()
            .collect();
        merged_template.push(ProcTemplateComponent {
            template_component: TemplateComponent::default(),
            value: parts.join(&compound_config.sub_delimiter),
            pre_formatted: true,
            config: entry
                .template
                .first()
                .and_then(|component| component.config.clone()),
            ..Default::default()
        });

        ProcEntry {
            id: entry.id,
            template: merged_template,
            metadata: entry.metadata,
        }
    }

    fn merge_compound_entries<F>(&self, entries: Vec<ProcEntry>) -> Vec<ProcEntry>
    where
        F: OutputFormat<Output = String>,
    {
        let compound_groups = self.compound_groups.borrow();
        if compound_groups.is_empty() {
            return entries;
        }

        let Some(compound_config) = self.compound_numeric_config() else {
            return entries;
        };

        let ref_to_group = Self::build_compound_group_lookup(&compound_groups);
        if ref_to_group.is_empty() {
            return entries;
        }

        let rendered_strings = Self::render_compound_entry_bodies::<F>(&entries);
        let entries_by_id: HashMap<String, ProcEntry> = entries
            .iter()
            .map(|entry| (entry.id.clone(), entry.clone()))
            .collect();
        let group_members_present = Self::build_present_group_members(&entries, &ref_to_group);
        let first_present_by_group: HashMap<usize, String> = group_members_present
            .iter()
            .filter_map(|(&group_number, ids)| {
                ids.first()
                    .cloned()
                    .map(|first_id| (group_number, first_id))
            })
            .collect();

        let mut result = Vec::new();
        for entry in entries {
            if let Some(&group_number) = ref_to_group.get(&entry.id) {
                let Some(present_ids) = group_members_present.get(&group_number) else {
                    result.push(entry);
                    continue;
                };

                if present_ids.len() == 1 {
                    result.push(entry);
                    continue;
                }

                if first_present_by_group.get(&group_number) == Some(&entry.id) {
                    result.push(self.build_merged_compound_entry(
                        entry,
                        &compound_groups[&group_number],
                        &entries_by_id,
                        &rendered_strings,
                        &compound_config,
                    ));
                }
            } else {
                result.push(entry);
            }
        }

        result
    }

    fn extract_metadata(&self, reference: &Reference) -> ProcEntryMetadata {
        let options = RenderOptions {
            config: self.get_config(),
            locale: &self.locale,
            context: RenderContext::Bibliography,
            mode: citum_schema::citation::CitationMode::NonIntegral,
            suppress_author: false,
            locator: None,
            locator_label: None,
        };

        ProcEntryMetadata {
            author: reference
                .author()
                .map(|authors| format_contributors_short(&authors.to_names_vec(), &options)),
            year: reference.issued().map(|issued| issued.year().to_string()),
            title: reference.title().map(|title| title.to_string()),
        }
    }

    fn resolve_group_heading(&self, heading: &GroupHeading) -> Option<String> {
        match heading {
            GroupHeading::Literal { literal } => Some(literal.clone()),
            GroupHeading::Term { term, form } => self
                .locale
                .general_term(term, form.unwrap_or(citum_schema::locale::TermForm::Long))
                .map(ToOwned::to_owned),
            GroupHeading::Localized { localized } => self.resolve_localized_heading(localized),
        }
    }

    fn resolve_localized_heading(&self, localized: &HashMap<String, String>) -> Option<String> {
        fn language_tag(locale: &str) -> &str {
            locale.split('-').next().unwrap_or(locale)
        }

        let mut candidates = Vec::new();
        let mut push_candidate = |locale: &str| {
            let candidate = locale.to_string();
            if !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        };

        push_candidate(&self.locale.locale);
        push_candidate(language_tag(&self.locale.locale));

        if let Some(default_locale) = self.style.info.default_locale.as_deref() {
            push_candidate(default_locale);
            push_candidate(language_tag(default_locale));
        }

        push_candidate("en-US");
        push_candidate("en");

        for locale in candidates {
            if let Some(value) = localized.get(&locale) {
                return Some(value.clone());
            }
        }

        localized
            .iter()
            .min_by(|left, right| left.0.cmp(right.0))
            .map(|(_locale, value)| value.clone())
    }

    fn collect_matching_group_refs<'a>(
        &'a self,
        bibliography: &'a [ProcEntry],
        assigned: &HashSet<String>,
        evaluator: &SelectorEvaluator<'_>,
        group: &BibliographyGroup,
    ) -> Vec<&'a Reference> {
        bibliography
            .iter()
            .filter(|entry| !assigned.contains(&entry.id))
            .filter_map(|entry| {
                self.bibliography
                    .get(&entry.id)
                    .filter(|reference| evaluator.matches(reference, &group.selector))
            })
            .collect()
    }

    fn mark_group_members_assigned(assigned: &mut HashSet<String>, references: &[&Reference]) {
        for reference in references {
            if let Some(id) = reference.id() {
                assigned.insert(id);
            }
        }
    }

    fn build_group_local_hints(
        &self,
        sorted_refs: &[&Reference],
        group: &BibliographyGroup,
    ) -> Option<HashMap<String, ProcHints>> {
        if !matches!(group.disambiguate, Some(DisambiguationScope::Locally)) {
            return None;
        }

        let mut group_bibliography = Bibliography::new();
        for reference in sorted_refs {
            group_bibliography.insert(reference.id().unwrap_or_default(), (*reference).clone());
        }

        let resolved_sort = group.sort.as_ref().map(|sort| sort.resolve());
        let disambiguator = if let Some(sort) = resolved_sort.as_ref() {
            Disambiguator::with_group_sort(
                &group_bibliography,
                self.get_config(),
                &self.locale,
                sort,
            )
        } else {
            Disambiguator::new(&group_bibliography, self.get_config(), &self.locale)
        };

        Some(disambiguator.calculate_hints())
    }

    fn effective_group_style<'a>(
        &'a self,
        group: &'a BibliographyGroup,
    ) -> Cow<'a, citum_schema::Style> {
        if let Some(group_template) = &group.template {
            let mut local_style = self.style.clone();
            if let Some(bibliography) = local_style.bibliography.as_mut() {
                bibliography.template = Some(group_template.clone());
            }
            Cow::Owned(local_style)
        } else {
            Cow::Borrowed(&self.style)
        }
    }

    fn render_group_entries(
        &self,
        bibliography: &[ProcEntry],
        sorted_refs: Vec<&Reference>,
        group: &BibliographyGroup,
        local_hints: Option<&HashMap<String, ProcHints>>,
    ) -> Vec<ProcEntry> {
        if local_hints.is_none() && group.template.is_none() {
            return sorted_refs
                .into_iter()
                .filter_map(|reference| {
                    let id = reference.id()?;
                    bibliography.iter().find(|entry| entry.id == id).cloned()
                })
                .collect();
        }

        let hints = local_hints.unwrap_or(&self.hints);
        let bibliography_config = self.get_bibliography_config();
        let effective_style = self.effective_group_style(group);
        let renderer = Renderer::new(
            &effective_style,
            &self.bibliography,
            &self.locale,
            &bibliography_config,
            hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );

        sorted_refs
            .into_iter()
            .enumerate()
            .map(|(index, reference)| ProcEntry {
                id: reference.id().unwrap_or_default(),
                template: renderer
                    .process_bibliography_entry(reference, index + 1)
                    .unwrap_or_default(),
                metadata: self.extract_metadata(reference),
            })
            .collect()
    }

    fn append_rendered_group<F>(
        &self,
        result: &mut String,
        group: &BibliographyGroup,
        entries: Vec<ProcEntry>,
    ) where
        F: OutputFormat<Output = String>,
    {
        if !result.is_empty() {
            result.push_str("\n\n");
        }

        if let Some(heading) = group
            .heading
            .as_ref()
            .and_then(|group_heading| self.resolve_group_heading(group_heading))
        {
            result.push_str(&format!("# {}\n\n", heading));
        }

        result.push_str(&crate::render::refs_to_string_with_format::<F>(
            entries, None, None,
        ));
    }

    fn append_unassigned_entries<F>(
        &self,
        result: &mut String,
        bibliography: &[ProcEntry],
        assigned: &HashSet<String>,
    ) where
        F: OutputFormat<Output = String>,
    {
        let unassigned: Vec<ProcEntry> = bibliography
            .iter()
            .filter(|entry| !assigned.contains(&entry.id))
            .cloned()
            .collect();

        if unassigned.is_empty() {
            return;
        }

        if !result.is_empty() {
            result.push_str("\n\n");
        }

        result.push_str(&crate::render::refs_to_string_with_format::<F>(
            unassigned, None, None,
        ));
    }

    fn render_with_custom_groups<F>(
        &self,
        bibliography: &[ProcEntry],
        groups: &[BibliographyGroup],
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let cited_ids = self.cited_ids.borrow();
        let evaluator = SelectorEvaluator::new(&cited_ids);
        let sorter = GroupSorter::new(&self.locale);

        let mut assigned = HashSet::new();
        let mut result = String::new();

        for group in groups {
            let matching_refs =
                self.collect_matching_group_refs(bibliography, &assigned, &evaluator, group);
            if matching_refs.is_empty() {
                continue;
            }

            Self::mark_group_members_assigned(&mut assigned, &matching_refs);

            let sorted_refs = if let Some(sort_spec) = &group.sort {
                sorter.sort_references(matching_refs, &sort_spec.resolve())
            } else {
                matching_refs
            };
            let local_hints = self.build_group_local_hints(&sorted_refs, group);
            let entries =
                self.render_group_entries(bibliography, sorted_refs, group, local_hints.as_ref());

            self.append_rendered_group::<F>(&mut result, group, entries);
        }

        self.append_unassigned_entries::<F>(&mut result, bibliography, &assigned);
        fmt.finish(result)
    }

    fn render_with_legacy_grouping<F>(&self, bibliography: &[ProcEntry]) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let cited_ids = self.cited_ids.borrow();
        let cited_entries: Vec<ProcEntry> = bibliography
            .iter()
            .filter(|entry| cited_ids.contains(&entry.id))
            .cloned()
            .collect();

        let mut result = String::new();
        if !cited_entries.is_empty() {
            result.push_str(&crate::render::refs_to_string_with_format::<F>(
                cited_entries,
                None,
                None,
            ));
        }

        fmt.finish(result)
    }

    fn render_bibliography_for_group<F>(&self, group: &BibliographyGroup) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let processed = self.process_references();
        let merged_bibliography = self.merge_compound_entries::<F>(processed.bibliography);
        let fmt = F::default();
        let cited_ids = self.cited_ids.borrow();
        let evaluator = SelectorEvaluator::new(&cited_ids);
        let sorter = GroupSorter::new(&self.locale);

        let matching_refs: Vec<&Reference> = merged_bibliography
            .iter()
            .filter_map(|entry| {
                self.bibliography
                    .get(&entry.id)
                    .filter(|reference| evaluator.matches(reference, &group.selector))
            })
            .collect();

        if matching_refs.is_empty() {
            return fmt.finish(String::new());
        }

        let sorted_refs = if let Some(sort_spec) = &group.sort {
            sorter.sort_references(matching_refs, &sort_spec.resolve())
        } else {
            matching_refs
        };

        let entries: Vec<ProcEntry> = sorted_refs
            .into_iter()
            .filter_map(|reference| {
                let id = reference.id()?;
                merged_bibliography
                    .iter()
                    .find(|entry| entry.id == id)
                    .cloned()
            })
            .collect();

        fmt.finish(crate::render::refs_to_string_with_format::<F>(
            entries, None, None,
        ))
    }

    /// Process all bibliography references and render them.
    ///
    /// Returns sorted and formatted bibliography entries. For numeric styles,
    /// citations must have been processed first to assign citation numbers.
    pub fn process_references(&self) -> ProcessedReferences {
        self.initialize_numeric_citation_numbers();
        let sorted_refs = self.sort_references(self.bibliography.values().collect());
        let mut bibliography = Vec::new();
        let mut previous_reference: Option<&Reference> = None;

        let bibliography_config = self.get_config().bibliography.as_ref();
        let substitute =
            bibliography_config.and_then(|config| config.subsequent_author_substitute.as_ref());

        for (index, reference) in sorted_refs.iter().enumerate() {
            let ref_id = reference.id().unwrap_or_default();
            let entry_number = self
                .citation_numbers
                .borrow()
                .get(&ref_id)
                .copied()
                .unwrap_or(index + 1);

            if let Some(mut processed) = self.process_bibliography_entry(reference, entry_number) {
                if let Some(substitute_string) = substitute
                    && let Some(previous) = previous_reference
                    && self.contributors_match(previous, reference)
                {
                    let bibliography_config = self.get_bibliography_config();
                    let renderer = Renderer::new(
                        &self.style,
                        &self.bibliography,
                        &self.locale,
                        &bibliography_config,
                        &self.hints,
                        &self.citation_numbers,
                        CompoundRenderData {
                            set_by_ref: &self.compound_set_by_ref,
                            member_index: &self.compound_member_index,
                            sets: &self.compound_sets,
                        },
                    );
                    renderer.apply_author_substitution(&mut processed, substitute_string);
                }

                bibliography.push(ProcEntry {
                    id: ref_id,
                    template: processed,
                    metadata: self.extract_metadata(reference),
                });
                previous_reference = Some(reference);
            }
        }

        ProcessedReferences {
            bibliography,
            citations: None,
        }
    }

    /// Process and render a bibliography entry.
    ///
    /// Returns a processed template with metadata if the entry matches the style.
    pub fn process_bibliography_entry(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate> {
        let bibliography_config = self.get_bibliography_config();
        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            &bibliography_config,
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );
        renderer.process_bibliography_entry(reference, entry_number)
    }

    /// Process a bibliography entry with specific format.
    pub fn process_bibliography_entry_with_format<F>(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate>
    where
        F: OutputFormat<Output = String>,
    {
        let bibliography_config = self.get_bibliography_config();
        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            &bibliography_config,
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );
        renderer.process_bibliography_entry_with_format::<F>(reference, entry_number)
    }

    /// Check whether primary contributors match between two references.
    ///
    /// Used for subsequent author substitution in bibliographies.
    pub fn contributors_match(&self, prev: &Reference, current: &Reference) -> bool {
        let matcher = Matcher::new(&self.style, &self.default_config);
        matcher.contributors_match(prev, current)
    }

    /// Replace the primary contributor in a bibliography entry with a substitution string.
    ///
    /// Used for subsequent author substitution (e.g., "———" for repeating authors).
    pub fn apply_author_substitution(&self, proc: &mut ProcTemplate, substitute: &str) {
        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            self.get_config(),
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );
        renderer.apply_author_substitution(proc, substitute);
    }

    /// Render the bibliography to a string using a specific format.
    pub fn render_bibliography_with_format<F>(&self) -> String
    where
        F: OutputFormat<Output = String>,
    {
        self.render_selected_bibliography_with_format::<F, _>(
            self.bibliography.keys().cloned().collect::<Vec<_>>(),
        )
    }

    /// Render a selected bibliography subset to a string using a specific format.
    pub fn render_selected_bibliography_with_format<F, I>(&self, item_ids: I) -> String
    where
        F: OutputFormat<Output = String>,
        I: IntoIterator<Item = String>,
    {
        self.initialize_numeric_citation_numbers();
        let selected: HashSet<String> = item_ids.into_iter().collect();
        let sorted_refs = self.sort_references(self.bibliography.values().collect());
        let mut bibliography = Vec::new();
        let mut previous_reference: Option<&Reference> = None;

        let bibliography_config = self.get_config().bibliography.as_ref();
        let substitute =
            bibliography_config.and_then(|config| config.subsequent_author_substitute.as_ref());

        for (index, reference) in sorted_refs.iter().enumerate() {
            let ref_id = reference.id().unwrap_or_default();
            if !selected.contains(&ref_id) {
                continue;
            }

            let entry_number = self
                .citation_numbers
                .borrow()
                .get(&ref_id)
                .copied()
                .unwrap_or(index + 1);

            if let Some(mut processed) =
                self.process_bibliography_entry_with_format::<F>(reference, entry_number)
            {
                if let Some(substitute_string) = substitute
                    && let Some(previous) = previous_reference
                    && self.contributors_match(previous, reference)
                {
                    let bibliography_config = self.get_bibliography_config();
                    let renderer = Renderer::new(
                        &self.style,
                        &self.bibliography,
                        &self.locale,
                        &bibliography_config,
                        &self.hints,
                        &self.citation_numbers,
                        CompoundRenderData {
                            set_by_ref: &self.compound_set_by_ref,
                            member_index: &self.compound_member_index,
                            sets: &self.compound_sets,
                        },
                    );
                    renderer.apply_author_substitution_with_format::<F>(
                        &mut processed,
                        substitute_string,
                    );
                }

                bibliography.push(ProcEntry {
                    id: ref_id,
                    template: processed,
                    metadata: self.extract_metadata(reference),
                });
                previous_reference = Some(reference);
            }
        }

        let bibliography = self.merge_compound_entries::<F>(bibliography);
        crate::render::refs_to_string_with_format::<F>(bibliography, None, None)
    }

    /// Render the entire bibliography to a formatted string.
    pub fn render_bibliography(&self) -> String {
        self.render_bibliography_with_format::<crate::render::plain::PlainText>()
    }

    /// Render the bibliography with grouping for uncited (nocite) items.
    ///
    /// If `style.bibliography.groups` is defined, uses configurable grouping
    /// with per-group sorting. Otherwise, falls back to hardcoded cited/uncited
    /// grouping for backward compatibility.
    pub fn render_grouped_bibliography_with_format<F>(&self) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let processed = self.process_references();
        let merged_bibliography = self.merge_compound_entries::<F>(processed.bibliography);

        if let Some(groups) = self
            .style
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.groups.as_ref())
        {
            return self.render_with_custom_groups::<F>(&merged_bibliography, groups);
        }

        self.render_with_legacy_grouping::<F>(&merged_bibliography)
    }

    /// Render frontmatter-defined bibliography groups for document output.
    ///
    /// This is the document layer's facade for frontmatter-defined grouped
    /// bibliographies so it does not need to call lower-level grouping helpers
    /// directly.
    pub(crate) fn render_document_bibliography_groups<F>(
        &self,
        groups: &[BibliographyGroup],
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        self.render_with_custom_groups::<F>(&self.process_references().bibliography, groups)
    }

    /// Render one bibliography block for document output, returning heading and body separately.
    ///
    /// The returned body omits any document-level heading wrapper so callers can
    /// choose how to insert headings into their own output format.
    pub(crate) fn render_document_bibliography_block<F>(
        &self,
        group: &BibliographyGroup,
    ) -> RenderedBibliographyGroup
    where
        F: OutputFormat<Output = String>,
    {
        let mut headingless = group.clone();
        let heading = headingless
            .heading
            .take()
            .and_then(|group_heading| self.resolve_group_heading(&group_heading));
        let body = self.render_bibliography_for_group::<F>(&headingless);

        RenderedBibliographyGroup { heading, body }
    }
}
