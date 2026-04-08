---
name: style-qa
type: agent-invocable
description: Standardized QA gate for style work. Verifies fidelity, SQI drift, formatting defects, and regression surface. Produces approve/reject verdict with numbered findings.
model: haiku
---

# Style QA Gate

Authoritative shared process docs:
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md`

## Gate Inputs
- Style path(s) changed.
- Oracle JSON result(s).
- Optional baseline metrics for comparison.
- Optional docs/beans diff when task updates `.md` or `.beans/*`.

## Required Checks
1. Fidelity summary.
2. SQI drift summary as a secondary metric only.
3. Formatting audit.
4. Regression surface.
5. Docs/beans hygiene when docs or beans are touched.

## Decision Rules
- Reject when fidelity regresses.
- Reject when a registered divergence is reported as an unexplained defect.
- Reject when formatting defects are introduced.
- Approve when fidelity is preserved or improved and formatting is clean.

## Standard Output
- Verdict: `approve` or `reject`
- Metrics line: citations + bibliography + SQI delta
- Findings: short numbered list
- Next step: merge, iterate, or escalate to planner/processor
