---
# csl26-zrz5
title: implement `disambiguate.ignore` signature exclusion option
status: completed
type: task
priority: normal
created_at: 2026-05-29T11:14:56Z
updated_at: 2026-05-29T12:44:10Z
---

Implement the `disambiguate.ignore` list option specified in docs/specs/DISAMBIGUATION.md §6.

## Steps

- [x] Add `ignore: Option<Vec<ReferenceVariable>>` to the `Disambiguation` struct in `crates/citum-schema-style/src/options/processing.rs:393`
- [x] Thread into `DisambiguationFlags` and plumb into `build_group_key` in `crates/citum-engine/src/processor/disambiguation.rs`
- [x] Regenerate JSON schemas: `cargo run --bin citum --features schema -- schema --out-dir docs/schemas`
- [x] Add tests for `ignore: [original-published]` variant
- [x] Set spec status to Active (pending xrc5 completion)

## Summary of Changes

- Added `ignore: Option<Vec<DateVariable>>` to `Disambiguation` struct in processing.rs.
- Added `ignore_original_date: bool` to `DisambiguationFlags`; populated from `DateVariable::OriginalPublished` in the ignore list.
- Consumed flag in `calculate_hints` with a comment explaining the no-op (Citum already keys on issued only).
- Regenerated `docs/schemas/style.json` (+10 lines for the new ignore field).
- Added two unit tests: YAML round-trip of `ignore: [original-published]` and default-None omission from serialization.
