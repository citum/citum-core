---
# csl26-aals
title: 'citum_migrate: extract global et-al shorten to options.contributors'
status: todo
type: feature
priority: high
created_at: 2026-03-22T20:21:01Z
updated_at: 2026-03-22T20:21:01Z
---

## Summary

When a CSL style applies `et-al-min` / `et-al-use-first` at the bibliography
level, `citum_migrate` writes the threshold into individual type-templates only,
not into `options.contributors.shorten`. Entry types without an explicit
template get no truncation at all.

## Reproduction

Migrate `styles-legacy/american-medical-association.csl`. The CSL declares
et-al globally on `<bibliography et-al-min="7" et-al-use-first="3">`.

Expected post-migration:
```yaml
options:
  contributors:
    shorten:
      min: 7
      use-first: 3
      and-others: et-al
```

Actual: `options.contributors` has no `shorten` block. Threshold appears only
inside specific type-template name nodes, leaving other entry types uncapped.

## Root Cause

The bibliography options extraction pass reads `et-al-min` / `et-al-use-first`
from `<bibliography>` but treats them as per-type overrides rather than
detecting that they are declared globally and promoting them to
`options.contributors.shorten`.

## Impact

All CSL-migrated styles where et-al applies globally. Post-migration et-al
behaviour is silently wrong for any entry type without its own explicit template.
Affects numeric and author-date styles alike.

## Fix Direction

In `citum_migrate`'s bibliography options pass: when `et-al-min` /
`et-al-use-first` appear on `<bibliography>` unconditionally (not inside a
type-branch `<if>`), emit `options.contributors.shorten` globally.

Found during style-evolve eval run — AMA and ACM migrations (2026-03-22).

