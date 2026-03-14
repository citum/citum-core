---
# csl26-17of
title: Enforce clippy complexity and line-count lints after simplify frontier closes
status: todo
type: task
priority: deferred
tags:
    - clippy
    - lint
    - cleanup
created_at: 2026-03-14T16:02:50Z
updated_at: 2026-03-14T16:02:50Z
blocked_by:
    - csl26-rsw1
---

Track enabling explicit enforcement for `clippy::too_many_lines` and `clippy::cognitive_complexity` once the current simplify frontier across engine and migrate code is complete.

## Objective

Promote the currently ad hoc hotspot-checking pass into repo-default enforcement only after the refactor backlog is reduced enough that the signal becomes actionable instead of noisy.

## Enforcement Target

When this lands, enforce these lints both locally and in CI so developers hit the same contract before push that CI enforces after push.

## Gating Rule

Do not turn these lints into local default `cargo clippy` failures or CI errors while the active engine/migrate simplify frontier is still open. Start with `csl26-rsw1` and any follow-on hotspot beans needed to finish engine and migrate cleanup.

## Checklist

- [ ] Confirm the engine hotspot frontier is closed
- [ ] Confirm the migrate hotspot frontier is closed
- [ ] Decide whether enforcement belongs in workspace lints, CI flags, or both
- [ ] Enable `clippy::too_many_lines` enforcement locally
- [ ] Enable `clippy::cognitive_complexity` enforcement locally
- [ ] Enable matching enforcement in CI
- [ ] Update docs or contributor guidance if the default verification contract changes
- [ ] Re-run full verification after enabling enforcement

## Notes

- Today these lints are not enabled by default in workspace Clippy config.
- We already use targeted explicit passes to measure progress during simplify work; this bean is for making that enforcement official later.
