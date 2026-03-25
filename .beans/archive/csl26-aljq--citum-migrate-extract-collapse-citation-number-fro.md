---
# csl26-aljq
title: 'citum_migrate: extract collapse: citation-number from CSL'
status: completed
type: feature
priority: high
created_at: 2026-03-22T20:20:57Z
updated_at: 2026-03-25T21:51:03Z
---

## Summary

`citum_migrate` does not emit `citation.collapse: citation-number` when the
source CSL has `<citation collapse="citation-number">`. The field is silently
dropped during migration, requiring a manual style-defect fix after every
numeric migration.

## Reproduction

Migrate any numeric CSL style that uses `collapse="citation-number"` (e.g.,
`styles-legacy/acm-sig-proceedings.csl`). The resulting Citum YAML has no
`collapse` key under `citation:`.

CSL source:
```xml
<citation collapse="citation-number">
```

Expected output (post-migration):
```yaml
citation:
  collapse: citation-number
```

Actual output: `collapse` key absent from `citation:`.

## Root Cause

The migration pipeline's citation option extraction (likely in `citum_migrate`
upsampler) does not map `collapse="citation-number"` to
`CitationCollapse::CitationNumber`. The schema already supports this value —
it's a pipeline omission, not a schema gap.

## Impact

Affects all CSL-migrated numeric styles that declare citation collapse:
ACM variants, IEEE variants, ACS, NLM, and others. Each requires a manual
post-migration fix currently caught only by oracle comparison or style review.

## Fix Direction

In `citum_migrate`: during citation options extraction, check for the
`collapse` attribute on `<citation>` and map `"citation-number"` →
`CitationCollapse::CitationNumber` in the output struct.

Found during style-evolve eval run — acm-sig-proceedings migration (2026-03-22).



## Resolution
Implemented in commit `6bd78b8d` on branch `codex/bean-fixes-rqcp-aljq`. `csl-legacy` now parses `<citation collapse="citation-number">` and migration maps it to `citation.collapse: citation-number`, with parser and migration regression coverage.
