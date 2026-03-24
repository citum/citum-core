---
# csl26-u3zy
title: Remove per-component overrides field
status: scrapped
type: feature
priority: normal
created_at: 2026-03-23T20:17:37Z
updated_at: 2026-03-23T20:18:14Z
---

## Goal

Remove `ComponentOverride` enum and the `overrides: Option<HashMap<TypeSelector, ComponentOverride>>`
field from every `TemplateComponent` variant. Replace with `type-variants` at the spec level
(already landed in `spec/template-v2`).

## Why Deferred

The `convert-overrides-to-type-variants.py` script does not handle `ComponentOverride::Rendering`
suppression (the most common pattern in note styles like `chicago-notes-18th`). Running the script
as-is stripped 627-line style files down to ~162 lines, dropping all type-specific rendering — causing
fidelity to collapse from 1.0 to 0.052.

## Scriptable Approach (no LLM authoring needed)

1. Fix `convert-overrides-to-type-variants.py` to invert suppression:
   - For each group with `overrides: { <type>: { suppress: true } }`:
     emit a `type-variants:<type>` entry that is a copy of the default template
     with the suppressed component omitted.
   - Handle `ComponentOverride::Component` (full replacement): substitute the
     replacement component in the per-type template.
2. Run on `chicago-notes-18th.yaml` and verify fidelity with oracle.
3. Bulk-run on all styles with `overrides:` keys.
4. Remove `ComponentOverride` from `crates/citum-schema-style/src/template.rs`.
5. Remove `overrides` field from all template compiler output in `citum-migrate`.
6. All changes in one commit (§2.4 constraint): schema field removal + compiler fix.

## Key Files

- `scripts/convert-overrides-to-type-variants.py`
- `crates/citum-schema-style/src/template.rs` (remove ComponentOverride + overrides field)
- `crates/citum-migrate/src/template_compiler/compilation.rs` (stop emitting overrides)
- `crates/citum-migrate/src/template_compiler/overrides.rs` (remove module)
- All `styles/*.yaml` files that use overrides: (bulk script pass)

## Spec Reference

`docs/specs/TEMPLATE_V2.md` §2 — Step 5 in the ordering.
