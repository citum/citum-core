---
# csl26-5plq
title: Implement migration variable debugger
status: completed
type: feature
priority: normal
created_at: 2026-02-07T06:53:18Z
updated_at: 2026-02-27T16:08:18Z
parent: csl26-u1in
---

Add --debug-variable flag to citum_migrate to trace variable provenance.

50% reduction in migration debugging time expected.

Features:
- Track CSL source nodes → intermediate → final YAML
- Show deduplication decisions
- Display override propagation
- Output ordering transformations

Example:
citum_migrate styles/apa.csl --debug-variable volume

Output shows: Source CSL nodes, compiled template position, rendering options, overrides

Refs: GitHub #124, WORKFLOW_ANALYSIS.md Phase 2

## Summary of Changes

Verified as already implemented during staleness audit 2026-02-27. The --debug-variable flag is fully functional in citum-migrate/src/main.rs with comprehensive provenance tracking: CSL source node tracking, intermediate compilation steps, override propagation, and ordering transformations. Implementation includes structured debug output and variable context display.
