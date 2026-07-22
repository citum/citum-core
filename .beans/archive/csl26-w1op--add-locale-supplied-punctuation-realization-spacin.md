---
# csl26-w1op
title: Add locale-supplied punctuation realization spacing
status: completed
type: feature
priority: normal
tags:
    - multilingual
    - punctuation
    - locale
created_at: 2026-07-22T15:03:35Z
updated_at: 2026-07-22T17:54:26Z
parent: csl26-0ugp
---

Add locale-owned realization strings for semantic punctuation, including French and Québec NBSP/narrow-NBSP spacing variants.

Resolution order is: explicit style `scripts.*.realization` override, effective item-locale realization, then the selected punctuation-width preset or engine default. The feature must respect both `term-locale: style` and `term-locale: item`.

Quote-glyph selection and punctuation normalization/collision policy are out of scope.

## Acceptance Criteria

- [x] Locale realization strings select from the effective locale.
- [x] Style realization overrides take precedence over locale entries.
- [x] Missing locale entries fall back to the selected preset or engine default.
- [x] French/Québec spacing variants render correctly under both term-locale modes.

## Summary of Changes

Implemented locale-owned semantic punctuation realization with style override precedence, preset/engine fallback, and fr-FR/fr-CA spacing variants under both term-locale modes.
