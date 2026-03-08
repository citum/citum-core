# Docs Map

This directory is organized for progressive disclosure: start with concise status,
then drill into strategy and implementation details only as needed.

## Start Here (Canonical)

1. [`TIER_STATUS.md`](./TIER_STATUS.md) - current strict oracle style status.
2. [`compat.html`](https://citum.github.io/citum-core/compat.html) - published compatibility snapshot.
3. [`guides/RENDERING_WORKFLOW.md`](./guides/RENDERING_WORKFLOW.md) - operational rendering and verification workflow.
4. [`architecture/ROADMAP.md`](./architecture/ROADMAP.md) - strategic direction and phase sequencing.

Generate the compatibility snapshot locally with `node scripts/report-core.js --output-html docs/compat.html`.

## Active Guides

1. [`guides/CODING_STANDARDS.md`](./guides/CODING_STANDARDS.md)
2. [`guides/STYLE_EVOLVE_WORKFLOW.md`](./guides/STYLE_EVOLVE_WORKFLOW.md)
3. [`guides/DOMAIN_EXPERT.md`](./guides/DOMAIN_EXPERT.md)
4. [`guides/style-author-guide.md`](./guides/style-author-guide.md)

## Architecture (Entry)

Use [`architecture/README.md`](./architecture/README.md) to navigate active
architecture/design docs versus historical snapshots.

## Historical Snapshot Policy

Date-stamped architecture docs (for example `*_2026-02-21.md`) are snapshots of
specific execution windows. They are useful for audit history but are not
canonical status sources.
