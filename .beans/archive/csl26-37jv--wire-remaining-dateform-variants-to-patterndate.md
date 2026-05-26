---
# csl26-37jv
title: Wire remaining DateForm variants to pattern.date-*
status: completed
type: task
priority: low
tags:
    - dates
created_at: 2026-05-16T11:28:27Z
updated_at: 2026-05-26T20:14:22Z
---

## Goal

Wire the four `DateForm` variants left untouched by `csl26-v6ok` into the locale-authored `pattern.date-*` mechanism. The spec already reserves the IDs (`docs/specs/LOCALE_MESSAGES.md` §1.5); only the engine plumbing in `crates/citum-engine/src/values/date.rs::format_single_date` is missing.

## Variants

| `DateForm` | Reserved message ID | Notes |
|---|---|---|
| `YearMonth` | `pattern.date-year-month` | English fallback `{month} {year}` — used in author-date in-text citations |
| `YearMonthDay` | `pattern.date-year-month-day` | English fallback `{year}, {month} {day}` |
| `DayMonthAbbrYear` | `pattern.date-day-month-abbr-year` | Uses short-month list |
| `MonthAbbrDayYear` | `pattern.date-month-abbr-day-year` | Uses short-month list |

## Todo

- [x] Wire each `DateForm` arm in `format_single_date` to consult the matching `pattern.date-*` (mirror the `Full` / `MonthDay` graft)
- [x] Author `pattern.date-year-month` for `es-ES` (`"{$month} de {$year}"`) and `eu-ES` (`"{$year}ko {$month}"`)
- [x] Add unit tests in `date.rs::locale_pattern_tests` covering each new variant + missing-component fallback
- [x] Verify portfolio quality gate stays at fidelity 1.0

## Trigger

This bean is low priority until a locale actually authors one of the reserved IDs and needs the engine to consume it. The motivating case in `csl26-v6ok`'s smoke test (`(2023, ekaina)` — mixing Basque month with English assembly) is a visible defect that would justify pulling this bean forward.

## Related

- Parent feature: `csl26-v6ok`
- Spec: `docs/specs/LOCALE_MESSAGES.md` §1.5

## Summary of Changes

Wired `YearMonth`, `YearMonthDay`, `DayMonthAbbrYear`, and `MonthAbbrDayYear` variants in `format_single_date` to consult locale `pattern.date-*` messages before falling through to hardcoded English assembly — matching the existing `Full` / `MonthDay` pattern. Added `pattern.date-year-month` to `es-ES` and `eu-ES` locales. Added 7 new unit tests in `locale_pattern_tests`. Updated `docs/specs/LOCALE_MESSAGES.md` to mark all six IDs as Active. Archived csl26-ubya (GitResolver wiring was already complete). All 1400 tests pass; portfolio quality gate holds.
