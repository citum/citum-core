---
# csl26-kqji
title: Descendant-of-preset-base wrapper rewrite (PR2)
status: completed
type: feature
priority: high
created_at: 2026-05-20T17:37:32Z
updated_at: 2026-05-20T22:16:47Z
parent: csl26-f1u7
blocked_by:
    - csl26-e7yw
---

PR2 of the citum-migrate post-publish quality wave (epic [[csl26-f1u7]]).

Extend the alias-wrapper machinery landed in PR1 ([[csl26-e7yw]]) into a full descendant-of-preset-base rewrite: when a legacy CSL ID is not an exact registry alias but the registry / lineage data can identify it as part of a known family (apa-7th, chicago-author-date-18th, chicago-notes-18th, or a future numeric base), emit `extends: <family-id>` + diff-form template variants instead of standalone templates.

## Scope sketch (refine before starting)
- Descendant manifest: derive from CSL `<info><link rel="independent-parent">` plus existing `lineage.rs` resolution, or hand-author a starter manifest covering the highest-impact descendants.
- Rewrite pass: for matched descendants, route through the existing `ExistingWrapper { preserve_template_deltas: true }` path, plumb diff-form bibliography type-variants from `template_diff` against the parent's resolved template (not the migrated child's primary).
- Refresh `scripts/report-migrate-sqi.js` baseline; expect long-tail neutral/negative styles to move into the `+5..+10` band per the baseline observations.

## Acceptance
- No fidelity regression on any sentinel; reuse existing oracle + nextest gate.
- Corpus mean SQI lifts measurably above PR1's `98.17` baseline.
- Updates `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` in place.


## Hand-off context

### Where PR1 left off

PR1 ([[csl26-e7yw]], merged commit `3863b7f3` on branch `feat/migrate-sqi-scorecard-citation-variants`) closed the alias case in `crates/citum-migrate/src/lineage.rs`:

- `StyleLineage::resolve` now maps an alias targets semantic class from the registry entry kind, so `(Base|Profile|Journal, Alias)` flows to `output_plan`.
- `output_plan` adds the `(Base|Profile|Journal, Alias)` arm returning `ExistingWrapper { preserve_template_deltas: false }`.
- `diff_value` short-circuits at `ATOMIC_CONFIG_PARENTS` x `ATOMIC_CONFIG_LEAVES` paths (atomic untagged-enum mappings).

What PR1 did *not* touch:
- The `current_style.is_none() && alias_target.is_none()` path — i.e. legacy CSL ids that have no registry record at all but *are* structurally descendants of a known canonical style. That is PR2s job.
- Citation type-variant emission. There is no `compile_citation_with_types` analog to `compile_bibliography_with_types`; adding one is a sub-feature of either PR2 or a separate ticket.

### Concrete starting points

1. **Survey the long-tail.** Run `node scripts/report-migrate-sqi.js --skip-fidelity --out /tmp/sqi.json` and inspect rows where migrated SQI is below `98` *and* the public YAML has `hasRootExtends: true`. Those are the candidates: the public form already extends something, but the converter still emits standalone.
2. **Manifest source.** CSL files carry `<info><link rel="independent-parent" href="..."/></info>`. `csl-legacy::parser` already parses these — check `csl_legacy::model::StyleInfo.links` (or equivalent). The href hash (`http://www.zotero.org/styles/<name>`) maps cleanly to canonical citum ids via the alias scan already in `lineage.rs:137-140`.
3. **Reuse the diff machinery.** `apply_to_migrated_style` and `diff_value` are already general; the path that needs to change is `StyleLineage::resolve` so that a `<link rel=independent-parent>` hit produces `parent_style_id = Some(canonical)` even when the local YAML doesnt exist.
4. **Watch the `preserve_template_deltas` knob.** For aliases of `kind: base` we used `false` (drop local templates entirely; trust the canonical). For descendants of a `base` parent, the right choice is likely `true` — keep diff-form bibliography type-variants in the wrapper, drop the primary template if it matches.

### Suggested test additions

- Add `chicago-notes` and `oscola` to the scorecard sentinels (`SENTINELS` in `scripts/report-migrate-sqi.js`). They are not yet in the corpus and are the styles most likely to exercise citation type-variants when that work lands.
- Mirror `aliased_legacy_style_resolves_as_existing_wrapper` for a descendant case (pick a real `<link rel=independent-parent>` style from `styles-legacy/`).

### Acceptance (refined)

- No fidelity regression on the PR1 sentinel corpus or on `chicago-notes` / `oscola` once they are added.
- Corpus mean SQI lifts measurably above PR1s `98.17` baseline (target: `>= 99.0`).
- The long-tail neutral/negative deltas (`american-medical-association` `-0.27`, `institute-of-physics-numeric` `-0.27`) move into positive territory.
- `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` updated in place with new numbers and an observations entry for the descendant rewrite.

## Summary of Changes

Delivered as PR #766 (`feat(migrate): use template parent wrappers`, commit `d919e930`).

- Extended `StyleLineage::resolve` to follow `<info><link rel="independent-parent">` hits to a canonical citum id even when no local YAML exists for the legacy id.
- Added a `(_, DescendantOfBase)` arm in `output_plan` returning `ExistingWrapper { preserve_template_deltas: true }`, so descendants emit `extends: <parent>` plus diff-form template variants instead of standalone templates.
- Added `chicago-notes` and `oscola` to the SQI scorecard sentinels.
- Added a `pathologicalOutput` diagnostic to `report-migrate-sqi.js` for downstream consumers.

Follow-up tracked in [[csl26-39tm]] (output-driven compression + alias UX evidence).
