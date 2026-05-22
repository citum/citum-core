---
# csl26-yzmc
title: Remove incorrect FullThenShort from MLA; add ShortOnly to IntegralNameRule
status: completed
type: task
priority: normal
created_at: 2026-05-22T23:17:39Z
updated_at: 2026-05-22T23:21:22Z
---

MLA's integral-names config incorrectly applied name-memory (FullThenShort). Research shows ShortOnly is correct for all major styles including MLA. Fix: remove integral-names block from MLA YAML, add ShortOnly variant to IntegralNameRule enum for future explicitness.

## Summary of Changes

- Added `ShortOnly` variant to `IntegralNameRule` in `citum-schema-style/src/options/integral_names.rs`
- Removed the incorrect `integral-names` block from MLA's embedded style YAML; research confirmed FullThenShort name-memory doesn't match how any major style (including MLA) works in practice
- Replaced the MLA-specific schema config test with a behavioral test for `ShortOnly`: verifies that a Subsequent state is ignored (full name rendered) when the rule is `short-only`
