---
# csl26-p2lb
title: 'Phase 2: Create citum/labs repository'
status: completed
type: task
priority: low
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-03-01T14:23:38Z
blocking:
    - csl26-modz
    - csl26-p1rn
---

Create a `citum/labs` GitHub repository to house proof-of-concept
integrations and experimental bindings that use citum-core.

## Initial Contents

* Move `bindings/lua/` (LuaLaTeX integration) as the first experiment
  from `citum/citum-core` into `citum/labs`
* Document clearly as non-stable / proof-of-concept
* Establish the pattern for future experimental integrations (WASM, etc.)

## Guidelines for Labs

* No stability guarantees
* Each experiment is a standalone directory with its own README
* Experiments graduate to `citum-bindings` only when API is stable

Refs: csl26-modz, docs/architecture/CITUM_MODULARIZATION.md
