---
# csl26-mlt3
title: Add preferred-transliteration style config
status: completed
type: feature
priority: normal
created_at: 2026-02-12T00:00:00Z
updated_at: 2026-02-27T23:41:55Z
---

Add preferred-transliteration field to MultilingualConfig supporting priority lists.

Implement matching algorithm (exact BCP 47 → script prefix → fallback).

Refs: expert feedback on transliteration methods,docs/architecture/MULTILINGUAL.md Section 1.3

## Summary of Changes

Added  field to  as an ordered priority list of BCP 47 tags. Implemented  helper with 4-step matching (priority-list exact → priority-list substring → preferred_script exact → preferred_script substring). Threaded new parameter through all call sites in , , and  (9 sites total). Added 4 unit tests and updated 22 integration tests in . All 397 tests pass.
