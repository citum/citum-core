---
# csl26-b801
title: Unify Sorter into GroupSorter with cached keys
status: completed
type: task
priority: normal
tags:
    - sorting
    - performance
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-06T15:47:34Z
parent: csl26-8m2p
---

processor/sorting.rs Sorter recomputes author/title sort keys (collation, article stripping) on every comparison; grouping/sorting.rs GroupSorter already has the cached Schwartzian pattern. Unify the two stacks, dedupe compare_optional_years, and implement or explicitly reject the silent no-op SortKey::CitationNumber. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 8.

## Summary of Changes

Unified the two sorting stacks. Deleted processor/sorting.rs (uncached
`Sorter` + duplicate `compare_optional_years`); the config-level sort now
maps to a `GroupSort` via new `Sort::group_sort()` (Author→Author,
Year→Issued, Title→Title; CitationNumber skipped) and flows through the
cached Schwartzian `GroupSorter`. Legacy semantics preserved: entry-ID
tiebreak (now opt-in `GroupSorter::sort_references_with_id_tiebreak`,
with the ID cached per reference) applies only on the config-sort path,
and a missing `processing:` still defaults to the AuthorDate family sort.
`SortKey::CitationNumber` is now explicitly rejected: explicit (non-preset)
use emits the `citation_number_sort_not_supported` style-load warning via
the api/warnings.rs unknown-enum scan. Audit finding 8.
