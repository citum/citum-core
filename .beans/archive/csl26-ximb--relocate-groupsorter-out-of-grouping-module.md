---
# csl26-ximb
title: Relocate GroupSorter out of grouping module
status: completed
type: task
priority: normal
created_at: 2026-07-06T17:26:16Z
updated_at: 2026-07-06T17:59:17Z
---

Follow-up to csl26-b801 (PR #1019). GroupSorter in crates/citum-engine/src/grouping/sorting.rs is now the engine's single sorting stack — its callers are mostly not grouping-related (processor/setup.rs bibliography + citation-item sorting, processor/disambiguation.rs), so both the file location and the type name are historical.

Scope (engine-side only, mechanical):
- Move grouping/sorting.rs to crate root (src/sorting.rs), beside sort_support.rs and sort_partitioning.rs.
- Rename GroupSorter → ReferenceSorter (or Sorter; the name is free again). Update the pub use in grouping/mod.rs and all call sites.

Out of scope, decide separately: renaming the serde-facing schema types (citum_schema::grouping::GroupSort/GroupSortKey) — they touch the style schema surface and docs/schemas, and the mismatch predates csl26-b801 (bibliography.sort already resolves to a GroupSort).

## Summary of Changes

Moved crates/citum-engine/src/grouping/sorting.rs to crates/citum-engine/src/sorting.rs (crate root, beside sort_support.rs / sort_partitioning.rs) and renamed GroupSorter to ReferenceSorter. lib.rs now declares `pub mod sorting;` and re-exports ReferenceSorter at the crate root; grouping/mod.rs no longer owns sorting. Updated all consumers (processor/setup.rs, processor/disambiguation.rs, processor/bibliography/grouping.rs, benches/rendering.rs incl. the benchmark-group label). Schema types (citum_schema::grouping::GroupSort/GroupSortKey) untouched per scope. Pure move+rename: 1814/1814 tests, clippy clean, apa.csl fidelity unchanged.
