---
# csl26-6wab
title: 'Fix note-style fidelity: bib 0/N deflates score'
status: completed
type: bug
priority: normal
created_at: 2026-04-12T16:44:33Z
updated_at: 2026-04-12T17:11:20Z
---

Note styles (hasBibliography: false) have oracle bibliography totals of 0/56 included in fidelity calculation, deflating scores (e.g. chicago-notes-18th shows 37.4% instead of 72.9%). Fix computeFidelityScore to skip bibliography when hasBibliography===false, and fix overall aggregation.

## Summary of Changes

- Fixed `computeFidelityScore` to skip bibliography 0/N for note styles (`hasBibliography: false`); chicago-notes-18th corrected from 37.4% → 72.9%
- Fixed portfolio `bibliographyOverall` aggregation to exclude note-style bibliography totals
- Fixed oracle cache key in `runCiteprocSnapshotOracle` to include YAML hash, preventing stale results after style edits
- Fixed `karger-journals-author-date` citation template: `name-form: initials` → `family-only`, added missing `date: issued` component; fidelity 78.4% → 100%
- All other 78-82% styles (ACM variants, Springer, AMA) confirmed at div-004 ceiling — no style-level fix available
