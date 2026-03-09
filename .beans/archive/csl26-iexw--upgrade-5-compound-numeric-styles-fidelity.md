---
# csl26-iexw
title: Upgrade 5 compound-numeric styles fidelity
status: completed
type: task
priority: normal
created_at: 2026-03-07T19:53:28Z
updated_at: 2026-03-09T11:35:00-04:00
---

Upgrade all 5 compound-numeric styles to improve biblatex fidelity:
numeric-comp, chem-acs, angewandte-chemie, chem-rsc, and chem-biochem.
This bean is complete and stale at the root because the landing work already
shipped on `main`.

Closure evidence:
- `main` contains commit `75bcf71` with explicit `Refs: csl26-iexw`
- That commit lands the compound-numeric fidelity work and comparator fixes
- The active bean lifecycle policy requires completed stale beans to be
  archived promptly

## Summary of Changes

- `scripts/report-core.js` now expands compound bibliography snapshot blocks
  before comparison via `expandCompoundBibEntries()`, so grouped biblatex
  snapshots are scored against the same logical entries as the oracle.
- `scripts/report-core.test.js` contains regression coverage for the compound
  bibliography comparator path added in the landing work.
- The landed style work updated all five target styles:
  `styles/angewandte-chemie.yaml`, `styles/chem-acs.yaml`,
  `styles/chem-biochem.yaml`, `styles/chem-rsc.yaml`, and
  `styles/numeric-comp.yaml`.
- Current focused `report-core` results confirm the bean is closure-worthy
  rather than still active:
  - `angewandte-chemie`: citations `25/26`, bibliography `33/33`,
    fidelity `0.983`
  - `chem-acs`: citations `26/26`, bibliography `32/33`, fidelity `0.983`
  - `chem-biochem`: citations `25/26`, bibliography `32/33`,
    fidelity `0.966`
  - `chem-rsc`: citations `26/26`, bibliography `31/33`, fidelity `0.966`
  - `numeric-comp`: bibliography `33/33`, fidelity `1.000`
