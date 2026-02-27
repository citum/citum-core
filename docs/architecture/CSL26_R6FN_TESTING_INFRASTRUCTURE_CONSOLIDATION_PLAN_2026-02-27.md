# CSL26-R6FN Testing Infrastructure Consolidation Plan

## Summary

`csl26-r6fn` no longer represents greenfield testing work. The repo already has
working oracle baselines, core-quality reporting, specialty fixtures, and
component-coverage tracking. The remaining need is to define the testing stack
as one coherent contract so CI, docs, and beans describe the same system.

This document is the canonical testing-layer map for that consolidation.

## Layer 1: Rust Correctness

- Purpose: validate processor, schema, parser, and integration semantics.
- Source files:
  - `crates/**`
  - `tests/**`
- Commands:
  - `cargo nextest run`
  - Fallback where needed: `cargo test`
- Failure meaning:
  - Core engine behavior, parsing, or integration semantics have regressed.
- Gate status:
  - CI required.

## Layer 2: Oracle Fidelity

- Purpose: compare Citum rendering against citeproc-js for canonical style
  fidelity.
- Source files:
  - `scripts/oracle.js`
  - `scripts/oracle-batch-aggregate.js`
  - `scripts/check-oracle-regression.js`
  - `scripts/report-data/oracle-top10-baseline.json`
- Commands:
  - `node scripts/oracle.js styles-legacy/apa.csl --json`
  - `node scripts/oracle-batch-aggregate.js styles-legacy/ --top 10`
  - `node scripts/check-oracle-regression.js --baseline scripts/report-data/oracle-top10-baseline.json`
- Failure meaning:
  - Rendered citation or bibliography output regressed relative to the pinned
    oracle baseline.
- Gate status:
  - CI required for the top-10 baseline set.

## Layer 3: Portfolio Quality

- Purpose: measure fidelity and SQI-style quality across production core styles.
- Source files:
  - `scripts/report-core.js`
  - `scripts/check-core-quality.js`
  - `scripts/report-data/core-quality-baseline.json`
- Commands:
  - `node scripts/report-core.js > /tmp/core-report.json`
  - `node scripts/check-core-quality.js --report /tmp/core-report.json --baseline scripts/report-data/core-quality-baseline.json`
- Failure meaning:
  - A production core style dropped below the hard fidelity gate, or quality
    drift exceeded baseline tolerances.
- Gate status:
  - CI required.

## Layer 4: Fixture-Family Specialty Coverage

- Purpose: protect rendering domains that are not fully represented by the
  canonical top-10 oracle baseline.
- Coverage manifest:
  - `tests/fixtures/coverage-manifest.json`
- Fixture families:
  - Note: `tests/fixtures/citations-note-expanded.json`
  - Legal: `tests/fixtures/references-legal.json`
  - Scientific: `tests/fixtures/references-scientific.json`
  - Multilingual: `tests/fixtures/references-multilingual.yaml` and
    `tests/fixtures/multilingual/*.json`
  - Grouping: `tests/fixtures/grouping/*.json`
- Commands:
  - `node scripts/check-testing-infra.js`
  - Domain/runtime tests remain in `cargo nextest run`
- Failure meaning:
  - Fixture ownership, metadata contracts, or specialty coverage declarations
    drifted out of sync with the repo.
- Gate status:
  - CI required for manifest and metadata validation.
  - Individual specialty fixtures remain covered by targeted Rust tests and
    docs-driven workflows rather than a heavyweight oracle gate.

## Canonical Policies

### Canonical CI Baselines

- Oracle regression baseline:
  - `scripts/report-data/oracle-top10-baseline.json`
- Core-quality baseline:
  - `scripts/report-data/core-quality-baseline.json`

These are the only committed baseline artifacts used as CI gate inputs.

### Ad Hoc Local Baselines

Files under `baselines/` are for local milestone snapshots, refactor safety
checks, and pre/post comparison work. They are not authoritative CI inputs.

### Metadata Contract

Committed baseline/report artifacts must include:

- `metadata.timestamp`
- `metadata.gitCommit`
- `metadata.fixture`
- `metadata.generator`
- `metadata.styles` or `metadata.styleSelector`

### Fixture Governance

The manifest in `tests/fixtures/coverage-manifest.json` is the source of truth
for:

- which fixture files are canonical
- which risk classes they cover
- which scripts/tests own them
- whether they are CI-required or advisory

## Operational Workflow

1. Run Rust tests for semantic correctness.
2. Run oracle regression checks for pinned top-10 style fidelity.
3. Run core-quality reporting for production-style fidelity + SQI drift.
4. Run `node scripts/check-testing-infra.js` to verify fixture governance and
   metadata contracts.
5. Refresh committed baselines only in dedicated baseline PRs with explicit
   justification.

## Related Files

- `baselines/README.md`
- `docs/TIER_STATUS.md`
- `docs/guides/RENDERING_WORKFLOW.md`
- `tests/fixtures/coverage-manifest.json`
- `scripts/check-testing-infra.js`
