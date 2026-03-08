---
# csl26-l2hg
title: Numeric Style Rendering
status: completed
type: epic
priority: critical
created_at: 2026-02-07T07:40:14Z
updated_at: 2026-02-26T14:00:00Z
blocking:
    - csl26-yxvz
---

Harden numeric-style fidelity (Vancouver, Springer, etc.) across all
citation and bibliography rendering scenarios.

Canonical status: `docs/TIER_STATUS.md`

## Checklist

- [x] csl26-6whe — Fix year positioning for numeric styles
- [x] csl26-tbnq — Debug Springer citation regression
- [x] csl26-ul0p — Fix conference paper template formatting
- [x] csl26-aicv — Fix volume/issue ordering for numeric styles
- [x] csl26-j7h7 — Support superscript citation numbers

## Summary of Changes

All five child tasks completed. Oracle confirms:
- `elsevier-vancouver.csl` 32/32 bibliography ✓
- `elsevier-harvard.csl` 32/32 bibliography ✓ (including chapter/conference paper)
- `nature.csl` 13/13 citations ✓ (superscript handled in display layer)
- Core quality gate: 141/141 styles at fidelity 1.0 ✓

Remaining non-epic failures (nature publisher:extra, chicago-author-date patent type) are
tracked separately and do not block this epic.
