---
# csl26-3m45
title: Render EDTF seasons via locale season terms
status: completed
type: task
priority: normal
tags:
    - dates
    - localization
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-05T19:08:34Z
parent: csl26-8m2p
---

EDTF seasons (2023-21 = Spring 2023) silently render as bare years: Edtf::month() returns None for seasons and no engine code reads locale.dates.seasons despite en-US shipping four season names. Map seasons 21-24 through locale.dates.seasons wherever months resolve, or emit a structured warning while unsupported. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 6.

## Summary of Changes

- Added `Edtf::season()` accessor (`crates/citum-edtf/src/lib.rs`), mirroring `month()`, returning a 1-based index (1=Spring..4=Winter) for EDTF season codes 21-24, `None` otherwise.
- `extract_month` in `crates/citum-engine/src/values/date.rs` now falls back to `edtf.season()` resolved against `locale.dates.seasons` when `edtf.month()` is `None`. All 7 call sites in `format_single_date` (Month, YearMonth, MonthDay, Full, YearMonthDay, DayMonthAbbrYear, MonthAbbrDayYear) now pass the locale's seasons list.
- Closed-range formatting (`format_closed_range`/`format_same_year_range`) required no changes — it already treats `month_or_season.is_some()` as "has a month" and delegates to `format_single_date`, so same-year season ranges (e.g. \"Spring–Summer 2023\") collapse correctly for free.
- Added unit tests: 3 new `citum-edtf` tests for the `season()` accessor (bare season date, interval start, non-season date returns `None`), and 5 new `citum-engine` tests covering Month/YearMonth/Full forms in en-US, a localized (es-ES) season term, and a same-year season range collapse.
- `2023-21` now renders \"Spring 2023\" (en-US) matching citeproc-js instead of silently dropping to \"2023\".
