---
# csl26-drtv
title: Fix dependencies and complete Renovate automation
status: in-progress
type: bug
priority: high
created_at: 2026-11-14T17:01:18Z
updated_at: 2026-11-14T17:01:18Z
---

Patch four affected lockfile packages and repair Renovate configuration and future update coverage.

- [x] Apply targeted security updates
- [ ] Repair Renovate warning and enable OSV and daily lockfile maintenance
- [ ] Add strict pinned configuration validation in CI
- [ ] Validate audit, dependency policy, Rust gate, workflow security, and Renovate dry run
- [ ] Prepare review and verify PR CI
- [ ] Verify hosted Renovate configuration after merge

## Dependency validation

- just pre-commit: passed; fmt and clippy clean, 2,838 tests passed.
- cargo audit --deny warnings --no-yanked: passed.
- cargo deny check advisories licenses bans sources: passed with existing policy warnings.
