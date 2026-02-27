# csl26-gczk Execution Plan: 95%+ Migration Component Coverage (2026-02-27)

## Objective
Establish a repeatable gate that reports migration component coverage and flags unmapped CSL migration gaps before merging style-wave work.

This plan operationalizes bean `csl26-gczk` on a dedicated PR branch using `/style-evolve migrate` loops.

## Scope For This PR
1. Add explicit component coverage threshold reporting (default: `0.95`) to migration gap analysis.
2. Aggregate unmapped components from oracle component diffs across styles.
3. Define PR workflow commands and artifacts so each migration wave can prove coverage and identify repeated gaps.

## Workflow (New PR)
1. Create branch: `codex/csl26-gczk-component-coverage`
2. Baseline report snapshot:
```bash
node scripts/report-core.js > /tmp/core-report.before.json
node scripts/analyze-migration-gaps.js \
  --report /tmp/core-report.before.json \
  --min-occurrences 2 \
  --component-threshold 0.95 > /tmp/gczk.before.json
```
3. Style migration loop (fidelity-first) with `/style-evolve`:
```bash
/style-evolve migrate --legacy styles-legacy/<style>.csl --count 1
node scripts/oracle.js styles-legacy/<style>.csl --json
```
4. After-wave snapshot:
```bash
node scripts/report-core.js > /tmp/core-report.after.json
node scripts/analyze-migration-gaps.js \
  --report /tmp/core-report.after.json \
  --min-occurrences 2 \
  --component-threshold 0.95 > /tmp/gczk.after.json
```
5. Compare before/after for:
- `componentCoverage.failStyles`
- `componentCoverage.average`
- `unmappedComponents`

## Required PR Artifacts
1. Before/after JSON reports (`/tmp/gczk.before.json`, `/tmp/gczk.after.json`).
2. Table of styles below 95% component coverage.
3. Top repeated unmapped components and proposed migration-engine follow-ups.

## Acceptance Criteria
1. Coverage metrics are available in one command (`analyze-migration-gaps.js`).
2. Styles below 95% component coverage are explicitly listed for triage.
3. Repeated unmapped components are aggregated across styles.
4. No fidelity regressions are introduced by this PR.

## Notes
- Fidelity remains the hard gate. Coverage is used to detect silent migration loss patterns earlier.
- This PR adds measurement and gating; it does not claim full closure of all unmapped components yet.
