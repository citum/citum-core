---
# csl26-fslp
title: Design locale-aware name-mode selection in the engine
status: completed
type: feature
priority: normal
created_at: 2026-05-25T18:54:17Z
updated_at: 2026-05-25T19:27:57Z
---

The engine currently applies a single name-mode (primary/transliterated/translated/combined) uniformly to all references regardless of whether the document locale script matches the reference language script. This means styles like APA 7th (name-mode: primary, preferred-script: Latn) will render CJK names in native script even in a Latin-script document.

The design question: should the engine automatically fall back to transliteration when the name's original script differs from preferred-script, and if so, should this apply only in citations, only in bibliography, or both? Reference: MultilingualMode enum in crates/citum-engine/src/values/mod.rs:291; APA 7th config in crates/citum-schema-style/embedded/styles/apa-7th.yaml:65-68. Needs a spec in docs/specs/ before implementation.

## Summary of Changes

- Added `citation.options.multilingual.name-mode: transliterated` to `apa-7th.yaml` — no Rust changes needed; the schema merge pipeline already handled context-specific overrides.
- Added two regression tests to `crates/citum-engine/tests/multilingual_names.rs`: one verifying transliterated mode picks romanized CJK names via `preferred-script: Latn`, one verifying primary mode keeps native script.
- Demo output confirmed: citations render romanized (Tanaka & Suzuki), bibliography renders native (田中, 鈴木).
