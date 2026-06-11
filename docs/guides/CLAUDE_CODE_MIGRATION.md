# Citum Repo-Local Harness

## Overview

This repository no longer treats a home-directory harness as its governing
control plane. The active Citum harness is repo-local: contributors should be
able to understand the workflow from repository files alone.

## Control Surfaces

| Surface | Role |
|---|---|
| `CLAUDE.md` | Single authored Citum project instructions |
| `AGENTS.md` | Symlink to `CLAUDE.md` for AGENTS-aware tools |
| `.skills/` | Canonical public skill tree |
| `.claude/skills/` | Host-specific skills and wrappers |
| `.codex/agents/` | Thin internal Codex role contracts |
| `docs/policies/AGENT_HARNESS_POLICY.md` | Repo-owned agent role and artifact rules |
| `docs/guides/AGENT_ORCHESTRATION.md` | Repo-owned planner/worker/reviewer handoff guide |

See `docs/specs/REPO_LOCAL_HARNESS.md` for the governing design.

## What Changed

### Before

The repo still carried language that implied Citum behavior was layered on top
of:

- a global home-directory agent layer
- a generic home-directory verification wrapper
- a prior host-native task system that is no longer the active local workflow

### Now

The active contract is:

- repo-owned instructions are the source of truth
- `AGENTS.md` resolves to `CLAUDE.md`; do not duplicate root instructions
- optional host installers are convenience layers only
- contributor task tracking uses `/beans`
- Claude and Codex entrypoints are documented in-repo

## Working Model

### Tasks

Use `/beans` for local task tracking. Do not rely on a host-native task panel as
the repo's governing workflow.

### Skills and agents

- Public reusable skills live under `.skills/`.
- Claude/Copilot-specific skills live under `.claude/skills/`.
- Internal Codex role contracts live under `.codex/agents/`.

Shared workflow logic should live in docs and policies, not be duplicated into
every host wrapper.

### Claude hooks

Repo-local Claude settings may expose project conveniences such as `beans
prime`, but personal tool nudges, token-saving hooks, and exact tool routing
belong in user config. The repo states required capabilities and verification
rules; the user layer decides whether those capabilities are provided by a
language server, symbol index, shell wrapper, or another local tool.

### Optional install steps

Some hosts may require optional local installation steps to expose repo-owned
skills. Those steps do **not** change the source of truth:

- the repo remains authoritative
- home-directory state is optional convenience only

For Codex skill installation, see `docs/guides/AGENT_SKILLS.md`.

## Maintainer Notes

When updating the harness:

1. Keep `AGENTS.md` symlinked to `CLAUDE.md` unless a future spec moves both
   entrypoints to a third shared source.
2. Prefer repo docs over host-specific wrapper text for shared process logic.
3. Avoid new references that make `~/` content part of the required Citum
   contract.
4. Keep exact model IDs and personal tools out of repo wrappers.
5. Update the harness spec when changing the control-surface model.

## Historical Note

This file used to document a Claude-native-task migration. It now serves as the
current maintainer guide for the repo-local harness so existing links continue
to land on active guidance instead of stale workflow history.
