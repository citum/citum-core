---
# csl26-wggv
title: 'Complete doc and test coverage: csl-legacy and citum-cli'
status: completed
type: task
priority: normal
created_at: 2026-03-02T16:54:21Z
updated_at: 2026-03-02T17:19:14Z
pr: https://github.com/citum/citum-core/pull/271
---

PR #270 completed coverage for citum-engine, citum-migrate, and citum_schema. Two crates remain unaudited: csl-legacy (CSL 1.0 XML parser) and citum-cli (binary). Also a handful of low-value schema gaps were deferred.

## Remaining Work

### High priority
- [x] Audit `csl-legacy` crate: identify undocumented public items and untested logic; add `///` and unit tests where meaningful
- [x] Audit `citum-cli` crate: identify undocumented public items and untested logic; add `///` and unit tests where meaningful

### Low priority (deferred, do only if auto-doc generation reveals gaps)
- [ ] Add `//!` to `citum_schema` grouping.rs, locale/raw.rs, options/ submodules, embedded/*.rs (items already documented at field level)

## Approach
Same pattern as PR #270: audit with Explore agent → builder adds docs + tests → reviewer verifies → clean branch with no Co-Authored-By footers → PR.
