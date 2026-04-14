# Style Workflow Execution

**Status:** Active
**Version:** 1.0
**Date:** 2026-04-04
**Related:** `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`, `docs/specs/MIGRATE_RESEARCH_RICH_INPUTS.md`

## Purpose
This guide defines the shared execution flow for Citum style workflows so Claude skills and Codex agents can reference the same process without duplicating it.

## Scope
In scope:
- style-oriented routing and verification loops
- evidence order for migration and QA passes
- output contract shape for shared workflow roles
- common escalation boundaries and stop conditions

Out of scope:
- host-specific frontmatter and model settings
- one-off mode deltas that only apply to a single wrapper
- Rust implementation details

## Design
### Shared execution order
1. Establish the workflow mode and target scope.
2. Capture the smallest trustworthy evidence surface first.
3. Use reduced-cluster evidence before broad supplemental reruns.
4. Classify each failure using the shared policy.
5. Apply at most one tightly scoped fix per bounded cluster pass.
6. Re-run the reduced evidence set, then the broader oracle or report surface.
7. Stop when the cluster is reclassified, converged, or proven out of scope.

### Shared verification logic
- Fidelity is the hard gate.
- SQI or other secondary metrics are advisory unless a workflow explicitly promotes them.
- QA must reject regressions and formatting defects.
- Supplemental rich-input evidence is confirmation, not the first debugging surface.

### Shared output shape
Every workflow should report:
- target or cluster chosen
- classification and rationale
- before/after evidence
- exact change made, if any
- whether the pass should continue, stop, or escalate

### Shared escalation
- `migration-artifact` stays in migration work until the converter is fixed or disproven.
- `style-defect` routes to style-local YAML repair.
- `processor-defect` routes to processor or engine follow-up.
- `intentional divergence` is recorded and excluded from fix counts.

## Implementation Notes
- Use this guide as the canonical place for the evidence ladder and convergence language currently repeated across style workflows.
- Keep host wrappers short and refer back here instead of restating the loop.

## Acceptance Criteria
- [x] Shared style workflows reference this guide instead of duplicating the same loop text.
- [x] The evidence ladder is defined here exactly once.
- [x] The shared output contract is expressed here in host-neutral terms.

## Changelog
- 2026-04-04: Initial version.
