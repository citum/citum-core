---
# csl26-b801
title: Unify Sorter into GroupSorter with cached keys
status: todo
type: task
tags:
    - sorting
    - performance
parent: csl26-8m2p
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-04T17:49:02Z
---

processor/sorting.rs Sorter recomputes author/title sort keys (collation, article stripping) on every comparison; grouping/sorting.rs GroupSorter already has the cached Schwartzian pattern. Unify the two stacks, dedupe compare_optional_years, and implement or explicitly reject the silent no-op SortKey::CitationNumber. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 8.
