---
# csl26-bpuw
title: Chicago Notes humanities-note fixture recovery
status: completed
type: task
created_at: 2026-03-07T13:49:55Z
updated_at: 2026-03-09T15:30:00Z
---

5 failures remain after expanding humanities-note family fixtures (44/49 = 0.898):

## Engine Gaps (archive variable wiring)
- `dead-sea-scrolls` (manuscript): archive + archive_location not wired in variable.rs
- `austen-ms` (manuscript): same archive gap
- `derrida-letter` (personal_communication): archive field not rendering

Fix: Add `SimpleVariable::Archive => reference.archive()` and `SimpleVariable::ArchiveLocation => reference.archive_location()` in `crates/citum-engine/src/values/variable.rs`, plus accessor methods on Reference.

## Style Template Gaps
- `ginzburg1976` (book with translator): Missing `contributor: translator` component in chicago-notes.yaml template, plus original-title handling
- `foucault-interview` (interview): Container-title + pages not rendering (interview suppressed in chapter-like block), title quoting may have engine issue

## Acceptance Criteria
- All 5 failures pass (49/49)
- Chicago Notes baseline restored to 1.0
- No regressions in core fixture (37/37 still passing)

## Summary of Changes

- Confirmed the archive/archive-location and humanities-note recovery work was
  already landed on `main`; no new Chicago Notes code was needed in this PR.
- Verified the focused regression coverage still passes on the branch:
  - `cargo test -p citum-engine test_humanities_note_fixture_preserves_archive_and_interview_fields -- --exact --nocapture`
  - `cargo test -p citum-engine chicago_notes -- --nocapture`
- Archived this bean and resolved the duplicate follow-on as `csl26-6i1c`.
