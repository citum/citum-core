---
# csl26-qjwq
title: 'fix(docs): update rust library example'
status: in-progress
type: bug
priority: normal
created_at: 2026-11-16T21:56:12Z
updated_at: 2026-11-16T22:04:25Z
---

Renovate PR #1310 cannot generate artifacts because the standalone Rust documentation example requires unavailable citum-engine ^0.50.

- [x] Reproduce dependency resolution failure from Renovate evidence.
- [x] Update the example to published 0.84 and the current request fields.
- [x] Remove stale pre-publication instructions.
- [x] Validate the example and required Rust gate.
- [ ] Submit fix PR and verify CI.
- [ ] After the user merges the fix, retry #1310 and verify all checks.

The user approved Cargo.toml changes and this PR title. Keep the Renovate branch bot-owned.

Validation passed: published 0.84 dependency resolution, cargo check, fixture formatting, cargo run (two citations and bibliography), just pre-commit, Rust review-smell audit, and git diff --check.
