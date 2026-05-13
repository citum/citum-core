---
# csl26-zv1y
title: Add document-level abbreviation-map
status: completed
type: feature
priority: normal
created_at: 2026-05-13T11:13:26Z
updated_at: 2026-05-13T11:23:42Z
---

Add a document-level `abbreviation-map` feature as a clean break from CSL/Pandoc's `citation-abbreviations` JSON format.

## Checklist

- [x] Define `AbbreviationMap` newtype in `crates/citum-engine/src/api/types.rs`
- [x] Add `abbreviation_map: Option<AbbreviationMap>` to `DocumentOptions`
- [x] Add `abbreviation_map: Option<&'a AbbreviationMap>` to `RenderOptions`
- [x] Add `abbreviation_map` field to `Processor` struct and wire from `DocumentOptions`
- [x] Propagate into `RenderOptions` construction in `processor/rendering/mod.rs`
- [x] Add `apply_abbreviation` helper and call from title/variable value extraction
- [x] Add integration test with BDD naming and `assert_eq!` full-string assertions
- [x] Create `docs/specs/ABBREVIATION_MAP.md` (status: Active)
- [x] Run pre-commit checks and commit on feature branch

## Summary of Changes

Added `AbbreviationMap` newtype wrapping `HashMap<String, String>` to `DocumentOptions`, threaded it through `Processor` and `RenderOptions`, and applied `apply_abbreviation()` post-extraction in `values/title.rs` and `values/variable.rs`. 1257 tests pass. Spec at `docs/specs/ABBREVIATION_MAP.md`.
