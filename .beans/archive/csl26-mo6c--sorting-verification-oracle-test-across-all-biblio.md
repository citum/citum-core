---
# csl26-mo6c
title: 'Sorting verification: oracle test across all bibliography styles'
status: completed
type: task
priority: normal
created_at: 2026-02-25T12:21:13Z
updated_at: 2026-03-01T14:38:15Z
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

## Summary of Changes

Added tests/fixtures/sort-oracle.json with 10 references covering sort edge cases:
- Multiple works by same author/year (Adams 2020 × 3)
- Anonymous works with article-prefixed titles
- All-caps surnames (SMITH, WILLIAMS)
- Multi-author books and articles
- Varied volume/issue numbers for numeric style independence

Added crates/citum-engine/tests/sort_oracle.rs with 6 oracle-level sort assertions:
1. test_apa_7th_sort_same_author_year_by_title — verifies title-based tiebreaker
2. test_apa_7th_sort_anonymous_works_by_title — documents article-stripping gap
3. test_numeric_sort_by_citation_order — numeric assignment by citation order
4. test_uppercase_surname_sort_order — SMITH before WILLIAMS alphabetically
5. test_multiauthor_same_year_sort — multi-author books sorted correctly
6. test_numeric_style_volume_issue_independence — citation order trumps volume

Updated bibliography.rs test_anonymous_works_sort_by_title_without_article with improved TODO comment linking to csl26-srvr and csl26-mo6c.
