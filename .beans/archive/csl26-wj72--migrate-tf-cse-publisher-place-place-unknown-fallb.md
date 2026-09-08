---
# csl26-wj72
title: 'Migrate T&F-CSE publisher-place [place unknown] fallback to select: first'
status: scrapped
type: task
priority: normal
tags:
    - style
    - locale
    - fidelity
created_at: 2026-09-08T15:13:45Z
updated_at: 2026-09-08T19:26:47Z
---

docs/specs/GROUP_SELECT.md's worked-example Acceptance Criteria item. taylor-and-francis-council-of-science-editors-author-date.csl:77-86 uses a literal <text value="[place unknown]"/>, gated on no type (universal). Citum's TemplateComponent has no literal-text variant (only Message/Term, both locale-keyed), so this needs a new locale term added first -- an authored-content decision (term key + English text) requiring user sign-off, not made unilaterally in the select:first engine PR. Once a term exists: migrate the publisher-place macro to select:first: [variable: publisher-place, message: term.<key>], verify via report-core.js diff (expect 0 regressions plus the 7 parity rows the render-when disposition audit identified).

## Reasons for Scrapping

Created as a deferral bean based on an overcautious read (the locale term's text is copied verbatim from the shipped CSL, not an invented content decision). User pushed back ("I don't understand why you don't just use a message there?"), so this was implemented directly in the select:first PR (csl26-tzs3 / GROUP_SELECT.md) instead of deferred. Brackets come from wrap: { punctuation: brackets }, not baked into the term text (second user correction). See csl26-tzs3's Summary of Changes.
