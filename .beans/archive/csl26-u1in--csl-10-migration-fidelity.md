---
# csl26-u1in
title: CSL 1.0 Migration & Fidelity
status: completed
type: epic
priority: critical
created_at: 2026-02-07T12:11:33Z
updated_at: 2026-03-09T15:00:00Z
---

Core migration-fidelity program completed for the maintained portfolio.
Parser completeness, migration fidelity, oracle verification, and style coverage
now have a stable closure point for the core styles and top-10 oracle cohort.

Canonical references:
- docs/TIER_STATUS.md
- docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md
- docs/policies/SQI_REFINEMENT_PLAN.md

Exit criteria met:
- Core quality gate passes with fidelity `1.0` for all `146` styles in the
  current maintained portfolio.
- Top-10 oracle batch passes at `10/10` citation-perfect and `10/10`
  bibliography-perfect styles under the current strict fixture set.
- Residual long-tail parity work is tracked as narrower follow-on beans instead
  of remaining attached to this umbrella epic.

Follow-on work:
- csl26-iexw retains compound-numeric style fidelity improvements.
- Documented residual portfolio gaps remain tracked in
  `docs/policies/SQI_REFINEMENT_PLAN.md` until split into narrower beans or
  resolved.

## Summary of Changes

- Removed the stale blocker on `csl26-yxvz`, which had already been completed.
- Re-scoped this epic to the completed core migration-fidelity milestone rather
  than an unbounded long-tail parity umbrella.
- Refreshed the canonical SQI reference to `docs/policies/SQI_REFINEMENT_PLAN.md`.
- Closed the epic based on current repo evidence:
  - `node scripts/report-core.js > /tmp/core-report.json`
  - `node scripts/check-core-quality.js --report /tmp/core-report.json --baseline scripts/report-data/core-quality-baseline.json`
  - `node scripts/oracle-batch-aggregate.js styles-legacy/ --top 10`
