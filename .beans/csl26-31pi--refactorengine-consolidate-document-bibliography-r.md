---
# csl26-31pi
title: 'refactor(engine): unify document bibliography rendering into single Processor method'
status: todo
type: task
priority: normal
created_at: 2026-06-09T12:08:06Z
updated_at: 2026-06-09T12:09:51Z
---


## Background

The `feat(engine): nocite bibliography-only entries` PR (#891) fixed
the behavioral inconsistency between paths but left **two separate implementations**
of "restrict bibliography to cited refs":

1. **Document-string path** (`pipeline.rs` → `render_grouped_document_bibliography_with_format`)
   Calls `render_grouped_bibliography_inner(restrict_to_cited: true, …)` in
   `crates/citum-engine/src/processor/bibliography/grouping.rs:569`.

2. **Batch / session path** (`document.rs::format_bibliography`)
   Passes `cited_ids_vec` explicitly to `render_selected_bibliography_with_format_and_annotations`
   and calls `process_selected_references_with_format` for per-entry data.
   `crates/citum-engine/src/api/document.rs:638`.

Both are behaviorally correct but share no code. The `restrict_to_cited` flag in
`render_grouped_bibliography_inner` already models the concept cleanly; the batch/session
path just bypasses it via a different entry point.

## Goal

A single `Processor::render_document_bibliography<F>` method (or equivalent name) that:
- Consults `self.cited_ids` internally (already populated by the nocite/citation pipeline)
- Returns both `content: String` and `entries: Vec<BibliographyEntry>` in one call
- Handles all three sub-paths: custom groups, sort-partitioning, and flat
- Is the only caller of `render_grouped_bibliography_inner(restrict_to_cited: true, …)`
- Replaces the `format_bibliography` function in `document.rs` as the orchestrator

The standalone / FFI `render_bibliography_with_format` (`restrict_to_cited: false`) stays
unchanged — those are genuinely no-document-context calls.

## Key files

- `crates/citum-engine/src/processor/bibliography/mod.rs` — add new method here
- `crates/citum-engine/src/processor/bibliography/grouping.rs:569` — wire through
- `crates/citum-engine/src/api/document.rs:611` — replace `format_bibliography` to call new method
- `crates/citum-engine/src/processor/document/pipeline.rs:244` — update to use new method
- `crates/citum-cli/` — calls `format_document`, which routes through `document.rs:611`; no direct
  bibliography rendering. The CLI picks up the unified method automatically — no changes needed
  here, but verify no bypass when implementing.

## Constraints

- No behavior change: all existing tests must pass after refactor
- The `entries` field must remain consistent with `content` (subsequent-author substitution
  must iterate only the cited subset — the bug `process_selected_references_with_format`
  fixed in the batch path must be preserved in the unified method)
- `process_selected_references_with_format` added in the nocite PR can be folded into
  the new method or kept as an internal helper — defer to implementer
