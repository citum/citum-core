# Engine Migrate Co-Evolution Wave Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-09
**Supersedes:** None
**Related:** csl26-ufx4, csl26-9a89, csl26-ctw8, csl26-oo5q, csl26-paok, csl26-6i1c, csl26-bpuw

## Purpose
Define an engine-first co-evolution wave that converts repeated style-fidelity failures into shared `citum_migrate` and `citum_engine` fixes before any residual style-local cleanup.

## Scope
In scope:
- repeated bibliography and citation gap reduction across a representative 10-style cohort
- `citum-migrate` template-resolution, merge-guardrail, and suppression improvements
- `citum-engine` fixes only when a valid migrated template still misrenders
- limited style-local cleanup for improved cohort styles
- selective preset extraction only when a new public preset is reused unchanged across at least three improved cohort styles

Out of scope:
- new CLI surface
- `Style.version` validation
- locator syntax redesign
- note-position feature expansion
- store/server work
- broad style-catalog expansion outside the selected cohort

## Design
The wave uses a fixed cohort of citation-cluster and bibliography-cluster styles to drive shared fixes:

- citation/locator cohort:
  - `association-for-computing-machinery`
  - `springer-fachzeitschriften-medizin-psychologie`
  - `institute-of-mathematics-and-its-applications`
  - `american-geophysical-union`
  - `annual-reviews-author-date`
- bibliography/migration cohort:
  - `gost-r-7-0-5-2008-author-date`
  - `mhra-author-date-publisher-place`
  - `future-medicine`
  - `royal-society-of-chemistry`
  - `international-union-of-crystallography`

Sentinel styles for regression checks:
- `apa-7th`
- `chicago-notes`
- `oscola`
- `oscola-no-ibid`

Implementation order:
1. Capture before-state artifacts for core quality and migration-gap clustering.
2. Reduce repeated bibliography clusters first in `citum-migrate`.
3. Reduce repeated locator, suppress-author, and citation-template recovery clusters in `citum-migrate`.
4. Apply `citum-engine` fixes only when the cohort proves the engine is wrong for a valid template.
5. Regenerate cohort styles and apply limited style-local cleanup.
6. Reassess preset extraction and only add a public preset if the reuse gate is met.

Allowed new preset families in this wave:
- numeric or chemistry journal bibliography preset
- author-date publisher or place bibliography preset

## Implementation Notes
Use `scripts/report-core.js` and `scripts/analyze-migration-gaps.js` as the wave-level scorecard.
Use `/style-evolve` for any residual style-local repair after shared fixes land.
Keep fidelity as the hard gate and treat SQI as secondary.

## Acceptance Criteria
- [ ] No regression in core quality or top-10 oracle regression checks.
- [ ] Shared fixes improve or preserve fidelity for all cohort styles before style-local cleanup.
- [ ] At least six cohort styles improve from shared migrate or engine changes.
- [ ] The top repeated citation and bibliography cluster totals each drop by at least one third without increasing any tracked cluster.
- [ ] `apa-7th` and `chicago-notes` retain fidelity and do not lose SQI.
- [ ] Any new public preset is adopted unchanged by at least three improved cohort styles in the same PR.

## Changelog
- v1.0 (2026-03-09): Initial draft and implementation activation.
