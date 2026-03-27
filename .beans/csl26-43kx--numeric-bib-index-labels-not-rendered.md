---
# csl26-43kx
title: Numeric bib index labels not rendered
status: in-progress
type: bug
priority: high
created_at: 2026-03-27T19:19:19Z
updated_at: 2026-03-27T23:36:07Z
---

Bibliography entries for numeric styles (karger-journals, thieme-german, institute-of-physics-numeric, multidisciplinary-digital-publishing-institute) are missing the citation number label ([21], 21, etc.) in Citum output. Oracle shows e.g. '[21] Johnson D, Lee S...' but csln renders 'Johnson D, Lee S...'. Discovered during migrate-research session-3 corpus measurement. All affected styles score 32/33 or 31/33 on the bibliography fixture; the single/double miss is the patent or legal-case entry that also requires the index label.
