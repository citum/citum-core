---
# csl26-qml8
title: Fix editor-only bib rendering in new-harts-rules note styles
status: completed
type: bug
priority: normal
created_at: 2026-03-28T01:25:58Z
updated_at: 2026-03-28T10:24:18Z
---

Bibliography item 32 (editor-only book, no author): oracle expects 'Reis, Harry T., and Charles M. Judd, editors., Handbook of Research Methods...' but Citum outputs the title first with no editors. Fix: add a book/edited-collection bibliography type-variant that leads with contributor: editor form: verb for cases where there is no author. Affects new-harts-rules-notes-label-page and -no-url.

## Summary of Changes

- Fixed article-journal and chapter bib templates: removed spurious publisher-place + broken volume prefix migration artifact; added proper volume/issue rendering with wrap: parentheses for year.
- Fixed interview bib type-variant: changed interviewer from form: long + hardcoded prefix to form: verb (design-correct, localized).
- Fixed citation template interviewer: same form: verb change.
- Added personal_communication / personal-communication suppression to bibliography type-variants (was rendering extra Citum-only entry).
- Divergence div-006 documented for patent author name-order.
- Oracle: 18/18 citations, 32/32 bibliography.
