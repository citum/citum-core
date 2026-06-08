---
# csl26-dpep
title: 'fix(engine): same-author multicite collapse — first-class rule for both modes'
status: completed
type: bug
priority: normal
created_at: 2026-06-08T22:32:36Z
updated_at: 2026-06-08T22:41:51Z
---

Same-author multicites render incorrectly in both modes. Integral: 'Chen (2017); Chen (2020)' instead of 'Chen (2017, 2020)'. Non-integral: '(Chen, 2017); (Chen, 2020)' instead of '(Chen, 2017, 2020)'. PR #889 has a partial fix (integral only) with template-mutation hacks. This bean tracks the first-class refactor: capture-and-apply-once wrap, explicit routing rule in render_grouped_citation_group_with_format, spec rename/broadening, and tests for both modes + bracket wrap.

## Summary of Changes

- ****: Replaced two implementation-specific workarounds ( template mutation +  early-return in ) with a clean first-class rule:
  -  now explicitly routes multi-item same-author groups to the collapse (fallback) path for integral mode; single-item groups keep the per-item explicit path
  -  captures the year-group  from the first item's filtered template and strips it from all items (integral multi-item groups only); the captured wrap is returned to the caller
  -  threads the captured wrap to 
  -  uses  (new helper) instead of hardcoded  — bracket-wrapped styles now render correctly
  - Deleted 
- ****: Added two new tests alongside the existing integral collapse test:
  -  — non-integral flat template + cluster wrap → "(Chen, 2017, 2020)"
  -  — integral bracket wrap → "Chen [2017, 2020]"
- ****: Renamed  → ; broadened scope to cover both modes; added "Same-Author Collapse" section documenting the invariant and implementation
