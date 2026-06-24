# Style Workflow Execution

**Status:** Active
**Version:** 1.0
**Date:** 2026-04-04
**Related:** [STYLE_WORKFLOW_DECISION_RULES.md](../policies/STYLE_WORKFLOW_DECISION_RULES.md),
[MIGRATE_RESEARCH_RICH_INPUTS.md](../specs/MIGRATE_RESEARCH_RICH_INPUTS.md),
[MIGRATION_STRATEGY_ANALYSIS.md](../architecture/MIGRATION_STRATEGY_ANALYSIS.md)

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
2. Classify the target on **three** axes before editing:
   - semantic class (`base`, `profile`, `journal`, `independent`)
   - implementation form (`alias`, `config-wrapper`, `structural-wrapper`, `standalone`)
   - portfolio tier (`embedded-core` or `dependent`) — see the decision rules for
     the predicate (`citum style list --source embedded`)
3. Establish source authority before reading implementation artifacts:
   publisher guide first, then publisher house rules, then parent-style guidance.
4. Capture the smallest trustworthy evidence surface first.
5. Use reduced-cluster evidence before broad supplemental reruns.
6. Classify each failure using the shared policy.
7. Apply at most one tightly scoped fix per bounded cluster pass.
8. Re-run the reduced evidence set, then the broader oracle or report surface.
9. Stop when the cluster is reclassified, converged, or proven out of scope.

### Shared verification logic
- Fidelity to the declared primary authority is a hard gate for all tiers.
- **SQI is a hard gate for `embedded-core` styles** (both fidelity and SQI must
  be green for a pass verdict). For `dependent` styles, SQI is advisory and a
  tie-breaker only.
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
- semantic class, implementation form, and portfolio tier
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

A style wave is a bounded cohort executed through repeated `upgrade`, `migrate`,
`create`, or `tune` passes under this same execution flow.

- Keep one wave to one family or one clearly related cohort per PR.
- For profile-family work, it is valid to use `create` to author a hidden family
  root first and then `upgrade` to reduce the public handles.
- Do not add a separate public "wave" command surface; waves are an execution
  pattern, not a new mode.

## The `tune` loop (embedded-core styles)

`tune` is the correct mode whenever the goal is to bring an `embedded-core`
style to **100% fidelity and clean SQI**. Deterministic migration (`citum-migrate`)
cannot reliably reach this bar on its own — see
[MIGRATION_STRATEGY_ANALYSIS.md](../architecture/MIGRATION_STRATEGY_ANALYSIS.md). The converter's output is a
**seed**: a starting candidate whose oracle score and SQI baseline ground the
first iteration.

### Execution order
1. **Seed:** run `citum-migrate` (or accept the existing Citum YAML) to produce
   a concrete candidate. Record oracle fidelity baseline and SQI baseline.
2. **Fidelity loop:**
   a. Run the oracle (`node scripts/oracle.js <legacy-style> --json`).
   b. Classify each failure per the shared decision rules.
   c. Apply the smallest correct YAML change toward the target reference output.
   d. Re-run oracle. Repeat until fidelity is 100%.
   e. If a residual is clearly a `processor-defect` or `intentional divergence`,
      reclassify and exclude — do not keep iterating.
3. **SQI loop (begins only when fidelity is green):**
   a. Run `node scripts/report-core.js --style <name>` to get the SQI score.
   b. Apply SQI improvements — hoist shared options, use presets, introduce
      diff-based `type-variants`, prune redundant defaults — without regressing
      fidelity. Re-run oracle after each SQI change to confirm.
   c. Continue until SQI is clean (no actionable SQI findings remain).
4. **QA gate:** hand off to `style-qa` with tier = `embedded-core`.

### Stop conditions (same as all shared workflows)
- Two distinct approaches fail on the same cluster → reclassify.
- Residual explained by a registered divergence → record ID, do not count as failure.
- Residual is a `processor-defect` → escalate to Rust workflow; do not keep
  cycling YAML.
- Migrate cannot produce a usable seed → hand-author from guide evidence
  directly (the `create` path).

## Implementation Notes
- Use this guide as the canonical place for the evidence ladder and convergence language currently repeated across style workflows.
- Keep host wrappers short and refer back here instead of restating the loop.

## Acceptance Criteria
- [x] Shared style workflows reference this guide instead of duplicating the same loop text.
- [x] The evidence ladder is defined here exactly once.
- [x] The shared output contract is expressed here in host-neutral terms.
- [x] Portfolio tier is part of the classification and output contract.
- [x] `tune` loop is defined here once, not in individual skill files.

## Changelog
- 2026-06-24: Added portfolio tier to the three-axis classification and output
  shape. Replaced the universal "SQI is advisory" rule with a tier-aware rule
  (embedded-core promotes SQI to a hard gate; dependent stays advisory). Added
  the `tune` execution loop section, including seed, fidelity loop, SQI loop,
  and stop conditions. Cross-linked `MIGRATION_STRATEGY_ANALYSIS.md`. Added
  `tune` to the wave pass types.
- 2026-04-23: Added explicit semantic-class vs implementation-form
  classification, profile-contract verification, journal structural-wrapper
  acceptance, and bounded-wave guidance.
- 2026-04-04: Initial version.
