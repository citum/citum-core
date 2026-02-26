---
name: style-evolve
type: user-invocable, agent-invocable
description: Single human-facing command for Citum style co-evolution (style + processor). Use for all style work: upgrade (improve existing), migrate (CSL 1.0 to Citum), create (from source evidence). Fidelity is the hard gate; SQI is secondary.
model: sonnet
routes-to: style-maintain, style-migrate-enhance, style-qa
---

# Style Evolve

## Human UX (Public Entry Point)
Use this command for all style work:
- `/style-evolve upgrade ...`
- `/style-evolve migrate ...`
- `/style-evolve create ...`

Do not require users to choose internal pipeline skills.

## Modes
1. `upgrade`
- Improve existing Citum styles.
- Route to `../style-maintain/SKILL.md`.

2. `migrate`
- Convert one or more CSL 1.0 styles to high-fidelity Citum.
- Route to `../style-migrate-enhance/SKILL.md`.

3. `create`
- Build a style from source evidence.
- Accept one or more sources:
  - `--source-url`
  - `--source-text`
  - `--source-issue`
  - `--source-file`
- Default planner path: `@dstyleplan` -> `@styleplan` -> `@builder`.

## Co-Evolution Rule (Mandatory)
Every iteration must assess two tracks:
- Track A: style/template edits
- Track B: reusable code opportunities (presets, missing features, processor fixes)

A task is not complete until Track B is explicitly marked as:
- implemented, or
- deferred with rationale.

## Shared Gates
- Fidelity regression is never allowed.
- SQI is optimization only after fidelity.
- All modes must pass `../style-qa/SKILL.md` before completion.

## Output Contract
- Fidelity metrics (citations and bibliography pass counts).
- SQI delta.
- Code-opportunity table:
  - preset candidate
  - missing feature
  - processor defect
  - defer rationale

## Internal Skills (Pipeline Components)
- `style-maintain`
- `style-migrate-enhance`
- `style-qa`
- `pr-workflow-fast`
