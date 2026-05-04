---
# csl26-isrv
title: Interactive server API — document-batch and session modes
status: todo
type: feature
priority: normal
created_at: 2026-05-04T00:00:00Z
updated_at: 2026-05-04T12:00:04Z
---

Implement the interactive server API as specified in
`docs/specs/SERVER_INTERACTIVE_API.md`.

Two tiers:

1. **`format_document`** (stateless, document-shaped) — primary delivery.
   A single call takes the full ordered citation list and reference set,
   returns all formatted citations plus bibliography. Enables correct
   note-position inference, ibid, and disambiguation — impossible with the
   current per-citation `render_citation` method.

2. **Session lifecycle** (`open_session` … `close_session`) — advanced,
   behind a `session` feature flag (implies `http`). Amortizes style
   parsing and deserialization for word processors with large bibliographies.

Core types (`FormatDocumentRequest`, `DocumentSession`, etc.) go in
`citum-engine`. `citum-bindings` and `citum-server` remain thin adapters.

Existing `render_citation` / `render_bibliography` / `validate_style`
methods are preserved unchanged.

## Acceptance Criteria

See `docs/specs/SERVER_INTERACTIVE_API.md` acceptance criteria section.
