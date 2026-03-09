---
# csl26-modz
title: 'Citum modularization (epic)'
status: completed
type: milestone
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-03-09T15:20:00Z
blocking: []
---

Completed phased reorganization of this workspace into the Citum ecosystem:
cleaner crate boundaries, GitHub org rename, and a public bindings strategy.

See docs/architecture/CITUM_MODULARIZATION.md for the full plan.

## Phase 0 (completed)
- csl26-p0cl: Remove unused clap dep from citum_engine ✅
- csl26-p0dc: Decouple citum_schema from csl_legacy and biblatex ✅

## Phase 1 (completed)
- csl26-p1rn: GitHub org transfer + crate rename ✅

## Phase 2 (completed)
- csl26-p2bn: Define citum-bindings public API ✅
- csl26-p2lb: Create citum/labs repository with LuaLaTeX binding ✅

Exit criteria met:
- All five linked modularization phase beans are completed.
- The dependency-boundary cleanup landed on `main`.
- The crate/org rename and bindings strategy work landed on `main`.

## Summary of Changes

- Closed this epic because every phase bean it tracked is now completed:
  `csl26-p0cl`, `csl26-p0dc`, `csl26-p1rn`, `csl26-p2bn`, and `csl26-p2lb`.
- Reframed the bean from an active umbrella milestone to a completed
  implementation milestone.
- Archived after verification so the bean no longer remains open at the repo
  root.
