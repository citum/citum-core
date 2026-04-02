# Migrate-Research Rich-Input Pass Contract

**Status:** Draft
**Date:** 2026-04-02
**Related:** [FIDELITY_RICH_INPUTS.md](./FIDELITY_RICH_INPUTS.md), [DIVERGENCE_REGISTER.md](../adjudication/DIVERGENCE_REGISTER.md), `.beans/csl26-tpmn--chicago-author-date-chicago-18-rich-fidelity-follo.md`

## Purpose
Define the bounded rich-input workflow for `migrate-research` when a style-fidelity pass needs supplemental benchmark evidence without falling back to full-corpus exploration as the first debugging surface.

## Pass Contract
Each pass must target exactly one bounded cluster.

Required pass fields:

- target style
- target cluster
- cluster selector
- primary oracle before and after
- official supplemental before and after
- cluster before and after
- classification
- hypothesis
- stop reason

## Evidence Ladder
Use evidence in this order:

1. Primary oracle hard gate
2. Official style-scoped report from `report-core`
3. Cluster-scoped supplemental evidence from a reduced rich-input fixture
4. Full supplemental benchmark rerun as confirmation

The full supplemental corpus is confirmation evidence, not the first debugging surface.

## Classification
Every pass must classify the chosen cluster before edits:

- `style-defect`
- `migration-artifact`
- `processor-defect`
- `intentional divergence`

Action routing:

- `migration-artifact` stays in `migrate-research`
- `style-defect` routes to `style-evolve upgrade`
- `processor-defect` routes to engine follow-up
- `intentional divergence` routes to adjudication

## Operator Commands
Primary oracle:

```bash
node scripts/oracle.js styles-legacy/chicago-author-date.csl --json > /tmp/chicago-primary.json
```

Official style report:

```bash
node scripts/report-core.js --style chicago-author-date > /tmp/chicago-report.json
```

Cluster extraction:

```bash
node scripts/extract-rich-benchmark-cluster.js \
  --style chicago-author-date \
  --benchmark chicago-zotero-bibliography \
  --type entry-dictionary,entry-encyclopedia \
  --only-mismatches \
  --out-dir /tmp/chicago-entry-cluster
```

Reduced cluster rerun:

```bash
node scripts/oracle.js styles-legacy/chicago-author-date.csl \
  --json \
  --scope bibliography \
  --refs-fixture /tmp/chicago-entry-cluster/cluster-fixture.json \
  > /tmp/chicago-entry-cluster/cluster-oracle.json
```

## Output Artifacts
The extractor writes:

- `cluster-fixture.json`
- `cluster-before.json`
- `cluster-summary.json`

These are operator artifacts only. They do not redefine official reporting or hard gates.

If `--only-mismatches` cannot safely map every failing row back to a source item
id, the extractor must preserve the selected cluster and report the unresolved
row count instead of over-reducing to a non-reproducible subset.

## V1 Limits
- bibliography-only supplemental extraction
- explicit selectors only: `--type` or `--ids`
- no changes to `report-core` published JSON shape
- no heuristic cluster inference in the extractor
