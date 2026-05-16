---
# csl26-v6ok
title: Locale-authored date patterns for inflected languages
status: in-progress
type: feature
priority: normal
created_at: 2026-05-16T10:47:07Z
updated_at: 2026-05-16T10:47:07Z
---

## Goal

Let locales author date assembly in MF2 so non-English languages can express their own year/month/day order and morphology. Basque is the motivating example (CSL upstream issue [#6369](https://github.com/citation-style-language/styles/issues/6369), csln prototype issue [#107](https://github.com/bdarcus/csln/issues/107)); the same gap applies to other inflecting languages (Finnish, Hungarian, …).

Today the engine hard-codes English assembly (`format!("{month} {d}, {year}")`) in `crates/citum-engine/src/values/date.rs::format_single_date`. This bean introduces `pattern.date-*` message IDs so a locale can override that assembly. The current English path becomes the fallback when no pattern is authored, so all existing locales/styles stay bit-identical.

## Scope (this PR)

Two `DateForm` variants only: `Full` and `MonthDay`. Other variants (`YearMonth`, `YearMonthDay`, `DayMonthAbbrYear`, `MonthAbbrDayYear`) keep their existing hardcoded assembly. The architecture extends trivially when needed.

## Todo

- [ ] Extend `MessageArgs` with `year` / `month` / `day` slots; teach `Mf2MessageEvaluator` to substitute `{$year}` / `{$month}` / `{$day}`
- [ ] Add unit tests for the three new variables (alongside existing `$count`/`$gender` tests)
- [ ] Wire `pattern.date-full` into `format_single_date::DateForm::Full` (fallback unchanged)
- [ ] Wire `pattern.date-month-day` into `format_single_date::DateForm::MonthDay` (fallback unchanged)
- [ ] Add `locales/eu-ES.yaml` with Apertium-attested `YYYYko {month}ren {day}a` shape — **mark provisional pending native-speaker review**
- [ ] Update `docs/specs/LOCALE_MESSAGES.md` (§2 namespace + new subsection; bump version)
- [ ] Update `docs/guides/AUTHORING_LOCALES.md` (date-patterns subsection)
- [ ] Add integration test: Basque APA render vs. en-US APA regression
- [ ] Pre-commit gate: `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run`
- [ ] Portfolio quality gate: `report-core.js` + `check-core-quality.js` show zero regressions
- [ ] Open PR, request Basque-speaker review for `eu-ES.yaml` content

## Plan reference

`/Users/brucedarcus/.claude/plans/i-created-the-vast-emerson.md`

## Related

- `csl26-qrpo` — ICU4X swap (this PR's pre-formatted approach is the spec'd path until ICU4X stabilizes `:citum-date`)
- `docs/specs/LOCALE_MESSAGES.md` §1.5 — anticipates exactly this seam

## Follow-ups (separate beans, do not bundle)

- Title / proper-noun inflection (the broader cross-language ask in CSL #6369) — needs design
- Remaining `DateForm` variants (`YearMonth`, `YearMonthDay`, abbr-month forms) once a locale needs them
- Month-form variants (`dates.months.{nominative,locative,…}`) if a single locale ever needs more than one inflected form
