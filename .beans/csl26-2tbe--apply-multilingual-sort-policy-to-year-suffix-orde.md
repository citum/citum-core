---
# csl26-2tbe
title: Apply multilingual sort policy to year suffix ordering
status: todo
type: bug
priority: normal
tags:
    - engine
    - sorting
    - multilingual
created_at: 2026-07-10T21:50:45Z
updated_at: 2026-07-10T21:50:45Z
parent: csl26-8m2p
---

The final bibliography uses `ReferenceSorter::with_bibliography_config`, but
`Disambiguator::sort_group_for_year_suffix` constructs `ReferenceSorter::new`.
As a result, a bibliography configured for `sorting.multilingual: romanized`
or an explicit `sorting.locale` can assign a/b suffixes using original uniform
keys while presenting entries in romanized/locale-configured order.

## Checklist

- [ ] Add a same-author/same-year multilingual fixture whose original and `sort-as` title orders differ
- [ ] Thread the effective bibliography sort-key policy into `Disambiguator`
- [ ] Assert year-suffix order matches final bibliography order
- [ ] Cover group-local disambiguation as well as the global path

Audit: `docs/architecture/audits/2026-07-10_CITUM_ENGINE_FOLLOW_UP_REVIEW.md`
