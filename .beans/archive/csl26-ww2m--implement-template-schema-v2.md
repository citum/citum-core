---
# csl26-ww2m
title: Implement template schema v2
status: completed
type: feature
priority: normal
created_at: 2026-03-23T13:34:42Z
updated_at: 2026-03-23T20:17:07Z
---

Implement docs/specs/TEMPLATE_V2.md on branch spec/template-v2 (PR #426) in four scoped commits:

- [x] Commit 1: items→group rename (TemplateList→TemplateGroup, ~145 match arms, serde alias)
- [x] Commit 2: type-variants on CitationSpec; type-templates→type-variants on BibliographySpec; engine lookup
- [x] Commit 3: TypeSelector validation + Style::validate() stub
- [ ] Commit 4: Remove overrides field/enum from schema+engine; migrate compiler emits type-variants; bulk migrate 145 styles (BREAKING — Schema-Bump: major)

Spec: docs/specs/TEMPLATE_V2.md
Related spec bean: csl26-da9f

## Summary of Changes

Steps 1–4 landed in PR `spec/template-v2` (Schema-Bump: patch):

- TemplateList → TemplateGroup rename with `#[serde(alias = "items")]` for compat
- `type-variants` added to CitationSpec; `type-templates` → `type-variants` alias on BibliographySpec
- `VALID_TYPE_NAMES`, `TypeSelector::unknown_type_names()`, and `Style::validate()` with `SchemaWarning::UnknownTypeName`
- Schema regenerated (patch bump); all oracle scenarios pass
- Spec `docs/specs/TEMPLATE_V2.md` promoted to Active

Step 5 (overrides removal, Schema-Bump: major) deferred. `ComponentOverride` and `overrides`
field remain in schema. Follow-up bean created for scriptable conversion approach.
