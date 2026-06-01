---
# csl26-vz0t
title: render doc emits full bibliography for grouped styles
status: completed
type: bug
priority: high
created_at: 2026-06-01T15:27:38Z
updated_at: 2026-06-01T15:32:17Z
---

## Summary of Changes

- Added  private method to  with a  param — groups and partition branches now intersect with  when true.
- Added  pub(crate) method that delegates with .
- Swapped  document path to use the new cited-only method.
- Added two regression tests in  covering: no citations → no bibliography; one citation → only cited ref in bibliography.
- All 1466 tests pass.
