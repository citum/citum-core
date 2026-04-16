---
# csl26-d59c
title: 'engine: benchmarked disambiguation hot-path optimization'
status: completed
type: task
priority: normal
created_at: 2026-03-26T19:21:54Z
updated_at: 2026-04-16T18:43:23Z
parent: csl26-fk0w
---

Follow-up performance slice from csl26-3oq0.

Scope this bean to disambiguation hot-path allocation work that was explicitly deferred from the low-risk rendering optimization PR. Require benchmark numbers before and after changes, and keep the work separate from correctness fixes unless a benchmarked refactor exposes a behavioral regression that must be fixed in the same slice.

Primary hotspot to target:
- Disambiguation builds many short-lived strings and vectors

## Tasks
- [x] Capture a fresh baseline for disambiguation-heavy rendering scenarios
- [x] Identify the highest-allocation disambiguation path with focused benchmarks or profiling
- [x] Implement a benchmarked optimization slice without changing rendering semantics
- [x] Record before/after numbers in the PR description or bean summary

Parent context: csl26-fk0w
Deferred from: csl26-3oq0

## Summary of Changes

Reduced allocations in `Disambiguator::calculate_hints` hot path by
applying scratch-buffer reuse to two inner loops in
`crates/citum-engine/src/processor/disambiguation.rs`:

- `check_givenname_resolution` now builds each collision key into a
  single reused `String` buffer, cloning into the `seen` HashSet only on
  actual insertion (via `std::mem::take` after a `contains` pre-check).
- `partition_by_name_expansion` uses the existing
  `append_name_expansion_key` to write into a reused buffer, then a
  `get_mut` + fallback pattern to avoid a fresh allocation when the key
  already exists in the partition map.

Removed the now-unused wrappers `make_name_expansion_key` and
`make_givenname_resolution_key`; callers write into the mutable buffer
directly.

No semantic changes — identical disambiguation output.

## Benchmark Notes

Captured with `./scripts/bench-check.sh capture csl26-d59c-before`, then
`capture csl26-d59c-after` after the scratch-buffer commit. Criterion
p-values and change intervals below are from the after run (same host).

Targeted disambiguation benches (all p = 0.00 < 0.05):

| Scenario                                  | Before     | After      | Change |
|-------------------------------------------|------------|------------|--------|
| No collisions                             | 3.73 µs    | 2.49 µs    | -45.4% |
| Given-name collisions                     | 1.64 µs    | 1.53 µs    | -26.9% |
| Name partition + suffix fallback          | 3.23 µs    | 2.78 µs    |  -7.2% |
| Label-mode suffix collisions              | 2.81 µs    | 1.64 µs    | -37.0% |
| Default title-order suffix collisions     | 2.16 µs    | 1.78 µs    | -31.0% |

End-to-end benches (include disambiguation on the path):

| Scenario                                  | Change |
|-------------------------------------------|--------|
| Process Citation (APA)                    | -49.3% |
| Process Bibliography (APA, 10 items)      | -52.0% |

Unrelated benches were at or near the noise floor:

- `Processor::render_bibliography_with_format/Compound bibliography merge`: p = 0.57 (no change)
- `Style Deserialization/YAML`: p = 0.76 (no change)
- Other format/deserialization benches swung by a few percent on the
  same host; csl26-3oq0 previously documented that these can drift with
  machine state and should be treated as noise for this PR.

Deferred: `HashMap<String, _>` → `HashMap<&str, _>` in
`group_references` (requires lifetime plumbing through
`ReferenceCache`) was left for a follow-up slice. Current gains were
large enough to ship without it.
