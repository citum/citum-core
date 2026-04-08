---
name: style-migrate-enhance
type: agent-invocable
description: High-throughput migration waves converting priority parent CSL 1.0 styles to Citum with repeatable before/after metrics and migration-engine gap recommendations. Fidelity is the hard gate.
model: sonnet
---

# Style Migrate+Enhance

Authoritative shared process docs:
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md`

## Use This Skill When
- The task is portfolio migration.
- You need repeatable before/after/rerun metrics.
- You want concrete recommendations for `citum_migrate` improvements from observed gaps.

## Input Contract
- Legacy style path(s) under `styles-legacy/`.
- Target Citum style path(s) under `styles/`.
- Batch size and priority source.
- Optional target metric.

## Output Contract
- Updated style YAML file(s).
- Shared metrics and rerun evidence in the format described by the shared execution guide.
- Migration-pattern gaps and recommended converter/preset follow-up when observed.

## Autonomous Operation

Run the full wave without pausing between styles. Use the shared docs for the common evidence order, decision rules, and output shape. Only interrupt for `Cargo.toml`/`Cargo.lock` changes or `git push origin main`.

## Workflow
1. Select the next priority wave.
2. Seed the baseline with the smallest trustworthy evidence surface.
3. Apply the fix according to the shared policy and execution guide.
4. Re-run apples-to-apples comparison evidence.
5. Treat supplemental rich-input evidence as confirmation when configured.
6. Commit each passing style and produce final metrics plus follow-up recommendations.

## Hard Gates
- Never accept a fidelity regression.
- Never classify a registered divergence as a migration or engine bug without updating adjudication first.
- SQI is tie-breaker and optimization only.
- After bounded retries with no progress, note it in the wave summary and move to the next style rather than halting the entire wave.

## Required Artifacts
- Iteration log.
- Final wave summary table.
- Code Opportunities table in the same shape as the router skill when engine gaps are observed.

## Verification
- Structured oracle: `node scripts/oracle.js <legacy-style> --json`
- Core quality report: `node scripts/report-core.js`
- Supplemental official style report for configured rich-input styles: `node scripts/report-core.js --style <name>`
- Optional full workflow impact: `./scripts/workflow-test.sh <legacy-style>`
