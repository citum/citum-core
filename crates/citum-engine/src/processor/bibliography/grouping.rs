/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Grouped bibliography rendering with configurable selectors and sorting.

use super::RenderedBibliographyGroup;
use crate::grouping::{GroupSorter, SelectorEvaluator};
use crate::processor::Processor;
use crate::processor::disambiguation::Disambiguator;
use crate::processor::rendering::{CompoundRenderData, Renderer, RendererResources};
use crate::reference::{Bibliography, Reference};
use crate::render::ProcEntry;
use crate::render::format::{OutputFormat, ProcEntryMetadata};
use crate::values::{ProcHints, RenderContext, RenderOptions, format_contributors_short};
use citum_schema::grouping::{BibliographyGroup, DisambiguationScope, GroupHeading};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

impl Processor {
    pub(super) fn resolve_group_heading(&self, heading: &GroupHeading) -> Option<String> {
        match heading {
            GroupHeading::Literal { literal } => Some(literal.clone()),
            GroupHeading::Term { term, form } => self
                .locale
                .resolved_general_term(term, form.unwrap_or(citum_schema::locale::TermForm::Long)),
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

        let resolved_sort = group
            .sort
            .as_ref()
            .map(citum_schema::GroupSortEntry::resolve);
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
        let effective_style = self.effective_group_style(group);
        let bibliography_config = self.get_bibliography_config();
        let renderer = Renderer::new(
            RendererResources {
                style: &effective_style,
                bibliography: &self.bibliography,
                locale: &self.locale,
                config: &bibliography_config,
            },
            hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
            self.show_semantics,
            self.inject_ast_indices,
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
            result.push_str(&format!("# {heading}\n\n"));
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

    pub(super) fn render_with_custom_groups<F>(
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
    pub(crate) fn render_document_bibliography_groups<F>(
        &self,
        groups: &[BibliographyGroup],
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        self.render_with_custom_groups::<F>(&self.process_references().bibliography, groups)
    }

    /// Render one bibliography block for document output.
    ///
    /// Returns heading and body separately so callers can insert headings
    /// in their own output format.
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

    pub(super) fn extract_metadata(&self, reference: &Reference) -> ProcEntryMetadata {
        let options = RenderOptions {
            config: self.get_config(),
            locale: &self.locale,
            context: RenderContext::Bibliography,
            mode: citum_schema::citation::CitationMode::NonIntegral,
            suppress_author: false,
            locator_raw: None,
            ref_type: None,
            show_semantics: self.show_semantics,
            current_template_index: None,
        };

        ProcEntryMetadata {
            author: reference
                .author()
                .map(|authors| format_contributors_short(&authors.to_names_vec(), &options)),
            year: reference.issued().map(|issued| issued.year().clone()),
            title: reference.title().map(|title| title.to_string()),
        }
    }
}
