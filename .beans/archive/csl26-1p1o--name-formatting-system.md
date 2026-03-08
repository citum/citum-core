---
# csl26-1p1o
title: Name Formatting System
status: completed
type: epic
priority: high
created_at: 2026-02-07T07:40:14Z
updated_at: 2026-02-27T16:11:19Z
blocking:
    - csl26-yxvz
---

Implement complete name formatting system with support for initials without periods, custom delimiters, and internationalization.

## Summary of Changes

Verified as already implemented during staleness audit 2026-02-27:
- Complete name formatting system with `format_names` and `format_single_name` functions in citum_engine
- Support for initials with/without periods via `initialize_with` and `initialize_with_hyphen` options
- Custom delimiters and `sort_separator` configuration
- Full internationalization via multilingual name resolution with script preference
- All name parts (family, given, particles, suffix) supported
- Disambiguation via name expansion (`min_names_to_show`)
