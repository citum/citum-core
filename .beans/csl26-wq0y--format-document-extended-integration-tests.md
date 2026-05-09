---
# csl26-wq0y
title: format_document — extended integration tests
status: todo
type: task
priority: normal
created_at: 2026-05-09T00:21:22Z
updated_at: 2026-05-09T00:21:22Z
---

Expand integration test coverage for `citum_engine::format_document` beyond the
4 tests landed in csl26-isrv. Tier 1 minimum tests covered:

- Empty citations → empty result
- Missing ref → warning, no hard error
- Inline YAML style → renders
- StyleInput::Uri → UnresolvedInput error

## Add

- Author-date mixed integral + non-integral citations in document order;
  assert order preserved and integral renders as narrative ("Smith (2020)").
- Note-style document with repeat citations to the same work; assert ibid
  on 2nd/3rd where the style requires it.
- `DocumentOptions.annotations` map present → annotations appear in
  bibliography output for matching ref IDs.
- HTTP-mode dispatch test (axum route): the rpc.rs `format_document` arm
  works through a real HTTP request.

Use real styles under `styles/` (e.g. `apa-7th.yaml`,
`chicago-notes-18th.yaml`) loaded via `StyleInput::Path`. Build minimal
`tests/fixtures/` JSON for refs if reusable ones don't exist.
