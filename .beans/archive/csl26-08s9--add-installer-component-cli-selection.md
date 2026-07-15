---
# csl26-08s9
title: Add installer component CLI selection
status: completed
type: bug
priority: normal
tags:
    - cli
    - release
created_at: 2026-07-14T23:17:04Z
updated_at: 2026-07-15T10:39:24Z
---

Implement --components support in the POSIX installer while preserving CITUM_COMPONENTS compatibility.

- [x] Add argument parsing and help text
- [x] Add isolated installer regression coverage
- [x] Update installation documentation


## Summary of Changes

Added --components installer selection, preserved CITUM_COMPONENTS compatibility, documented the new command, and opened PR #1060 after 22 targeted regression tests passed.
