/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Bibliography processing and rendering.
//!
//! This module owns bibliography entry generation, grouped rendering,
//! subsequent-author substitution, and the document-facing facade methods used
//! by the document processor.

mod compound;
mod grouping;

use super::matching::Matcher;
use super::rendering::{CompoundRenderData, Renderer, RendererResources};
use super::{ProcessedReferences, Processor};
use crate::reference::Reference;
use crate::render::format::OutputFormat;
use crate::render::{ProcEntry, ProcTemplate};
use std::collections::HashSet;

/// Rendered bibliography block data for document integration.
#[derive(Debug, Clone, Default)]
pub(crate) struct RenderedBibliographyGroup {
    /// The resolved group heading, if one exists.
    pub(crate) heading: Option<String>,
    /// The rendered bibliography body without any document-level heading wrapper.
    pub(crate) body: String,
}

impl Processor {
    /// Create a bibliography renderer with effective shared and bibliography-only config.
    fn with_bibliography_renderer<T>(&self, render: impl FnOnce(Renderer<'_>) -> T) -> T {
        let bibliography_shared_config = self.get_bibliography_config();
        let bibliography_config = self.get_bibliography_options().into_owned();
        let renderer = Renderer::new(
            RendererResources {
                style: &self.style,
                bibliography: &self.bibliography,
                locale: &self.locale,
                config: &bibliography_shared_config,
                bibliography_config: Some(bibliography_config),
            },
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
            self.show_semantics,
            self.inject_ast_indices,
        );

        render(renderer)
    }

    /// Process sorted references and apply subsequent-author substitution.
    ///
    /// Returns bibliography entries with optional author substitution applied.
    fn process_sorted_refs<'a, I, F>(
        &self,
        sorted_refs: I,
        process_fn: impl Fn(&Reference, usize) -> Option<ProcTemplate>,
    ) -> Vec<ProcEntry>
    where
        I: Iterator<Item = &'a Reference>,
        F: OutputFormat<Output = String>,
    {
        let mut bibliography = Vec::new();
        let mut previous_reference: Option<&Reference> = None;

        let bibliography_options = self.get_bibliography_options();
        let substitute = bibliography_options.subsequent_author_substitute.as_ref();

        for (index, reference) in sorted_refs.enumerate() {
            let ref_id = reference.id().unwrap_or_default().to_string();
            let entry_number = self
                .citation_numbers
                .borrow()
                .get(&ref_id)
                .copied()
                .unwrap_or(index + 1);

            if let Some(mut processed) = process_fn(reference, entry_number) {
                if let Some(substitute_string) = substitute
                    && let Some(previous) = previous_reference
                    && self.contributors_match(previous, reference)
                {
                    self.with_bibliography_renderer(|renderer| {
                        renderer.apply_author_substitution_with_format::<F>(
                            &mut processed,
                            substitute_string,
                        );
                    });
                }

                bibliography.push(ProcEntry {
                    id: ref_id,
                    template: processed,
                    metadata: self.extract_metadata(reference),
                });
                previous_reference = Some(reference);
            }
        }

        bibliography
    }

    /// Process all bibliography references and render them.
    ///
    /// Returns sorted and formatted bibliography entries. For numeric styles,
    /// citations must have been processed first to assign citation numbers.
    pub fn process_references(&self) -> ProcessedReferences {
        self.initialize_numeric_bibliography_numbers();
        let sorted_refs = self.sort_references(self.bibliography.values().collect());
        let bibliography = self.process_sorted_refs::<_, crate::render::plain::PlainText>(
            sorted_refs.iter().copied(),
            |reference, entry_number| self.process_bibliography_entry(reference, entry_number),
        );

        ProcessedReferences {
            bibliography,
            citations: None,
        }
    }

    /// Process and render a bibliography entry.
    pub fn process_bibliography_entry(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate> {
        self.with_bibliography_renderer(|renderer| {
            renderer.process_bibliography_entry(reference, entry_number)
        })
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
        self.with_bibliography_renderer(|renderer| {
            renderer.process_bibliography_entry_with_format::<F>(reference, entry_number)
        })
    }

    /// Check whether primary contributors match between two references.
    ///
    /// Used for subsequent author substitution in bibliographies.
    pub fn contributors_match(&self, prev: &Reference, current: &Reference) -> bool {
        let matcher = Matcher::new(&self.style, &self.default_config);
        matcher.contributors_match(prev, current)
    }

    /// Replace the primary contributor with a substitution string.
    ///
    /// Used for subsequent author substitution (e.g., "———").
    pub fn apply_author_substitution(&self, proc: &mut ProcTemplate, substitute: &str) {
        self.with_bibliography_renderer(|renderer| {
            renderer.apply_author_substitution(proc, substitute);
        });
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
        let selected: HashSet<String> = item_ids.into_iter().collect();

        // 1. Check for custom bibliography groups
        if let Some(groups) = self
            .style
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.groups.as_ref())
        {
            let all_entries = self.process_references().bibliography;
            return self.render_with_custom_groups_filtered::<F>(&all_entries, groups, &selected);
        }

        // 2. Check for automatic sort partitioning with sections
        let bibliography_options = self.get_bibliography_options();
        if let Some(partitioning) = bibliography_options.sort_partitioning.as_ref()
            && crate::sort_partitioning::should_render_sections(partitioning)
        {
            self.initialize_numeric_bibliography_numbers();
            let all_sorted = self.sort_references(self.bibliography.values().collect());
            let selected_sorted: Vec<&Reference> = all_sorted
                .into_iter()
                .filter(|r| r.id().as_deref().is_some_and(|id| selected.contains(id)))
                .collect();
            return self.render_with_partition_sections::<F>(selected_sorted, partitioning);
        }

        // 3. Fallback to flat rendering
        self.initialize_numeric_bibliography_numbers();
        let sorted_refs = self.sort_references(self.bibliography.values().collect());

        let bibliography = self.process_sorted_refs::<_, F>(
            sorted_refs
                .iter()
                .filter(|r| r.id().as_deref().is_some_and(|id| selected.contains(id)))
                .copied(),
            |reference, entry_number| {
                self.process_bibliography_entry_with_format::<F>(reference, entry_number)
            },
        );

        let bibliography = self.merge_compound_entries::<F>(bibliography);
        crate::render::refs_to_string_with_format::<F>(bibliography, None, None)
    }

    /// Render the entire bibliography to a formatted string.
    pub fn render_bibliography(&self) -> String {
        self.render_bibliography_with_format::<crate::render::plain::PlainText>()
    }
}
