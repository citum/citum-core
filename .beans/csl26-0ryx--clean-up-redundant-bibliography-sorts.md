---
# csl26-0ryx
title: Clean up redundant bibliography sorts
status: completed
type: task
created_at: 2026-02-28T14:50:41Z
updated_at: 2026-02-28T15:05:00Z
---

## Goal

Finish the explicit default sorting rollout by removing redundant `bibliography.sort` declarations from styles that now inherit the same family default, and stop `citum-migrate` from emitting those redundant sorts.

## Todo

- [x] classify removable style-level `bibliography.sort` entries
- [x] remove redundant `bibliography.sort` entries from matching styles
- [x] update migration output to omit redundant bibliography sort defaults
- [x] add regression coverage for migrate + style cleanup assumptions
- [x] run Rust and docs/beans verification
- [x] add summary and mark completed

## Summary

- Classified the current style corpus and confirmed only `styles/alpha.yaml`, `styles/mhra-notes.yaml`, and `styles/mhra-shortened-notes-publisher-place.yaml` had redundant `bibliography.sort` declarations under the new processing-family defaults.
- Updated `citum-migrate` to suppress migrated bibliography sorts only when the extracted legacy sort exactly matches the processing-family default, while preserving numeric and note-family exception sorts.
- Added migrate regression tests covering author-date and note default suppression plus numeric and note-exception preservation.
- Ran `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo nextest run`.
- Ran `./scripts/check-docs-beans-hygiene.sh`; it still fails on pre-existing unrelated bean `.beans/csl26-5axq--evaluate-icu-library-for-datetime-internationaliza.md` using status `scrapped`.
