---
# csl26-chc9
title: 'feat: numeric-compound styles PR'
status: completed
type: feature
priority: normal
created_at: 2026-03-06T17:30:33Z
updated_at: 2026-03-06T18:05:12Z
---

Engine verification + snapshot testing + PR for the 5 compound styles (numeric-comp, angewandte-chemie, chem-acs, chem-biochem, chem-rsc)

## Summary of Changes

- Engine test: all 14 compound-numeric tests pass (no fixes needed)
- Created scripts/oracle-native.js: snapshot oracle for native styles
- Modified scripts/report-core.js: isNative detection + runNativeOracle routing
- Generated 5 golden snapshots in tests/snapshots/compound/
- Updated baseline: 16 tracked styles, all fidelity 1.0
- PR #296 created and CI passed
