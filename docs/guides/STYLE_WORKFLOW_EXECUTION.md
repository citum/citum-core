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
2. Classify the target on two axes before editing:
   semantic class (`base`, `profile`, `journal`, `independent`) and
   implementation form (`alias`, `config-wrapper`, `structural-wrapper`,
   `standalone`).
3. Establish source authority before reading implementation artifacts:
   publisher guide first, then publisher house rules, then parent-style guidance.
4. Capture the smallest trustworthy evidence surface first.
5. Use reduced-cluster evidence before broad supplemental reruns.
6. Classify each failure using the shared policy.
7. Apply at most one tightly scoped fix per bounded cluster pass.
8. Re-run the reduced evidence set, then the broader oracle or report surface.
9. Stop when the cluster is reclassified, converged, or proven out of scope.

### Shared verification logic
- Fidelity to the declared primary authority is the hard gate.
- SQI or other secondary metrics are advisory unless a workflow explicitly promotes them.
- QA must reject regressions and formatting defects.
- Supplemental rich-input evidence is confirmation, not the first debugging surface.
- CSL structure is verification evidence, not the source of truth for wrapper thickness.
- For `profile` targets, verify that the file still satisfies the config-wrapper
  contract: no local templates, no local `type-variants`, and no
  template-clearing `null`.
- For `journal` targets, accept `structural-wrapper` as a legitimate endpoint
  when guide-backed deltas or current merge mechanics prevent a meaningful thin
  reduction.

### Shared output shape
Every workflow should report:
- target or cluster chosen
- semantic class and implementation form
- classification and rationale
- before/after evidence
- exact change made, if any
- whether the pass should continue, stop, or escalate

### Shared escalation
- `migration-artifact` stays in migration work until the converter is fixed or disproven.
- `style-defect` routes to style-local YAML repair.
- `processor-defect` routes to processor or engine follow-up.
- `intentional divergence` is recorded and excluded from fix counts.
- If parentage is guide-backed but current merge semantics still force a bulky
  wrapper, escalate as an infrastructure constraint rather than preserving or
  reintroducing duplicated structure as if it were authority.

## Waves

A style wave is a bounded cohort executed through repeated `upgrade`,
`migrate`, or `create` passes under this same execution flow.

- Keep one wave to one family or one clearly related cohort per PR.
- For profile-family work, it is valid to use `create` to author a hidden family
  root first and then `upgrade` to reduce the public handles.
- Do not add a separate public "wave" command surface; waves are an execution
  pattern, not a new mode.

## Implementation Notes
- Use this guide as the canonical place for the evidence ladder and convergence language currently repeated across style workflows.
- Keep host wrappers short and refer back here instead of restating the loop.

## Acceptance Criteria
- [x] Shared style workflows reference this guide instead of duplicating the same loop text.
- [x] The evidence ladder is defined here exactly once.
- [x] The shared output contract is expressed here in host-neutral terms.

## Changelog
- 2026-04-23: Added explicit semantic-class vs implementation-form
  classification, profile-contract verification, journal structural-wrapper
  acceptance, and bounded-wave guidance.
- 2026-04-04: Initial version.
