---
# csl26-v3b2
title: Document select:first, locator label-case/attach, online-access in author docs
status: completed
type: task
priority: normal
tags:
    - docs
    - styles
created_at: 2026-09-20T12:35:28Z
updated_at: 2026-09-20T12:39:27Z
---

Three author-facing features landed Sep 6-10 (706a6c0ca select:first groups, faf18fd9b locator label-case/attach, d383f95b1 online-access designator) with no coverage in docs/guides/style-author-guide.md or docs/guides/style-authoring/*.html. Add worked-example documentation to both surfaces, plus the group: primitive lead-in (undocumented prerequisite) and term.place-unknown to AUTHORING_LOCALES.md.

## Summary of Changes

- Added a **Groups and First-Match Fallback** section to `style-author-guide.md` (`group:` primitive lead-in + `select: first`, both worked examples from apa-7th/chicago-author-date-18th), with a matching sidebar entry and cross-links from Substitution/Missing Dates.
- Added a **Locators** subsection under Global Options (preset table, `label-case`, `attach`, APA/MLA examples) and added `year-month-abbr-day` to the Date form list.
- Added an **Online Access Designator** subsection under Scoped Options (table row + full field docs + the Springer per-type-exclusion limitation, sourced from MEDIUM_DESIGNATOR.md).
- Mirrored all three in `docs/guides/style-authoring/templates.html` (Groups) and `options.html` (Locator Rendering, Online Access Designator).
- Added `term.place-unknown`, `term.cited`, `term.internet` rows to `AUTHORING_LOCALES.md`'s message ID catalog.
- Verified via `build-doc-pages.js` + `build-author-guide.js` + `check-doc-links.js` (no broken links) + `alint check` (29/29 passed).
