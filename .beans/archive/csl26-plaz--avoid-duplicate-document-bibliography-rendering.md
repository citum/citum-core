---
# csl26-plaz
title: Avoid duplicate document bibliography rendering
status: completed
type: task
priority: normal
tags:
    - engine
    - performance
created_at: 2026-07-10T21:50:45Z
updated_at: 2026-07-11T09:34:12Z
parent: csl26-8m2p
---

`Processor::render_document_bibliography` builds content and entries in
separate render passes. In the flat `restrict_to_cited` path, content first
renders every loaded reference and `render_with_legacy_grouping` then clones
and keeps only cited entries; entries are rendered again from the cited subset.
Custom-group and partition paths likewise render content separately from flat
per-entry output.

## Checklist

- [x] Benchmark document rendering with a large library and small cited subset
- [x] Produce content and per-entry data from one cited, correctly ordered `ProcEntry` pass where semantics allow
- [x] Preserve group-local templates, disambiguation, annotations, and subsequent-author substitution
- [x] Add public-output equivalence tests for flat, custom-group, and partitioned paths

Audit: `docs/architecture/audits/2026-07-10_CITUM_ENGINE_FOLLOW_UP_REVIEW.md`

## Additional Audit Scope

Measure and, if warranted, eliminate repeated whole-bibliography sorting in
`entries_for_bibliography_group` and deep `Reference` cloning in
`build_group_local_hints`. Prefer caching the sorted ID spine per
document-facade call before widening `Disambiguator` lifetimes.

## Summary of Changes

`Processor::render_document_bibliography` (`processor/bibliography/grouping.rs`)
now special-cases the common flat/sort-partitioned document shape: it renders
each cited reference's template exactly once and derives both the appended
`content` string and the per-entry `entries` API data from that one pass,
instead of rendering once for content and again for entries (and, in the flat
case, once for the *entire loaded library* before throwing most of it away).

- New `render_flat_document_bibliography` renders the cited, sorted subset
  once via `render_numbered_refs`, then applies the (cheap)
  subsequent-author-substitution post-pass either once flat (for `entries`)
  or once per sort-partition section (for `content`, preserving the
  historical per-section substitution reset).
- The fast path only activates when `restrict_to_cited` is true, the style
  has no custom bibliography groups, and the run has no active
  compound-numeric groups — those three cases keep the historical two-pass
  render because their semantics genuinely need it (group-local
  disambiguation/templates; compound merging needs every configured group
  member, cited or not; the unrestricted all-refs path needs the whole
  library). Regression-tested directly (`test_render_document_bibliography_compound_groups_use_full_render`).
- Benchmark: `cargo bench --bench rendering -- "Process Document Bibliography"`
  on a 400-loaded/10-cited APA document, driven through the public
  `process_document` entry point: **~15.87 ms → ~0.97 ms, ~16x faster**
  (before/after measured on the same harness by temporarily reverting just
  the facade change).
- New equivalence/regression tests: large-library exclusion
  (`crates/citum-engine/tests/document.rs::flat_and_partitioned_bibliography_fast_path`),
  substitution across an uncited gap, and — the highest-risk case —
  subsequent-author substitution correctly resetting at each sort-partition
  section boundary via the document facade (not just the pre-existing
  standalone-render test for that behavior).
- All 942 `citum-engine` tests and the full `just pre-commit` gate (1860
  workspace tests) pass. `report-core.js` quality score 0.956 (unchanged
  baseline); `workflow-test.sh styles-legacy/apa.csl` 20/20 citations, 45/46
  bibliography (pre-existing unrelated gap).

Deferred P2 (sorted-ID-spine caching + `build_group_local_hints` cloning,
which affect `render_document_bibliography_blocks`, not this facade) to
follow-up bean `csl26-u2kb`.
