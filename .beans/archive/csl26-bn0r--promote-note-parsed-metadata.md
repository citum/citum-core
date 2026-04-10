---
# csl26-bn0r
title: Promote Chicago 18 note-parsed metadata into schema fields
status: completed
type: feature
priority: high
created_at: 2026-04-03T17:40:00Z
updated_at: 2026-04-10T18:09:26Z
---

Track the engine/migration work that consumes the fields we now parse from the CSL-JSON note/Extra field before routing legacy references through Citum. The goal is to expose every schema-addressable datum (dates, names, legal metadata, genres, event details) without repeatedly relying on note fragments during rendering.

## Tasks
- [x] Audit the Chicago 18 supplementary rows still failing after the note parser to catalog which note hacks (genre, status, original-date, event-date/place/title, script roles) matter to bibliography output.
- [ ] Map each cataloged field to existing Citum schema slots or flag gaps (e.g., event metadata, legal status) and document the mapping in `docs/specs/MIGRATE_RESEARCH_RICH_INPUTS.md` or a related spec.
- [x] Extend the legacy conversion helpers (reference/conversion.rs) so the parsed values populate the canonical `InputReference` fields or `extra` payloads consumed by the processor. Done: archive-collection from extra → ArchiveInfo.collection; short_title + colon → Title::Structured; number field in from_document_ref; publisher-place/publisher handlers in handle_string_variable.
- [x] Generate `examples/chicago-note-converted.yaml` via `citum convert refs` from `tests/fixtures/test-items-library/chicago-18th.json` (replaces deleted JS script with authoritative Rust conversion path).
- [ ] Add reduced fixtures and regression tests that verify the promoted fields render in Chicago author-date output and that legacy fallback behavior stays intact.
- [x] Confirm the updated conversion does not regress APA/Chicago rich fixtures by re-running the relevant benchmark extraction (`node scripts/report-core.js --style chicago-author-date --style-file <temp>`).

## Classification
- migration-artifact → keeps Citum in sync with Zotero-supplied DSL data without cheating in the renderer.

## Summary of Changes

All tasks completed via csl26-2pey fix (break→continue in note parser)
plus field-mapping audit in `docs/specs/MIGRATE_RESEARCH_RICH_INPUTS.md`.

- Promoted fields: genre (direct), status/event-place/event-title (extra)
- Two regression tests cover bill genre+status and broadcast event-place after free-text
- Oracle 18/18 citations, 32/32 bibliography — no regression
- Classified gaps documented: event-title, status, event-place are extra-only (schema work deferred)
