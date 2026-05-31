---
# csl26-54jn
title: Display-mode-aware multilingual disambiguation keys
status: completed
type: feature
priority: normal
tags:
    - engine
    - multilingual
    - disambiguation
    - spec
created_at: 2026-05-31T20:36:49Z
updated_at: 2026-05-31T21:25:28Z
---

DISAMBIGUATION.md §4 requires the author collision key to reflect the same multilingual surface form the style renders (transliteration/translation/original) when display mode != primary. build_author_key in crates/citum-engine/src/processor/disambiguation.rs calls append_lowercased_families on raw family names without selecting the appropriate multilingual variant, so colliding transliterations are treated as distinct authors rather than a collision. Acceptance criterion 'Multilingual key generation respects display mode' in docs/specs/DISAMBIGUATION.md is open with no covering bean. Implement render_name_for_disambiguation to select the variant matching the active display mode before building the collision key. Spec: docs/specs/DISAMBIGUATION.md §4.

## Summary of Changes

Implemented `render_name_for_disambiguation` in `crates/citum-engine/src/processor/disambiguation.rs`.
This private helper wraps the existing `resolve_multilingual_name` function,
pulling `name_mode`, `preferred_transliteration`, and `preferred_script` from
`self.config.multilingual`. `build_reference_cache` now calls it instead of
`Contributor::to_names_vec()`, so the author collision key always reflects the
same surface form the style renders.

Added unit test `test_multilingual_key_generation_respects_display_mode`:
- Transliterated mode + shared romanisation → same author key (collision detected)
- Primary mode + distinct originals → different author keys (no spurious collision)

Updated `docs/specs/DISAMBIGUATION.md` §4 acceptance criterion and Changelog.
All 1459 tests pass.
