---
# csl26-p12i
title: Automate security dependency updates via Renovate vulnerability alerts
status: completed
type: task
priority: normal
created_at: 2026-06-22T23:31:54Z
updated_at: 2026-06-22T23:33:33Z
---

Add vulnerabilityAlerts to renovate.json so security-fixable advisories get immediate PRs (bypassing the weekly Monday schedule). Also consolidate duplicate RUSTSEC ignore list from ci.yml into deny.toml.

## Summary of Changes

- Added `vulnerabilityAlerts` block to `renovate.json` with `"schedule": ["at any time"]`, `prPriority: 10`, `groupName: null`
- Removed 4 duplicate `--ignore RUSTSEC-*` flags from `cargo audit` step in `ci.yml`; deny.toml is now sole source of truth
- PR: https://github.com/citum/citum-core/pull/959
