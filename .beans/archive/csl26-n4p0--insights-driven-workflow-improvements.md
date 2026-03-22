---
# csl26-n4p0
title: Insights-driven workflow improvements
status: completed
type: task
priority: normal
created_at: 2026-03-22T15:42:17Z
updated_at: 2026-03-22T15:50:48Z
---

Implement high-ROI fixes from /insights report: pre-commit Rust gate hook, /squash-pr skill, CLAUDE.md schema-regen and path-confirmation rules.

## Summary of Changes

- Created `~/.claude/scripts/hooks/pre-commit-rust-gate.sh` — intercepts git commit in Rust workspaces, runs cargo fmt --check + clippy + schema regen check
- Wired hook into `~/.claude/settings.json` PreToolUse[Bash] array
- Created `~/.claude/skills/squash-pr/SKILL.md` — safe squash-rebase skill with branch guard, approval gate, temp-file GIT_SEQUENCE_EDITOR, hygiene rules
- Added schema regen rule to CLAUDE.md pre-commit section
- Added ~/.claude path confirmation rule to CLAUDE.md confirmations
- PR #420 for the CLAUDE.md changes
