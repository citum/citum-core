---
# csl26-wtwi
title: Fix changelog generation in release workflow
status: completed
type: bug
priority: normal
created_at: 2026-05-26T12:11:34Z
updated_at: 2026-05-26T12:12:57Z
---

git-cliff runs inside cargo-release pre-release-hook with ambiguous commit range; changelog entries are missing or just rename the version number. Fix: move git-cliff call to explicit workflow step before cargo-release, use --prepend with explicit LAST_TAG..HEAD range.

## Summary of Changes

Moved git-cliff call from cargo-release pre-release-hook into an explicit workflow step. Now uses LAST_TAG..HEAD range with --prepend. PR #810.
