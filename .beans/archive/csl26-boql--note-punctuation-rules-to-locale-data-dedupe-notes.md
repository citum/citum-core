---
# csl26-boql
title: Note punctuation rules to locale data; dedupe notes.rs
status: completed
type: task
priority: normal
tags:
    - notes
    - localization
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-06T00:31:13Z
parent: csl26-8m2p
---

locale_note_rule hardcodes en-US/fr punctuation-placement defaults in engine code rather than locale files (overridable via options.notes but defaults belong in data), and process_note_document/_html plus prepare_note_citations/_html are ~70-line near-verbatim duplicates with silent render-error fallbacks. Move defaults to locale data and extract the shared note pipeline. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 21.

## Summary of Changes

- Added `note-punctuation` / `note-number` / `note-marker-order` fields to
  `GrammarOptions` (citum-schema-style), reusing the existing style-override
  enums (`NoteQuotePlacement`, `NoteNumberPlacement`, `NoteMarkerOrder`).
  Gave each enum a `#[default]` variant matching the engine's former
  fallback branch. Set explicit values in `en-US.yaml` and `fr-FR.yaml`
  matching prior hardcoded behavior; other embedded locales rely on the new
  struct defaults, which reproduce prior fallback behavior unchanged.
- Rewrote `locale_note_rule` in `notes.rs` to read
  `self.locale.grammar_options` directly instead of string-matching the
  locale ID — simpler and more correct, since `self.locale` is already the
  resolved `Locale` for the style's `default_locale`.
- Removed the now-unused `language_tag` helper from `note_support.rs`.
- Collapsed `process_note_document`/`process_note_document_html` and
  `prepare_note_citations`/`prepare_note_citations_html` (~140 duplicated
  lines) into one generic implementation per stage, using a small
  `NoteOutputSink` trait to carry the one real difference (HTML output
  routes through `HtmlPlaceholderRegistry::push_inline`; plain text does
  not). Public call sites in `pipeline.rs` are unchanged.
- Verified: `cargo nextest run` (1798 tests), `cargo clippy -D warnings`,
  `cargo fmt --check` all pass. `workflow-test.sh` on
  `chicago-notes-bibliography-17th-edition` shows identical results
  before/after (20/20 citations, 42/47 bibliography — the 5 pre-existing
  bibliography mismatches are unrelated to notes and confirmed present on
  `main` too). Regenerated `docs/schemas/locale.json` via `just schema-gen`.
- PR: (added after opening)
