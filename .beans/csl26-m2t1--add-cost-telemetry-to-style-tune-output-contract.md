---
# csl26-m2t1
title: Add cost telemetry to style-tune output contract
status: todo
type: task
priority: normal
tags:
    - dx
    - style
created_at: 2026-07-17T15:43:04Z
updated_at: 2026-07-17T16:02:35Z
---

If the tuning->pipeline flywheel works, cost per tuned style should fall over time. Extend the style-tune skill output contract (and style-qa record) to capture wall-time and approximate token/effort spent per tune pass, so the trend is measurable instead of anecdotal. Context: docs/architecture/audits/2026-07-17_MIGRATION_APPROACH_STRATEGIC_REVIEW.md

## Tasks

- [ ] Extend the style-tune Output Contract (.claude/skills/style-tune/SKILL.md and .skills/ counterpart if present) with two required fields: wall-clock time per tune pass and approximate token/effort spend
- [ ] Mirror the fields in the style-qa record so the QA gate preserves them
- [ ] Record values in the tune bean's Summary of Changes so history is queryable via beans
- [ ] After 3+ tuned styles carry the fields, add a short trend note (falling/flat/rising cost per style) to the next migrate/tune audit
