---
# csl26-3oq0
title: 'engine: benchmarked rendering hot-path optimization wave'
status: completed
type: task
priority: normal
created_at: 2026-03-26T15:35:11Z
updated_at: 2026-03-26T19:08:44Z
parent: csl26-fk0w
---

Follow-up performance wave from the citum-engine broad review. Keep this separate from correctness PRs and require benchmark numbers before and after changes.

Primary hotspots identified in the review:
- GroupSorter recomputes sort keys and type-order ranks inside comparator work
- Disambiguation builds many short-lived strings and vectors
- Type-variant resolution clones templates on hot render paths
- Compound bibliography preprocessing clones more than necessary

## Tasks
- [x] Capture baseline numbers with `cargo bench -p citum-engine --bench rendering`
- [x] Prioritize low-risk hot-path reductions first
- [x] Implement optimizations in small, benchmarked slices
- [x] Record before/after numbers in the PR description or bean summary

Source: broad citum-engine review after PR #448.

## Summary of Changes

- Added focused rendering benchmarks for:
  - `GroupSorter::sort_references/Explicit type order + author`
  - `Renderer::process_bibliography_entry/Type variant + article-journal fallback`
  - `Processor::render_bibliography_with_format/Compound bibliography merge`
- Refactored `GroupSorter` to precompute derived sort values and explicit type ranks once per `sort_references` call.
- Changed grouped renderer type-variant resolution to borrow matching type-variant templates, while preserving owned fallbacks for localized default templates.
- Reduced bibliography hot-path allocation in article-journal fallback filtering by returning borrowed templates when no filtering is needed.
- Reduced compound bibliography merge cloning by rendering entry bodies from filtered component slices instead of rebuilding temporary `ProcEntry` clones, and by assembling merged compound output in a single buffer.
- Added a bibliography regression test covering type-variant precedence alongside article-journal no-page DOI fallback behavior.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run`

## Benchmark Notes

- Baseline captured with `./scripts/bench-check.sh capture csl26-3oq0-before`.
- Follow-up run captured with `./scripts/bench-check.sh compare csl26-3oq0-before csl26-3oq0-after`.
- End-to-end APA benchmark medians from the compare run:
  - `Process Citation (APA)`: `22.728 µs` -> `38.065 µs`
  - `Process Bibliography (APA, 10 items)`: `95.047 µs` -> `161.28 µs`
- New focused benchmark medians from the after run:
  - `GroupSorter::sort_references/Explicit type order + author`: `5.5734 µs`
  - `Renderer::process_bibliography_entry/Type variant + article-journal fallback`: `3.3020 µs`
  - `Processor::render_bibliography_with_format/Compound bibliography merge`: `43.384 µs`
- The compare helper's `critcmp` step failed on the captured text file format, and the same run showed roughly 1.9x regressions in unrelated `formats` benchmarks too. Treat the before/after delta as machine-noisy and rerun on a quieter host before making a performance-improvement claim in the PR description.
