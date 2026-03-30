---
# csl26-0ueb
title: Modularize types.rs into 4 submodules
status: completed
type: task
priority: normal
created_at: 2026-03-29T17:31:40Z
updated_at: 2026-03-30T18:31:31Z
---

Split crates/citum-schema-data/src/reference/types.rs (900 lines, exceeds 300-line guideline) into 4 modules as specified in ENUM_VOCABULARY_POLICY.md:

- types/common.rs: NumOrStr, MultilingualString, Title, RefDate, ArchiveInfo, EprintInfo (~180 lines)
- types/structural.rs: Monograph, MonographType, Collection, CollectionType, SerialComponent, SerialComponentType, Serial, SerialType, Parent, ParentReference (~280 lines)
- types/legal.rs: LegalCase, Statute, Treaty, Hearing, Regulation, Brief (~200 lines)
- types/specialized.rs: Classic, Patent, Dataset, Standard, Software (~200 lines)
- types/mod.rs: re-exports only (~30 lines)

Public API unchanged. Requires full build verification (cargo fmt --check && cargo clippy && cargo nextest run).

## Summary of Changes

Split crates/citum-schema-data/src/reference/types.rs (928 lines) into 4 focused submodules under types/:

- common.rs (215 lines): NumOrStr, MultilingualString, MultilingualComplex, ArchiveInfo, EprintInfo, Title, StructuredTitle, Subtitle, RefDate, and the 3 type aliases
- structural.rs (295 lines): Monograph, MonographType, Collection, CollectionType, CollectionComponent, MonographComponentType, SerialComponent, SerialComponentType, Serial, SerialType, Parent<T>, ParentReference
- legal.rs (235 lines): LegalCase, Statute, Treaty, Hearing, Regulation, Brief
- specialized.rs (215 lines): Classic, Patent, Dataset, Standard, Software
- mod.rs (17 lines): submodule declarations + pub use * re-exports

Public API unchanged. All 894 tests pass.
