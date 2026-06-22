---
# csl26-2rs1
title: 'fix(style): AMS-label Wave 3 fixes'
status: completed
type: task
priority: normal
created_at: 2026-06-22T21:52:37Z
updated_at: 2026-06-22T22:12:23Z
---

## Summary of Changes

- Engine fix: `update_label_mode(Numeric)` now counts `CitationLabel` as satisfying the label check in `crates/citum-schema-style/src/options/scoped.rs`, preventing spurious prepend of `citation-number` when a `citation-label` component is already present.
- Style fix: `styles/american-mathematical-society-label.yaml`
  - Added `name-form: full` to both `options.contributors` and `bibliography.options.contributors`.
  - Added an `all:` type-variant override that uses `citation-label` instead of the inherited `citation-number` from `elsevier-with-titles-core`. This overrides the parent's catch-all at position 2 in the merged IndexMap.
- Result: 72.1% → 93.2% fidelity (bibliography: 32/48 → 45/47; citations: 17/20)
