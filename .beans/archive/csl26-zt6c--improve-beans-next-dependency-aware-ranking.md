---
# csl26-zt6c
title: 'Improve /beans next: dependency-aware ranking'
status: completed
type: task
priority: high
created_at: 2026-03-06T13:47:09Z
updated_at: 2026-04-24T12:13:54Z
---

Improve the citum-bean next script and beans SKILL.md so /beans next can be trusted without manual override.

- [x] Rewrite citum-bean next to use beans query GraphQL for leverage scoring
- [x] Rank by: priority → leverage (unblock count) desc → type → age
- [x] Show in-progress beans as context header
- [x] Display 'Unblocks N' in output when leverage > 0
- [x] Drop redundant beans prime instruction from SKILL.md
- [x] Add body-modification cheat sheet to SKILL.md
- [x] Expand description trigger phrases in SKILL.md

## Summary of Changes

Rewrote citum-bean next to use a single GraphQL query (beans query) to fetch
all non-terminal beans with blockingIds in one round trip. Leverage score
is computed per bean as the count of its blockingIds that are still open.
Ranking now uses: priority → leverage desc → type → age.

Output now includes an in-progress context header showing what is already
running, and a '· unblocks N' badge for high-leverage candidates.

SKILL.md updated: dropped redundant beans prime call, added body-modification
cheat sheet with exact syntax for checklist check-off, and expanded description
trigger phrases.
