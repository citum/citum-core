---
# csl26-k6ty
title: Route date ranges through format_single_date
status: completed
type: task
priority: normal
tags:
    - dates
    - rendering
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T20:39:43Z
parent: csl26-8m2p
---

format_range_start is a ~100-line near-duplicate of format_single_date without locale pattern resolution, and extract_range_end hardcodes English month-day-year assembly with long month names. Route both range endpoints through format_single_date (year-suppressed variant for same-year ends) and delete the duplicate. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 5.

## Summary of Changes

Deleted format_range_start (~100-line duplicate without locale patterns) and
extract_range_end (hardcoded English month-day-year assembly). Both range
endpoints now route through format_single_date, so locale date patterns
(e.g. es-ES day-first) apply symmetrically. Same-year closed ranges collapse
the repeated year (May 14–June 2, 2023); the open-ended-from-start form
(../2020) no longer renders the buggy 2020–2020. Added a range_tests module
covering regression, collapse, locale-pattern, and IntervalTo cases.
