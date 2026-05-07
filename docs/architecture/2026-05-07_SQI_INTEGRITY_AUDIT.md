# SQI Integrity Audit

**Date:** 2026-05-07
**Scope:** `scripts/report-core.js`, full `styles/` report corpus
**Related:** `docs/reference/SQI.md`, `docs/specs/EMBEDDED_ROOT_WRAPPER_MIGRATION.md`

## Summary

The pre-audit SQI average was not a reliable maintainability signal for inherited
styles. Root `extends:` wrappers with no authored templates were scored from
resolved parent templates, so inherited parent complexity was charged to the
thin wrapper. Diff-form `type-variants` were also excluded from component
budgets but still participated in duplicate and near-duplicate penalties.

The audited calculation keeps fidelity unchanged, scores authored wrapper shape
from authored YAML, and treats Template V3 diff variants as patch operations
rather than copied full templates.

## Measurements

Commands:

```bash
node scripts/report-core.js --timings > /tmp/core-report-current.json
node scripts/report-core.js --timings > /tmp/core-report-audited.json
node scripts/check-core-quality.js \
  --report /tmp/core-report-audited.json \
  --baseline scripts/report-data/core-quality-baseline.json
```

Results:

| Metric | Before | After |
|---|---:|---:|
| Styles | 154 | 154 |
| Average SQI | 0.929591 | 0.957409 |
| Reported SQI | 0.930 | 0.957 |
| Minimum SQI | 0.776 | 0.831 |
| Median SQI | 0.938 | 0.968 |
| Styles below 0.90 SQI | 34 | 15 |
| Styles below 0.95 SQI | 92 | 52 |
| Quality-gate warnings | 0 | 0 |

The audited average improved by `+0.027818`, exceeding the `+0.010` PR target.

## Findings

1. Root wrappers were misattributed.
   - `cell-numeric`, `disability-and-rehabilitation`, and
     `elsevier-with-titles` all inherited the same resolved parent concision
     burden: 236 components, 19 scopes, and SQI concision `22.3`.
   - After the fix, those wrappers report `inheritedPreset` and score local
     concision as a pure root wrapper.

2. Diff variants were over-penalized.
   - Object-form `modify`, `remove`, and `add` operations are now reported as
     `diffVariantScopes` and `diffVariantOperations`.
   - They still count selector breadth, but no longer create duplicate,
     near-duplicate, or repeated-pattern penalties.

3. Preset reuse needed the same authorship boundary.
   - Pure root wrappers now treat top-level `extends:` as strong embedded preset
     reuse.
   - This prevents the corrected authorship data from producing false preset
     usage regressions against the CI baseline.

## Residual Risk

The audit corrects SQI attribution; it does not claim remaining low-tail SQI
styles are clean. The low tail after this pass is more meaningful: styles such
as `mhra-notes-publisher-place-no-url`, `ieee`, and structural journal wrappers
still require style or infrastructure work rather than metric repair.
