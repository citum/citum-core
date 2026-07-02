---
# csl26-2uq2
title: SQI sprawl penalty double-counts merged type selectors
status: todo
type: bug
priority: low
created_at: 2026-07-02T23:42:57Z
updated_at: 2026-07-02T23:42:57Z
---

report-core.js's concision metric counts every type in a combined type-variant selector key individually: collectTemplateScopes does variantSelectorCount += typeSelectors.length || 1 (scripts/report-core.js:1681), and sprawlPenalty charges 0.8 per selector past 18 (scripts/report-core.js:2150). A merged 'manuscript, collection' key therefore scores as 2 selectors even though merging is the DRY move — the metric penalizes exactly the consolidation it should reward. Observed on PR #996: chicago-author-date-18th SQI dipped 0.926 -> 0.925 (~0.001) from a functionally-correct 2-type merge that lifted 6 bibliography refs.

Likely fix: count merged selector keys once for sprawl purposes (or discount additional types in one key), keeping the targetBonus math consistent so existing SQI baselines do not jump. Audit other consumers of variantSelectorCount (targetBonus at scripts/report-core.js:2145) before changing semantics.

## Todo
- [ ] Decide counting semantics (1 per key vs discounted) and check baseline impact across embedded styles
- [ ] Implement + update scripts tests
- [ ] Confirm chicago-author-date-18th SQI no longer penalized for the manuscript, collection merge
