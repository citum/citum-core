# Codex Skills

## Purpose

This repo keeps Codex skills as repo-owned source under `.codex/skills/` and installs
them into `$CODEX_HOME/skills` with a local sync script.

## Public Skills

- `style-evolve`
- `migrate-research`
- `rust-simplify`

These are the user-facing skill names for Codex. They should point at thin wrappers
that delegate shared workflow logic to repo docs and internal role contracts.

## Legacy Alias

- `styleauthor`

Keep this only as a compatibility alias for older workflows that still mention the
legacy name.

## Internal Roles

The files in `.codex/agents/` remain internal execution roles:

- `spec-planner`
- `style-qa-reviewer`
- `migration-researcher`
- `rust-implementer`
- `docs-curator`

Use them for the lower-level role contracts, not as the primary user-facing entrypoint.

## Installation

Run `./scripts/install-codex-skills.sh` to refresh symlinks in `$CODEX_HOME/skills`.
The script is idempotent and fails if a target path is already a non-symlink.

## Mirror Rules

- Keep workflow logic in `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md` and
  `docs/guides/STYLE_WORKFLOW_EXECUTION.md`.
- When `.jj` is present, Codex skills may use
  `docs/guides/JJ_AI_CHANGE_STACK.md` for local change isolation, intent
  capture, and stack curation. Keep Git/GitHub as the published interface.
- Keep Codex skills thin and host-focused.
- Keep `./scripts/codex <role> <target...>` as a fallback for direct role execution.
