---
# csl26-aicv
title: Fix volume/issue ordering for numeric styles
status: completed
type: bug
priority: high
created_at: 2026-02-07T06:44:03Z
updated_at: 2026-02-26T14:00:00Z
blocking:
    - csl26-l2hg
---

Numeric styles like Vancouver show volume incorrectly.

Current: volume(issue)
Expected: volume(issue) or volume: issue depending on style

Refs: GitHub #129

## Summary of Changes

Verified resolved via oracle: `elsevier-vancouver.csl` passes 32/32 bibliography.
Journal articles render as `Nature 2015;521:436–44` (year;volume:pages), which matches
the Vancouver semicolon convention exactly. No code changes required — fix was applied
in prior migration work.
