---
# csl26-vpae
title: 'migrate: delta-based extraction to fix eager disambiguation-default materialization'
status: completed
type: bug
priority: normal
created_at: 2026-06-20T18:51:16Z
updated_at: 2026-07-06T23:38:57Z
---

The migrate options-extractor eagerly materializes disambiguation defaults (e.g. year_suffix: unwrap_or(true)) before comparing against named Processing presets, so fold_to_named_processing can never match. Fix: leave unset CSL attributes as None and record only explicit overrides that differ from the preset — the same delta philosophy as extends:. Design decisions captured in docs/reference/PROCESSING_MIGRATION.md (csl26-1861); implementation deferred from that bean's doc-only scope.

## Root Cause (2026-07-06 review)

The original `year_suffix: unwrap_or(true)` sites are gone; `options_extractor/processing.rs` now seeds from the class preset and applies only explicit attributes (lines 88-97). Two residual defects keep materialization eager:

1. **Unconditional `givenname_rule` assignment** (`processing.rs:98-104`): the `_ => GivennameRule::default()` arm overwrites the seeded preset value even when the CSL attribute is absent. Harmless while `default() == ByCite` (the author-date presets' rule), but it is a fold-breaker the moment any preset uses a different rule (`AuthorDateFull` already uses `PrimaryName`; a style folding toward it via explicit attrs gets clobbered back to default).
2. **All-or-nothing folding** (`fold_to_named_processing`): the derived config is compared for exact equality against each named preset's `config()`. Any single divergence — most commonly an explicit non-preset `sort` — fails every candidate and falls to `Processing::Custom(custom)`, whose serialization dumps the **entire** materialized `ProcessingCustom`, including disambiguation defaults the CSL never stated. That is the eager-materialization symptom in emitted YAML.

## Fix Design

Delta philosophy per docs/reference/PROCESSING_MIGRATION.md, two layers:

1. **Migrate-side sparse extraction (no schema change, do first):**
   - Introduce a local `ProcessingOverrides { sort, group, disamb_names, disamb_add_givenname, disamb_year_suffix, givenname_rule }` with all-`Option` fields populated only from explicit CSL attributes. Fix defect 1 by assigning `givenname_rule` only when the attribute is present.
   - Fold per facet, not whole-struct: pick the disambiguation-nearest named preset from the folding table (bare→author-date, +givenname→author-date-givenname, +names→author-date-names, both→author-date-full), then check whether the remaining overrides (sort/group) also match that preset's config. If yes → named preset. If only sort/group diverge → emit `Custom` with the explicit sort/group but `disambiguate` set to the preset's value ONLY if it differs from what resolution would produce; today Custom has no default-resolution, so keep the materialized disambiguation but add a serializer skip for values equal to the class default once layer 2 lands.
2. **Schema-side custom-as-delta (preferred end state, separate commit):** allow `ProcessingCustom` to carry an optional `base: Processing` (named presets only); resolution overlays the sparse fields onto `base.config()`. YAML reads `processing: { base: author-date, sort: ... }` — the same philosophy as `extends:`. Serializer omits fields equal to the base config. Touches citum-schema-style (serde + resolution) and engine call sites of `.config()`; schema regen required (`just schema-gen`).

Tests: rstest over the folding table in PROCESSING_MIGRATION.md (each row → expected emitted YAML), plus a regression asserting a style with explicit non-preset sort no longer emits materialized disambiguation defaults.

Sizing: layer 1 is Sonnet-executable against this design; layer 2 needs a schema review pass first.

## Summary of Changes

Implemented layer 1 only (per design doc sizing note): introduced a sparse
`ProcessingOverrides` struct in `options_extractor/processing.rs` capturing only
explicitly-stated CSL attributes. `fold_to_named_processing` now picks the
disambiguation-nearest named preset directly from the names/given-name signal
(per `docs/reference/PROCESSING_MIGRATION.md`'s folding table), then overlays only
the remaining explicit overrides (sort/group/year-suffix/givenname-rule), falling
to `Processing::Custom` only when those diverge. This fixes the actual fold-breaker:
a style with `disambiguate-add-names`+`disambiguate-add-givenname` but no explicit
`givenname-disambiguation-rule` now correctly folds to `author-date-full` instead of
dumping a full custom block with eagerly-materialized defaults. Added rstest cases
over the folding table plus a regression confirming an explicit non-preset sort
diverges to `Custom` without disturbing the disambiguation portion. Updated one
existing test (`test_extract_full_disambiguation_default_rule_stays_by_cite_custom`
-> `..._folds_to_full`) that had encoded the old buggy behavior. Verified against a
real style (`american-anthropological-association.csl`) that now emits
`processing: author-date-full` instead of a materialized custom block.

Layer 2 (schema-side `ProcessingCustom.base` field + resolution overlay) is
deferred to a follow-up bean pending schema review, per the original bean's sizing
note and Bruce's confirmed scope decision.
