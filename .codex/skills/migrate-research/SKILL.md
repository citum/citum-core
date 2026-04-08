---
name: migrate-research
description: Autonomous Codex research loop for Citum migration fidelity gaps and converter-focused Rust fixes.
---

# Migrate Research

Use this skill when multiple styles share a migration failure pattern or when a rich
input pass needs classification before changing `citum_migrate`.

Read first:
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md`
- `docs/specs/MIGRATE_RESEARCH_RICH_INPUTS.md`
- `docs/guides/CODEX_SKILLS.md`

## Use When

- The goal is converter improvement, not a style YAML patch.
- Several styles fail for the same reason.
- You need bounded evidence before deciding whether to fix migration, style, processor,
  or adjudication.

## Operating Rules

- Start with the smallest trustworthy evidence surface.
- Classify each cluster as `migration-artifact`, `style-defect`, `processor-defect`,
  or `intentional divergence`.
- Stop when the cluster is clearly out of scope or converged.
- Keep the loop bounded to one cluster per pass.

## Output

Report the chosen cluster, classification, before/after evidence, exact change made if
any, and whether the pass should continue, stop, or escalate.

