---
# csl26-wq0y
title: format_document — extended integration tests
status: completed
type: task
priority: normal
created_at: 2026-05-09T00:21:22Z
updated_at: 2026-06-22T12:51:26Z
---

Expand integration test coverage for `citum_engine::format_document` beyond the
4 tests landed in csl26-isrv. Tier 1 minimum tests covered:

- Empty citations → empty result
- Missing ref → warning, no hard error
- Inline YAML style → renders
- StyleInput::Uri → UnresolvedInput error

## Add

- [x] Author-date mixed integral + non-integral citations in document order;
  assert order preserved and integral renders as narrative ("Smith (2020)").
- [x] Note-style document with repeat citations to the same work; assert ibid
  on 2nd/3rd where the style requires it.
- [x] `DocumentOptions.annotations` map present → annotations appear in
  bibliography output for matching ref IDs.
- [x] HTTP-mode dispatch test (axum route): the rpc.rs `format_document` arm
  works through a real HTTP request.

Use real styles under `styles/` (e.g. `apa-7th.yaml`,
`chicago-notes-18th.yaml`) loaded via `StyleInput::Path`. Build minimal
`tests/fixtures/` JSON for refs if reusable ones don't exist.

## Summary of Changes

Added four extended integration tests for `format_document` via real styles loaded with `StyleInput::Path`:

- **`format_document_author_date_mixed_citation_modes_order_preserved`** (document.rs) — APA 7th, integral renders outside parentheses, non-integral is parenthetical, document order preserved.
- **`format_document_note_style_repeat_citations_produce_ibid`** (document.rs) — Chicago Notes 18th, three consecutive same-item citations; 2nd and 3rd auto-detect `Position::Ibid` and render "Ibid.".
- **`format_document_annotations_appear_in_bibliography`** (document.rs) — APA 7th, `DocumentOptions.annotations` map appended to bibliography output.
- **`rpc_handler_format_document_returns_citations_and_bibliography`** (http.rs) — axum `rpc_handler` full HTTP stack test; asserts formatted_citations array and bibliography object in response.
