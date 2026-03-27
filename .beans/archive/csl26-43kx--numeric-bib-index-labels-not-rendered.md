---
# csl26-43kx
title: Numeric bib index labels not rendered
status: completed
type: bug
priority: high
created_at: 2026-03-27T19:19:19Z
updated_at: 2026-03-27T23:36:45Z
---

Bibliography entries for numeric styles (karger-journals, thieme-german, institute-of-physics-numeric, multidisciplinary-digital-publishing-institute) are missing the citation number label ([21], 21, etc.) in Citum output. Oracle shows e.g. '[21] Johnson D, Lee S...' but csln renders 'Johnson D, Lee S...'. Discovered during migrate-research session-3 corpus measurement. All affected styles score 32/33 or 31/33 on the bibliography fixture; the single/double miss is the patent or legal-case entry that also requires the index label.

## Summary of Changes

Added citation-number component (wrap: brackets, suffix: ' ') as first bibliography template item in four numeric styles: karger-journals, thieme-german, institute-of-physics-numeric, and mdpi. Oracle bibliography scores: all four styles now pass with 33/33 bibliography matches.
