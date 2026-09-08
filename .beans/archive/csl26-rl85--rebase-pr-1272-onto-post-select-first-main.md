---
# csl26-rl85
title: 'Rebase PR #1272 onto post-select-first main'
status: completed
type: task
priority: high
tags:
    - engine
    - fidelity
created_at: 2026-09-10T21:07:19Z
updated_at: 2026-09-10T21:22:48Z
---

PR #1272 (feat/csl26-medium-designator) was branched before the select:first stack merged into main; left off last session's stack sync, branch became CONFLICTING/DIRTY. Fixing: rebase the single commit (35c1ac87f) onto main, resolve template.rs/date.rs additive conflicts, regenerate docs/schemas, force-push, restore green CI.

## Summary of Changes

Rebased `35c1ac87f` onto main via `git rebase`. Resolved 2 real conflicts (`template.rs`, `date.rs` — both additive `DateForm` variants from this branch and the meanwhile-merged ASME work), plus 2 generated-schema conflicts resolved by regenerating with `just schema-gen`. `just pre-commit` green (2806/2806). Force-pushed; PR body updated to reflect the rebase. CI green, PR #1272 now MERGEABLE/CLEAN.
