---
# csl26-u2kb
title: Cache sorted-ID spine and reduce Reference cloning in bibliography group rendering
status: completed
type: task
priority: normal
created_at: 2026-07-11T09:33:54Z
updated_at: 2026-07-11T17:41:11Z
parent: csl26-8m2p
blocked_by:
    - csl26-plaz
---

Follow-up to csl26-plaz (P2 / \"Additional Audit Scope\" from the 2026-07-10
engine follow-up audit, deferred to keep csl26-plaz focused on the P1
duplicate-render fix).

`entries_for_bibliography_group` (processor/bibliography/grouping.rs) calls
`sorted_id_stubs`, which sorts the *entire* bibliography for every
independently rendered group/block — for `g` blocks over `n` references this
approaches `g * O(n log n)`. This is the primary target: cache the sorted ID
spine once per document-facade call (e.g. per
`render_document_bibliography_blocks` invocation) instead of re-sorting per
block.

`build_group_local_hints` (same file) clones every selected `Reference` into
a temporary `Bibliography` before running group-local disambiguation, to give
`Disambiguator` a simple, safe lifetime boundary. Only widen `Disambiguator`
to accept a borrowed subset if profiling after the spine-caching fix still
shows this cloning as a measurable cost — the audit explicitly flags the
owned-bibliography approach as a defensible trade-off, not a bug.

## Checklist

- [x] Benchmark `render_document_bibliography_blocks` with many groups over a
      large library to quantify the repeated-sort cost
- [x] Cache the sorted ID spine per document-facade call, threaded through
      `entries_for_bibliography_group`
- [x] Re-benchmark; only pursue a borrowed-subset `Disambiguator` API if
      `build_group_local_hints` cloning still shows up as a measurable cost
- [x] Preserve existing group-local disambiguation/sort/heading behavior
      (regression tests already covering `render_document_bibliography_blocks`
      and `render_document_bibliography_block` must stay green)

Audit: docs/architecture/audits/2026-07-10_CITUM_ENGINE_FOLLOW_UP_REVIEW.md (P2)

## Summary of Changes

Cached the sorted-ID bibliography spine (`sorted_id_stubs`) once per
`render_document_bibliography_blocks` call instead of once per group/block:

- `entries_for_bibliography_group` and `render_document_bibliography_block`
  now take a `spine: &[ProcEntry]` parameter instead of calling
  `sorted_id_stubs` internally.
- `render_document_bibliography_blocks` computes the spine once and passes
  the same slice to every block, collapsing the sort cost from
  `g * O(n log n)` to a single `O(n log n)` over `g` blocks.
- `sorted_id_stubs` visibility bumped `pub(super)` -> `pub(crate)` so the
  three standalone-block regression tests (`processor/tests.rs`) can build
  their own spine for the still-supported single-block call shape.
- No change needed to `render_grouped_bibliography_inner`'s custom-groups
  path or `process_selected_references_with_format`'s caller in
  `processor/bibliography/mod.rs` — both already call `sorted_id_stubs`
  exactly once.

**Benchmark** (new permanent Criterion bench, `bench_document_bibliography_blocks`
in `benches/rendering.rs`; 400 loaded references split across 8 disjoint
`language` groups, driven through the public
`process_document_with_caller_blocks` entry point):

| | median time |
|---|---|
| Before (main) | 29.4 ms |
| After (this change) | 21.1 ms |

~28% reduction (criterion: -28.12%, p < 0.05), consistent with removing 7 of
8 redundant full-bibliography sorts.

**`build_group_local_hints` clone reduction (deferred):** per the bean and
audit, only pursued if profiling after the spine fix still showed the
per-group `Reference` clone as a measurable cost. The ~28% reduction is
fully explained by removing the redundant sorts; no separate profiling signal
pointed at the clone as a remaining bottleneck, so the owned-`Bibliography`
boundary in `Disambiguator` is kept as-is (matches the audit's assessment
that it's a defensible trade-off, not a bug). No new bean needed unless a
future measurement says otherwise.

Verification: `cargo nextest run -p citum-engine` (948/948 passed),
`just pre-commit` (fmt + clippy -D warnings + full workspace nextest) green.
