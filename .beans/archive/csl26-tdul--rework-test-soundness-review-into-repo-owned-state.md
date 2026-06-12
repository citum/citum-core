---
# csl26-tdul
title: Rework test-soundness-review into repo-owned state-tracking skill
status: completed
type: feature
priority: normal
created_at: 2026-06-12T10:47:35Z
updated_at: 2026-06-12T10:53:36Z
---

Move test-soundness-review into .skills/, replace JSON output with a living ledger + dated audit records, promote spec review to first-class with blocking/advisory gating, add a 'redundant' verdict for low-value tests, and consolidate a shared test-value contract in CODING_STANDARDS.md. Two commits: (1) skill, (2) docs.

## Summary of Changes

Commit 1 (skill): added .skills/test-soundness-review/SKILL.md — repo-owned
auditor. Replaced JSON-to-stdout with durable ledger + dated audit records,
promoted spec review to a first-class step with blocking/advisory gating,
added a 'redundant' verdict for low-value tests, and a no-arg ledger
navigation mode.

Commit 2 (docs): created docs/architecture/TEST_SOUNDNESS_STATUS.md (living
ledger seeded from the spec inventory; DISAMBIGUATION backfilled as audited
from commit aa7af758). Added the shared 'What makes a test worth keeping'
contract to CODING_STANDARDS.md and pointed TEST_STRATEGY.md, the
test-coverage skill, and the soundness skill at it. Registered the ledger in
the architecture README.
