---
# csl26-dw4u
title: Investigate note full-citation locator grouping in MHRA and New Hart's note styles
status: todo
type: bug
priority: high
tags:
    - style-evolve
    - note-styles
created_at: 2026-03-27T00:05:30Z
updated_at: 2026-03-27T00:05:30Z
---

## Summary
Note-style full citations still mishandle article-journal locator rendering in several migrated styles after the 2026-03-26 threshold-recovery wave.

## Observed styles
- `mhra-notes`
- `mhra-notes-publisher-place`
- `mhra-notes-publisher-place-no-url`
- `new-harts-rules-notes-label-page`
- `new-harts-rules-notes-label-page-no-url`

## Current behavior
For the `et-al-with-locator` oracle scenario, Citum still renders malformed journal detail/grouping around volume, year, and locator placement. Representative output patterns include:
- `Journal of Climate Analytics, 12, (2021),, (p. 205)` in MHRA note styles
- `Journal of Climate Analytics, ), 2/12, (2021)` in New Hart's Rules label-page variants

## Expected behavior
- MHRA note styles should render `12.3 (2021), pp. 201–19 (p. 205)` and retain note-style shortened author output (`John Smith and others`)
- New Hart's Rules label-page variants should render `12/3 (2021), p. 205`

## Classification
- `no applicable divergence found`
- Likely shared `processor-defect` or migrate/template grouping defect in note full-citation rendering for journal detail + locator combinations

## Why deferred
This wave fixed style-local bibliography failures and one Chicago citation gap, but the remaining note-journal+locator issue appears shared across multiple styles and did not yield to bounded YAML-only retries. It needs a focused engine/migration investigation rather than more style churn in PR #454.
