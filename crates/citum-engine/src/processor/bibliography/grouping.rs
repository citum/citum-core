/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Grouped bibliography rendering with configurable selectors and sorting.

use super::RenderedBibliographyGroup;
use crate::api::AnnotationStyle;
use crate::grouping::SelectorEvaluator;
use crate::processor::FinalizedRun;
use crate::processor::Processor;
use crate::processor::disambiguation::Disambiguator;
use crate::processor::rendering::{CompoundRenderData, Renderer, RendererResources};
use crate::reference::{Bibliography, Reference};
use crate::render::ProcEntry;
use crate::render::format::{OutputFormat, ProcEntryMetadata};
use crate::sorting::ReferenceSorter;
use crate::values::{
    ProcHints, RenderContext, RenderOptions, format_contributors_short, resolve_multilingual_name,
    resolve_multilingual_string,
};
use citum_schema::grouping::{BibliographyGroup, DisambiguationScope, GroupHeading};
use citum_schema::options::{BibliographyPartitionHeading, BibliographySortPartitioning};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

impl Processor {
    /// Resolve a localized or literal group heading.
    pub(super) fn resolve_group_heading(&self, heading: &GroupHeading) -> Option<String> {
        match heading {
            GroupHeading::Literal { literal } => Some(literal.clone()),
            GroupHeading::Term { term, form } => self.locale.resolved_general_term(
                term,
                &form.clone().unwrap_or(citum_schema::locale::TermForm::Long),
                None,
            ),
            GroupHeading::Localized { localized } => self.resolve_localized_heading(localized),
        }
    }

    /// Resolve a localized heading map based on the processor locale.
    ///
    /// Matches in order:
    /// 1. Exact locale (e.g., "en-GB")
    /// 2. Primary language (e.g., "en")
    /// 3. Style default locale
    /// 4. en-US fallback
    /// 5. First alphabetically defined key
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

    /// Resolve a bibliography partition heading.
    fn resolve_partition_heading(&self, heading: &BibliographyPartitionHeading) -> Option<String> {
        match heading {
            BibliographyPartitionHeading::Literal { literal } => Some(literal.clone()),
            BibliographyPartitionHeading::Term { term, form } => self.locale.resolved_general_term(
                term,
                &form.clone().unwrap_or(citum_schema::locale::TermForm::Long),
                None,
            ),
            BibliographyPartitionHeading::Localized { localized } => {
                self.resolve_localized_heading(localized)
            }
        }
    }

    /// Find unassigned bibliography entries that match a group's selector.
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

    /// Returns `ProcEntry` stubs with only `id` populated, in sort order.
    ///
    /// Used for grouping paths that only need IDs for selector matching — avoids
    /// the full PlainText render pass that `process_references` performs.
    pub(super) fn sorted_id_stubs(&self) -> Vec<ProcEntry> {
        // Numeric citation numbers are already populated by `Processor::begin_run`,
        // which every `FinalizedRun` this module consumes was produced from.
        self.sort_references(self.bibliography.values().collect())
            .into_iter()
            .filter_map(|r| {
                r.id().map(|id| ProcEntry {
                    id: id.to_string(),
                    template: vec![],
                    metadata: ProcEntryMetadata::default(),
                })
            })
            .collect()
    }

    /// Mark references as assigned to a bibliography group.
    fn mark_group_members_assigned(assigned: &mut HashSet<String>, references: &[&Reference]) {
        for reference in references {
            if let Some(id) = reference.id() {
                assigned.insert(id.to_string());
            }
        }
    }

    /// Calculate disambiguation hints locally within a bibliography group.
    ///
    /// Only calculates hints if the group specifies local disambiguation scope.
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
            group_bibliography.insert(
                reference.id().unwrap_or_default().to_string(),
                (*reference).clone(),
            );
        }

        let resolved_sort = group
            .sort
            .as_ref()
            .map(citum_schema::GroupSortEntry::resolve);
        let bibliography_config = self.get_bibliography_config();
        let disambiguator = if let Some(sort) = resolved_sort.as_ref() {
            Disambiguator::with_group_sort(
                &group_bibliography,
                &bibliography_config,
                &self.locale,
                sort,
            )
        } else {
            Disambiguator::new(&group_bibliography, &bibliography_config, &self.locale)
        };

        Some(disambiguator.calculate_hints())
    }

    /// Resolve the effective style to use for a bibliography group.
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

    /// Render bibliography entries for a specific group.
    fn render_group_entries<F>(
        &self,
        _bibliography: &[ProcEntry],
        sorted_refs: Vec<&Reference>,
        group: &BibliographyGroup,
        local_hints: Option<&HashMap<String, ProcHints>>,
        run: &FinalizedRun,
    ) -> Vec<ProcEntry>
    where
        F: OutputFormat<Output = String>,
    {
        // Always process entries with format F so that group components (pre_formatted=true)
        // contain markup in the target format rather than PlainText (_..._).
        let hints = local_hints.unwrap_or(&self.hints);
        let effective_style = self.effective_group_style(group);
        let bibliography_config = self.get_bibliography_config();
        let bibliography_options = self.get_bibliography_options().into_owned();
        let substitute = bibliography_options.subsequent_author_substitute.clone();
        let renderer = Renderer::new(
            RendererResources {
                style: &effective_style,
                bibliography: &self.bibliography,
                locale: &self.locale,
                config: Arc::new(bibliography_config.into_owned()),
                bibliography_config: Some(Arc::new(bibliography_options)),
                first_note_by_id: None,
            },
            hints,
            &run.state().citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
            self.show_semantics,
            self.inject_ast_indices,
            self.abbreviation_map.as_ref(),
        );

        let mut entries = Vec::new();
        let mut previous_reference: Option<&Reference> = None;

        for (index, reference) in sorted_refs.into_iter().enumerate() {
            let ref_id = reference.id().unwrap_or_default().to_string();
            let entry_number = run
                .state()
                .citation_numbers
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&ref_id)
                .copied()
                .unwrap_or(index + 1);

            if let Some(mut processed) =
                renderer.process_bibliography_entry_with_format::<F>(reference, entry_number)
            {
                if let Some(substitute_string) = substitute.as_deref()
                    && let Some(previous) = previous_reference
                    && self.contributors_match(previous, reference)
                {
                    renderer.apply_author_substitution_with_format::<F>(
                        &mut processed,
                        substitute_string,
                    );
                }

                entries.push(ProcEntry {
                    id: ref_id,
                    template: processed,
                    metadata: self.extract_metadata(reference),
                });
                previous_reference = Some(reference);
            }
        }

        entries
    }

    /// Append a rendered bibliography group to the output string.
    fn append_rendered_group<F>(
        &self,
        result: &mut String,
        group: &BibliographyGroup,
        entries: Vec<ProcEntry>,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        suppress_heading: bool,
    ) where
        F: OutputFormat<Output = String>,
    {
        if !result.is_empty() {
            result.push_str("\n\n");
        }

        if !suppress_heading
            && let Some(heading) = group
                .heading
                .as_ref()
                .and_then(|group_heading| self.resolve_group_heading(group_heading))
        {
            result.push_str(&self.render_group_heading::<F>(&heading));
        }

        result.push_str(&crate::render::refs_to_string_with_format::<F>(
            entries,
            annotations,
            annotation_style,
        ));
    }

    /// Append a rendered bibliography partition to the output string.
    fn append_rendered_partition<F>(
        &self,
        result: &mut String,
        heading: Option<&BibliographyPartitionHeading>,
        entries: Vec<ProcEntry>,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
    ) where
        F: OutputFormat<Output = String>,
    {
        if !result.is_empty() {
            result.push_str("\n\n");
        }

        if let Some(heading) =
            heading.and_then(|group_heading| self.resolve_partition_heading(group_heading))
        {
            result.push_str(&self.render_group_heading::<F>(&heading));
        }

        result.push_str(&crate::render::refs_to_string_with_format::<F>(
            entries,
            annotations,
            annotation_style,
        ));
    }

    /// Orchestrate the rendering of automatic bibliography partitions with headings.
    pub(super) fn render_with_partition_sections<F>(
        &self,
        sorted_refs: Vec<&Reference>,
        partitioning: &BibliographySortPartitioning,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let mut result = String::new();

        for (partition_key, references) in
            crate::sort_partitioning::partition_references(sorted_refs, &self.locale, partitioning)
        {
            let heading = partition_key
                .as_ref()
                .and_then(|key| partitioning.headings.get(key));
            let entries = self.merge_compound_entries::<F>(
                self.process_sorted_refs::<_, F>(
                    references.into_iter(),
                    |reference, entry_number| {
                        self.process_bibliography_entry_with_format::<F>(
                            reference,
                            entry_number,
                            run,
                        )
                    },
                    run,
                ),
                run,
            );
            self.append_rendered_partition::<F>(
                &mut result,
                heading,
                entries,
                annotations,
                annotation_style,
            );
        }

        fmt.finish(result)
    }

    /// Render a filtered subset of entries using custom bibliography grouping.
    ///
    /// This uses a two-pass grouping strategy:
    /// 1. Collect and render all populated groups.
    /// 2. Determine if heading suppression applies (only one group populated).
    /// 3. Append groups and any remaining unassigned entries.
    pub(super) fn render_with_custom_groups_filtered<F>(
        &self,
        all_entries: &[ProcEntry],
        groups: &[BibliographyGroup],
        selected: &HashSet<String>,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let cited_ids = &run.state().cited_ids;
        let evaluator = SelectorEvaluator::new(cited_ids);
        let bibliography_config = self.get_bibliography_config();
        let sorter = ReferenceSorter::with_bibliography_config(&self.locale, &bibliography_config);

        let mut assigned = HashSet::new();
        let mut result = String::new();

        // First pass: collect all populated groups with their rendered entries
        let mut populated_groups: Vec<(&BibliographyGroup, Vec<ProcEntry>)> = Vec::new();

        for group in groups {
            let matching_refs =
                self.collect_matching_group_refs(all_entries, &assigned, &evaluator, group);

            let matching_refs: Vec<&Reference> = matching_refs
                .into_iter()
                .filter(|r| r.id().as_deref().is_some_and(|id| selected.contains(id)))
                .collect();

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
            let entries = self.merge_compound_entries::<F>(
                self.render_group_entries::<F>(
                    all_entries,
                    sorted_refs,
                    group,
                    local_hints.as_ref(),
                    run,
                ),
                run,
            );

            populated_groups.push((group, entries));
        }

        // Compute unassigned entries to determine if heading suppression applies
        let unassigned_refs: Vec<&Reference> = all_entries
            .iter()
            .filter(|entry| !assigned.contains(&entry.id) && selected.contains(&entry.id))
            .filter_map(|entry| self.bibliography.get(&entry.id))
            .collect();

        let suppress_heading = populated_groups.len() == 1 && unassigned_refs.is_empty();

        // Second pass: render populated groups with optional heading suppression
        for (group, entries) in populated_groups {
            self.append_rendered_group::<F>(
                &mut result,
                group,
                entries,
                annotations,
                annotation_style,
                suppress_heading,
            );
        }

        self.append_unassigned_entries_filtered::<F>(
            &mut result,
            all_entries,
            &assigned,
            selected,
            annotations,
            annotation_style,
            run,
        );
        fmt.finish(result)
    }

    /// Append unassigned bibliography entries to the output string.
    #[allow(
        clippy::too_many_arguments,
        reason = "internal helper, all params load-bearing"
    )]
    fn append_unassigned_entries_filtered<F>(
        &self,
        result: &mut String,
        bibliography: &[ProcEntry],
        assigned: &HashSet<String>,
        selected: &HashSet<String>,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) where
        F: OutputFormat<Output = String>,
    {
        let unassigned_refs: Vec<&Reference> = bibliography
            .iter()
            .filter(|entry| !assigned.contains(&entry.id) && selected.contains(&entry.id))
            .filter_map(|entry| self.bibliography.get(&entry.id))
            .collect();

        if unassigned_refs.is_empty() {
            return;
        }

        // Re-process references to ensure correct author substitution and disambiguation
        // within the unassigned subset.
        let unassigned = self.merge_compound_entries::<F>(
            self.process_sorted_refs::<_, F>(
                unassigned_refs.into_iter(),
                |reference, entry_number| {
                    self.process_bibliography_entry_with_format::<F>(reference, entry_number, run)
                },
                run,
            ),
            run,
        );

        if !result.is_empty() {
            result.push_str("\n\n");
        }

        result.push_str(&crate::render::refs_to_string_with_format::<F>(
            unassigned,
            annotations,
            annotation_style,
        ));
    }

    /// Render bibliography using legacy (cited/uncited) grouping.
    fn render_with_legacy_grouping<F>(
        &self,
        bibliography: &[ProcEntry],
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let cited_ids = &run.state().cited_ids;
        let cited_entries: Vec<ProcEntry> = bibliography
            .iter()
            .filter(|entry| cited_ids.contains(&entry.id))
            .cloned()
            .collect();

        let mut result = String::new();
        if !cited_entries.is_empty() {
            result.push_str(&crate::render::refs_to_string_with_format::<F>(
                cited_entries,
                annotations,
                annotation_style,
            ));
        }

        fmt.finish(result)
    }

    /// Render the bibliography with grouping for uncited (nocite) items.
    ///
    /// If `style.bibliography.groups` is defined, uses configurable grouping
    /// with per-group sorting. Group selectors apply to individual references
    /// before compound numeric rows are merged, so each rendered group only
    /// includes the members that matched its selector. Otherwise, falls back to
    /// hardcoded cited/uncited grouping for backward compatibility.
    pub fn render_grouped_bibliography_with_format<F>(&self, run: &FinalizedRun) -> String
    where
        F: OutputFormat<Output = String>,
    {
        self.render_grouped_bibliography_with_format_and_annotations::<F>(None, None, run)
    }

    /// Render the bibliography with grouping and annotations.
    pub fn render_grouped_bibliography_with_format_and_annotations<F>(
        &self,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        self.render_grouped_bibliography_inner::<F>(false, annotations, annotation_style, run)
    }

    /// One-shot convenience for [`Processor::render_grouped_bibliography_with_format`]:
    /// begins a throwaway run internally.
    pub fn render_grouped_bibliography_with_format_standalone<F>(&self) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let run = self.begin_run().finalize();
        self.render_grouped_bibliography_with_format::<F>(&run)
    }

    /// One-shot convenience for
    /// [`Processor::render_grouped_bibliography_with_format_and_annotations`]:
    /// begins a throwaway run internally.
    pub fn render_grouped_bibliography_with_format_and_annotations_standalone<F>(
        &self,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let run = self.begin_run().finalize();
        self.render_grouped_bibliography_with_format_and_annotations::<F>(
            annotations,
            annotation_style,
            &run,
        )
    }

    /// Unified document bibliography facade — returns content and per-entry data together.
    ///
    /// This is the single entry point for all document-context bibliography rendering:
    /// batch (`format_document`), interactive session (`DocumentSession`), and the
    /// document-string (`process_document`) path all funnel through here.
    ///
    /// When `restrict_to_cited` is `true` (the document case), only references present
    /// in `run`'s `cited_ids` — cited in-text or registered via `nocite` — are included.
    /// When `false`, all loaded references are eligible; this hook is reserved for the
    /// `allrefs` escape hatch (csl26-f9ri) and is not yet exposed publicly.
    ///
    /// Both `content` and `entries` are computed from the same cited subset so
    /// subsequent-author substitution stays consistent across both outputs.
    pub(crate) fn render_document_bibliography<F>(
        &self,
        restrict_to_cited: bool,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> super::DocumentBibliography
    where
        F: OutputFormat<Output = String>,
    {
        let content = self.render_grouped_bibliography_inner::<F>(
            restrict_to_cited,
            annotations,
            annotation_style,
            run,
        );
        let cited_ids: Vec<String> = run.state().cited_ids.iter().cloned().collect();
        let entries = if restrict_to_cited {
            self.process_selected_references_with_format::<F, _>(cited_ids, run)
                .bibliography
        } else {
            self.process_references_with_format::<F>(run).bibliography
        };
        super::DocumentBibliography { content, entries }
    }

    /// Shared implementation for grouped bibliography rendering.
    ///
    /// When `restrict_to_cited` is `true`, each branch limits its candidate
    /// set to references present in `run`'s `cited_ids`. When `false`, all
    /// loaded references are eligible (the original all-refs behaviour used
    /// by standalone `render refs`, FFI, and tests).
    fn render_grouped_bibliography_inner<F>(
        &self,
        restrict_to_cited: bool,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        if let Some(groups) = self
            .style
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.groups.as_ref())
        {
            let id_stubs = self.sorted_id_stubs();
            let selected = if restrict_to_cited {
                let cited = &run.state().cited_ids;
                id_stubs
                    .iter()
                    .filter(|e| cited.contains(&e.id))
                    .map(|e| e.id.clone())
                    .collect::<HashSet<_>>()
            } else {
                id_stubs
                    .iter()
                    .map(|e| e.id.clone())
                    .collect::<HashSet<_>>()
            };
            return self.render_with_custom_groups_filtered::<F>(
                &id_stubs,
                groups,
                &selected,
                annotations,
                annotation_style,
                run,
            );
        }

        let bibliography_options = self.get_bibliography_options();
        if let Some(partitioning) = bibliography_options.sort_partitioning.as_ref()
            && crate::sort_partitioning::should_render_sections(partitioning)
        {
            let mut refs: Vec<&Reference> = self.bibliography.values().collect();
            if restrict_to_cited {
                let cited = &run.state().cited_ids;
                refs.retain(|r| r.id().as_deref().is_some_and(|id| cited.contains(id)));
            }
            let sorted_refs = self.sort_references(refs);
            return self.render_with_partition_sections::<F>(
                sorted_refs,
                partitioning,
                annotations,
                annotation_style,
                run,
            );
        }

        let all_entries = self.process_references_with_format::<F>(run).bibliography;
        self.render_with_legacy_grouping::<F>(
            &self.merge_compound_entries::<F>(all_entries, run),
            annotations,
            annotation_style,
            run,
        )
    }

    /// Extract and render entries for a bibliography group.
    ///
    /// Returns the individual processed entries for the group, threading
    /// the `assigned` dedup set to ensure each reference appears in only one group.
    fn entries_for_bibliography_group<F>(
        &self,
        group: &BibliographyGroup,
        assigned: &mut HashSet<String>,
        run: &FinalizedRun,
    ) -> Vec<crate::render::ProcEntry>
    where
        F: OutputFormat<Output = String>,
    {
        let bibliography = self.sorted_id_stubs();
        let cited_ids = &run.state().cited_ids;
        let evaluator = SelectorEvaluator::new(cited_ids);
        let bibliography_config = self.get_bibliography_config();
        let sorter = ReferenceSorter::with_bibliography_config(&self.locale, &bibliography_config);

        let matching_refs =
            self.collect_matching_group_refs(&bibliography, assigned, &evaluator, group);
        Self::mark_group_members_assigned(assigned, &matching_refs);

        if matching_refs.is_empty() {
            return Vec::new();
        }

        let sorted_refs = if let Some(sort_spec) = &group.sort {
            sorter.sort_references(matching_refs, &sort_spec.resolve())
        } else {
            matching_refs
        };

        let local_hints = self.build_group_local_hints(&sorted_refs, group);
        self.merge_compound_entries::<F>(
            self.render_group_entries::<F>(
                &bibliography,
                sorted_refs,
                group,
                local_hints.as_ref(),
                run,
            ),
            run,
        )
    }

    /// Render one bibliography block for document output.
    ///
    /// Returns heading and body separately so callers can insert headings
    /// in their own output format.
    #[allow(
        clippy::too_many_arguments,
        reason = "internal helper, all params load-bearing"
    )]
    pub(crate) fn render_document_bibliography_block<F>(
        &self,
        group: &BibliographyGroup,
        assigned: &mut HashSet<String>,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> RenderedBibliographyGroup
    where
        F: OutputFormat<Output = String>,
    {
        let mut headingless = group.clone();
        let heading = headingless
            .heading
            .take()
            .and_then(|group_heading| self.resolve_group_heading(&group_heading));

        let entries = self.entries_for_bibliography_group::<F>(&headingless, assigned, run);
        let body = crate::render::refs_to_string_slice_with_format::<F>(
            &entries,
            annotations,
            annotation_style,
        );

        RenderedBibliographyGroup {
            heading,
            body,
            entries,
        }
    }

    /// Render an ordered sequence of sectional bibliography blocks.
    ///
    /// Threads a single `assigned` dedup set so each reference appears in
    /// only one block. Returns rendered groups with heading, body, and entries.
    pub(crate) fn render_document_bibliography_blocks<F>(
        &self,
        groups: &[BibliographyGroup],
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> Vec<RenderedBibliographyGroup>
    where
        F: OutputFormat<Output = String>,
    {
        let mut assigned = std::collections::HashSet::new();
        groups
            .iter()
            .map(|group| {
                self.render_document_bibliography_block::<F>(
                    group,
                    &mut assigned,
                    annotations,
                    annotation_style,
                    run,
                )
            })
            .collect()
    }

    pub(super) fn extract_metadata(&self, reference: &Reference) -> ProcEntryMetadata {
        let bibliography_config = Arc::new(self.get_bibliography_config().into_owned());
        let options = RenderOptions {
            config: bibliography_config.clone(),
            bibliography_config: Some(Arc::new(self.get_bibliography_options().into_owned())),
            locale: &self.locale,
            context: RenderContext::Bibliography,
            mode: citum_schema::citation::CitationMode::NonIntegral,
            suppress_author: false,
            locator_raw: None,
            ref_type: None,
            show_semantics: self.show_semantics,
            current_template_index: None,
            abbreviation_map: self.abbreviation_map.as_ref(),
        };

        let ml = bibliography_config.multilingual.as_ref();
        let preferred_transliteration = ml.and_then(|m| m.preferred_transliteration.as_deref());
        let preferred_script = ml.and_then(|m| m.preferred_script.as_ref());

        ProcEntryMetadata {
            author: reference.author().map(|author| {
                let names = resolve_multilingual_name(
                    &author,
                    ml.and_then(|m| m.name_mode.as_ref()),
                    preferred_transliteration,
                    preferred_script,
                    &self.locale.locale,
                );
                format_contributors_short(&names, &options)
            }),
            year: reference
                .effective_issued_date()
                .map(|issued| issued.year().clone()),
            title: reference.title().map(|title| {
                use citum_schema::reference::types::{MultilingualString, Title};
                match &title {
                    Title::Multilingual(m) => resolve_multilingual_string(
                        &MultilingualString::Complex(m.clone()),
                        ml.and_then(|ml| ml.title_mode.as_ref()),
                        preferred_transliteration,
                        preferred_script,
                        &self.locale.locale,
                    ),
                    _ => title.to_string(),
                }
            }),
        }
    }

    fn render_group_heading<F>(&self, heading: &str) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let fmt = F::default();
        fmt.finish(fmt.unnumbered_heading(2, fmt.text(heading)))
    }
}
