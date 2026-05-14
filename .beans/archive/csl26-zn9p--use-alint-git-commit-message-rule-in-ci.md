---
# csl26-zn9p
title: Use alint git_commit_message rule in CI
status: completed
type: task
priority: normal
created_at: 2026-05-14T12:03:28Z
updated_at: 2026-05-14T12:03:59Z
---

Replace the manual bash commit-msg validation step in ci.yml with the alint git_commit_message rule (available in v0.9.21). Bump alint action to v0.9.21. Keep .githooks/commit-msg for local enforcement.

## Summary of Changes

- Added  rule (kind: ) to  with the same constraints as the removed bash step: conventional commit pattern, 50-char subject limit, body required, Revert/fixup!/squash! bypass.
- Removed the manual 'Validate commit messages' bash step from .
- Bumped alint action from v0.9.20 to v0.9.21 (first version with git_commit_message support).
-  retained for local enforcement.
