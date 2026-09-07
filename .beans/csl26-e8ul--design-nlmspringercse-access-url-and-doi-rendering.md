---
# csl26-e8ul
title: Design NLM/springer/CSE access-URL and DOI rendering
status: todo
type: task
priority: normal
tags:
    - style
    - engine
    - schema
    - fidelity
created_at: 2026-09-08T13:18:42Z
updated_at: 2026-09-08T13:18:48Z
parent: csl26-ccdt
blocked_by:
    - csl26-8z39
---

Split out of docs/specs/MEDIUM_DESIGNATOR.md after an adversarial review found the 'access-phrase' bundle wasn't actually shared across the three target styles. Each style's access macro is genuinely different, verified against the shipped CSL:

- NLM (taylor-and-francis-national-library-of-medicine.csl:72-91): if type=article-journal, DOI-if-no-page-volume (csl26-8z39's scope) else nothing; else-if URL, 'Retrieved from: URL'. The phrase never renders for article-journal.
- Springer (springer-vancouver-brackets.csl:88-107): if DOI, doi.org URL; else-if URL, 'URL. Accessed DATE' -- no 'retrieved' wording, a date inline in this macro (separate from the [cited ...] bracket rendered elsewhere).
- CSE (taylor-and-francis-council-of-science-editors-author-date.csl:48-56): if DOI, doi.org URL; else bare URL. No phrase, no date.

This needs its own per-style design (DOI-preference-over-URL is shared across all three; the phrase/date wording is not), entangled with csl26-8z39's DOI-preference logic for NLM specifically. MEDIUM_DESIGNATOR.md's online-access option now covers only the [Internet] marker and cited-date bracket, which are genuinely uniform.
