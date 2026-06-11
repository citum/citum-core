# Codex Agent Drafts

This directory contains repo-local Codex agent drafts for `citum-core`.

Repo-owned public skills live under `.skills/` and are installed into
`$CODEX_HOME/skills` with `./scripts/install-skills.sh`.

The repo-local control surfaces are:

- `CLAUDE.md` — single authored Citum project instructions
- `AGENTS.md` — symlink to `CLAUDE.md` for AGENTS-aware tools
- `docs/guides/AGENT_SKILLS.md` — installation and boundary guide
- `docs/policies/AGENT_HARNESS_POLICY.md` — planner/worker/reviewer rules
- `docs/guides/AGENT_ORCHESTRATION.md` — task-packet and handoff workflow

Authoritative shared process docs:
- `docs/policies/AGENT_HARNESS_POLICY.md`
- `docs/guides/AGENT_ORCHESTRATION.md`
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md`

These files are intentionally thin host wrappers:
- use the shared docs for process logic
- keep only host-local purpose, role tier, reasoning tier, scope, and output-contract metadata here
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
- Local user config chooses exact model IDs, reasoning controls, and tool backends.
- Treat the repo-root `AGENTS.md` as the Codex-facing control surface; it must remain symlinked to `CLAUDE.md` unless a future spec moves both entrypoints to a shared source.
