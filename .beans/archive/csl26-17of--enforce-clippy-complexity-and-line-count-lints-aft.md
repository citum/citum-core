---
# csl26-17of
title: Enforce clippy complexity and line-count lints after simplify frontier closes
status: completed
type: task
priority: deferred
tags:
    - clippy
    - lint
    - cleanup
created_at: 2026-03-14T16:02:50Z
updated_at: 2026-03-15T16:27:08Z
blocked_by:
    - csl26-ey6s
    - csl26-5zzb
---

Track enabling explicit enforcement for `clippy::too_many_lines` and `clippy::cognitive_complexity` once the current simplify frontier across engine and migrate code is complete.

## Objective

Promote the currently ad hoc hotspot-checking pass into repo-default enforcement only after the refactor backlog is reduced enough that the signal becomes actionable instead of noisy.

## Enforcement Target

When this lands, enforce these lints both locally and in CI so developers hit the same contract before push that CI enforces after push.

## Gating Rule

Do not turn these lints into local default `cargo clippy` failures or CI
errors while the active engine/migrate simplify frontier is still open. The
remaining frontier is represented by the follow-on hotspot beans below.

## Current Follow-on Frontier

The remaining hotspot frontier is tracked in:
- `csl26-ey6s` — grouped citation rendering frontier
- `csl26-5zzb` — Djot adapter and shared document pipeline seams

Close those follow-on beans before enabling repo-default complexity and
line-count lint enforcement.

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

## Summary of Changes

Enabled too_many_lines = "deny" and cognitive_complexity = "deny" in workspace lints. Scoped #[allow] with FIXME trackers applied to all violating functions, referencing deferral bean csl26-44gu. Frontier was clear; both original blockers archived.
