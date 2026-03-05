---
# csl26-zafv
title: Support numeric compound citation styles
status: completed
type: feature
priority: normal
created_at: 2026-03-05T15:58:00Z
updated_at: 2026-03-05T17:52:00Z
---

CSL schema issue #437: numeric compound styles group multiple references under a single citation number with sub-labels (a, b, c...).

Example: "2. a) Zwart KB, et al. (1983) ..., b) van der Klei IJ, et al. (1991) ..."

This is prevalent in chemistry journals and was never supported in CSL 1.0.

Design questions to resolve:
- How does input signal compound grouping? Style-level `processing: numeric-compound`? Explicit field on `Citation`?
- Sub-label rendering: alphabetic (a, b, c) per locale? Configurable?
- Delimiter between sub-items within a compound citation
- Interaction with existing numeric citation numbering

Orthogonal to compound locators (csl26-z4t6, completed).
Architecture plan: docs/architecture/CSL26_ZAFV_NUMERIC_COMPOUND_CITATIONS.md

## Acceptance Criteria

- [x] `CompoundNumericConfig` and `SubLabelStyle` in schema, serde round-trips
- [x] Compound sets modeled as top-level bibliography `sets` (no per-reference `group-key`)
- [x] Compound groups assigned same citation number
- [x] Bibliography renders sub-labeled entries for grouped refs
- [x] Citation rendering supports style-controlled addressing:
  - `subentry: true` => `[2a]`/`[2b]`
  - `subentry: false` => `[2]`
- [x] Integration tests for sets validation, numbering, and bibliography merge behavior

## Final Notes (2026-03-05)

This bean ships the entry-set model (`InputBibliography.sets`) and style-level
compound behavior controls, including `subentry` for whole-group vs sub-item
citation addressing.

mciteplus-style cite-site override behavior remains explicitly out of scope for
this bean and is tracked in `csl26-di0r`.
