---
# csl26-modz
title: 'Citum modularization (epic)'
status: in-progress
type: milestone
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-02-22T00:00:00Z
blocking: []
---

Epic tracking the phased reorganization of this workspace into the Citum
ecosystem: cleaner crate boundaries, GitHub org rename, and a public
bindings strategy.

See docs/architecture/CITUM_MODULARIZATION.md for the full plan.

## Phase 0 (now)
- csl26-p0cl: Remove unused clap dep from csln_processor
- csl26-p0dc: Move csl_legacy / biblatex conversion impls to csln_migrate

## Phase 1 (at next wave break)
- csl26-p1rn: GitHub org transfer + crate rename

## Phase 2 (before Phase 4)
- csl26-p2bn: Define citum-bindings public API
- csl26-p2lb: Create citum/labs repository with LuaLaTeX binding
