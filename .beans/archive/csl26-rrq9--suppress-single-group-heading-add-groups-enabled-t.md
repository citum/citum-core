---
# csl26-rrq9
title: Suppress single-group heading + add groups-enabled toggle
status: completed
type: bug
priority: normal
created_at: 2026-05-10T11:19:31Z
updated_at: 2026-05-10T11:27:48Z
---

Bibliography heading prints for default group even when all refs are in that single group. Fix with two-pass collection in render_with_custom_groups_filtered, plus add groups_enabled boolean (default true, with TODO) to BibliographySpec.

## Tasks

- [x] Add `groups_enabled: bool` (default true, TODO comment) to BibliographySpec
- [x] Gate custom-groups path on `bibliography.groups_enabled`
- [x] Two-pass heading suppression in render_with_custom_groups_filtered
- [x] BDD test for single-group heading suppression
- [x] Regenerate schemas

## Summary of Changes

- Added `groups_enabled: bool` (default true, with TODO) on `BibliographySpec`; manual `Default` impl preserves the new default.
- Gated the custom-groups render path on `bibliography.groups_enabled` in `processor/bibliography/mod.rs`.
- Refactored `render_with_custom_groups_filtered` to a two-pass approach: collect populated `(group, entries)` pairs and unassigned entries, then suppress heading when exactly one group populates and unassigned is empty.
- `append_rendered_group` gained a `suppress_heading` parameter; honored via guard clause.
- Added BDD tests for the single-group (heading suppressed) and two-group (both headings present) cases; updated three existing processor tests whose fixtures now produce a single populated group.
- Regenerated `docs/schemas/style.json`.

Commit: `4a13f272` on `fix/single-group-heading-suppression`. All tests pass (1243), fmt+clippy clean.
