---
# csl26-hbxx
title: 'Oracle normalizer: handle HTML-wrapped bibliography numbering'
status: completed
type: bug
priority: normal
created_at: 2026-02-14T15:05:47Z
updated_at: 2026-02-26T00:00:00Z
---

The normalizeText() in oracle-utils.js now strips leading numbering (e.g. '1. ') but citeproc-js wraps the number in <div class='csl-left-margin'>1. </div>. After HTML stripping, the number prefix may be preceded by whitespace and not at the start of the string, so the ^\d+\.\s+ regex doesn't match. Need to handle csl-left-margin/csl-right-inline div structure properly, or strip after whitespace normalization.

## Summary of Changes

Moved the bibliography numbering strip regex from line 58 to line 61 (after whitespace normalization) in `scripts/oracle-utils.js`. The regex now fires on an already-normalized string where all `\s+` has been collapsed to single spaces. Simplified the regex from `/^[\s\u200e\u200f\u202a-\u202e\u2066-\u2069]*\d+\.\s*/` to `/^[\u200e\u200f\u202a-\u202e\u2066-\u2069]*\d+\.\s*/` (removed leading `\s*` since whitespace is already collapsed). This ensures HTML-wrapped bibliography numbers like `<div class='csl-left-margin'>1. </div>...` are properly normalized after tag stripping.
