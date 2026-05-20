---
# csl26-e7yw
title: citum-migrate SQI scorecard + citation type-variant diff emission
status: completed
type: feature
priority: high
created_at: 2026-05-20T17:04:52Z
updated_at: 2026-05-20T17:47:20Z
parent: csl26-f1u7
---

First PR of the post-publish converter quality wave (epic [[csl26-f1u7]]). Establishes the measurement substrate plus the cheapest structural emission improvement, so subsequent PRs can be evaluated against a real baseline.

Plan: `~/.claude/plans/with-crates-now-published-reflective-snowglobe.md`.

## Todo
- [x] Verify `template_diff::build_type_variants` is reusable for citation templates (deferred — see scope-pivot note)
- [x] Build CompiledOutput: capture per-type citation templates (deferred — see scope-pivot note)
- [x] In `build_final_style`: citation type-variant emission (deferred — see scope-pivot note)
- [x] Drop redundant scope options (subsumed by alias-wrapper diff)
- [x] Add unit tests for the two converter fixes (aliased-style routing, atomic-config diff)
- [x] `scripts/report-migrate-sqi.js`: runs converter over 15-style corpus, reports fidelity + migrated/public SQI + per-style delta + concision diagnostics
- [x] `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` published with pre/post numbers
- [x] `docs/TIER_STATUS.md` carries the converter-SQI summary paragraph
- [x] Pre-commit gate (fmt + clippy + nextest 1312 passing)
- [x] Oracle sentinels: 270/270 citations, 498/507 bibliography (9 misses are pre-existing engine-gaps)
- [x] `node scripts/check-core-quality.js` unaffected (no `styles/` mutations)

## Acceptance
- Top-10 sentinels remain 100% strict-match.
- Migrated-YAML corpus mean SQI lifts measurably (target ≥ +0.05; revise once baseline lands).
- Scorecard JSON + Markdown reproducible from the new script.
- Bean files squashed into the concluding PR commit.


## Scope pivot

The planned citation type-variant emission + option pruning were dropped after the scorecard baseline showed the SQI lift was concentrated in two unrelated converter defects: a `diff_value` partial-mapping bug at atomic-config paths (blocking `elsevier-vancouver` entirely) and an `output_plan` miss for `(Base|Profile|Journal, Alias)` (forcing standalone duplicates for `apa`, `chicago-author-date`, and their siblings). Fixing those two issues delivered a `+4.6`-point corpus mean lift and full corpus completion (15/15) without the citation-side machinery. Citation type-variant emission remains a structural-completeness goal but moves to a follow-up wave child.

## Summary of Changes

- `crates/citum-migrate/src/lineage.rs`: add `ATOMIC_CONFIG_PARENTS`/`ATOMIC_CONFIG_LEAVES` and route `diff_value` to emit full child mappings at those paths; map alias-target semantic class from the registered entry kind; extend `output_plan` to treat `(Base|Profile|Journal, Alias)` as `ExistingWrapper`; add unit tests for both fixes.
- `scripts/report-migrate-sqi.js` (new): reproducible scorecard composing `citum-migrate`, `oracle-migrate-batch`, and `report-core` exports.
- `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` (new): baseline numbers, per-style table, observations, sequencing.
- `docs/TIER_STATUS.md`: converter-output SQI section linking the baseline doc.


## Hand-off pointers

- **PR:** [citum-core#763](https://github.com/citum/citum-core/pull/763)
- **Branch:** `feat/migrate-sqi-scorecard-citation-variants`
- **Commit:** `3863b7f3 fix(migrate): aliased + atomic-config diff` (+ `01dea56c` bean scaffolding)
- **Baseline numbers (15-style corpus):** migrated mean SQI `93.57` -> `98.17`; public mean `94.92` (delta `+3.25` mean, `+1.77` p50, `+10.87` p90). Headline movers: `apa` from `-17.67` to `+11.63`, `chicago-author-date` from `-31.56` to `+1.77`, `elsevier-vancouver` recovered from hard-failure to `100/100`.
- **Files touched:** `crates/citum-migrate/src/lineage.rs` (logic + 2 new tests), `scripts/report-migrate-sqi.js` (new), `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` (new), `docs/TIER_STATUS.md` (append).
- **Gates:** `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo nextest run` (1314 passing, +2 new lineage tests), `oracle-migrate-batch` 270/270 citations + 498/507 bibliography (9 misses are pre-existing engine-gaps tracked in session-3 / session-4 lab notes), `check-core-quality.js` 154 styles fidelity 1.0 `warnings=0`.
- **What did *not* land:** citation type-variant emission via `compile_citation_with_types` (no compiler analog exists for the citation side today); option-pruning (subsumed by alias-wrapper diff for the styles where it would have helped). Both deferred to PR2/PR3 of the wave, not lost.
