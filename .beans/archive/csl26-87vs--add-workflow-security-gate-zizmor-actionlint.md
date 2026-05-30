---
# csl26-87vs
title: Add workflow security gate (zizmor + actionlint)
status: completed
type: task
priority: normal
created_at: 2026-05-30T15:33:46Z
updated_at: 2026-05-30T15:47:13Z
---

Add a new .github/workflows/actions-security.yml with zizmor + actionlint jobs, fix any existing findings so CI lands green, then open a PR. See plan at ~/.claude/plans/this-is-for-a-sorted-melody.md


## Summary of Changes

- New  with  and  jobs
- SHA-pinned all action references across all 4 existing workflow files (97 → 18 findings)
- Fixed High/High template-injection in ci.yml ( → env var ) and release.yml (/ → env vars /)
- Added  to fidelity.yml (Medium/Medium excessive-permissions)
- Added  to all checkout steps that don't need credentials for git-push
- Configured zizmor with  — filters remaining Low-confidence artipacked/cache-poisoning findings while keeping High/High and Medium/Medium gates active
- 0 blocking findings after remediation
