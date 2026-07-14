---
# csl26-08s9
title: Add installer component CLI selection
status: in-progress
type: bug
priority: normal
tags:
    - cli
    - release
created_at: 2026-07-14T23:17:04Z
updated_at: 2026-07-14T23:22:34Z
---

Implement --components support in the POSIX installer while preserving CITUM_COMPONENTS compatibility.

- [ ] Add argument parsing and help text
- [ ] Add isolated installer regression coverage
- [x] Update installation documentation
- [x] Validate, open PR, and record results

## Summary of Changes

Added --components installer selection, preserved CITUM_COMPONENTS compatibility, documented the new command, and opened PR #1060 after 22 targeted regression tests passed.
