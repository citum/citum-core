---
# csl26-fvlx
title: Multilingual style presets + Pattern mode
status: completed
type: feature
priority: normal
created_at: 2026-06-02T21:53:36Z
updated_at: 2026-06-02T22:15:17Z
---

Add MultilingualPreset enum (apa/mla/chicago/ieee), a Pattern engine mode for 3-way CJK views (original+romanized+[translated]), and wire the preset into all embedded core styles.

## Todos

- [x] Engine: add Pattern mode + segment types to multilingual.rs
- [x] Engine: implement Pattern arm in resolve_multilingual_string / resolve_multilingual_name
- [x] Schema: add MultilingualPreset enum to presets.rs
- [x] Schema: add MultilingualConfigEntry + deserialize_multilingual_config in options/mod.rs
- [x] Styles: wire preset into all embedded core files
- [x] Optionally migrate apa-7th.yaml + modern-language-association.yaml to preset form
- [x] Tests: schema unit test (preset parses + resolves)
- [x] Tests: engine unit tests (Pattern mode: dedup, 3-way, wrapping)
- [x] Tests: end-to-end tests (Chicago + MLA Japanese reference)
- [x] Docs: update MULTILINGUAL.md spec
- [x] Gate: cargo fmt/clippy/nextest
- [x] Schema regen

## Summary of Changes

Engine, schema, preset, and style changes to extend multilingual romanization/translation to all embedded core styles.

Pattern mode added: ordered segment list with dedup; powers Chicago (3-way) and MLA (original+translation) presets.
MultilingualPreset enum added: apa/mla/chicago/ieee, each resolving to a concrete MultilingualConfig.
14 embedded styles gained multilingual preset annotations. APA migrated to preset form.
11 new tests added (schema unit + engine unit + end-to-end). Portfolio quality gate: 154 styles passing.
