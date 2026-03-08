---
# csl26-yrri
title: 'fix(labels): AMS et-al takes 4 initials not 3'
status: completed
type: bug
priority: normal
created_at: 2026-02-21T15:25:34Z
updated_at: 2026-02-27T14:28:31Z
---

In labels.rs, et_al branch hardcodes .take(3) + marker. AMS/citeproc-js takes first 4 initials with no marker for 5+ authors. Fix: make the take count configurable (4 for ams, 3 for alpha/din). Workaround in styles/american-mathematical-society-label.yaml already sets et-al-marker:'' but still emits 3 initials. Tracked in PR #211.

## Summary of Changes

Added `et_al_names` field to `LabelConfig` and `LabelParams`. AMS preset now defaults to 4, Alpha/Din to 3. Engine `labels.rs` replaced hardcoded `.take(3)` with `.take(params.et_al_names as usize)`. Tests added to verify AMS uses 4 initials, Alpha uses 3.
