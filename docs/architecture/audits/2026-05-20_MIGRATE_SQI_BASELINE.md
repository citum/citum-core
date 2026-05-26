# citum-migrate SQI baseline

- Generated: 2026-05-21T15:58:07.054Z
- Commit: 7753f82d
- Corpus: both (18 styles)

## Aggregate

| Subject | n | mean | p10 | p50 | p90 |
|---|---:|---:|---:|---:|---:|
| Migrated YAML SQI | 18 | 96.3 | 89.1 | 99.07 | 100 |
| Public YAML SQI | 17 | 92.46 | 83.93 | 96.27 | 100 |
| Migrated − Public | 17 | 3.87 | -0.27 | 2.4 | 10.87 |

## Per-style

| Style | Fidelity | Migrated SQI | Public SQI | Δ | LOC | Migrated dup/near/rep | Public dup/near/rep |
|---|---:|---:|---:|---:|---:|---|---|
| apa | 18/18 • 33/33 | 100 | 88.37 | 11.63 | 5 | 0/0/0 | 1/2/161 |
| apa-6th-edition | 18/18 • 30/34 | 95.57 | n/a | - | 993 | 0/0/84 | - |
| elsevier-harvard | 18/18 • 34/34 | 100 | 100 | 0.00 | 67 | 0/0/0 | 0/0/0 |
| elsevier-with-titles | 18/18 • 34/34 | 100 | 100 | 0.00 | 24 | 0/0/0 | 0/0/0 |
| elsevier-vancouver | 18/18 • 34/34 | 100 | 100 | 0.00 | 52 | 0/0/0 | 0/0/0 |
| springer-basic-author-date | 18/18 • 34/34 | 100 | 100 | 0.00 | 65 | 0/0/0 | 0/0/0 |
| ieee | 18/18 • 34/34 | 94.8 | 83.93 | 10.87 | 265 | 0/0/31 | 4/14/95 |
| american-medical-association | 18/18 • 31/34 | 97.23 | 97.5 | -0.27 | 398 | 0/0/51 | 0/1/52 |
| nature | 18/18 • 32/34 | 99.07 | 96.67 | 2.40 | 178 | 0/0/9 | 0/0/0 |
| cell | 18/18 • 32/34 | 99.47 | 96.27 | 3.20 | 177 | 0/0/3 | 0/0/3 |
| chicago-author-date | 18/18 • 33/33 | 100 | 98.23 | 1.77 | 5 | 0/0/0 | 0/0/43 |
| chicago-notes | 18/18 • 0/0 | 66.67 | 58.93 | 7.74 | 5 | 0/0/0 | 1/2/109 |
| oscola | 11/18 • 32/34 | 98.53 | 89.1 | 9.43 | 332 | 0/0/21 | 0/0/8 |
| karger-journals | 18/18 • 33/34 | 99.03 | 89.27 | 9.76 | 253 | 0/0/4 | 0/0/5 |
| institute-of-physics-numeric | 18/18 • 34/34 | 89.1 | 89.37 | -0.27 | 156 | 0/0/7 | 0/0/3 |
| thieme-german | 18/18 • 34/34 | 98.7 | 95.73 | 2.97 | 276 | 0/0/10 | 0/0/10 |
| multidisciplinary-digital-publishing-institute | 18/18 • 33/34 | 95.17 | 88.53 | 6.64 | 237 | 0/0/21 | 0/0/20 |
| taylor-and-francis-chicago-author-date | 18/18 • 33/33 | 100 | 100 | 0.00 | 50 | 0/0/0 | 0/0/0 |

Columns: Migrated/Public SQI is a simple mean of `concision`, `fallbackRobustness`, and `presetUsage` (0–100). LOC is migrated YAML output lines. dup/near/rep counts come from `qualityBreakdown.subscores.concision` diagnostics in `report-core.js`. `n/a` means the style has no public YAML baseline in the scorecard corpus.

## Compression candidates

Styles where the migrator discovered a candidate parent via the registry, a source CSL link, or a reverse `<info><link rel="template">` in an embedded canonical style. The scorecard tries the minimized wrapper form (`--family-candidate auto --minimize-wrapper`) for each candidate and accepts it only when oracle citation pass ≥ standalone, bibliography pass ≥ standalone, LOC decreases, and every minimized citation/bibliography entry is strictly equivalent after normalization.

| Style | Candidate parent | Discovery source | Standalone LOC → Minimized LOC | Standalone fidelity → Minimized fidelity | Accepted |
|---|---|---|---:|---|:---:|
| apa | apa-7th | registry-alias | 2481 → - | - → - | – |
| apa-6th-edition | apa-7th | reverse-template-link | 993 → 5 | 1/1 • 30/34 → 0/1 • 33/34 | ✗ |
| elsevier-harvard | elsevier-harvard-core | local-extends | 324 → - | - → - | – |
| elsevier-with-titles | elsevier-with-titles-core | local-extends | 124 → - | - → - | – |
| elsevier-vancouver | elsevier-vancouver-core | local-extends | 182 → - | - → - | – |
| springer-basic-author-date | springer-basic-author-date-core | local-extends | 231 → - | - → - | – |
| chicago-author-date | chicago-author-date-18th | registry-alias | 8610 → - | - → - | – |
| chicago-notes | chicago-notes-18th | registry-alias | 74 → - | - → - | – |
| multidisciplinary-digital-publishing-institute | american-chemical-society | template-link | 237 → 237 | 1/1 • 33/34 → 1/1 • 33/34 | ✗ |
| taylor-and-francis-chicago-author-date | taylor-and-francis-chicago-author-date-core | local-extends | 9034 → - | - → - | – |

## Output Diagnostics

| Style | LOC | Template components | Bibliography template | Bibliography variants | Pathological |
|---|---:|---:|---:|---:|---|
| apa-6th-edition | 993 | 189 | 30 | 35 | no |

Note: `apa-6th-edition` remains standalone. The LOC reduction comes from
pathological Rust XML-fallback bibliography cleanup in `citum-migrate`, not from
emitting an unsafe `apa-7th` wrapper; strict minimization still rejects that
candidate.
