# Style Quality Index (SQI)

SQI is a secondary quality metric for CSLN styles.

Use SQI to improve style maintainability only after fidelity is correct.

## Priority Order

1. Fidelity (hard gate): output must match the citeproc-js oracle.
2. SQI (secondary): choose cleaner, more robust style definitions when fidelity is comparable.

Never accept an SQI gain that causes a fidelity regression.

## What SQI Measures

SQI is computed per style from four subscores:

1. `typeCoverage`: how broadly the style succeeds across observed reference types.
2. `fallbackRobustness`: whether core types still render correctly via shared templates/fallback paths.
3. `concision`: duplication, override density, and repeated component patterns.
4. `presetUsage`: reuse of shared presets (`processing`, `contributors`, `dates`, `titles`, `substitute`, template presets).

Overall SQI is reported as a 0.0-1.0 score in JSON and as a percentage in `docs/compat.html`.

## Working Thresholds

Current wave target:

- `>= 0.95` fidelity
- `>= 0.90` SQI

These thresholds are used for wave planning and tracking, not as a replacement for oracle checks.

## Commands

Generate the core report:

```bash
node scripts/report-core.js > /tmp/core-report.json
```

Regenerate the compatibility dashboard:

```bash
node scripts/report-core.js --write-html
```

Check drift against CI baseline:

```bash
node scripts/check-core-quality.js \
  --report /tmp/core-report.json \
  --baseline scripts/report-data/core-quality-baseline.json
```

## Related

- [SQI refinement plan](../architecture/SQI_REFINEMENT_PLAN.md)
- [Rendering workflow](../guides/RENDERING_WORKFLOW.md)
- [Style author guide](../guides/style-author-guide.md)
