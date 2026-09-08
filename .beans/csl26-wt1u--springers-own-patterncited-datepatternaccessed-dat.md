---
# csl26-wt1u
title: Springer's own pattern.cited-date/pattern.accessed-date bracket has date-form and capitalization bugs
status: todo
type: bug
priority: low
tags:
    - style
    - fidelity
created_at: 2026-09-08T23:50:05Z
updated_at: 2026-09-08T23:50:05Z
parent: csl26-ccdt
---

springer-vancouver-brackets-core.yaml's pre-existing pattern.cited-date/pattern.accessed-date messages (unrelated to docs/specs/MEDIUM_DESIGNATOR.md's online-access option, which springer doesn't use for this bracket) render '[Cited 2024, January 15]' where oracle expects '[cited 2024 Jan 15]' -- wrong capitalization, wrong date form (long month + comma instead of abbreviated no-comma), and a stray period before the bracket ('2023.' vs '2023'). Found via report-core.js per-entry inspection while verifying an adversarial-review fix to the (separate) online-access cited-date bracket for NLM/CSE -- confirmed this is Springer's own, independent, pre-existing code path (this style has never set online-access.cited-date-label). Same underlying date-form gap as MEDIUM_DESIGNATOR.md's now-fixed DateForm::YearMonthAbbrDay -- likely fixable by migrating pattern.cited-date/pattern.accessed-date onto that same DateForm, but the capitalization/period issues need their own investigation.
