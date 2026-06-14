---
title: Citation Regime
status: Active
created: 2026-06-14
---

# Citation Regime

## Summary

A **citation regime** is the primary identity key a citation style uses to render
in-text citations. Citum encodes regimes as `Processing` enum variants and enforces
regime coherence when one style inherits from another.

## Regimes

| Regime | `Processing` variants | Primary key | Example |
|---|---|---|---|
| Author-date | `AuthorDate`, `AuthorDateGivenname`, `AuthorDateNames`, `AuthorDateFull` | `(Author, Year)` | APA, Chicago author-date |
| Numeric | `Numeric` | Citation-order number | IEEE, Nature, bio-protocol |
| Note | `Note` | Footnote / endnote | Chicago notes-bibliography |
| Label | `Label` | Trigraph label | Alpha, DIN 1505-2 |
| Custom | `Custom` | User-defined | Any fully-custom style |

## Scope: citation surface only

Regimes govern the **citation surface** — the in-text citation template, integral
and non-integral sub-specs, and disambiguation strategies. They do **not** govern
bibliography sort order or bibliography grouping, which are independently
authorable and inheritable across regime boundaries.

A numeric style may therefore still sort its bibliography alphabetically by author
(e.g., CSE citation-sequence). Those fields live outside the regime-scoped
surface and must not be reset by regime enforcement.

## `RegimeFamily`: coarse cross-regime comparison

For compatibility checks, `Processing::regime_family()` returns a `RegimeFamily`
discriminant:

- All `AuthorDate*` variants → `RegimeFamily::AuthorDate`
- `Numeric` → `RegimeFamily::Numeric`
- `Note` → `RegimeFamily::Note`
- `Label` → `RegimeFamily::Label`
- `Custom` → `RegimeFamily::Custom` — never triggers automatic regime resets

## Engine inheritance invariant

When a child style resolves its effective style by merging over a parent,
inherited citation-mode sub-specs (`citation.integral`, `citation.non_integral`)
**are reset to the child's own declared values** (often `None`) if both of the
following hold:

1. The child's effective `Processing` has a `regime_family()` that differs from
   the parent's effective `Processing` family.
2. The child supplies its own base `citation.template`.

Bibliography spec is untouched by this reset.

Implementation: `merge_style_overlay` in
`crates/citum-schema-style/src/style/overlay.rs`.

### Rationale

A numeric child that overrides only `citation.template` but inherits an
author-date parent's `non_integral` sub-spec will have its numeric template
overwritten at render by `CitationSpec::resolve_for_mode(NonIntegral)`. The
regime guard closes this leak.

### Author escape hatch

A style author who explicitly wants to inherit cross-regime sub-specs may declare
them directly in the child YAML. Declared overlay values are preserved; only
inherited-but-not-overridden values are reset.

## Migrate lineage invariant

During CSL migration, a `<link rel="template">` parent is **dropped** (the child
migrates as standalone) when `detect_processing_mode` classifies the child's
detected regime family as incompatible with the parent Citum style's declared
`processing` regime family.

`<link rel="independent-parent">`, registry aliases, and local `extends` are
**not subject** to this check — they are explicit, author-asserted relationships.

Implementation: `StyleLineage::apply_regime_guard` in
`crates/citum-migrate/src/lineage.rs`.

### Guard condition

The guard fires only when the parent Citum style has an explicitly declared
`processing` field. A parent with no `processing` key is treated as unknown and
passed through without dropping.

## Related specs

- [MIGRATION_TAXONOMY_AWARE_WRAPPERS](MIGRATION_TAXONOMY_AWARE_WRAPPERS.md) —
  defines allowed wrapper cases; the regime invariant adds a compatibility gate
  on `template`-rel parent links.
- [DISAMBIGUATION](DISAMBIGUATION.md) — disambiguation is regime-scoped;
  author-date and label disambiguation must not leak into numeric styles.
- [EXPLICIT_DEFAULT_SORTING](EXPLICIT_DEFAULT_SORTING.md) — bibliography sort
  is orthogonal to regime and is not reset by the regime guard.
