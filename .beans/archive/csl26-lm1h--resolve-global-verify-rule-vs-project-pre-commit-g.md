---
# csl26-lm1h
title: Resolve global verify rule vs project pre-commit gate conflict
status: completed
type: task
priority: normal
created_at: 2026-05-28T22:56:30Z
updated_at: 2026-05-28T22:56:49Z
---

Global kernel (~/.claude/CLAUDE.md) says never call cargo directly, use verify.sh. But the global rust adapter is strictly weaker than this repo's pre-commit gate (swallows clippy failures, no fmt --check, cargo test not nextest, no schema regen). Add a project-override line in citum-core/CLAUDE.md Pre-Commit Gate section so the repo gate is authoritative. Doc-only.

- [x] Add override line to citum-core/CLAUDE.md Pre-Commit Gate section
- [x] Confirm it reads cleanly and does not contradict the gate command

## Summary of Changes

Added a project-override paragraph to the `## Pre-Commit Gate (Rust)` section of `citum-core/CLAUDE.md` declaring the repo gate authoritative and exempting it from the global `verify.sh` rule (which uses a weaker adapter). Doc-only; no behavior code touched.
