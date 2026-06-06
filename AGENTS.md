# Citum — Repo-Local Agent Contract

This repository owns its agent harness. Do not assume required behavior from
`~/.sober`, `~/.claude`, `~/.codex`, or any other home-directory policy layer.

`CLAUDE.md` is the authored project-instructions source. This file is the
repo-local, host-neutral entrypoint for AGENTS-aware tools and must stay aligned
with the same core rules.

## Core rules

1. **Prefer repo-owned instructions.** Use this repository's control surfaces
   first: `CLAUDE.md`, this file, `.skills/`, `.claude/skills/`,
   `.codex/agents/`, and the docs under `docs/`.
2. **Keep the harness repo-local.** Optional host installers may copy or expose
   repo-owned skills, but they are convenience layers, not the source of truth.
3. **Use deterministic search and verification.** Search before deep reading,
   make the smallest safe change, and run the repo's own validation commands.
4. **Follow the repo's documented workflow.** Tasks use `/beans`; documentation
   belongs in the correct `docs/` class; PR work happens on a branch.

## Canonical surfaces

| Surface | Role |
|---|---|
| `CLAUDE.md` | Authored Citum project instructions |
| `AGENTS.md` | Host-neutral repo entrypoint |
| `.skills/` | Canonical public skill tree |
| `.claude/skills/` | Host-specific skills and wrappers |
| `.codex/agents/` | Thin internal Codex role contracts |
| `docs/guides/AGENT_SKILLS.md` | Skill installation and boundary guide |

## Verification

For Rust changes, the required gate is:

```bash
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run
```

Do not substitute a generic wrapper that weakens this gate.

## Documentation placement

| Kind | Directory |
|---|---|
| Feature / design specs | `docs/specs/` |
| Active behavioral rules | `docs/policies/` |
| Architectural snapshots / audits | `docs/architecture/` |
| Operational guides | `docs/guides/` |
| Reference material | `docs/reference/` |

## Task and PR workflow

- Track local work with `/beans`.
- Create a spec in `docs/specs/` before non-trivial feature work; set it to
  `Active` in the implementation change.
- Work on a branch when a PR is planned.
- Do not manually bump workspace or schema versions in feature/fix PRs.

## Pointers

- `CLAUDE.md`
- `docs/specs/REPO_LOCAL_HARNESS.md`
- `docs/guides/AGENT_SKILLS.md`
- `docs/guides/CLAUDE_CODE_MIGRATION.md`
