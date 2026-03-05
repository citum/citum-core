---
# csl26-z4t6
title: Support numeric compound citations in native model
status: completed
type: feature
priority: high
created_at: 2026-03-05T15:22:32Z
updated_at: 2026-03-05T15:57:47Z
---

Tentative conclusion from PR #285 analysis:

- This should be supported, but not as a quick patch on top of current CSL 1.0
  locator parsing.
- The "numeric compound" shape is not representable in CSL 1.0 citation-item
  data (`label + locator` only), so lossless behavior requires a native model
  extension.
- Recommended path: add a native locator AST for citation items that can
  represent mixed numeric segments (e.g., chapter/section/paragraph chains)
  and render labels with locale terms at output time.
- Migration stance: CSL 1.0 input should continue to degrade gracefully to the
  existing flat locator model; native Citum input can opt into full compound
  semantics.

Definition of done (tentative):

- Native schema supports structured compound locators without breaking existing
  citation-item serialization.
- Processor and renderers handle compound locators deterministically.
- Backward compatibility tests confirm unchanged output for existing
  non-compound styles.


## Summary of Changes

Added compound locator support as an additive, backward-compatible schema extension:

- `LocatorSegment` struct (`label` + `value`) in `citum-schema`
- `ResolvedLocator` enum (Flat | Compound) with `CitationItem::resolved_locator()`
- `locators: Option<Vec<LocatorSegment>>` on `CitationItem` (takes priority when present)
- `collapse_compound_locator()` and `resolve_item_locator()` in rendering engine
- 6 call sites updated to use resolver; variable.rs and templates unchanged
- 6 new unit tests (serde roundtrip, priority, flat fallback, none, skip-serializing)
- All 504 tests pass, no regressions

Branch: `feat/compound-locators`
