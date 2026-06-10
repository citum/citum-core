---
# csl26-ymal
title: Add romanized-original-translated preset + fix CNE prose
status: completed
type: feature
priority: normal
created_at: 2026-06-10T11:21:35Z
updated_at: 2026-06-10T11:30:49Z
---

Two-part change: (1) add `RomanizedScriptTranslated` preset to `MultilingualPreset` in citum-core
(romanized + original-script + [translated] titles, romanized + original-script names, Latn preferred
script, Han/Hangul `use-native-ordering: true` baked in), collapsing the embedded style to a one-liner;
(2) fix misleading CNE framing in the citum-org multilingual news post.

## Summary of Changes

- Added `RomanizedScriptTranslated` variant to `MultilingualPreset` in
  `crates/citum-schema-style/src/presets.rs`; preset resolves to Pattern title mode
  (romanized + original-script + [translated]) and Pattern name mode (romanized + original-script),
  with `preferred_script: Latn` and Han/Hangul `use_native_ordering: true` baked in.
- Added `test_multilingual_preset_romanized_script_translated_parses_and_resolves` in
  `crates/citum-schema-style/src/options/mod.rs` (278/278 pass, full suite 1542/1542).
- Updated `docs/specs/MULTILINGUAL.md` §2.1 preset table and note.
- Collapsed `crates/citum-schema-style/embedded/styles/chicago-notes-18th-script.yaml` explicit
  block to one-liner `multilingual: romanized-script-translated` (render output identical).
- Regenerated `docs/schemas/style.json` (new `romanized-script-translated` enum const only).
- citum-org prose fix (de-CNE the news post) shipped in PR B / citum-org #21 (merged).
