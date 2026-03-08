---
# csl26-l3vu
title: Complete variable-once rule implementation
status: completed
type: feature
priority: critical
created_at: 2026-02-07T12:11:49Z
updated_at: 2026-02-26T00:00:00Z
parent: csl26-u1in
---

Ensure variable-once rule is fully implemented across all migration scenarios.

Tasks:
- Prevent duplicate list variables during migration
- Add suppress overrides automatically where needed
- Handle edge cases (contributor+date, title+container-title, etc.)
- Verify no silent duplication in rendered output

Recent work: Commits on duplicate list variable prevention (csl26-6whe fix)

Impact: Critical for bibliography accuracy

## Summary of Changes

Implemented comprehensive cross-list variable deduplication in citum-migrate:

1. **New Pass**: `deduplicate_variables_cross_lists` in `passes/deduplicate.rs`
   - Tracks seen variables globally across sibling lists and components
   - Suppresses duplicate occurrences using `ComponentOverride::Rendering`
   - Handles all variable types: SimpleVariable, Contributor, Title, Date, Number

2. **Integration**: Wired into migration pipeline in `main.rs`
   - Runs after title deduplication, before list overrides propagation
   - Maintains correct ordering for deduplication passes

3. **Regression Tests**: Added `tests/variable_once.rs` with 4 comprehensive test cases
   - Contributor cross-list duplicates
   - Date cross-list duplicates
   - Simple variable cross-list duplicates
   - Nested list variable scope handling

All tests passing, no clippy warnings, code follows Citum style guidelines.