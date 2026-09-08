---
# csl26-zgdf
title: 'Cited-date bracket: date-form/case mismatches for NLM/springer/CSE'
status: completed
type: bug
priority: low
tags:
    - style
    - fidelity
created_at: 2026-09-08T21:21:30Z
updated_at: 2026-09-08T23:50:30Z
parent: csl26-ccdt
---

docs/specs/MEDIUM_DESIGNATOR.md's injected cited-date bracket has two cosmetic gaps found via report-core.js per-entry inspection after implementation: (1) NLM's cited-date-form: full doesn't reproduce CSL's abbreviated form="text" (citum '2024 January 15' vs oracle '2024 Jan 15'); CSE's year-month-day choice is similarly approximate. (2) springer/CSE's [cited ...]/[accessed ...] bracket renders capitalized ([Cited ...]) vs oracle's lowercase, apparently a sentence-initial auto-capitalization interaction specific to that insertion point -- NLM's differently-positioned bracket doesn't show this. Neither blocks the marker/anchor/term correctness (all verified correct); low priority polish.

## Summary of Changes

Both original gaps fixed: (1) DateForm::YearMonthAbbrDay added, reproducing CSL's form="text" shape exactly (year, abbreviated month with periods stripped, day, no comma) -- wired for NLM (cited-date-form) and CSE (cited-date-form). (2) text_case: AsIs forced on the injected cited-date label, fixing CSE's capitalization. Verified via direct CLI rendering: TLIB-SEL-MAP-1 now reads '[accessed 2019 Oct 27]', byte-identical to oracle's cited-date fragment.

The original bean conflated Springer's observed '[Cited ...]' output with this same gap -- it isn't. Springer doesn't use online-access.cited-date-label at all; its bracket comes from a separate, pre-existing pattern.cited-date/pattern.accessed-date message pair untouched by this PR. Split that out as csl26-wt1u.
