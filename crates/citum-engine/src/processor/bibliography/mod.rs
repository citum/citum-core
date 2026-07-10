/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Bibliography processing and rendering.
//!
//! This module owns bibliography entry generation, grouped rendering,
//! subsequent-author substitution, and the document-facing facade methods used
//! by the document processor.

mod compound;
mod grouping;

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

use super::matching::Matcher;
use super::rendering::{CompoundRenderData, Renderer, RendererResources};
use super::run_state::FinalizedRun;
use super::{ProcessedReferences, Processor};
use crate::api::AnnotationStyle;
use crate::reference::Reference;
use crate::render::format::OutputFormat;
use crate::render::{ProcEntry, ProcTemplate};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Rendered bibliography block data for document integration.
#[derive(Debug, Clone, Default)]
pub(crate) struct RenderedBibliographyGroup {
    /// The resolved group heading, if one exists.
    pub(crate) heading: Option<String>,
    /// The rendered bibliography body without any document-level heading wrapper.
    pub(crate) body: String,
    /// Individual entries rendered in this block.
    pub(crate) entries: Vec<crate::render::ProcEntry>,
}

/// Combined document bibliography rendering output.
///
/// Returned by [`Processor::render_document_bibliography`] — the unified facade
/// used by the batch, session, and document-string rendering paths. Both fields
/// are computed from the same cited subset so subsequent-author substitution
/// stays consistent between the rendered string and the per-entry data.
#[derive(Debug, Clone, Default)]
pub(crate) struct DocumentBibliography {
    /// The full rendered bibliography string for the document.
    pub(crate) content: String,
    /// Flat per-entry data, one entry per cited reference.
    pub(crate) entries: Vec<crate::render::ProcEntry>,
}

/// Bibliography entries render in parallel (behind the `parallel` feature,
/// on by default) once a bibliography reaches this many entries; below the
/// threshold, thread-pool dispatch overhead isn't worth paying and entries
/// render sequentially instead. The value is a conservative starting
/// estimate, not yet bench-tuned (`cargo bench --bench rendering` runs on a
/// loaded desktop were inconclusive); see
/// `docs/specs/PARALLEL_BIBLIOGRAPHY_RENDERING.md` for the rationale.
#[cfg(feature = "parallel")]
pub(crate) const PARALLEL_MIN_ENTRIES: usize = 32;

/// Resolve `(reference, entry_number)` pairs for `sorted_refs`, in order.
///
/// Reads each reference's already-assigned citation number from `run` when
/// present — numeric styles pre-assign these at `begin_run`
/// (`initialize_numeric_bibliography_numbers`) — and falls back to its
/// 1-based position in `sorted_refs` otherwise. This is a sequential,
/// read-only pass over `run`'s shared `citation_numbers` map, done up front
/// so the render step that follows (parallel or not) is free of further
/// lock contention.
fn number_sorted_refs<'a>(
    sorted_refs: impl Iterator<Item = &'a Reference>,
    run: &FinalizedRun,
) -> Vec<(&'a Reference, usize)> {
    sorted_refs
        .enumerate()
        .map(|(index, reference)| {
            let ref_id = reference.id().unwrap_or_default().to_string();
            let entry_number = run
                .state()
                .citation_numbers
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&ref_id)
                .copied()
                .unwrap_or(index + 1);
            (reference, entry_number)
        })
        .collect()
}

/// Render each numbered reference through `process_fn`, preserving input
/// order.
///
/// Sequential fallback used when the `parallel` feature is disabled, or the
/// bibliography is below [`PARALLEL_MIN_ENTRIES`] (see [`render_entries`]).
fn render_entries_sequential<'a>(
    numbered_refs: &[(&'a Reference, usize)],
    process_fn: &impl Fn(&Reference, usize) -> Option<ProcTemplate>,
) -> Vec<(&'a Reference, Option<ProcTemplate>)> {
    numbered_refs
        .iter()
        .map(|&(reference, entry_number)| (reference, process_fn(reference, entry_number)))
        .collect()
}

/// Render each numbered reference through `process_fn` across the rayon
/// thread pool, preserving input order.
///
/// Reference order in the output matches `numbered_refs` exactly (`par_iter`
/// over a slice is order-preserving under `collect`), so the caller's
/// subsequent-author-substitution post-pass sees the same sequence it would
/// under [`render_entries_sequential`].
#[cfg(feature = "parallel")]
fn render_entries_parallel<'a>(
    numbered_refs: &[(&'a Reference, usize)],
    process_fn: &(impl Fn(&Reference, usize) -> Option<ProcTemplate> + Sync),
) -> Vec<(&'a Reference, Option<ProcTemplate>)> {
    use rayon::prelude::*;
    numbered_refs
        .par_iter()
        .map(|&(reference, entry_number)| (reference, process_fn(reference, entry_number)))
        .collect()
}

/// Choose the sequential or parallel render path for `numbered_refs` and
/// apply it.
///
/// Parallel rendering requires both the `parallel` feature and
/// `numbered_refs.len() >= PARALLEL_MIN_ENTRIES`; otherwise this falls back
/// to [`render_entries_sequential`].
fn render_entries<'a>(
    numbered_refs: &[(&'a Reference, usize)],
    process_fn: &(impl Fn(&Reference, usize) -> Option<ProcTemplate> + Send + Sync),
) -> Vec<(&'a Reference, Option<ProcTemplate>)> {
    #[cfg(feature = "parallel")]
    if numbered_refs.len() >= PARALLEL_MIN_ENTRIES {
        return render_entries_parallel(numbered_refs, process_fn);
    }
    render_entries_sequential(numbered_refs, process_fn)
}

impl Processor {
    /// Create a bibliography renderer with effective shared and bibliography-only config.
    fn with_bibliography_renderer<T>(
        &self,
        run: &FinalizedRun,
        render: impl FnOnce(Renderer<'_>) -> T,
    ) -> T {
        let bibliography_shared_config = self.get_bibliography_config();
        let bibliography_config = self.get_bibliography_options().into_owned();
        let renderer = Renderer::new(
            RendererResources {
                style: &self.style,
                bibliography: &self.bibliography,
                locale: &self.locale,
                config: Arc::new(bibliography_shared_config.into_owned()),
                bibliography_config: Some(Arc::new(bibliography_config)),
                first_note_by_id: None,
            },
            &self.hints,
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

        render(renderer)
    }

    /// Process sorted references and apply subsequent-author substitution.
    ///
    /// Returns bibliography entries with optional author substitution applied.
    ///
    /// This is the core iterator for bibliography rendering, handling the choice
    /// between entry-specific rendering and subsequent-author placeholders.
    ///
    /// Entry numbers are resolved sequentially first (a single pass over
    /// `run`'s shared `citation_numbers` map), then entries render via
    /// [`render_entries`] — sequentially, or across the rayon thread pool
    /// once the bibliography is large enough (see [`PARALLEL_MIN_ENTRIES`]) —
    /// and finally [`apply_substitution_post_pass`](Self::apply_substitution_post_pass)
    /// walks the (order-preserved) results sequentially to apply
    /// subsequent-author substitution, which depends on cite-order.
    fn process_sorted_refs<'a, I, F>(
        &self,
        sorted_refs: I,
        process_fn: impl Fn(&Reference, usize) -> Option<ProcTemplate> + Send + Sync,
        run: &FinalizedRun,
    ) -> Vec<ProcEntry>
    where
        I: Iterator<Item = &'a Reference>,
        F: OutputFormat<Output = String>,
    {
        let bibliography_options = self.get_bibliography_options();
        let substitute = bibliography_options.subsequent_author_substitute.as_ref();

        let numbered_refs = number_sorted_refs(sorted_refs, run);
        let rendered = render_entries(&numbered_refs, &process_fn);

        self.apply_substitution_post_pass::<F>(rendered, substitute, run)
    }

    /// Apply subsequent-author substitution to already-rendered entries and
    /// assemble [`ProcEntry`]s, in order.
    ///
    /// This is the sequential part of bibliography rendering: substitution
    /// depends on the *previous successfully rendered* reference, so it
    /// cannot itself run in parallel. `rendered` must already be in final
    /// bibliography order (as produced by [`render_entries`], parallel or
    /// not); entries where `process_fn` returned `None` are skipped
    /// entirely and do not advance the "previous reference" used for
    /// contributor matching.
    fn apply_substitution_post_pass<F>(
        &self,
        rendered: Vec<(&Reference, Option<ProcTemplate>)>,
        substitute: Option<&String>,
        run: &FinalizedRun,
    ) -> Vec<ProcEntry>
    where
        F: OutputFormat<Output = String>,
    {
        let mut bibliography = Vec::with_capacity(rendered.len());
        let mut previous_reference: Option<&Reference> = None;

        for (reference, processed) in rendered {
            let Some(mut processed) = processed else {
                continue;
            };

            if let Some(substitute_string) = substitute
                && let Some(previous) = previous_reference
                && self.contributors_match(previous, reference)
            {
                self.with_bibliography_renderer(run, |renderer| {
                    renderer.apply_author_substitution_with_format::<F>(
                        &mut processed,
                        substitute_string,
                    );
                });
            }

            let ref_id = reference.id().unwrap_or_default().to_string();
            bibliography.push(ProcEntry {
                id: ref_id,
                template: processed,
                metadata: self.extract_metadata(reference),
            });
            previous_reference = Some(reference);
        }

        bibliography
    }

    /// Process all bibliography references and render them.
    ///
    /// This is a one-shot convenience wrapper: it begins a throwaway
    /// [`super::run_state::RunState`] internally, so it has no continuity
    /// with any citations processed elsewhere. Use
    /// [`Processor::process_references_with_format`] with an explicit,
    /// shared `FinalizedRun` to render a bibliography that reflects prior
    /// citation registration in the same document.
    pub fn process_references(&self) -> ProcessedReferences {
        let run = self.begin_run().finalize();
        self.process_references_with_format::<crate::render::plain::PlainText>(&run)
    }

    /// Process all bibliography references using the requested output format.
    ///
    /// This preserves format-specific inline markup in per-entry API output.
    /// `run` should reflect all citations already processed for this
    /// document (or be a fresh [`Processor::begin_run`] for a standalone
    /// bibliography with no citations); see
    /// [`Processor::process_references_with_format_standalone`] for a
    /// one-shot convenience.
    pub fn process_references_with_format<F>(&self, run: &FinalizedRun) -> ProcessedReferences
    where
        F: OutputFormat<Output = String>,
    {
        let sorted_refs = self.sort_references(self.bibliography.values().collect());
        let bibliography = self.process_sorted_refs::<_, F>(
            sorted_refs.iter().copied(),
            |reference, entry_number| {
                self.process_bibliography_entry_with_format::<F>(reference, entry_number, run)
            },
            run,
        );
        ProcessedReferences {
            bibliography,
            citations: None,
        }
    }

    /// Process only the selected bibliography entries, in bibliography sort order.
    ///
    /// Mirrors the flat path inside
    /// [`render_selected_bibliography_with_format_and_annotations`] so that
    /// per-entry `text` and subsequent-author substitution are computed against
    /// the same subset that produced `content` — not the full loaded
    /// bibliography. This matters for subsequent-author substitution: an uncited
    /// predecessor must not cause the first cited entry to receive `———`.
    pub(crate) fn process_selected_references_with_format<F, I>(
        &self,
        item_ids: I,
        run: &FinalizedRun,
    ) -> ProcessedReferences
    where
        F: OutputFormat<Output = String>,
        I: IntoIterator<Item = String>,
    {
        let selected: HashSet<String> = item_ids.into_iter().collect();
        let sorted_refs = self.sort_references(self.bibliography.values().collect());
        let bibliography = self.process_sorted_refs::<_, F>(
            sorted_refs
                .iter()
                .filter(|r| r.id().as_deref().is_some_and(|id| selected.contains(id)))
                .copied(),
            |reference, entry_number| {
                self.process_bibliography_entry_with_format::<F>(reference, entry_number, run)
            },
            run,
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
        run: &FinalizedRun,
    ) -> Option<ProcTemplate> {
        self.with_bibliography_renderer(run, |renderer| {
            renderer.process_bibliography_entry(reference, entry_number)
        })
    }

    /// Process a bibliography entry with specific format.
    pub fn process_bibliography_entry_with_format<F>(
        &self,
        reference: &Reference,
        entry_number: usize,
        run: &FinalizedRun,
    ) -> Option<ProcTemplate>
    where
        F: OutputFormat<Output = String>,
    {
        self.with_bibliography_renderer(run, |renderer| {
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

    /// Render the bibliography to a string using a specific format.
    pub fn render_bibliography_with_format<F>(&self, run: &FinalizedRun) -> String
    where
        F: OutputFormat<Output = String>,
    {
        self.render_bibliography_with_format_and_annotations::<F>(None, None, run)
    }

    /// Render the bibliography to a string with annotations.
    pub fn render_bibliography_with_format_and_annotations<F>(
        &self,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        self.render_selected_bibliography_with_format_and_annotations::<F, _>(
            self.bibliography.keys().cloned().collect::<Vec<_>>(),
            annotations,
            annotation_style,
            run,
        )
    }

    /// Render a selected bibliography subset to a string using a specific format.
    pub fn render_selected_bibliography_with_format<F, I>(
        &self,
        item_ids: I,
        run: &FinalizedRun,
    ) -> String
    where
        F: OutputFormat<Output = String>,
        I: IntoIterator<Item = String>,
    {
        self.render_selected_bibliography_with_format_and_annotations::<F, _>(
            item_ids, None, None, run,
        )
    }

    /// Render a selected bibliography subset to a string with annotations.
    ///
    /// Orchestrates the choice between:
    /// 1. Custom bibliography groups (selectors and headings).
    /// 2. Automatic sort partitioning with sections (headings only).
    /// 3. Standard flat rendering.
    pub fn render_selected_bibliography_with_format_and_annotations<F, I>(
        &self,
        item_ids: I,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
        run: &FinalizedRun,
    ) -> String
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
            .filter(|bibliography| bibliography.groups_enabled)
            .and_then(|bibliography| bibliography.groups.as_ref())
        {
            let all_entries = self.sorted_id_stubs();
            return self.render_with_custom_groups_filtered::<F>(
                &all_entries,
                groups,
                &selected,
                annotations,
                annotation_style,
                run,
            );
        }

        // 2. Check for automatic sort partitioning with sections
        let bibliography_options = self.get_bibliography_options();
        if let Some(partitioning) = bibliography_options.sort_partitioning.as_ref()
            && crate::sort_partitioning::should_render_sections(partitioning)
        {
            let all_sorted = self.sort_references(self.bibliography.values().collect());
            let selected_sorted: Vec<&Reference> = all_sorted
                .into_iter()
                .filter(|r| r.id().as_deref().is_some_and(|id| selected.contains(id)))
                .collect();
            return self.render_with_partition_sections::<F>(
                selected_sorted,
                partitioning,
                annotations,
                annotation_style,
                run,
            );
        }

        // 3. Fallback to flat rendering
        let sorted_refs = self.sort_references(self.bibliography.values().collect());

        let bibliography = self.process_sorted_refs::<_, F>(
            sorted_refs
                .iter()
                .filter(|r| r.id().as_deref().is_some_and(|id| selected.contains(id)))
                .copied(),
            |reference, entry_number| {
                self.process_bibliography_entry_with_format::<F>(reference, entry_number, run)
            },
            run,
        );

        let bibliography = self.merge_compound_entries::<F>(bibliography, run);
        crate::render::refs_to_string_with_format::<F>(bibliography, annotations, annotation_style)
    }

    /// Render the entire bibliography to a formatted string.
    ///
    /// One-shot convenience wrapper: begins a throwaway run internally, so
    /// it has no continuity with any citations processed elsewhere. Use
    /// [`Processor::render_bibliography_with_format`] with an explicit,
    /// shared `FinalizedRun` for a bibliography that reflects prior citation
    /// registration.
    pub fn render_bibliography(&self) -> String {
        let run = self.begin_run().finalize();
        self.render_bibliography_with_format::<crate::render::plain::PlainText>(&run)
    }

    /// One-shot convenience for [`Processor::process_references_with_format`]:
    /// begins a throwaway run internally.
    pub fn process_references_with_format_standalone<F>(&self) -> ProcessedReferences
    where
        F: OutputFormat<Output = String>,
    {
        let run = self.begin_run().finalize();
        self.process_references_with_format::<F>(&run)
    }

    /// One-shot convenience for [`Processor::process_bibliography_entry`]:
    /// begins a throwaway run internally.
    pub fn process_bibliography_entry_standalone(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate> {
        let run = self.begin_run().finalize();
        self.process_bibliography_entry(reference, entry_number, &run)
    }

    /// One-shot convenience for [`Processor::process_bibliography_entry_with_format`]:
    /// begins a throwaway run internally.
    pub fn process_bibliography_entry_with_format_standalone<F>(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate>
    where
        F: OutputFormat<Output = String>,
    {
        let run = self.begin_run().finalize();
        self.process_bibliography_entry_with_format::<F>(reference, entry_number, &run)
    }

    /// One-shot convenience for [`Processor::render_bibliography_with_format`]:
    /// begins a throwaway run internally.
    pub fn render_bibliography_with_format_standalone<F>(&self) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let run = self.begin_run().finalize();
        self.render_bibliography_with_format::<F>(&run)
    }

    /// One-shot convenience for
    /// [`Processor::render_bibliography_with_format_and_annotations`]:
    /// begins a throwaway run internally.
    pub fn render_bibliography_with_format_and_annotations_standalone<F>(
        &self,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
    ) -> String
    where
        F: OutputFormat<Output = String>,
    {
        let run = self.begin_run().finalize();
        self.render_bibliography_with_format_and_annotations::<F>(
            annotations,
            annotation_style,
            &run,
        )
    }

    /// One-shot convenience for [`Processor::render_selected_bibliography_with_format`]:
    /// begins a throwaway run internally.
    pub fn render_selected_bibliography_with_format_standalone<F, I>(&self, item_ids: I) -> String
    where
        F: OutputFormat<Output = String>,
        I: IntoIterator<Item = String>,
    {
        let run = self.begin_run().finalize();
        self.render_selected_bibliography_with_format::<F, I>(item_ids, &run)
    }

    /// One-shot convenience for
    /// [`Processor::render_selected_bibliography_with_format_and_annotations`]:
    /// begins a throwaway run internally.
    pub fn render_selected_bibliography_with_format_and_annotations_standalone<F, I>(
        &self,
        item_ids: I,
        annotations: Option<&HashMap<String, String>>,
        annotation_style: Option<&AnnotationStyle>,
    ) -> String
    where
        F: OutputFormat<Output = String>,
        I: IntoIterator<Item = String>,
    {
        let run = self.begin_run().finalize();
        self.render_selected_bibliography_with_format_and_annotations::<F, I>(
            item_ids,
            annotations,
            annotation_style,
            &run,
        )
    }
}
