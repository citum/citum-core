---
name: style-evolve
type: user-invocable, agent-invocable
description: "Single human-facing command for all Citum style work. Use whenever someone asks to fix, improve, convert, or create a citation style. Routes to upgrade, migrate, or create. Always use this rather than calling style-maintain or style-migrate-enhance directly."
model: sonnet
routes-to: style-maintain, style-migrate-enhance, style-qa
---

# Style Evolve

## Human UX (Public Entry Point)

```
/style-evolve upgrade <style-path>
/style-evolve migrate <csl-path>
/style-evolve create
```

Do not ask users to call internal skills directly.

## Setup (read first)

Before doing anything else, read:
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md` — failure classification and stop conditions
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md` — decision flow, evidence ladder, shared gates

## Autonomous Operation

Run the full pipeline without stopping to ask questions between steps. Only interrupt for the explicit permission gates in `CLAUDE.md`.

## Modes

### 1. upgrade
Route to `../style-maintain/SKILL.md`.

### 2. migrate
Route to `../style-migrate-enhance/SKILL.md`.

### 3. create
Build a new Citum style from source evidence. Escalate to `@dplanner` for design.

## Co-Evolution Rule

Use `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md` for failure classification and stop conditions. Keep the router focused on dispatch and host entrypoint behavior.

## Shared Gates

- Compatibility fidelity regression is never allowed unless the task explicitly chooses a documented semantic divergence from legacy CSL behavior.
- SQI is optimization-only after fidelity is stable.
- Before editing a style, classify it by semantic class and implementation form
  using the shared workflow docs and `docs/specs/STYLE_TAXONOMY.md`.
- Profile-family work may require a `create` pass for a hidden family root
  followed by `upgrade` reduction of the public handles.
- Journal/profile reductions must choose parents from guide-backed authority,
  not nearest CSL or template similarity.
- Keep waves bounded to one family or one clearly related cohort per PR.
- For styles with configured `benchmark_runs`, run `node scripts/report-core.js --style <name>` after the primary oracle pass and treat the rich-input results as official supplemental evidence.
- All modes must pass `../style-qa/SKILL.md` before completion.
- If docs or beans are changed: `./scripts/check-docs-beans-hygiene.sh` must pass.

## Output Contract

Every completed task delivers:
- fidelity metrics
- SQI delta
- authority basis
- rich input evidence summary
- divergences
- code opportunities table
- QA verdict
- research value
- commit SHA and message, or `none`

Include a routed sub-skill header line in the final output.
