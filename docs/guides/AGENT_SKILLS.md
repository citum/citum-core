# Agent Skills

## Purpose

Contributor AI skills for working on citum-core. These skills run inside a citum-core
checkout and reference internal docs and CLI tooling.

Install locally:

```bash
./scripts/install-skills.sh
```

This syncs the canonical `.skills/` tree into Codex's `$CODEX_HOME/skills`
directory. Legacy `.codex/skills/` paths remain as compatibility shims for
existing Codex installs.

## Repo-Local Harness Boundary

This repository owns its harness contract.

- `CLAUDE.md` is the single authored Citum project-instructions file.
- `AGENTS.md` is a symlink to `CLAUDE.md` for AGENTS-aware tools.
- `.skills/` is the canonical public skill tree.
- `.claude/skills/` and `.codex/agents/` are host-specific surfaces.

`./scripts/install-skills.sh` is an optional installer for host exposure. It is
not the source of truth, and contributors should not need to inspect `~/`
content to understand Citum's workflow model.

Do not duplicate root instruction content between `CLAUDE.md` and `AGENTS.md`.
If the entrypoint model changes, move both to a shared source rather than
maintaining parallel copies.

## Contributor Skills

| Skill | Description |
|-------|-------------|
| `style-evolve` | Route style work: upgrade, migrate, or create a Citum style |
| `migrate-research` | Autonomous research loop for citum_migrate fidelity gaps |
| `rust-simplify` | One-file Rust quality pass using local symbol-aware analysis |

These skills are for citum-core contributors only. They require the repo to be present
locally and reference internal docs by path.

## External Style Authoring

To author Citum styles without a citum-core checkout, install the external skill:

```bash
npx skills add citum/skills
```

See [github.com/citum/skills](https://github.com/citum/skills).

## Supported Agents

The `.skills/` tree is host-neutral and includes agent-specific config under
`.skills/<name>/agents/`. The local installer currently targets Codex only;
other skills-compatible hosts can consume `.skills/` directly unless a
host-specific installer is added later.

## Legacy Alias

`styleauthor` is kept as a compatibility alias for older workflows.

## Internal Roles

Files in `.codex/agents/` remain internal execution roles:

- `spec-planner`
- `style-qa-reviewer`
- `migration-researcher`
- `rust-implementer`
- `docs-curator`

Use them for lower-level role contracts, not as primary user-facing entrypoints.
Exact model IDs, tool routing, token-saving wrappers, and local hooks belong in
user config, not these repo-owned role contracts.

## Mirror Rules

- Keep general orchestration logic in
  [../policies/AGENT_HARNESS_POLICY.md](../policies/AGENT_HARNESS_POLICY.md)
  and [AGENT_ORCHESTRATION.md](./AGENT_ORCHESTRATION.md).
- Keep style workflow logic in
  [../policies/STYLE_WORKFLOW_DECISION_RULES.md](../policies/STYLE_WORKFLOW_DECISION_RULES.md)
  and [STYLE_WORKFLOW_EXECUTION.md](./STYLE_WORKFLOW_EXECUTION.md).
- Keep skills thin and host-focused.
