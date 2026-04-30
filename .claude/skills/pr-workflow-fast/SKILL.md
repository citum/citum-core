---
name: pr-workflow-fast
description: >
  Fast, consistent branch/PR workflow with explicit change-type quality gates
  and concise evidence-first PR descriptions.
---

# PR Workflow Fast

**Type:** User-Invocable, Agent-Invocable
**LLM Access:** Yes
**Purpose:** Efficient branch/PR workflow with minimal ceremony and explicit quality gates.

## Use This Skill When
- You want a branch + PR quickly.
- You need consistent, reviewable PR descriptions from oracle/test outputs.
- You want checks selected by change type.

## Branch Policy
- Branch prefix: `codex/`.
- Name pattern: `codex/<scope>-<short-goal>`.
- Keep PR scope narrow and mergeable.
- When `.jj` is present, `docs/guides/JJ_AI_CHANGE_STACK.md` may be used for
  local change-stack curation before pushing the Git branch.

## Change-Type Gates
1. Docs/styles only (`.md`, `styles/*.yaml`):
   - syntax sanity + targeted rendering/oracle checks
2. Rust-touching (`.rs`, `Cargo.toml`, `Cargo.lock`):
   - `cargo fmt --check` ← use `--check`, never bare `cargo fmt` (fmt is a fix, not a gate)
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo nextest run` (fallback: `cargo test`)
   - **Gate must pass before the first push.** If gate fails after builder work,
     fix + amend the unpushed commit (amend is allowed on unpushed commits),
     re-verify, then push. Never push a failing commit and follow up with a
     cleanup commit.
   - Every commit on the branch must individually be gate-clean.
3. Hot path/perf claims:
   - benchmark baseline/after via `./scripts/bench-check.sh`

### If Gate Fails After Push
If you discover a gate failure after pushing, squash + force-push
(`git push --force-with-lease`) rather than adding a cleanup commit.
Requires CONFIRM (critical action).

## PR Body Template
- Summary: what changed and why.
- Scope: files and affected workflows.
- Validation: exact commands run + key results.
- Risk: potential regressions and mitigation.
- Follow-ups: non-blocking next steps.

## Efficiency Rules
- One oracle snapshot per iteration unless structure changes materially.
- Stop at target metric; avoid unbounded polish loops.
- Escalate to planner when style-only fixes stall.
- Prefer smallest diff that achieves gate pass.

## Merge Readiness Checklist
- Checks passed for touched change type.
- PR body includes objective evidence.
- No unresolved high-severity findings.
