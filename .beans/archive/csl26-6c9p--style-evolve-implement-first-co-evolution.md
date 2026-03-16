---
# csl26-6c9p
title: 'style-evolve: implement-first co-evolution'
status: completed
type: task
priority: normal
created_at: 2026-03-16T10:22:21Z
updated_at: 2026-03-16T10:23:19Z
---

Rewrite co-evolution logic across style-evolve, style-maintain, and style-migrate-enhance to default to attempting Rust fixes rather than deferring. Add experiment-first bias, root-cause grouping, bean templates, engine-fix-unlocks artifact, and cross-skill table alignment.

## Summary of Changes

- **style-maintain**: Rewrote Co-Evolution section with implement-first bias. Added root-cause grouping step before jCodeMunch lookups, replaced tractability pre-assessment with experiment-first loop (write + test, defer only on hard blockers), added rich bean template (symbol path, oracle diff, fix sketch, unlocks), added Unlocks column to Code Opportunities table.
- **style-evolve**: Updated Co-Evolution Rule to reflect implement-first default. Tightened deferred rules to three hard blockers only. Added Unlocks column to table format. Added dedup check (beans list -S) before filing.
- **style-migrate-enhance**: Aligned Required Artifacts to use Code Opportunities table format; migration-engine gaps now follow the same implement-first rule.
