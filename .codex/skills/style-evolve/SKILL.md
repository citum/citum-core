---
name: style-evolve
description: "Public Codex entrypoint for Citum style work. Activate on: 'upgrade', 'migrate', 'create', any style authoring request, or any request to fix/improve/convert a Citum or CSL citation style. Route to the shared workflow docs and internal roles."
---

# Style Evolve

Use this skill for any Citum style request that should be handled through the shared
style workflow.

Read first:
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md`
- `docs/guides/CODEX_SKILLS.md`

## Public Modes

- `upgrade`: improve an existing Citum style.
- `migrate`: convert CSL 1.0 source into Citum style YAML.
- `create`: author a new Citum style from source evidence.

## Routing

- Use `spec-planner` when the request needs architecture or schema decisions.
- Use `migration-researcher` when the evidence points to `citum_migrate`.
- Use `rust-implementer` for bounded Rust fixes.
- Use `style-qa-reviewer` for the final style QA gate.

## Operating Rules

- Keep fidelity as the hard gate.
- Treat SQI as secondary optimization only.
- Before editing a style, classify it by semantic class and implementation
  form using `docs/specs/STYLE_TAXONOMY.md` and the shared workflow docs.
- Profile-family work may require a `create` pass for a hidden family root
  followed by `upgrade` reduction of the public handles.
- Journal/profile reductions must choose parents from guide-backed authority,
  not nearest CSL or template similarity.
- Keep waves bounded to one family or one clearly related cohort per PR.
- Do not duplicate the shared decision rules or evidence ladder here.
- Keep this skill focused on routing and host-facing behavior.
