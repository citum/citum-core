---
# csl26-ctw8
title: 'citum-migrate: extract strip-periods from CSL source'
status: completed
type: task
priority: normal
created_at: 2026-02-14T15:05:35Z
updated_at: 2026-03-09T15:30:00Z
---

The citum-migrate options extractor does not detect strip-periods from CSL sources. The upsampler.rs map_formatting() hardcodes strip_periods to None (line 648). Many CSL styles use strip-periods='true' on labels (especially editor labels). This should be extracted and set at the appropriate config tier (global options or bibliography.options). Related: strip-periods is available at Tier 1 (global options) and Tier 2 (citation/bibliography options) in Citum, not just per-component.

## Summary of Changes

- Preserved node-local `strip_periods` values in `citum-migrate` upsampling for
  legacy `Text` and `Label` nodes.
- Carried label-driven strip-period behavior through template compilation so
  compiled number, contributor, and locator template output retains the
  existing engine semantics.
- Added migrate regression tests covering:
  - term upsampling with `strip-periods="true"`
  - label upsampling with `strip-periods="true"`
  - absent/false preservation behavior
  - template-compilation preservation for term, number, contributor, and
    locator label cases
