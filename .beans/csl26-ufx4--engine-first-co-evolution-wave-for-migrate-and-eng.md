---
# csl26-ufx4
title: Engine-first co-evolution wave for migrate and engine fidelity
status: in-progress
type: feature
priority: high
created_at: 2026-03-09T15:53:54Z
updated_at: 2026-03-09T17:33:00Z
---

Spec: docs/specs/ENGINE_MIGRATE_COEVOLUTION_WAVE.md

Related beans:
- csl26-9a89
- csl26-ctw8
- csl26-oo5q
- csl26-paok
- csl26-6i1c
- csl26-bpuw

Cohort:
- association-for-computing-machinery
- springer-fachzeitschriften-medizin-psychologie
- institute-of-mathematics-and-its-applications
- american-geophysical-union
- annual-reviews-author-date
- gost-r-7-0-5-2008-author-date
- mhra-author-date-publisher-place
- future-medicine
- royal-society-of-chemistry
- international-union-of-crystallography

Sentinels:
- apa-7th
- chicago-notes
- oscola
- oscola-no-ibid

Artifacts:
- /tmp/core-report.before.json
- /tmp/gaps.before.json
- /tmp/coev-wave-oracles/*.before.json

Checklist:
- [x] Create draft spec and baseline artifacts
- [x] Add divergence-aware verification and skill preflight so `div-004`
  mismatches stop counting as defects
- [x] Keep valid shared fixes already on branch: locator normalization in
  `citum-migrate` and numeric-only citation-file batching in `citum`
- [x] Remove the invalid cited-subset numbering experiment from the engine
- [x] Run full verification and open PR
- [ ] Decide whether any remaining narrow pre-1.0 cleanup merits follow-on work

Rescope notes (2026-03-09):
- The original broad engine-first wave was too wide because a major portion of
  the apparent oracle gap was the documented `div-004` sort divergence rather
  than an engine defect.
- Remaining pre-1.0 work should be tracked in narrower beans rather than as one
  large co-evolution push.
- Current narrow targets are already represented by existing beans:
  - `csl26-9a89` for real bibliography rendering edge cases
  - `csl26-6i1c` and `csl26-bpuw` for Chicago notes/humanities fixture recovery
  - `csl26-ctw8` for migration strip-periods extraction follow-up
