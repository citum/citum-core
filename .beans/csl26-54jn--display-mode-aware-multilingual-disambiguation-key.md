---
# csl26-54jn
title: Display-mode-aware multilingual disambiguation keys
status: todo
type: feature
priority: normal
tags:
    - engine
    - multilingual
    - disambiguation
    - spec
created_at: 2026-05-31T20:36:49Z
updated_at: 2026-05-31T20:36:49Z
---

DISAMBIGUATION.md §4 requires the author collision key to reflect the same multilingual surface form the style renders (transliteration/translation/original) when display mode != primary. build_author_key in crates/citum-engine/src/processor/disambiguation.rs calls append_lowercased_families on raw family names without selecting the appropriate multilingual variant, so colliding transliterations are treated as distinct authors rather than a collision. Acceptance criterion 'Multilingual key generation respects display mode' in docs/specs/DISAMBIGUATION.md is open with no covering bean. Implement render_name_for_disambiguation to select the variant matching the active display mode before building the collision key. Spec: docs/specs/DISAMBIGUATION.md §4.
