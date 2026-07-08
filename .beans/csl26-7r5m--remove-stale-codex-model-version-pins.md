---
# csl26-7r5m
title: Remove stale Codex model version pins
status: in-progress
type: task
priority: normal
created_at: 2026-07-08T21:08:30Z
updated_at: 2026-07-08T21:09:03Z
---

Replace versioned Codex model IDs in repo-local agent contracts and launcher with stable aliases.

- [x] Create stable model aliases in agent contracts
- [x] Update scripts/codex fallback launcher
- [x] Verify no stale gpt-5.x pins remain in Codex agent surfaces
- [x] Run shell syntax and beans hygiene checks
- [ ] Commit, push, and open draft PR
