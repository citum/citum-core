---
# csl26-v6ok
title: Locale-authored date patterns for inflected languages
status: completed
type: feature
priority: normal
tags:
    - dates
    - multilingual
created_at: 2026-05-16T10:47:07Z
updated_at: 2026-05-18T00:43:07Z
---

## Goal

Let locales author date assembly in MF2 so non-English languages can express their own year/month/day order and morphology. Basque is the motivating example (CSL upstream issue [#6369](https://github.com/citation-style-language/styles/issues/6369), csln prototype issue [#107](https://github.com/bdarcus/csln/issues/107)); the same gap applies to other inflecting languages (Finnish, Hungarian, …).

Today the engine hard-codes English assembly (`format!("{month} {d}, {year}")`) in `crates/citum-engine/src/values/date.rs::format_single_date`. This bean introduces `pattern.date-*` message IDs so a locale can override that assembly. The current English path becomes the fallback when no pattern is authored, so all existing locales/styles stay bit-identical.

## Scope (this PR)

Two `DateForm` variants only: `Full` and `MonthDay`. Other variants (`YearMonth`, `YearMonthDay`, `DayMonthAbbrYear`, `MonthAbbrDayYear`) keep their existing hardcoded assembly. The architecture extends trivially when needed.

## Todo

- [x] Extend `MessageArgs` with `year` / `month` / `day` slots; teach `Mf2MessageEvaluator` to substitute `{$year}` / `{$month}` / `{$day}`
- [x] Add unit tests for the three new variables (alongside existing `$count`/`$gender` tests)
- [x] Wire pattern.date-full into format_single_date DateForm Full (fallback unchanged)
- [x] Wire pattern.date-month-day into format_single_date DateForm MonthDay (fallback unchanged)
- [x] Add locales/eu-ES.yaml (provisional) and Spanish pattern.date-* as second worked example
- [x] Update docs/specs/LOCALE_MESSAGES.md (section 2 namespace plus new subsection; bumped to 1.4)
- [x] Update docs/guides/AUTHORING_LOCALES.md (date-patterns subsection)
- [x] Add unit tests: en-US regression plus es-ES and eu-ES MF2 path plus missing-component fallback (7 new tests)
- [x] Pre-commit gate: fmt clean, clippy clean, 1293/1293 tests pass
- [x] Portfolio quality gate: 154 styles at fidelity 1.0, 0 warnings
- [x] Open PR, request Basque-speaker review for `eu-ES.yaml` content

## Implementation note — Spanish bonus

Added `pattern.date-full` and `pattern.date-month-day` to `es-ES.yaml` as a second worked example. This flips Spanish bibliography date rendering from `enero 12, 2023` (en-US fallback assembly, incorrect) to `12 de enero de 2023` (correct, matches Spanish APA convention and the already-declared `date-formats.bib-default` CLDR pattern). Portfolio gate confirms zero regressions across 154 styles.

## Plan reference

`/Users/brucedarcus/.claude/plans/i-created-the-vast-emerson.md`

## Related

- `csl26-qrpo` — ICU4X swap (this PR's pre-formatted approach is the spec'd path until ICU4X stabilizes `:citum-date`)
- `docs/specs/LOCALE_MESSAGES.md` §1.5 — anticipates exactly this seam

## Follow-ups

- `csl26-37jv` — Wire remaining `DateForm` variants (`YearMonth`, `YearMonthDay`, abbr-month forms) to `pattern.date-*`
- `csl26-hqy5` — Custom locale file invocation for builtin-alias styles (surfaced by this PR's smoke test)
- `csl26-1b4e` — Title / proper-noun inflection across languages (the broader CSL #6369 ask; draft, needs design)
- `csl26-dno4` — Per-case month-form maps (`dates.months.<case>`) (draft, defer until a real locale needs it)
