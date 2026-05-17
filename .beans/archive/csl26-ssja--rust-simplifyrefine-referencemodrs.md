---
# csl26-ssja
title: Rust simplify+refine reference/mod.rs
status: completed
type: task
priority: normal
created_at: 2026-05-17T09:11:07Z
updated_at: 2026-05-17T12:12:00Z
---

Extract 3377-line crates/citum-schema-data/src/reference/mod.rs into named siblings; move the `InputReference` shell and class discriminator types to focused modules; collapse pure class dispatch; remove 2 clippy-allow pragmas.

Plan: ~/.claude/plans/and-rust-refine-crates-citum-schema-data-federated-mochi.md

- [x] Create siblings (`input`, `classes`, `ctors`, `serde_impl`, `accessors`) — `mod.rs` is now the public facade
- [x] Kept accessors.rs as single file; per-class ref_type helpers extracted inline
- [x] Move 3 inline test mods (discriminator_tests, normalize_tests, numbering_tests) to sibling files
- [x] Added `class_dispatch!` for pure shared-field dispatch (`id`, `accessed`, `language`, `field_languages`, `set_id`)
- [x] Applied original_embedded helper (saved ~230 lines)
- [x] Split ref_type via 4 per-class helpers; clippy allow dropped
- [x] Rewrote collect_contributors_by_role using peek; indexing_slicing allow dropped
- [x] fmt + clippy -D warnings + 1298 nextest tests all green
- [x] Schema diff zero (docs/schemas vs regen)
- [x] Commit

## Summary of Changes

Reduced `crates/citum-schema-data/src/reference/mod.rs` from **3377 lines to a facade** by extracting focused sibling files; applied dispatch and accessor cleanups inside the extracted code.

### Files split out of mod.rs
- `input.rs` — `InputReference`, `UnknownClassData`, and unknown-class field-language sentinel.
- `classes.rs` — `ReferenceClass`, `ClassExtension`, class-name helpers, and `class_dispatch!`.
- `ctors.rs` — 19 transitional PascalCase constructors including `Unknown`, plus `from_boxed_*` constructors via `boxed_reference_constructor!`; private `from_known<T>` (now `pub(super)`).
- `serde_impl.rs` — `Deserialize`/`Serialize`/`JsonSchema` impls + `FlatClassProxy`, `duplicate_field_error`, `reference_schema_branch`, `deserialize_reference_body`.
- `accessors.rs` — every `impl InputReference` method except construction + serde; includes `collect_contributors_by_role` and `normalize_genre_medium`.
- `discriminator_tests.rs`, `normalize_tests.rs`, `numbering_tests.rs` — the three inline `#[cfg(test)] mod ...` blocks.

### Refinement wins inside accessors.rs
- `class_dispatch!` now collapses the pure 18-arm dispatch matches for class metadata and shared-field access/mutation while leaving behavior-heavy accessors explicit.
- New private `InputReference::original_embedded() -> Option<&InputReference>` collapses the four 68-line `original_date`/`original_title`/`original_publisher_str`/`original_publisher_place` accessors into one-liners — net -230 lines in accessors.rs.
- `ref_type` split into per-class helpers (`monograph_ref_type`, `collection_component_ref_type`, `serial_component_ref_type`, `event_ref_type`, `audio_visual_ref_type`). The `#[allow(clippy::too_many_lines)]` pragma is gone, not suppressed.
- `collect_contributors_by_role` rewritten to peek the filter iterator twice instead of collecting-then-indexing. The `#[allow(clippy::indexing_slicing)]` pragma is gone, not suppressed.

### Verification
- `cargo test -p citum-schema-data reference::discriminator_tests --all-features` — 16/16 passing.
- `cargo fmt --check` clean.
- `cargo clippy --all-targets --all-features -- -D warnings` clean.
- `cargo nextest run` — 1298/1298 passing.
- `cargo run --bin citum --features schema -- schema --out-dir docs/schemas` — zero diff (public schema unchanged).

### Follow-up status
- No class-dispatch follow-up remains for this PR. Irregular accessors such as contributors, titles, numbering, container/original traversal, and `ref_type` intentionally stay explicit because their behavior differs by class.
