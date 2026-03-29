---
# csl26-qqfa
title: Normalize genre/medium fixture values to kebab-case
status: todo
type: task
created_at: 2026-03-29T17:31:46Z
updated_at: 2026-03-29T17:31:46Z
---

Normalize existing fixture data and engine matching per ENUM_VOCABULARY_POLICY.md and GENRE_AND_MEDIUM_VALUES.md:

1. Migrate fixture values in tests/fixtures/ to kebab-case canonical forms (see migration table in GENRE_AND_MEDIUM_VALUES.md, e.g. 'PhD thesis' -> 'phd-thesis')
2. Add case-insensitive matching for genre/medium comparisons in the engine (citum-engine) so both old mixed-case and new kebab-case values match correctly during transition
3. Coordinate with a schema version bump

Do NOT change test expectations — only normalize the stored values; rendering should be unchanged.
