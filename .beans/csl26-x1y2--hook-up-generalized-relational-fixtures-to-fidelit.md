---
# csl26-x1y2
title: Hook up generalized relational fixtures to fidelity reporting
status: todo
type: task
priority: normal
created_at: 2026-04-01T15:00:00Z
updated_at: 2026-04-01T15:00:00Z
---

Following the implementation of the generalized relational container model (PR #485 / v0.20.0), we need to ensure the new high-fidelity fixtures (comprehensive.yaml, chicago-bib.yaml, etc.) are integrated into the automated fidelity reporting pipeline.

## Context
The architectural shift to recursive `WorkRelation` and `numbering` system requires updating the fidelity tools to correctly resolve and compare rendered output against these new relational structures.

## Tasks
- [ ] Update fidelity reporting tools to support recursive container resolution.
- [ ] Add `examples/comprehensive.yaml` to the fidelity benchmark suite.
- [ ] Verify that shorthand fields (volume, issue, etc.) are correctly handled by the reporting logic.
- [ ] Compare fidelity scores against the previous flat model baselines.
