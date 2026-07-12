---
# csl26-2tbe
title: Apply multilingual sort policy to year suffix ordering
status: completed
type: bug
priority: normal
tags:
    - engine
    - sorting
    - multilingual
created_at: 2026-07-10T21:50:45Z
updated_at: 2026-07-12T12:23:04Z
parent: csl26-8m2p
---

The final bibliography uses `ReferenceSorter::with_bibliography_config`, but
`Disambiguator::sort_group_for_year_suffix` constructs `ReferenceSorter::new`.
As a result, a bibliography configured for `sorting.multilingual: romanized`
or an explicit `sorting.locale` can assign a/b suffixes using original uniform
keys while presenting entries in romanized/locale-configured order.

## Checklist

- [x] Add a same-author/same-year multilingual fixture whose original and `sort-as` title orders differ
- [x] Thread the effective bibliography sort-key policy into `Disambiguator`
- [x] Assert year-suffix order matches final bibliography order
- [x] Cover group-local disambiguation as well as the global path

Audit: `docs/architecture/audits/2026-07-10_CITUM_ENGINE_FOLLOW_UP_REVIEW.md`

## Summary of Changes

Threaded the effective bibliography sort-key config into `Disambiguator` as a new `sort_config` field, separate from the existing citation/name-rule `config`. `sort_group_for_year_suffix` now builds its `ReferenceSorter` via `with_bibliography_config(locale, sort_config)` instead of the uniform `ReferenceSorter::new`, and the year-suffix tiebreak title key now goes through `title_sort_key_with_options` with `SortKeyOptions::from_config(sort_config)` instead of the uniform helper (which is now dead and was removed).

All three call sites now pass the effective bibliography config as `sort_config`: the global path (`processor/setup.rs::calculate_hints`), the group-local path (`processor/bibliography/grouping.rs::build_group_local_hints`), and the by-cite givenname overlay (`processor/citation.rs::citation_scoped_by_cite_hints`).

Added two regression tests in `disambiguation.rs`: a same-author/same-year multilingual fixture (`multilingual_year_suffix_pair`) whose Cyrillic-original and romanized `sort-as` title orders are deliberately opposite, asserting year-suffix `group_index` follows the romanized order (and ties that assertion to `ReferenceSorter::with_bibliography_config` directly, the same sorter the final bibliography uses), plus a contrasting uniform-policy assertion to pin the regression. A second test reproduces the group-local call-site pattern to confirm the fix applies there too. Verified both tests fail against the pre-fix behavior (temporarily reverted, ran, confirmed failure, restored) before finalizing.

All pre-existing `Disambiguator::new`/`with_group_sort` call sites (test module + `benches/rendering.rs`) updated for the new constructor arity. Full workspace `just pre-commit` (fmt, clippy -D warnings, 1904 tests) passes.
