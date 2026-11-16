---
# csl26-yr77
title: 'fix(tooling): add renovate commit bodies'
status: in-progress
type: bug
priority: normal
created_at: 2026-11-16T21:35:36Z
updated_at: 2026-11-16T21:58:26Z
---

Renovate PR #1310 has no commit body, which fails Citum commit policy.

- [x] Configure the approved Renovate commit body.
- [x] Verify original and corrected messages against the commit hook.
- [x] Validate Renovate configuration and submit a separate PR.
- [ ] After the user merges the fix, request Renovate regeneration of #1310 and verify all checks.

PR #1310 remains owned by Renovate; merging is the user’s action.

PR #1312 merged. Regenerated commit a8ecbdf includes the configured body and Commit Policy passed. Verification found a separate renovate/artifacts failure caused by the Rust example dependency on ^0.50; follow-up csl26-qjwq tracks recovery.

All GitHub Actions checks passed for regenerated PR #1310 (including Rust CI, Security Audit, and Release Dry Runs). Only renovate/artifacts remains red.
