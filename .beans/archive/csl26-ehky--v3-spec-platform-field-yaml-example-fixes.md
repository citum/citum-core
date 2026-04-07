---
# csl26-ehky
title: 'v3 spec: platform field, YAML example fixes'
status: completed
type: task
priority: normal
created_at: 2026-04-05T14:29:30Z
updated_at: 2026-04-05T14:32:55Z
---

Address inline comments in examples/chicago-note-converted.yaml from the type/refactor-3 branch. Includes: serde fix for StructuredTitle.full null serialization, platform field addition to AudioVisualWork spec, and data fixes (archive redundancy, H.R. comment, Toby Lee reclassify to conference paper).

## Summary of Changes

- Added `#[serde(skip_serializing_if = "Option::is_none")]` to `StructuredTitle.full` in `common.rs` — fixes null serialization bug
- Added `platform` field to `AudioVisualWork` spec (§3.2) with medium/platform distinction note
- Added H.R. legislative bills to spec §4 scope boundary table
- YAML fixes in `examples/chicago-note-converted.yaml`:
  - Removed two `full: null` lines (Prairie State, thesis)
  - `medium: YouTube` → `platform: YouTube` (with deferred-until-impl note)
  - Wiener Philharmoniker: updated TODO comment for audio-visual reclassify
  - Removed `archive-location: O'Laughlin Papers` (redundant with archive-info)
  - Fixed `archive-info` to use `collection` not non-existent `location` key
  - Removed `archive: ProQuest` (redundant)
  - H.R. comment updated to reference type system TODO
  - Toby Lee: reclassified from chapter/empty-container to monograph, genre: conference-paper
