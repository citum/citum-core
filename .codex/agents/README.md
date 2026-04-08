# Codex Agent Drafts

This directory contains repo-local Codex agent drafts for `citum-core`.

Repo-owned public Codex skills live under `.codex/skills/` and are installed into
`$CODEX_HOME/skills` with `./scripts/install-codex-skills.sh`.

Authoritative shared process docs:
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md`

These files are intentionally thin host wrappers:
- use the shared docs for process logic
- keep only host-local purpose, model, scope, and output-contract metadata here
- do not duplicate the shared workflow loop in each agent file

Runnable Codex fallback:
- `./scripts/codex <role> <target...>`

## Files
- `migration-researcher.md`
- `style-qa-reviewer.md`
- `rust-implementer.md`
- `spec-planner.md`
- `docs-curator.md`

## Shared Conventions
- Keep scope explicit and bounded.
- Prefer repo policies over generic behavior when they conflict.
- For Citum Rust work, follow the repo's documented verification and documentation rules.
- Treat `AGENTS.md` as the real control surface until Codex documents something stronger for custom agents.
