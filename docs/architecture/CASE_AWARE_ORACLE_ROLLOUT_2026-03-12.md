# Case-Aware Oracle Rollout 2026-03-12

## Purpose

Record the repository state after making oracle comparison case-aware by
default, adding explicit case-mismatch reporting to `scripts/report-core.js`,
and rolling title-case configuration updates into the shipped core styles that
were previously masked by case-insensitive scoring.

## What Changed

- Oracle comparison now treats case-only bibliography differences as failures by
  default across `oracle.js`, `oracle-fast.js`, and `oracle-yaml.js`.
- `report-core.js` now records `caseMismatchesOverall` at the report level and
  `caseMismatches` per style.
- The oracle CLI scripts no longer call `process.exit()` immediately after
  writing large JSON payloads, which had been truncating piped output under
  `report-core`.
- Title rendering configuration was updated for:
  - `styles/apa-7th.yaml`
  - `styles/chicago-author-date.yaml`
  - `styles/elsevier-with-titles.yaml`
  - `styles/chem-rsc.yaml`

## Result

Running `node scripts/report-core.js` with case-aware scoring leaves title/text
case regressions at zero across the core style set.

Remaining case mismatch count:

- `american-mathematical-society-label`: `1`

That remaining delta is not a title-rendering issue. It is a citation-label case
difference (`Nasa24` vs `NASA24`) and should be handled as separate label
generation work rather than folded into title-case policy.

## Verification-Policy Impact

No new title-related verification-policy divergence was added in this rollout.
The remaining `american-mathematical-society-label` delta is intentionally left
unadjusted so reports continue to surface the unresolved citation-label case
behavior.
