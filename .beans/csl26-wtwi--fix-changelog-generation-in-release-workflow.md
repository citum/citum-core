---
# csl26-wtwi
title: Fix changelog generation in release workflow
status: in-progress
type: bug
created_at: 2026-05-26T12:11:34Z
updated_at: 2026-05-26T12:11:34Z
---

git-cliff runs inside cargo-release pre-release-hook with ambiguous commit range; changelog entries are missing or just rename the version number. Fix: move git-cliff call to explicit workflow step before cargo-release, use --prepend with explicit LAST_TAG..HEAD range.
