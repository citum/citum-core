---
# csl26-1i59
title: Add locale vocab layer for genre/medium display text
status: completed
type: feature
priority: normal
created_at: 2026-03-29T22:38:05Z
updated_at: 2026-03-30T11:44:09Z
blocked_by:
    - csl26-qqfa
---

Implement locale vocab lookup for canonical genre/medium keys. Create locale/en/vocab.yaml (and other locale files) mapping kebab-case keys to display strings (e.g. phd-thesis → 'PhD thesis'). Wire display-text resolution into the render layer so stored canonical values are localized at render time. Prerequisite: csl26-qqfa (normalization) must be complete first. See docs/reference/GENRE_AND_MEDIUM_VALUES.md §Localization for the planned shape.


## Summary of Changes

- Added `RawVocab` struct to `locale/raw.rs` with genre/medium HashMaps
- Added `VocabMap` struct to `locale/types.rs` with `is_empty()` helper
- Added `vocab` field + `lookup_genre()`/`lookup_medium()` methods to `Locale`
- Added `kebab_to_display()` fallback (capitalizes first word, replaces `-` with space)
- Wired `SimpleVariable::Genre` and `SimpleVariable::Medium` in `variable.rs` to call locale lookup
- Populated `locales/en-US.yaml` with 29 genre + 15 medium entries
- Updated 3 existing tests that expected raw keys to expect display text
- Added 4 new unit tests covering known key lookup and fallback behavior
