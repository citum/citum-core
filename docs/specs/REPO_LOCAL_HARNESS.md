# Repo-Local Harness Specification

**Status:** Active
**Date:** 2026-06-06
**Supersedes:** —
**Related:** `CLAUDE.md`; `AGENTS.md`; `docs/policies/AGENT_HARNESS_POLICY.md`; `docs/guides/AGENT_ORCHESTRATION.md`; `docs/guides/AGENT_SKILLS.md`; `docs/guides/JJ_AI_CHANGE_STACK.md`; bean `csl26-ddjg`

## Purpose

Define the Citum agent harness boundary. The repository owns project truth:
invariants, verification gates, durable artifacts, role contracts, and publish
safety. User-level harness config owns personal working style, model choices,
local toolchain, token-saving hooks, and exact tool routing.

## Scope

In scope:

- repo control surfaces such as `CLAUDE.md` and `AGENTS.md`
- boundaries between `.skills/`, `.claude/skills/`, and `.codex/agents/`
- neutral planner, worker, reviewer, and task-packet contracts
- durable Citum artifacts: beans, specs, policies, guides, and audit records
- safety rules for optional local `jj` intent capture

Out of scope:

- defining personal model choices, token-saving preferences, or local hooks
- requiring any hidden home-directory file to understand Citum workflow
- making Citum depend on one editor, host, toolchain plugin, or model family

## Design

### 1. Two-layer source of truth

The Citum source of truth lives in this repository. The user source of truth
lives outside the repository.

- `CLAUDE.md` is the single authored Citum project-instructions file.
- `AGENTS.md` is a symlink to `CLAUDE.md` for AGENTS-aware tools.
- The repo must not maintain duplicate root instruction bodies.
- If this model changes later, both root entrypoints should point to a new
  shared source rather than diverging.
- User config may choose tools and models, but must not override Citum's
  verification, docs placement, bean, commit, PR, or publish rules.

### 2. Repo-local skill surfaces

The harness exposes three distinct skill layers:

| Surface | Role |
|---|---|
| `.skills/` | Canonical public skills intended for cross-host reuse |
| `.claude/skills/` | Host-specific skills and wrappers |
| `.codex/agents/` | Thin internal Codex role contracts |

Shared workflow logic lives in docs or policy files rather than being
duplicated across all host surfaces:

- `docs/policies/AGENT_HARNESS_POLICY.md` for binding orchestration rules
- `docs/guides/AGENT_ORCHESTRATION.md` for task-packet and handoff mechanics
- task-domain policies and guides such as the style workflow docs

### 3. Tooling belongs to the user layer

Repo docs state required capabilities, not personal tool names.

Examples of user-layer concerns:

- model tiers and cost policy
- reasoning tier mapping and escalation permission gates
- symbol-navigation tools and language servers
- token-saving shell wrappers
- structural search tools
- local hooks and host-specific nudges
- dispatch-worker backends and aliases

Repo wrappers may mention that a capability is needed, but they must not require
a contributor to share one user's exact local toolchain, model IDs, or private
cost policy.

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

### 6. Durable and temporary artifacts

Durable project artifacts are tracked in the repo:

- `.beans/*.md` for concrete work units
- `docs/specs/` for non-trivial implementation contracts
- `docs/policies/` for binding recurring rules
- `docs/guides/` for operational how-tos
- `docs/architecture/audits/` for dated evidence records

Temporary local provenance may exist under `.ai-intents/` during `jj` drafting,
but it must not be published unless the user explicitly asks for durable prompt
provenance. Publish checks must reject accidental `.ai-intents/` paths.

## Implementation Notes

- Keep `CLAUDE.md` as the authored source for now to minimize churn in existing
  Claude/Copilot workflows.
- Keep `AGENTS.md` symlinked to `CLAUDE.md` so Codex/AGENTS-aware consumers
  receive the same contract without duplicated content.
- Keep host wrappers thin and point them at the shared policy/guide surface.
- Move personal model, reasoning, tool routing, and hooks to user config.
- Keep `.ai-intents/` publish-clean rather than ignored, so accidental durable
  provenance is visible and blocked.

## Acceptance Criteria

- [x] The repo defines a repo-local harness source of truth.
- [x] `AGENTS.md` exists as a repo-owned host-neutral entrypoint symlinked to
  `CLAUDE.md`.
- [x] Active control surfaces no longer imply a required home-directory
  harness.
- [x] The roles of `.skills/`, `.claude/skills/`, and `.codex/agents/` are
  documented and non-overlapping.
- [x] Personal tool routing is treated as user config, not Citum project truth.
- [x] Planner, worker, reviewer, and durable artifact rules are documented.
- [x] `.ai-intents/` is documented as temporary local provenance and blocked
  from accidental publication.
- [x] A contributor can understand the harness from repo files alone.

## Changelog

- 2026-06-11: Clarified repo/user harness boundary, moved personal tool routing
  out of repo truth, and added orchestration plus `.ai-intents/` safety.
- 2026-06-06: Initial version.
