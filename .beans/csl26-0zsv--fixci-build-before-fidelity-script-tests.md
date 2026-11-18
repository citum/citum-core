---
# csl26-0zsv
title: 'fix(ci): build before fidelity script tests'
status: in-progress
type: bug
priority: high
created_at: 2026-11-18T14:22:49Z
updated_at: 2026-11-18T14:29:50Z
---

Run 44315965913 cancelled the first report integration test after its 180-second timeout; the other 175 tests passed. The report resolves its default-feature Citum binary through a synchronous Cargo build, so cold compilation consumes the test budget.

- [x] Diagnose the failed Actions run
- [x] Build the default-feature Citum binary before timed script tests
- [x] Validate the corpus-aware suite and workflow configuration
Delivery: a new PR with green CI; keep this bean in progress until the fix lands on main.


## Summary of Changes

Added a scope-matched default-feature Citum build before timed script tests. The exact corpus-aware suite passed all 176 tests with no cancellations; actionlint, workflow YAML/order checks, and git diff --check passed. The separate local build took 3m50s. No Rust or dependency files changed.
