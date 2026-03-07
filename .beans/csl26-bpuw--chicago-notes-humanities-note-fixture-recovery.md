---
# csl26-bpuw
title: Chicago Notes humanities-note fixture recovery
status: todo
type: task
created_at: 2026-03-07T13:49:55Z
updated_at: 2026-03-07T13:49:55Z
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
