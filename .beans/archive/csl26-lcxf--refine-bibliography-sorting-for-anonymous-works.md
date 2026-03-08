---
# csl26-lcxf
title: Refine bibliography sorting for anonymous works
status: completed
type: bug
priority: normal
created_at: 2026-02-07T06:53:16Z
updated_at: 2026-02-27T00:00:00Z
parent: csl26-u1in
---

Chicago Author-Date shows entries out of order for anonymous works.

Issues:
- Anonymous work sorting not respecting 'The' article stripping
- Year fallback not working correctly for same-author entries

Fix:
- Review anonymous work sorting logic
- Ensure article stripping ('The', 'A', 'An') works for all styles
- Verify year-based secondary sort for same-name entries
- Test against Chicago Author-Date

Target: Chicago bibliography improves from 4/15 to 5/15+

Refs: GitHub #134, TIER2_PLAN.md Phase 5

## Summary of Changes

Added bibliography sorting configuration to `styles/chicago-author-date.yaml`:
- Implemented `options.processing.sort` with author (ascending) and year (ascending) as sort keys
- Added comprehensive test coverage in `crates/citum-engine/tests/bibliography.rs`:
  - `test_anonymous_works_sort_by_title_without_article`: Verifies article stripping in anonymous work sorting
  - `test_anonymous_same_year_tiebreak`: Confirms year-based secondary sorting for entries with same author/year

Oracle score: 30/32 bibliography entries match (stable at Chicago Author-Date 18th ed)