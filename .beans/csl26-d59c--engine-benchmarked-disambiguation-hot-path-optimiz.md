---
# csl26-d59c
title: 'engine: benchmarked disambiguation hot-path optimization'
status: todo
type: task
priority: normal
created_at: 2026-03-26T19:21:54Z
updated_at: 2026-03-26T19:21:54Z
parent: csl26-fk0w
---

Follow-up performance slice from csl26-3oq0.

Scope this bean to disambiguation hot-path allocation work that was explicitly deferred from the low-risk rendering optimization PR. Require benchmark numbers before and after changes, and keep the work separate from correctness fixes unless a benchmarked refactor exposes a behavioral regression that must be fixed in the same slice.

Primary hotspot to target:
- Disambiguation builds many short-lived strings and vectors

## Tasks
- [ ] Capture a fresh baseline for disambiguation-heavy rendering scenarios
- [ ] Identify the highest-allocation disambiguation path with focused benchmarks or profiling
- [ ] Implement a benchmarked optimization slice without changing rendering semantics
- [ ] Record before/after numbers in the PR description or bean summary

Parent context: csl26-fk0w
Deferred from: csl26-3oq0
