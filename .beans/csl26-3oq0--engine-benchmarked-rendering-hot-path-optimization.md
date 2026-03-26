---
# csl26-3oq0
title: 'engine: benchmarked rendering hot-path optimization wave'
status: todo
type: task
priority: normal
created_at: 2026-03-26T15:35:11Z
updated_at: 2026-03-26T15:35:11Z
parent: csl26-fk0w
---

Follow-up performance wave from the citum-engine broad review. Keep this separate from correctness PRs and require benchmark numbers before and after changes.

Primary hotspots identified in the review:
- GroupSorter recomputes sort keys and type-order ranks inside comparator work
- Disambiguation builds many short-lived strings and vectors
- Type-variant resolution clones templates on hot render paths
- Compound bibliography preprocessing clones more than necessary

## Tasks
- [ ] Capture baseline numbers with `cargo bench -p citum-engine --bench rendering`
- [ ] Prioritize low-risk hot-path reductions first
- [ ] Implement optimizations in small, benchmarked slices
- [ ] Record before/after numbers in the PR description or bean summary

Source: broad citum-engine review after PR #448.
