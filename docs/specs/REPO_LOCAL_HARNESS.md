# Repo-Local Harness Specification

**Status:** Active
**Date:** 2026-06-06
**Supersedes:** —
**Related:** `CLAUDE.md`; `AGENTS.md`; `.claude/settings.json`; `.claude/hooks/jcm-nudge.sh`; `docs/guides/AGENT_SKILLS.md`; bean `csl26-ddjg`

## Purpose

Define the Citum agent harness as a repository-owned system rather than an
overlay on top of a home-directory harness. This keeps contributor workflow
rules, skill boundaries, and Claude/Codex compatibility understandable from the
repo alone.

## Scope

In scope:

- repo control surfaces such as `CLAUDE.md` and `AGENTS.md`
- boundaries between `.skills/`, `.claude/skills/`, and `.codex/agents/`
- repo-owned Claude settings and hooks that reinforce project rules
- documentation for optional host installers
- removal of active references that imply a required global harness

Out of scope:

- re-creating Sober's installer model under a Citum name
- broad historical documentation cleanup unrelated to current control surfaces
- changing runtime behavior outside harness and documentation surfaces

## Design

### 1. Source of truth

The harness source of truth lives in this repository.

- `CLAUDE.md` is the single authored Citum project-instructions file.
- `AGENTS.md` is a symlink to `CLAUDE.md` for AGENTS-aware tools.
- The repo must not maintain duplicate root instruction bodies.
- If this model changes later, both root entrypoints should point to a new
  shared source rather than diverging.

### 2. Repo-local skill surfaces

The harness exposes three distinct skill layers:

| Surface | Role |
|---|---|
| `.skills/` | Canonical public skills intended for cross-host reuse |
| `.claude/skills/` | Host-specific skills and wrappers |
| `.codex/agents/` | Thin internal Codex role contracts |

Shared workflow logic should live in docs or policy files rather than being
duplicated across all host surfaces.

### 3. Repo-local Claude enforcement

Claude-specific behavior may use repo-owned settings and hooks when plain
instructions have proven unreliable.

- `.claude/settings.json` registers repo-local Claude hooks.
- `.claude/hooks/jcm-nudge.sh` reinforces the Rust code-search rule after
  `Read`, `Grep`, or `Glob` use against Rust/code paths.
- The jcodemunch/rust-analyzer priority table in `CLAUDE.md` is part of the
  core harness contract, not optional style guidance.
- A stale jcodemunch index must be refreshed before falling back to broad
  file reads or text search for named Rust symbols.

### 4. Optional installers

Host installers may exist to expose repo-owned skills to a specific tool, but
they are optional convenience layers only.

Allowed behavior:

- sync repo-owned assets outward from the repo
- explain host-specific setup requirements

Disallowed behavior:

- becoming the source of truth
- requiring a user to inspect home-directory files to understand Citum workflow
- introducing a new Citum-owned home-directory harness tree

### 5. Global-harness references

Active control surfaces must not describe Citum as governed by:

- `~/.sober`
- `~/.claude`
- `~/.codex`
- inherited global planner/builder/reviewer agents
- a generic `verify.sh` wrapper outside the repo

If a host provides extra capabilities, describe them as optional host behavior,
not as the governing Citum contract.

## Implementation Notes

- Keep `CLAUDE.md` as the authored source for now to minimize churn in existing
  Claude/Copilot workflows.
- Keep `AGENTS.md` symlinked to `CLAUDE.md` so Codex/AGENTS-aware consumers
  receive the same contract without duplicated content.
- Document repo-local Claude hooks where they enforce instructions that agents
  have historically ignored.
- Do not adopt Sober's home-directory install pattern.
- Rewrite the legacy migration guide rather than deleting it so existing links
  still land on current guidance.
- Update the specs index when adding this file.

## Acceptance Criteria

- [x] The repo defines a repo-local harness source of truth.
- [x] `AGENTS.md` exists as a repo-owned host-neutral entrypoint symlinked to
  `CLAUDE.md`.
- [x] Active control surfaces no longer imply a required home-directory
  harness.
- [x] The roles of `.skills/`, `.claude/skills/`, and `.codex/agents/` are
  documented and non-overlapping.
- [x] Claude-specific jcodemunch reinforcement is documented as repo-local
  enforcement rather than a home-directory dependency.
- [x] A contributor can understand the harness from repo files alone.

## Changelog

- 2026-06-06: Initial version.
