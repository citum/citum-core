---
# csl26-0ueb
title: Modularize types.rs into 4 submodules
status: todo
type: task
created_at: 2026-03-29T17:31:40Z
updated_at: 2026-03-29T17:31:40Z
---

Split crates/citum-schema-data/src/reference/types.rs (900 lines, exceeds 300-line guideline) into 4 modules as specified in ENUM_VOCABULARY_POLICY.md:\n- types/common.rs: NumOrStr, MultilingualString, Title, RefDate, ArchiveInfo, EprintInfo (~180 lines)\n- types/structural.rs: Monograph, MonographType, Collection, CollectionType, SerialComponent, SerialComponentType, Serial, SerialType, Parent, ParentReference (~280 lines)\n- types/legal.rs: LegalCase, Statute, Treaty, Hearing, Regulation, Brief (~200 lines)\n- types/specialized.rs: Classic, Patent, Dataset, Standard, Software (~200 lines)\n- types/mod.rs: re-exports only (~30 lines)\n\nPublic API unchanged. Requires full build verification (cargo fmt --check && cargo clippy && cargo nextest run).
