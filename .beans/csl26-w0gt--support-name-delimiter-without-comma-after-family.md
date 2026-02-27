---
# csl26-w0gt
title: Support name delimiter without comma after family
status: completed
type: feature
priority: high
created_at: 2026-02-07T06:53:10Z
updated_at: 2026-02-27T15:20:59Z
blocking:
    - csl26-1p1o
---

Some styles use space-only separator between family and initials.

Current: Smith, J, Anderson, M
Expected: Smith J, Anderson M

Fix:
- Extract sort-separator=' ' from CSL name element
- Apply to bibliography contributor config
- Test against styles using space-only separator
- Ensure family-given delimiter is configurable per style

Refs: GitHub #133, TIER3_PLAN.md Issue 2.3

## Summary of Changes

Confirmed sort_separator=' ' is handled correctly in contributor.rs. Added unit test verifying space separator produces 'Smith J' (no comma) when inverted, and default produces 'Smith, J'.
