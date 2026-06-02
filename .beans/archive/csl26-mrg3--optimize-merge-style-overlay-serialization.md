---
# csl26-mrg3
title: Optimize merge_style_overlay to avoid serialization round-trips
status: completed
type: task
priority: low
created_at: 2026-05-09T00:00:00Z
updated_at: 2026-06-02T10:49:35Z
---

The current implementation of `merge_style_overlay` in `citum-schema-style` relies
on re-serializing data structures (JSON/YAML) to perform deep merges. While
functional and flexible, this adds significant overhead to the style resolution
process.

## Scope

- Evaluate and implement a strongly-typed deep merge strategy for Citum styles
  that avoids intermediate serialization.
- The solution must account for the project's multi-format support, which
  includes JSON, YAML, and CBOR.
- Consider utilizing a custom macro or a specialized `Merge` trait to handle
  the recursive merging of templates and variants.
- Baseline the performance of the current serialization-based approach against
  the new implementation to verify gains.

## Summary of Changes

Replaced all serde_yaml round-trips in `merge_style_overlay` with typed field merges:

- `merge_info`: uses `merge_options!` macro over StyleInfo Option fields
- `merge_citation_spec` / `merge_bibliography_spec`: per-field Option merge, per-key
  merge for `type_variants` (IndexMap) and `templates`/`custom` (HashMap)
- `CitationOptions::merge` (already existed) and new `BibliographyOptions::merge` wired up
- Null-aware clearing (e.g. `ibid: ~`) preserved via targeted raw_yaml inspection
- Removed `merge_serialized`, `merge_serialized_value`, `merge_yaml_value` and all
  associated `#[allow(unwrap_used/expect_used)]` escape hatches
- Added regression test for per-key type_variants preservation (fixed silent bug)
- Added Criterion bench: `Style Resolution/merge_style_overlay`

**Bench results (Apple M-series, optimized build):**
- Before (serde round-trip): 36.5 µs
- After (typed merge): 31.3 µs
- Gain: ~14% faster

**Portfolio quality gate:** 154 styles, fidelity=1.0 (no regressions)

## Rationale

Eliminating the serialization round-trip during style merging will reduce
latency, particularly in the `citum-server` interactive mode where styles
may be resolved or overlaid frequently.

## Summary of Changes

Work completed in 98495bd. Dropped three serde_yaml serialization round-trips in merge_style_overlay in favour of typed per-field merges. Added BibliographyOptions::merge, a Criterion bench (36.5 µs → 31.3 µs, ~14% gain), and a regression test.
