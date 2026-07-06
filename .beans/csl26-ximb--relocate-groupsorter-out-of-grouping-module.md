---
# csl26-ximb
title: Relocate GroupSorter out of grouping module
status: todo
type: task
created_at: 2026-07-06T17:26:16Z
updated_at: 2026-07-06T17:26:16Z
---

Follow-up to csl26-b801 (PR #1019). GroupSorter in crates/citum-engine/src/grouping/sorting.rs is now the engine's single sorting stack — its callers are mostly not grouping-related (processor/setup.rs bibliography + citation-item sorting, processor/disambiguation.rs), so both the file location and the type name are historical.

Scope (engine-side only, mechanical):
- Move grouping/sorting.rs to crate root (src/sorting.rs), beside sort_support.rs and sort_partitioning.rs.
- Rename GroupSorter → ReferenceSorter (or Sorter; the name is free again). Update the pub use in grouping/mod.rs and all call sites.

Out of scope, decide separately: renaming the serde-facing schema types (citum_schema::grouping::GroupSort/GroupSortKey) — they touch the style schema surface and docs/schemas, and the mismatch predates csl26-b801 (bibliography.sort already resolves to a GroupSort).
