# citum-migrate SQI baseline (post-publish wave PR1)

**Date:** 2026-05-20
**Status:** Active
**Related:** `.beans/csl26-f1u7--citum-migrate-post-publish-quality-wave.md`, `.beans/archive/csl26-e7yw--citum-migrate-sqi-scorecard-citation-type-variant.md`, `docs/reference/SQI.md`, `docs/specs/APA_SQI_ALIGNMENT_AND_PRESET_REFACTOR.md`

## Purpose

Establish a reproducible scorecard for the structural quality (SQI) of `citum-migrate` output, so subsequent converter changes can be evaluated against a real baseline rather than gut feel. Record the lift produced by PR1's converter fixes:

- The `diff_value` recursion at atomic-config paths (`options.dates`, `options.contributors`, `options.titles`, `options.locators`, `options.processing`, and the scoped variants under `citation.options` / `bibliography.options`) was producing partial mappings that failed to deserialize as the corresponding untagged `Preset | Explicit` enums (e.g. an `Explicit` `DateConfig` missing required `month`, or a `processing.sort` losing sibling `group`/`disambiguate`). This fully blocked migration of `elsevier-vancouver` and silently dropped disambiguation fields whenever the migrated `processing` block was diffed against a parent. Atomic-config paths now emit the full child value when it differs.
- `StyleLineage::output_plan` did not route `(Base | Profile | Journal, Alias)` to `ExistingWrapper`. Legacy CSL IDs that are registered aliases of canonical embedded styles (e.g. `apa` → `apa-7th`, `chicago-author-date` → `chicago-author-date-18th`) were emitted as duplicated standalone styles, scoring far below the canonical embedded structure on concision. They now collapse to an `info` + `extends:` shell — the alias is *defined* to equal its canonical target, so any converter-derived deltas were noise rather than signal.
- `Processing::Custom` previously round-tripped with an externally-tagged YAML representation (`processing: !custom`). The serializer now emits a bare map, matching the deserializer's `visit_map` branch that already accepts that shape. Style YAMLs read by humans look the same as hand-authored styles (`processing: author-date` when the derived config matches a named variant; bare `processing:` + `sort`/`group`/`disambiguate` otherwise).
- `TitlesConfig` extraction now hoists rendering fields shared by every populated category to `titles.default`, removing categories that become empty. The engine already treats `default` as the fallback when a per-category rendering is absent (`crates/citum-engine/src/render/component.rs:348`), so this is a YAML compaction rather than a behavior change.

## Corpus

Lab + top-10 sentinels (15 styles): the migrate-research lab corpus plus the most-depended-on parents from `docs/TIER_STATUS.md`. Refresh with:

```bash
node scripts/report-migrate-sqi.js --out /tmp/sqi.json --markdown docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md
```

The scorecard runs the converter, scores both the migrated YAML and the public `styles/` YAML using the `concision`, `fallbackRobustness`, and `presetUsage` subscores from `scripts/report-core.js`, and pulls fidelity from `scripts/oracle-migrate-batch.js`. The fourth SQI subscore (`typeCoverage`) is intentionally not part of this measurement — it depends on full oracle per-type results, and folding it in would conflate converter-output quality with engine/style behavior.

## Aggregate

| Subject | n | mean | p10 | p50 | p90 |
|---|---:|---:|---:|---:|---:|
| Migrated YAML SQI | 15 | 98.17 | 94.80 | 99.47 | 100.00 |
| Public YAML SQI | 15 | 94.92 | 88.37 | 96.67 | 100.00 |
| Migrated − Public | 15 | +3.25 | -0.27 | +1.77 | +10.87 |

Pre-PR1 (commit `bdd5595d`) migrated mean was `93.57` and corpus completion was 14/15 (elsevier-vancouver hard-failed on the `DateConfigEntry` parse error). PR1 lifts the migrated mean by `+4.6` points and recovers the missing corpus entry to 100/100.

## Per-style

| Style | Fidelity | Migrated SQI | Public SQI | Δ | Migrated dup/near/rep | Public dup/near/rep |
|---|---:|---:|---:|---:|---|---|
| apa | 18/18 • 33/33 | 100.00 | 88.37 | **+11.63** | 0/0/0 | 1/2/161 |
| elsevier-harvard | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 0/0/0 | 0/0/0 |
| elsevier-with-titles | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 0/0/0 | 0/0/0 |
| elsevier-vancouver | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 0/0/0 | 0/0/0 |
| springer-basic-author-date | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 0/0/0 | 0/0/0 |
| ieee | 18/18 • 34/34 | 94.80 | 83.93 | **+10.87** | 0/0/31 | 4/14/95 |
| american-medical-association | 18/18 • 31/34 | 97.23 | 97.50 | -0.27 | 0/0/51 | 0/1/52 |
| nature | 18/18 • 32/34 | 99.07 | 96.67 | +2.40 | 0/0/9 | 0/0/0 |
| cell | 18/18 • 32/34 | 99.47 | 96.27 | +3.20 | 0/0/3 | 0/0/3 |
| chicago-author-date | 18/18 • 33/33 | 100.00 | 98.23 | +1.77 | 0/0/0 | 0/0/43 |
| karger-journals | 18/18 • 33/34 | 99.03 | 89.27 | **+9.76** | 0/0/4 | 0/0/5 |
| institute-of-physics-numeric | 18/18 • 34/34 | 89.10 | 89.37 | -0.27 | 0/0/7 | 0/0/3 |
| thieme-german | 18/18 • 34/34 | 98.70 | 95.73 | +2.97 | 0/0/10 | 0/0/10 |
| multidisciplinary-digital-publishing-institute | 18/18 • 33/34 | 95.17 | 88.53 | +6.64 | 0/0/21 | 0/0/20 |
| taylor-and-francis-chicago-author-date | 18/18 • 33/33 | 100.00 | 100.00 | 0.00 | 0/0/0 | 0/0/0 |

Columns: Migrated/Public SQI is a simple mean of `concision`, `fallbackRobustness`, and `presetUsage` (0–100). The `dup/near/rep` triple is the per-style `exactDuplicateScopes / nearDuplicateScopes / repeatedPatterns` diagnostic block from `qualityBreakdown.subscores.concision` in `report-core.js`. Fidelity is from `oracle-migrate-batch` (citations `passed/total` • bibliography `passed/total`).

## Observations

- The two pre-PR1 outliers (`apa` -17.67, `chicago-author-date` -31.56) collapsed to `+11.63` and `+1.77` respectively after the alias-wrapper routing change. Both styles are registry aliases for canonical embedded `kind: base` entries (`apa-7th`, `chicago-author-date-18th`); the converter now emits a thin `extends:` wrapper for them and inherits the canonical templates instead of duplicating them.
- `ieee` (+10.87), `karger-journals` (+9.76), `multidisciplinary-digital-publishing-institute` (+6.64), and `cell` (+3.20) are styles where the converter output is **more concise than the hand-authored public YAML**. The biggest publisher of this gap is `ieee`, where the public YAML carries 4 exact and 14 near-duplicate scopes that the converter's pattern compression collapses; this points at a follow-up YAML cleanup wave for those public styles, not a converter regression.
- Bibliography misses for `american-medical-association` (31/34), `nature`/`cell` (32/34), `karger-journals` (33/34), and `multidisciplinary-digital-publishing-institute` (33/34) are pre-existing engine-gaps (patent number, publisher:extra, magazine title suppression). They are not introduced by PR1; see session-3 / session-4 lab notes under `.claude/skills/migrate-research/lab/`.

## Sequencing

This baseline is the gate for the post-publish converter quality wave (`csl26-f1u7`):

- **PR2** will broaden the alias-wrapper machinery into a full descendant-of-preset-base rewrite: when a legacy CSL ID is not an exact alias but the registry can identify it as part of a known family, emit `extends: <family-id>` + diff-form template variants instead of standalone templates. Target: pulling more of the long tail (currently neutral or slightly negative) into the `+5..+10` band.
- **PR3** will author a Vancouver / numeric-journal preset base and repeat the rewrite pass for the numeric family (the `american-medical-association`, `karger`, `iop`, `thieme`, `mdpi` cluster).
- **PR4** will generalize family detection beyond the three hand-authored preset bases by deriving candidate families from cluster fingerprints.

Each subsequent PR refreshes this scorecard and updates the baseline numbers in place.
