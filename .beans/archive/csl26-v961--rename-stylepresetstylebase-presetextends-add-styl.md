---
# csl26-v961
title: Rename StylePreset→StyleBase, preset→extends; add style taxonomy
status: completed
type: task
priority: normal
created_at: 2026-04-20T11:20:00Z
updated_at: 2026-04-20T11:29:25Z
---

Breaking rename: StylePreset→StyleBase, YAML key preset:→extends: (no alias; all files updated), add StyleKind enum to RegistryEntry (base/profile/journal/independent), new STYLE_TAXONOMY.md spec, update STYLE_PRESET_ARCHITECTURE.md to v2.0.

## Summary of Changes

- Renamed `StylePreset` → `StyleBase` and module `style_preset.rs` → `style_base.rs`
- Renamed YAML key `preset:` → `extends:` — breaking rename, no serde alias; all styles/*.yaml and test fixtures updated
- Renamed `preset_detector.rs` → `base_detector.rs` in citum-migrate
- Added `StyleKind` enum (base/profile/journal/independent) to `RegistryEntry`
- Updated 6 embedded YAML files; 16 registry entries annotated with `kind:`
- New `docs/specs/STYLE_TAXONOMY.md` (four-tier model, Active)
- Updated `docs/specs/STYLE_PRESET_ARCHITECTURE.md` to v2.0
- All 1070 tests green; commit 68ff0c18
