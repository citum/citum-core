---
# csl26-57n6
title: 'Rust simplify pass: citum-migrate main.rs'
status: completed
type: task
priority: normal
created_at: 2026-03-14T16:44:01Z
updated_at: 2026-03-14T17:52:22Z
---

Ongoing simplify pass on crates/citum-migrate/src/main.rs. Target: concision, DRY, idioms, maintainability. PR branch: simplify/migrate-main-rs

## 2026-03-14\n- crates/citum-migrate/src/main.rs: DRY (merge_type_rendering helper, patent template reuse, scrub_overrides_map helper), concision (is_some_and for scrub_pages_year), 2998→2905 lines

## Summary of Changes
- Extracted ~60 fixup helpers from main.rs into new fixups.rs library module
- Added merge_type_rendering and scrub_overrides_map DRY helpers (6 call sites each)
- Reused base_media_template_from_bibliography inside patent template logic
- Fixed import hygiene (dead imports removed, test-only moved to test module)
- Fixed pre-existing citum-engine fmt and clippy issues surfaced by CI
- main.rs: 2998 → 1507 lines (−50%); fixups.rs: 1424 lines
