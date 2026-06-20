---
# csl26-c0lk
title: Integrate vouch for PR contributor gating
status: completed
type: task
priority: normal
created_at: 2026-06-20T14:07:52Z
updated_at: 2026-06-20T14:10:35Z
---

Add mitchellh/vouch to gate PR/issue creation against VOUCHED.td allowlist; rewrite CONTRIBUTING.md to document the policy.

## Tasks

- [x] Create .github/VOUCHED.td with bdarcus pre-vouched
- [x] Create .github/vouch-close.md (auto-close message template)
- [x] Create .github/workflows/vouch-check.yml (PR + issue gating)
- [x] Create .github/workflows/vouch-manage.yml (maintainer vouch management)
- [x] Rewrite CONTRIBUTING.md with vouch policy section

## Summary of Changes

Added vouch-based contributor gating: .github/VOUCHED.td allowlist, two GitHub Actions workflows (vouch-check.yml, vouch-manage.yml), a close-message template, and a rewritten CONTRIBUTING.md. Committed on branch ci/vouch-contributor-gating.
