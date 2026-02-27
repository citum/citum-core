---
# csl26-5axq
title: Evaluate ICU library for date/time internationalization
status: completed
type: task
priority: low
created_at: 2026-02-07T06:53:51Z
updated_at: 2026-02-27T22:43:52Z
---

From GitHub Issue #93: Add proper internationalization of dates and times. Evaluate pros and cons of using ICU library vs alternatives.

Consider:
- EDTF native date handling (already prioritized)
- Locale-specific date formatting
- Integration complexity and dependencies
- Performance implications
- Compatibility with existing date handling

Impact: Architecture decision
Effort: 1 week (research + doc)

Refs: GitHub #144, GitHub #93

## Summary of Changes\n\nEvaluation complete. Decision: proceed natively for time components; revisit ICU4X specifically for `icu_plurals` when ordinal day formatting becomes a requirement. ICU4X is overkill for citation time rendering (15-20 locales, simple AM/PM terms). icu4c eliminated due to WASM target incompatibility.
