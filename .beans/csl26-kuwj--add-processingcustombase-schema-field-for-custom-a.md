---
# csl26-kuwj
title: Add ProcessingCustom.base schema field for custom-as-delta processing
status: in-progress
type: task
priority: normal
created_at: 2026-07-06T23:39:03Z
updated_at: 2026-07-07T01:18:38Z
parent: csl26-al39
---

Layer 2 of the delta-based processing extraction design (docs/reference/PROCESSING_MIGRATION.md,
csl26-vpae layer 1 landed in csl26-vpae). Allow ProcessingCustom to carry an optional
base: Processing field (named presets only); resolution overlays the sparse custom
fields onto base.config() instead of requiring an exact whole-struct match. YAML would
read processing: { base: author-date, sort: ... } -- the same delta philosophy as
extends:. Touches citum-schema-style (serde + resolution) and citum-engine call sites
of Processing::config(); requires just schema-gen in the same commit.

Needs a schema review pass first (deferred from csl26-vpae per its own sizing note).

## Confirmed Design Decisions (2026-07-06, Bruce)

1. **Base type**: dedicated `ProcessingBase` enum (author-date, author-date-givenname, author-date-names, author-date-full, numeric, note, label) — nesting impossible by construction; label maps to `Label(LabelConfig::default())`.
2. **Overlay depth**: whole-field — each present field (sort/group/disambiguate) replaces the base's value wholesale; no sparse Disambiguation sub-merge.
3. **Base semantics**: delegate — `regime_family()` and `is_author_date_family()` follow the base; `default_bibliography_sort()` delegates only when `custom.sort` is None (explicit sort keeps the custom entry-ID-tiebreak path). Base with zero overrides behaves identically to the bare preset.
4. **Scope**: one PR, two commits — feat(schema) field+resolution+delegation+schema-gen, then feat(migrate) base+delta emission on Custom fallback.

## Todo

- [x] feat(schema): ProcessingBase enum, ProcessingCustom.base, resolved() overlay, delegation, deserializer, tests, schema-gen
- [ ] feat(migrate): fold_to_named_processing emits base+delta; update regression tests
- [ ] docs/reference/PROCESSING_MIGRATION.md: Custom-as-delta section
- [ ] just pre-commit green; end-to-end style verification; PR + CI
