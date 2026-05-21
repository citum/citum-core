# citum-migrate SQI baseline (post-publish wave PR1–PR3)

**Date:** 2026-05-20
**Status:** Active
**Related:** `.beans/csl26-f1u7--citum-migrate-post-publish-quality-wave.md`, `.beans/archive/csl26-e7yw--citum-migrate-sqi-scorecard-citation-type-variant.md`, `.beans/archive/csl26-kqji--descendant-of-preset-base-wrapper-rewrite-pr2.md`, `.beans/archive/csl26-39tm--output-driven-migration-compression-and-alias-ux-e.md`, `docs/reference/SQI.md`, `docs/specs/APA_SQI_ALIGNMENT_AND_PRESET_REFACTOR.md`

## Purpose

Establish a reproducible scorecard for the structural quality (SQI) of `citum-migrate` output. Subsequent converter changes are evaluated against this baseline. Records the lift produced by PR1–PR3 of the post-publish converter quality wave.

### PR1 (alias wrapper + atomic-config diff)

- `diff_value` recursion at atomic-config paths now emits the full child value when it differs, fixing fragment deserialization for the untagged `Preset | Explicit` enums (`options.dates`, `options.contributors`, `options.titles`, `options.locators`, `options.processing` and their scoped variants).
- `StyleLineage::output_plan` routes `(Base | Profile | Journal, Alias)` to `ExistingWrapper`. Aliases collapse to an `info` + `extends:` shell.
- `Processing::Custom` now round-trips as a bare map instead of an externally-tagged YAML representation.
- `TitlesConfig` extraction hoists rendering fields shared by every populated category to `titles.default`.

### PR2 (descendant-of-preset-base wrapper, #766)

- `StyleLineage::resolve` follows `<info><link rel="independent-parent">` into a canonical citum id even when no local YAML exists. A `(_, DescendantOfBase)` arm in `output_plan` routes the result through `ExistingWrapper { preserve_template_deltas: true }`.
- Added `chicago-notes` and `oscola` sentinels plus a `pathologicalOutput` diagnostic.

### PR3 (output-driven evidence + minimized wrapper)

- Added a `MigrationEvidence` record (`crates/citum-migrate/src/evidence.rs`) emitted as a JSON sidecar when `citum-migrate --emit-evidence <path>` is set. Captures registry alias status, discovered candidate parents with discovery source (registry-alias, template-link, independent-parent-link, reverse-template-link, local-extends), emitted form, preserved/discarded template paths, and standalone-vs-emitted LOC.
- Added reverse `<info><link rel="template">` discovery in `StyleLineage::resolve`. When a legacy style has no other parent link, the resolver scans embedded canonical styles for one that declares the legacy id as its historical template source. The discovered candidate is recorded inertly on `StyleLineage::family_candidate`.
- Added `--family-candidate off|auto|<id>` to opt into routing a discovered candidate through `ExistingWrapper { preserve_template_deltas: true }`.
- Added `--minimize-wrapper`: when used with `--family-candidate`, the migration emits a minimal wrapper (`info` + `extends` + only options that materially differ from parent), inheriting all template-bearing scopes from the candidate parent. Default off; existing callers see no behavior change.
- The SQI scorecard now A/B-tests every compression candidate: standalone form vs. minimized form, both oracle-verified. The minimized form replaces the standalone in scorecard reporting only when oracle citation pass ≥ standalone, bibliography pass ≥ standalone, and the minimized LOC is strictly smaller than standalone.
- `apa-6th-edition` is now a sentinel.

### Result on apa-6th-edition

| | Standalone | Minimized (PR3) |
|---|---:|---:|
| LOC | 5,661 | **5** |
| Migrated SQI | 66.67 | **100.00** |
| Oracle citations | 18/18 | 18/18 |
| Oracle bibliography | 10/37 | **33/34** |
| `pathologicalOutput` | yes | no |

The minimized form is not an alias of apa-7th: the migrator demonstrates output equivalence via the oracle reference corpus before swapping the standalone form for the inheritance shell. When the bibliography fixture set was expanded under the standalone form (37 references via converter-emitted type-variants), most rendered worse than the parent's template would have; inheriting apa-7th's templates verbatim recovers fidelity (33 of the 34 base references match) while reducing emitted LOC by 99.9 %.

## Corpus

13 sentinels plus 5 lab styles (18 total). Refresh with:

```bash
node scripts/report-migrate-sqi.js --out /tmp/sqi.json --markdown docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md
```

The scorecard runs `citum-migrate --emit-evidence`, reports fidelity via `scripts/oracle-migrate-batch.js`, scores both the migrated YAML and the public `styles/` YAML using `concision`, `fallbackRobustness`, and `presetUsage`. For every style whose evidence shows a discovered candidate AND whose initial emitted form is `standalone`, the scorecard also runs migrate with `--family-candidate auto --minimize-wrapper`, runs oracle on the minimized YAML directly, and swaps the row to the minimized form when the acceptance gate holds.

## Aggregate

| Subject | n | mean | p10 | p50 | p90 |
|---|---:|---:|---:|---:|---:|
| Migrated YAML SQI | 18 | 96.54 | 89.10 | 99.47 | 100.00 |
| Public YAML SQI | 17 | 92.46 | 83.93 | 96.27 | 100.00 |
| Migrated − Public | 17 | +3.87 | -0.27 | +2.40 | +10.87 |

PR3 lifted the mean from PR2's 94.69 to 96.54 by minimizing `apa-6th-edition` (66.67 → 100.00).

## Per-style

| Style | Fidelity | Migrated SQI | Public SQI | Δ | LOC | Form |
|---|---:|---:|---:|---:|---:|---|
| apa | 18/18 • 33/33 | 100.00 | 88.37 | **+11.63** | 5 | alias wrapper |
| apa-6th-edition | 18/18 • 33/34 | 100.00 | – | – | 5 | minimized wrapper (extends apa-7th) |
| elsevier-harvard | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 67 | descendant wrapper |
| elsevier-with-titles | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 24 | descendant wrapper |
| elsevier-vancouver | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 52 | descendant wrapper |
| springer-basic-author-date | 18/18 • 34/34 | 100.00 | 100.00 | 0.00 | 65 | descendant wrapper |
| ieee | 18/18 • 34/34 | 94.80 | 83.93 | **+10.87** | 265 | standalone |
| american-medical-association | 18/18 • 31/34 | 97.23 | 97.50 | -0.27 | 398 | standalone |
| nature | 18/18 • 32/34 | 99.07 | 96.67 | +2.40 | 178 | standalone |
| cell | 18/18 • 32/34 | 99.47 | 96.27 | +3.20 | 177 | standalone |
| chicago-author-date | 18/18 • 33/33 | 100.00 | 98.23 | +1.77 | 5 | alias wrapper |
| chicago-notes | 18/18 • 0/0 | 66.67 | 58.93 | +7.74 | 5 | alias wrapper; bib oracle pending |
| oscola | 11/18 • 32/34 | 98.53 | 89.10 | +9.43 | 332 | standalone |
| karger-journals | 18/18 • 33/34 | 99.03 | 89.27 | **+9.76** | 253 | standalone |
| institute-of-physics-numeric | 18/18 • 34/34 | 89.10 | 89.37 | -0.27 | 156 | standalone |
| thieme-german | 18/18 • 34/34 | 98.70 | 95.73 | +2.97 | 276 | standalone |
| multidisciplinary-digital-publishing-institute | 18/18 • 33/34 | 95.17 | 88.53 | +6.64 | 237 | standalone (compression candidate rejected) |
| taylor-and-francis-chicago-author-date | 18/18 • 33/33 | 100.00 | 100.00 | 0.00 | 50 | alias wrapper |

## Compression candidates

| Style | Candidate parent | Discovery source | Standalone LOC → Minimized LOC | Standalone fidelity → Minimized fidelity | Accepted |
|---|---|---|---:|---|:---:|
| apa-6th-edition | apa-7th | reverse-template-link | 5,661 → 5 | 18/18 • 10/37 → 18/18 • 33/34 | ✓ |
| multidisciplinary-digital-publishing-institute | american-chemical-society | template-link | 237 → 237 | 18/18 • 33/34 → 18/18 • 33/34 | ✗ |

The mdpi rejection is structural: its candidate parent is discovered via a `template-link` in the source CSL rather than a `reverse-template-link` from an embedded canonical style, so `promote_family_candidate` doesn't fire and `--minimize-wrapper` returns the standalone form unchanged. Bringing template-link candidates into the minimize path is the natural next step (see `csl26-ly8d`).

## Observations

- The two pre-PR1 outliers (`apa` -17.67, `chicago-author-date` -31.56) collapsed to `+11.63` and `+1.77` respectively after PR1's alias-wrapper routing. Both styles are registry aliases for canonical embedded `kind: base` entries; the converter emits a thin `extends:` wrapper and inherits the canonical templates instead of duplicating them.
- PR3's apa-6th-edition compression is the largest single-style lift of the wave: 5,661 LOC → 5 LOC, SQI 66.67 → 100.00, with a *fidelity gain* (10/37 → 33/34 bibliography) rather than a regression. The reference corpus exposed that apa-7th's templates render the apa-6th item set more accurately than the converter-derived apa-6th templates do — likely because the converter's coverage of the 37-reference type-variant set is incomplete while apa-7th's base templates handle the 34-reference core set cleanly.
- `ieee` (+10.87), `karger-journals` (+9.76), `multidisciplinary-digital-publishing-institute` (+6.64), and `cell` (+3.20) remain styles where the converter is more concise than the hand-authored public YAML. Public YAML cleanup is a separate follow-up.
- Bibliography misses for `american-medical-association` (31/34), `nature`/`cell` (32/34), `karger-journals` (33/34), and `mdpi` (33/34) are pre-existing engine gaps (patent number, publisher:extra, magazine title suppression) — not introduced by PR1–PR3.

## Sequencing

- **PR1** ([[csl26-e7yw]]): SQI scorecard, alias-wrapper routing, atomic-config diff fix.
- **PR2** ([[csl26-kqji]], #766): descendant-of-preset-base wrapper rewrite.
- **PR3** ([[csl26-39tm]], #767): evidence emission, reverse-template-link discovery, `--minimize-wrapper`, apa-6th-edition 5,661 → 5 LOC with fidelity improvement.
- **PR4** ([[csl26-ly8d]]): extend the minimize path to template-link candidates (mdpi and the numeric cluster) so they can be considered for compression too.
- **PR5** (not yet filed): author a Vancouver / numeric-journal preset base and repeat the rewrite pass.
- **PR6** (not yet filed): auto-derive candidate families from cluster fingerprints.

Each subsequent PR refreshes this scorecard and updates the baseline numbers in place.
