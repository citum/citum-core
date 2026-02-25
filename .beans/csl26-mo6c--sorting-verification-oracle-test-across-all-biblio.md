---
# csl26-mo6c
title: 'Sorting verification: oracle test across all bibliography styles'
status: todo
type: task
priority: normal
created_at: 2026-02-25T12:21:13Z
updated_at: 2026-02-25T12:21:13Z
---

## Problem

Sort templates are designed (PRIOR_ART.md Issue #61) but bibliography sorting order has not been systematically tested against oracle output across styles. This is a correctness gap that affects every bibliography style — a style that sorts correctly at 100% citation fidelity may silently produce wrong sort order.

## Scope

- Add sort-order assertions to the oracle test fixture or a dedicated sort fixture
- Run against all 10 top parent styles (author-date and numeric)
- Confirm sort behavior for: same-author same-year (year-suffix), anonymous works, all-caps sort keys, numeric ordering

## Known edge cases from ROADMAP.md

- Same author, same year disambiguation interaction
- Anonymous works (no author — sort by title)
- Numeric styles: sort by citation number, not author

## Success criteria

- Oracle includes sort-order assertions for ≥5 bibliography styles
- No sort regressions across top-10 parent styles
- Failure modes documented in `docs/guides/` if gaps found

## References

- PRIOR_ART.md (Issue #61, sort templates)
- ARCHITECTURAL_SOUNDNESS_2026-02-25.md (gap inventory)
- ROADMAP.md Phase 2 (numeric styles require correct sort)
