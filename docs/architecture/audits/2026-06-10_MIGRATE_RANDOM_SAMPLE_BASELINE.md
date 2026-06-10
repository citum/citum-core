# citum-migrate random-sample fidelity baseline

> Audit record for bean `csl26-rj4c` (epic `csl26-vmcr`). First converter
> measurement over a **random** corpus rather than the curated sentinel/lab
> set. Companion JSON snapshot:
> `scripts/report-data/migrate-random-baseline-2026-06-10.json`.

## Methodology

- Corpus: 100 of 2,844 independent parent styles in `styles-legacy/`,
  drawn by `report-migrate-sqi.js --corpus random --seed 20260610` —
  seeded, stratified by CSL `citation-format` class, fully reproducible.
- Fidelity: strict citeproc-js oracle comparison through
  `oracle.js --force-migrate` (checked-in `styles/*.yaml` cannot inflate
  results; every style renders from fresh `citum-migrate` output).
- Failures count against the headline: a style that cannot convert or
  render is in the denominator.
- Quality bar (decided 2026-06-10): publish-confident when ≥80% of sampled
  styles reach ≥90% combined strict citation+bibliography fidelity and no
  class falls below 60%.

## Verdict: below the bar

**43/100 styles at ≥90% combined strict fidelity** (bar: 80). Mean 83.2%,
median 89.7%. Per class share at threshold: author-date 57.5%, numeric
48.5%, author 20%, note 15.8%, label 0%. Both bar conditions fail; the
improvement wave (Phase 3 of the epic) activates before any public claims.

Two readings that shape the improvement plan:

1. **Citations are largely solved; bibliographies are the gap.** A majority
   of below-bar styles pass citations 20/20 while leaking 4–10 bibliography
   entries. The distribution is heavily banded at 82–90%: 24 styles are
   close misses. Converter fixes to a handful of shared bibliography
   patterns plausibly move the headline by tens of points.
2. **Structural quality is not the problem.** Migrated YAML SQI mean is
   94.9 — on par with the curated corpus (96.3 on 2026-05-20). The
   converter writes clean YAML that renders the wrong text.

## Failure clusters (initial classification)

Evidence: oracle entry diffs on representative styles per band, plus the
two hard failures. Classification follows the migrate-research taxonomy.

| # | Cluster | Evidence styles | Class mix | Classification | Est. reach |
|---|---|---|---|---|---|
| C1 | Emitted `type-variants` op references a component absent from the base template → processor hard-fails the whole style | zeitschrift-fur-fantastikforschung (`interview`), american-mathematical-society-label (`patent`) | any | migration-artifact (emit-time validation gap) | 2/100 direct, unknown corpus-wide |
| C2 | Bibliography drops container/periodical groups (journal name, year, volume vanish from rendered entries; citation-number prefix lost) | zeitschrift-fur-allgemeinmedizin (32/38), large 82–90% numeric/author-date band | numeric, author-date | migration-artifact | ~24 close-miss styles |
| C3 | Bibliography component order scrambled + label/affix leakage (`2017: 7: …: vol. 30: pp. …: available at`) | brazilian-journal-of-psychiatry (12/38), proceedings-of-the-estonian-academy-of-sciences-numeric (17/53) | numeric | migration-artifact | ~8 deep-failure styles |
| C4 | Note-class full-note citation templates lose delimiters/affixes wholesale (`"Title"PublisherPlace` run-on), wrong name form (full vs initials) | early-medieval-europe (cit 9/20), zeitschrift-fur-medienwissenschaft (7/20), note band | note, author | migration-artifact | note class: 16/19 below bar |
| C5 | Compact physics/abbreviated forms: article title should be suppressed, page-only locators (`37, 1 (1970)`) | springer-physics-author-date (11/38) | author-date | migration-artifact | small band, shared by physics family |

Cluster C1 is also a converter-correctness bug independent of fidelity:
`citum-migrate` must never emit YAML the processor rejects (validate
variant ops at emit time; drop or repair the op and record it in
evidence).

Secondary observation (not fidelity): 24 sampled styles had discovered
candidate parents but only 2 minimization attempts ran (both rejected).
The family-candidate path under-fires on the random corpus; worth a
separate look during the wave.

## Improvement plan

Tracked as child beans of epic `csl26-vmcr`; one bounded migrate-research
pass per cluster, re-measured with the same seed after each landing.
Priority order: C2 (largest reach) → C4 (worst class) → C1 (hard
failures, small fix) → C3 → C5. The wave target is the published bar:
80/100 at ≥90%, no class below 60%.

## Measured report (generated 2026-06-10)


- Generated: 2026-06-10T16:52:41.318Z
- Commit: 4016d7cf
- Corpus: random (100 of 2844 independent parents, seed 20260610)
- Strata (sampled/population): author 5/10, author-date 40/1335, label 3/3, note 19/542, numeric 33/954

## Fidelity headline

- Styles at ≥90% combined strict fidelity: 43/100 (43%) — non-ok: oracle_failed 2
- Combined strict fidelity (converted styles): mean 83.2%, p10 55.2%, median 89.7%, p90 100%

### Per class

| Class | Measured | ≥90% | Share | Mean | Median |
|---|---:|---:|---:|---:|---:|
| author | 5 | 1 | 20% | 86.9% | 87.7% |
| author-date | 40 | 23 | 57.5% | 87.8% | 94.8% |
| label | 3 | 0 | 0% | 46.6% | 74.1% |
| note | 19 | 3 | 15.8% | 70.6% | 75.9% |
| numeric | 33 | 16 | 48.5% | 86.4% | 89.8% |

### Failure taxonomy

| Style | Class | Status | Detail |
|---|---|---|---|
| zeitschrift-fur-fantastikforschung | author | oracle_failed | Processor failed: Error: template variant operation in `bibliography.type-variants[interview]` matched no component  |
| american-mathematical-society-label | label | oracle_failed | Processor failed: Error: template variant operation in `bibliography.type-variants[patent]` matched no component  |

## Aggregate

| Subject | n | mean | p10 | p50 | p90 |
|---|---:|---:|---:|---:|---:|
| Migrated YAML SQI | 98 | 94.9 | 86.03 | 97.5 | 99.27 |
| Public YAML SQI | 5 | 95.93 | 93.33 | 96.67 | 99.17 |
| Migrated − Public | 4 | -0.42 | -3.07 | -0.44 | 2.76 |

## Per-style

| Style | Fidelity | Migrated SQI | Public SQI | Δ | LOC | Migrated dup/near/rep | Public dup/near/rep |
|---|---:|---:|---:|---:|---:|---|---|
| chicago-in-text-shortened-author | 16/20 • 34/37 | 95.03 | err | - | 665 | 0/0/85 | - |
| chicago-in-text-shortened-author-no-url | 16/20 • 33/37 | 96.77 | err | - | 552 | 0/0/59 | - |
| chicago-in-text-shortened-author-title-no-url | 20/20 • 32/37 | 96.7 | err | - | 543 | 0/0/59 | - |
| modern-language-association-annotated-bibliography | 14/20 • 34/38 | 90.67 | err | - | 771 | 0/1/108 | - |
| zeitschrift-fur-fantastikforschung | -/- • -/- | err | err | - | - | - | - |
| antarctic-science | 19/20 • 38/38 | 92.77 | err | - | 224 | 0/0/3 | - |
| anthropologie-et-societes | 20/20 • 37/38 | 96.03 | err | - | 236 | 0/0/4 | - |
| australian-road-research-board | 18/20 • 38/38 | 85.57 | err | - | 213 | 0/0/15 | - |
| berlin-school-of-economics-and-law-international-marketing-management | 20/20 • 36/38 | 85.73 | err | - | 143 | 0/0/6 | - |
| bio-protocol | 6/20 • 31/38 | 76.27 | err | - | 1526 | 0/0/193 | - |
| canadian-biosystems-engineering | 20/20 • 32/38 | 92.87 | err | - | 510 | 0/0/103 | - |
| carolinea | 20/20 • 38/38 | 98.9 | err | - | 293 | 0/0/11 | - |
| civitas-revista-de-ciencias-sociais | 19/20 • 26/38 | 99.4 | err | - | 327 | 0/0/3 | - |
| evolution-letters | 20/20 • 37/38 | 99.37 | err | - | 209 | 0/0/2 | - |
| freshwater-crayfish | 20/20 • 38/38 | 98.63 | err | - | 344 | 0/0/15 | - |
| harvard-coventry-university | 19/20 • 33/38 | 84.03 | err | - | 1338 | 0/0/90 | - |
| harvard-durham-university-business-school | 19/20 • 36/38 | 97.77 | err | - | 449 | 0/0/44 | - |
| history-of-the-human-sciences | 20/20 • 38/38 | 86.03 | err | - | 177 | 0/0/3 | - |
| institut-national-de-sante-publique-du-quebec-napp | 20/20 • 32/38 | 99 | err | - | 214 | 0/0/10 | - |
| instituto-de-pesquisas-energeticas-e-nucleares | 7/20 • 28/38 | 99.13 | err | - | 225 | 0/0/7 | - |
| interkulturelle-germanistik-gottingen | 17/20 • 36/38 | 97.8 | err | - | 408 | 0/0/19 | - |
| jcom-journal-of-science-communication | 14/20 • 25/38 | 91 | err | - | 1271 | 0/1/147 | - |
| journal-of-advertising-research | 18/20 • 14/46 | 98.07 | err | - | 382 | 0/0/11 | - |
| journal-of-applied-entomology | 20/20 • 37/38 | 99.3 | err | - | 181 | 0/0/4 | - |
| journal-of-contemporary-water-research-and-education | 19/20 • 35/38 | 85.8 | err | - | 1173 | 0/0/208 | - |
| lien-social-et-politiques | 20/20 • 37/38 | 96.03 | err | - | 260 | 0/0/3 | - |
| mammalia | 20/20 • 24/39 | 98.03 | err | - | 354 | 0/0/24 | - |
| media-culture-and-society | 20/20 • 38/38 | 92.17 | err | - | 236 | 0/0/14 | - |
| molecular-psychiatry | 20/20 • 33/38 | 98.37 | err | - | 323 | 0/0/18 | - |
| ocean-and-coastal-research | 20/20 • 37/38 | 91.83 | err | - | 769 | 0/0/107 | - |
| oecologia-australis | 19/20 • 36/38 | 93.27 | err | - | 771 | 0/0/102 | - |
| pacific-science | 20/20 • 28/38 | 93.2 | err | - | 575 | 0/0/101 | - |
| pakistan-journal-of-agricultural-sciences | 20/20 • 30/38 | 98 | err | - | 324 | 0/0/16 | - |
| preslia | 19/20 • 36/38 | 85.8 | err | - | 188 | 0/0/5 | - |
| raptor-journal | 20/20 • 31/38 | 98.3 | err | - | 310 | 0/0/9 | - |
| reproduction | 17/20 • 38/38 | 96 | err | - | 221 | 0/0/3 | - |
| social-cognitive-and-affective-neuroscience | 19/20 • 38/38 | 92.53 | err | - | 235 | 0/0/5 | - |
| soil-science-and-plant-nutrition | 20/20 • 32/38 | 98.07 | err | - | 362 | 0/0/34 | - |
| springer-physics-author-date | 20/20 • 11/38 | 96.1 | 99.17 | -3.07 | 166 | 0/0/1 | 0/0/6 |
| the-holocene | 20/20 • 36/38 | 98.47 | err | - | 267 | 0/0/17 | - |
| the-international-spectator | 8/20 • 31/38 | 97.23 | err | - | 375 | 0/0/53 | - |
| the-open-university-harvard | 20/20 • 31/42 | 94.5 | err | - | 492 | 0/0/66 | - |
| universidade-estadual-do-oeste-do-parana-programa-institucional-de-bolsas-de-iniciacao-cientifica | 20/20 • 37/38 | 92.63 | err | - | 207 | 0/0/3 | - |
| universidade-estadual-paulista-campus-de-dracena-abnt | 9/20 • 34/38 | 95.13 | err | - | 320 | 0/0/23 | - |
| zeitschrift-fur-religionswissenschaft-author-date | 20/20 • 37/38 | 95.3 | err | - | 246 | 0/0/17 | - |
| american-mathematical-society-label | -/- • -/- | err | 93.8 | - | - | - | 0/2/27 |
| bibtex | 8/20 • 3/38 | 84.73 | err | - | 245 | 0/0/19 | - |
| din-1505-2-alphanumeric | 7/20 • 36/38 | 92.4 | 93.33 | -0.93 | 296 | 0/0/11 | 0/0/0 |
| anabases | 15/20 • 29/38 | 97.9 | err | - | 265 | 0/0/33 | - |
| bulletin-de-correspondance-hellenique | 15/20 • 31/38 | 98.43 | err | - | 187 | 0/0/15 | - |
| chicago-notes-bibliography-access-dates | 20/20 • 33/37 | 95.1 | err | - | 669 | 0/0/92 | - |
| chicago-shortened-notes-bibliography-classic-archive-place-first-no-url | 20/20 • 33/37 | 97.5 | err | - | 534 | 0/0/58 | - |
| china-information | 11/20 • 8/41 | 93.87 | err | - | 856 | 0/0/85 | - |
| donau-universitat-krems-department-fur-e-governance-in-wirthschaft-und-verwaltung | 17/20 • 35/38 | 96.03 | err | - | 518 | 0/0/72 | - |
| early-medieval-europe | 9/20 • 0/0 | 56.67 | err | - | 71 | 0/0/0 | - |
| histoire-at-politique | 18/20 • 34/38 | 94.37 | err | - | 263 | 0/1/36 | - |
| iso690-full-note-es | 5/20 • 34/38 | 99.2 | err | - | 309 | 0/0/9 | - |
| law-technology-and-humans | 18/20 • 35/38 | 95.9 | err | - | 612 | 0/0/83 | - |
| mohr-siebeck-recht | 12/20 • 23/38 | 89.33 | err | - | 120 | 0/0/3 | - |
| new-harts-rules-notes-initials-bracket-role-page-range-no-url | 18/20 • 30/37 | 99.1 | err | - | 242 | 0/0/9 | - |
| pravny-obzor | 9/20 • 33/38 | 98.83 | err | - | 240 | 0/0/10 | - |
| seminaire-saint-sulpice-ecole-theologie | 16/20 • 15/38 | 98.7 | err | - | 300 | 0/0/21 | - |
| stuttgart-media-university | 14/20 • 21/38 | 97.5 | err | - | 317 | 0/0/40 | - |
| the-journal-of-transport-history | 16/20 • 32/38 | 98.1 | err | - | 284 | 0/1/22 | - |
| universite-de-sherbrooke-histoire | 16/20 • 34/38 | 98.43 | err | - | 309 | 0/0/26 | - |
| vienna-legal | 14/20 • 16/38 | 97.4 | err | - | 486 | 0/0/49 | - |
| zeitschrift-fur-medienwissenschaft | 7/20 • 0/0 | 56.67 | err | - | 48 | 0/0/0 | - |
| advanced-science | 20/20 • 22/38 | 89.47 | err | - | 101 | 0/0/3 | - |
| animal-migration | 20/20 • 17/38 | 98.4 | err | - | 270 | 0/0/6 | - |
| brazilian-journal-of-psychiatry | 20/20 • 12/38 | 98.33 | err | - | 317 | 0/0/19 | - |
| cellular-and-molecular-bioengineering | 20/20 • 37/38 | 97.5 | err | - | 379 | 0/0/58 | - |
| clinical-journal-of-sport-medicine | 20/20 • 36/38 | 99.33 | err | - | 195 | 0/0/5 | - |
| din-1505-2-numeric | 20/20 • 35/38 | 99.27 | err | - | 315 | 0/0/5 | - |
| endoscopia | 20/20 • 38/38 | 97.47 | err | - | 384 | 0/0/43 | - |
| european-journal-of-emergency-medicine | 20/20 • 38/38 | 99.37 | err | - | 245 | 0/0/0 | - |
| frontiers-medical-journals | 20/20 • 38/38 | 96.23 | 96.67 | -0.44 | 197 | 0/0/0 | 0/0/0 |
| heart-rhythm | 20/20 • 38/38 | 96.53 | err | - | 379 | 0/0/54 | - |
| journal-of-cardiothoracic-and-vascular-anesthesia | 20/20 • 28/38 | 97.63 | err | - | 310 | 0/0/38 | - |
| journal-of-industrial-and-engineering-chemistry | 20/20 • 30/38 | 98.07 | err | - | 345 | 0/0/8 | - |
| journal-of-magnetic-resonance-imaging | 20/20 • 33/38 | 99 | err | - | 348 | 0/0/4 | - |
| journal-of-nanoscience-and-nanotechnology | 20/20 • 23/38 | 89.17 | err | - | 114 | 0/0/6 | - |
| journal-of-pediatric-gastroenterology-and-nutrition | 15/20 • 37/38 | 98.77 | err | - | 319 | 0/0/8 | - |
| journal-of-the-american-animal-hospital-association | 20/20 • 15/38 | 98.77 | err | - | 314 | 0/0/6 | - |
| journal-of-the-american-ceramic-society | 20/20 • 33/38 | 98.27 | err | - | 306 | 0/0/5 | - |
| malaysian-orthopaedic-journal | 20/20 • 37/38 | 97.7 | err | - | 435 | 0/0/38 | - |
| mycobiology | 20/20 • 37/38 | 99.5 | err | - | 177 | 0/0/2 | - |
| ophthalmic-genetics | 20/20 • 37/38 | 98.2 | err | - | 448 | 0/0/22 | - |
| opto-electronic-advances | 20/20 • 38/38 | 99.37 | err | - | 141 | 0/0/3 | - |
| pest-management-science | 20/20 • 33/38 | 99.5 | err | - | 241 | 0/0/3 | - |
| proceedings-of-the-estonian-academy-of-sciences-numeric | 20/20 • 17/53 | 97.13 | err | - | 486 | 0/0/38 | - |
| proceedings-of-the-royal-society-b | 20/20 • 37/38 | 99.43 | 96.67 | 2.76 | 150 | 0/0/3 | 0/0/0 |
| radiation-protection-dosimetry | 15/20 • 36/38 | 99 | err | - | 340 | 0/0/11 | - |
| sanamed | 20/20 • 38/38 | 98.47 | err | - | 397 | 0/0/10 | - |
| scientia-iranica | 20/20 • 33/39 | 98 | err | - | 319 | 0/0/20 | - |
| the-journal-of-adhesive-dentistry | 10/20 • 36/38 | 98.2 | err | - | 231 | 0/1/10 | - |
| the-journal-of-pure-and-applied-chemistry-research | 20/20 • 19/38 | 96.9 | err | - | 407 | 0/0/44 | - |
| veterinary-record | 20/20 • 28/38 | 92.83 | err | - | 204 | 0/0/1 | - |
| westfalische-wilhelms-universitat-munster-medizinische-fakultat | 20/20 • 25/38 | 97.8 | err | - | 360 | 0/0/39 | - |
| world-applied-sciences-journal | 20/20 • 30/38 | 98.63 | err | - | 268 | 0/0/4 | - |
| zeitschrift-fur-allgemeinmedizin | 20/20 • 32/38 | 98.43 | err | - | 289 | 0/0/5 | - |

Columns: Migrated/Public SQI is a simple mean of `concision`, `fallbackRobustness`, and `presetUsage` (0–100). LOC is migrated YAML output lines. dup/near/rep counts come from `qualityBreakdown.subscores.concision` diagnostics in `report-core.js`.

## Compression candidates

Styles where the migrator discovered a candidate parent via the registry, a source CSL link, or a reverse `<info><link rel="template">` in an embedded canonical style. The scorecard tries the minimized wrapper form (`--family-candidate auto --minimize-wrapper`) for each candidate and accepts it only when oracle citation pass ≥ standalone, bibliography pass ≥ standalone, LOC decreases, and every minimized citation/bibliography entry is strictly equivalent after normalization.

| Style | Candidate parent | Discovery source | Standalone LOC → Minimized LOC | Standalone fidelity → Minimized fidelity | Accepted |
|---|---|---|---:|---|:---:|
| modern-language-association-annotated-bibliography | modern-language-association | template-link | 789 → - | - → - | – |
| zeitschrift-fur-fantastikforschung | modern-language-association | template-link | 292 → - | - → - | – |
| bio-protocol | apa-7th | template-link | 1536 → - | - → - | – |
| journal-of-contemporary-water-research-and-education | chicago-author-date-18th | template-link | 1188 → - | - → - | – |
| mammalia | cse-name-year | template-link | 358 → - | - → - | – |
| oecologia-australis | apa-7th | template-link | 781 → - | - → - | – |
| pacific-science | apa-7th | template-link | 576 → - | - → - | – |
| springer-physics-author-date | american-physics-society | template-link | 166 → 166 | 1/1 • 11/38 → 1/1 • 11/38 | ✗ |
| the-holocene | sage-harvard | template-link | 286 → - | - → - | – |
| zeitschrift-fur-religionswissenschaft-author-date | american-sociological-association | template-link | 264 → - | - → - | – |
| american-mathematical-society-label | elsevier-with-titles | local-extends | 103 → - | - → - | – |
| new-harts-rules-notes-initials-bracket-role-page-range-no-url | new-harts-rules-notes | template-link | 260 → - | - → - | – |
| cellular-and-molecular-bioengineering | nlm-citation-sequence-superscript | template-link | 381 → - | - → - | – |
| endoscopia | elsevier-vancouver | template-link | 387 → - | - → - | – |
| european-journal-of-emergency-medicine | bmj | template-link | 258 → - | - → - | – |
| frontiers-medical-journals | frontiers | template-link | 197 → 197 | 1/1 • 38/38 → 1/1 • 38/38 | ✗ |
| journal-of-cardiothoracic-and-vascular-anesthesia | nlm-citation-sequence | template-link | 322 → - | - → - | – |
| journal-of-industrial-and-engineering-chemistry | elsevier-vancouver | template-link | 351 → - | - → - | – |
| journal-of-magnetic-resonance-imaging | biomed-central | template-link | 352 → - | - → - | – |
| journal-of-the-american-ceramic-society | american-medical-association | template-link | 318 → - | - → - | – |
| mycobiology | elsevier-vancouver | template-link | 183 → - | - → - | – |
| opto-electronic-advances | nature | template-link | 154 → - | - → - | – |
| scientia-iranica | the-optical-society | template-link | 339 → - | - → - | – |
| the-journal-of-pure-and-applied-chemistry-research | american-chemical-society | template-link | 421 → - | - → - | – |
| world-applied-sciences-journal | elsevier-with-titles | template-link | 287 → - | - → - | – |

## Output Diagnostics

| Style | LOC | Template components | Bibliography template | Bibliography variants | Pathological |
|---|---:|---:|---:|---:|---|
| apa-6th-edition | 994 | 189 | 30 | 35 | no |
