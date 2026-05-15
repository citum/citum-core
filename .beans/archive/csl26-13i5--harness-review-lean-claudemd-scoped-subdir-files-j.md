---
# csl26-13i5
title: 'Harness review: lean CLAUDE.md, scoped subdir files, jcodemunch enforcement'
status: completed
type: task
priority: high
created_at: 2026-05-15T09:46:01Z
updated_at: 2026-05-15T09:53:21Z
---

Apply Anthropic large-codebase best practices. Six steps: slim root CLAUDE.md, add scoped crate CLAUDE.md (engine/schema/migrate/csl-legacy), elevate jcodemunch+rust-analyzer tool priority, compact MEMORY.md, add crates/README.md map, add PostToolUse jcm-nudge hook. Plan: ~/.claude/plans/this-project-has-undergone-woolly-crab.md

## Todo

- [x] Slim root CLAUDE.md to ~120 lines (final: 136)
- [x] Add Code Search Tool Priority block (jcodemunch + rust-analyzer)
- [x] crates/citum-engine/CLAUDE.md
- [x] crates/citum-schema/CLAUDE.md
- [x] crates/citum-migrate/CLAUDE.md
- [x] crates/csl-legacy/CLAUDE.md
- [x] crates/README.md codebase map
- [x] Compact MEMORY.md to ~60 lines (final: 64)
- [x] Add ~/.claude/hooks/jcm-nudge.sh + register in user settings (placed in project .claude/hooks/ for scoping)
- [x] Run validate-frontmatter.sh and confirm no regressions
- [x] Push and open PR (#711)
- [x] Address Copilot review (hook hardening, drop user-local memory pointers, clarify file_outline vs repo_outline)
- [x] Add AGENTS.md symlinks for Codex parity (4 crates)
- [x] Fix amend rule in root CLAUDE.md (encourage on PR branches; never on main)

## Summary of Changes

- Root `CLAUDE.md`: 280 → 136 lines. Moved test catalog, locale detail, status snapshots, prior-art list out to pointers. Added unambiguous **Code Search Tool Priority** block elevating jcodemunch + rust-analyzer over Read/Grep, and a hard ban on `Explore` agent for code.
- Scoped `CLAUDE.md` added to four hot crates: `citum-engine`, `citum-schema` (facade), `citum-migrate`, `csl-legacy`. Each is ≤40 lines: layout table + 3-5 gotchas + symbol-query pointer.
- New `crates/README.md` — 15-crate map (pipeline / surface / support) with naming convention note, where-to-edit table, and navigation rules.
- Compacted auto-memory `MEMORY.md` from 145 → 64 lines: pure one-line index. Extracted 14 inline rule blocks into new detail files (`feedback_bean_commit_rule`, `feedback_bean_archive_rule`, `feedback_pr_merge_user_only`, `feedback_commit_message_rules`, `feedback_pr_workflow_branch_first`, `feedback_builder_agent_constraints`, `feedback_jcodemunch_usage`, `feedback_hook_restrictions`, `feedback_info_source_csl_only`, `feedback_brain_read_protocol`, `project_integral_multicite`, `project_ffi_layout`, `project_quality_gate`, `project_style_registry`).
- Added `PostToolUse` hook (`.claude/hooks/jcm-nudge.sh`) that fires on Read/Grep/Glob of Rust paths and injects a system reminder pointing to jcodemunch + rust-analyzer. Project-scoped via `$CLAUDE_PROJECT_DIR` so it does not affect other repos.

Plan reference: `~/.claude/plans/this-project-has-undergone-woolly-crab.md`.

## Verification done

- `./scripts/validate-frontmatter.sh --repo-only --copilot-strict` → 17 files OK.
- `jq` parse of `.claude/settings.json` succeeds.
- Hook smoke test with synthetic Read-on-citations.rs input emits valid PostToolUse JSON.
- Bean hygiene: this bean is the only in-progress one for the branch.
