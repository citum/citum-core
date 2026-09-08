---
# csl26-r4dn
title: Implement online-access medium designator (MEDIUM_DESIGNATOR.md)
status: completed
type: feature
priority: high
tags:
    - style
    - schema
    - fidelity
created_at: 2026-09-08T20:33:15Z
updated_at: 2026-09-08T21:23:40Z
parent: csl26-ccdt
blocking:
    - csl26-zs9y
---

Implement docs/specs/MEDIUM_DESIGNATOR.md: OnlineAccessConfig bibliography option (medium-marker + cited-date-label/form), anchor-selection resolving container-title vs own-title (including the legal-type flat-reporter edge case), and wiring taylor-and-francis-national-library-of-medicine, springer-vancouver-brackets, and taylor-and-francis-council-of-science-editors-author-date.

## Todo
- [x] OnlineAccessConfig added to BibliographyOptions, BibliographyConfig, to_bibliography_config()
- [x] medium_marker/cited_date_label reject bare-scalar input
- [x] Anchor selection resolves legal-type edge case (container_title_is_embedded_work)
- [x] Exact-output fixtures per style ([cited ...] NLM/springer, [accessed ...] CSE)
- [x] All three embedded styles wired; report-core.js diff: 0 regressions, 4 rows flip positive
- [x] just schema-gen run
- [x] Status promoted to Active in MEDIUM_DESIGNATOR.md (v1.1)

## Summary of Changes

Implemented docs/specs/MEDIUM_DESIGNATOR.md: OnlineAccessConfig bibliography option (medium-marker + cited-date-label/form), engine-injected via a new apply_online_access_bibliography_policy (mirrors the existing article-journal/anonymous-entries template-rewrite pattern), anchor selection resolving the legal-type flat-reporter edge case via a new container_title_is_embedded_work predicate, and wiring all three target styles. Found and fixed two spec inaccuracies during implementation: term.internet doesn't resolve via the generic term.X locale fallback (unlike term.cited/term.accessed) and needed a direct messages: entry; cited-date-form: text has no DateForm equivalent. report-core.js per-entry diff: 7 entries flip from failing to passing across the 3 styles, 0 regressions in the 35-style corpus. Filed csl26-zgdf for a residual cosmetic date-form/capitalization gap in the injected cited-date bracket.
