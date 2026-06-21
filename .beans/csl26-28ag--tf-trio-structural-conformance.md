---
# csl26-28ag
title: T&F trio structural conformance
status: todo
type: feature
priority: normal
created_at: 2026-06-21T10:49:22Z
updated_at: 2026-06-21T10:49:22Z
---

Finish structural conformance for the three embedded Taylor & Francis styles against the official PDFs (local-only at /tmp/tf/; not in-repo). Detail per-style in docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md (T&F section).

- NLM (tf_nlm.pdf): journal needs the `Year;Vol(Iss):pages` regroup and book/chapter `Place: Publisher; Year` reorder (same group shape as the AMA journal group). Author period + chapter `In: <names>, editors. <Book>` already landed in PR #946.
- CSE / Style C (tf_c.pdf): chapter `In:`/editors structure and journal punctuation (`37, (1); 1–13` -> `37(1):1–13`), author period.
- Chicago / Style F (tf_f.pdf): chapter title period-in-quote, redundant `(eds.)`, `: 683–703:` colons, journal `(year)`.

DECISION NEEDED (bucket B): T&F Style F prescribes sentence-case, UNQUOTED article titles; both Citum and the citeproc CSL render Title Case in quotes. Adopting the guide is an intentional divergence from the CSL reference — needs explicit sign-off before implementing.

Follow-up from the guide-conformance sweep (csl26-53zy / PR #946).
