# Style Quality Index (SQI)

SQI is a secondary quality metric for Citum styles.

Use SQI to improve style maintainability only after fidelity is correct.

## Priority Order

1. Fidelity (hard gate): output must match the citeproc-js oracle.
2. SQI (secondary): choose cleaner, more robust style definitions when fidelity is comparable.

Never accept an SQI gain that causes a fidelity regression.

SQI is not the structural lint. Deterministic style-shape rules such as anonymous-anchor rejection and dead-config detection are enforced separately by `scripts/style-structure-lint.js`.

## What SQI Measures

SQI is computed per style from four subscores:

1. `typeCoverage`: how broadly the style succeeds across observed reference types.
2. `fallbackRobustness`: whether core types still render correctly via shared templates/fallback paths.
3. `concision`: measures how efficiently the style achieves its rendering goals through template reuse.
   - Counts all template-bearing scopes in the resolved style, including `type-variants` and `type-templates`.
   - Penalizes high variant-selector counts, exact duplicate scopes, near-duplicate scopes, and repeated copied component/group patterns across scopes.
   - Uses structural fingerprints of whole components and groups rather than coarse field-name matching, so copied template forks are visible to the metric.
4. `presetUsage`: reuse of shared presets (`processing`, `contributors`, `dates`, `titles`, `substitute`, template presets).

Overall SQI is reported as a 0.0-1.0 score in JSON and as a percentage in `docs/compat.html`.

`qualityBreakdown.subscores.concision` now includes supporting diagnostics such as scope count, variant count, exact duplicates, near-duplicates, and repeated-pattern totals so score changes are explainable.

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

- [SQI refinement plan](../policies/SQI_REFINEMENT_PLAN.md)
- [Rendering workflow](../guides/RENDERING_WORKFLOW.md)
- [Style author guide](../guides/style-author-guide.md)
