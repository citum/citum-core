# APA SQI Alignment and Preset Refactor Specification

**Status:** Active
**Date:** 2026-04-11
**Related:** `docs/reference/SQI.md`, `styles/apa-7th.yaml`, `styles/preset-bases/apa-7th.yaml`

## Purpose

Correct the Style Quality Index so it measures real duplication in resolved Citum styles, then use that corrected metric to drive a preset-first cleanup of APA 7th. The goal is to stop over-scoring duplicate-heavy styles and to reduce APA duplication without changing rendered output.

## Scope

In scope: `scripts/report-core.js` SQI scoring, `scripts/report-core.test.js`, APA embedded template presets, APA preset-backed YAML styles, and generated report artifacts (`docs/compat.html`, `scripts/report-data/core-quality-baseline.json`).

Out of scope: new public schema keys, non-APA style refactors, processor behavior changes unrelated to preserving APA fidelity, and any weakening of the fidelity gate.

## Design

### SQI scoring alignment

SQI concision must evaluate all template-bearing scopes in the resolved style, including `type-variants` and `type-templates`. It must penalize:

- high counts of variant scopes
- exact duplicate variant scopes
- near-duplicate scopes that differ only by a small number of component changes
- repeated copied component/group patterns across scopes

The metric must use structural fingerprints of whole components and groups rather than the current coarse semantic-key approximation. The corrected duplicate-heavy APA structure should no longer land near `A`; it should score closer to the `C` band until the refactor removes the duplication.

### APA preset-first refactor

APA citation templates should not require inline `type-variants`. Shared citation and bibliography structure should move into the shared embedded APA template preset, with YAML using `use-preset: apa` plus only the minimum true deltas.

The public `styles/apa-7th.yaml` file should remain a thin wrapper over the preset base. The preset base should also be compacted so preset resolution does not reintroduce duplicated variant trees.

### Compatibility and reporting

No new user-facing schema surface is introduced. The report output may expose richer concision diagnostics so SQI changes are explainable in JSON and in `compat.html`.

## Implementation Notes

- Prefer component overrides and preset reuse over duplicated `type-variants`.
- Preserve fidelity exactly; if a compaction attempt changes output, keep the higher-fidelity structure and move reuse lower into the preset rather than re-expanding the public style.
- Update SQI docs to match the implemented metric after the refactor lands.

## Acceptance Criteria

- [ ] `scripts/report-core.js` counts `type-variants` and `type-templates` in concision scoring.
- [ ] Duplicate-heavy APA-like structures are penalized by automated SQI regression tests.
- [ ] `styles/apa-7th.yaml` and `styles/preset-bases/apa-7th.yaml` use `citation.use-preset: apa` and `bibliography.use-preset: apa`.
- [ ] APA citation configuration has no inline `type-variants`.
- [ ] APA fidelity remains unchanged in report/oracle verification.
- [ ] `docs/compat.html` and `scripts/report-data/core-quality-baseline.json` are regenerated from the final implementation.

## Changelog

- 2026-04-11: Initial version and activation with implementation.
