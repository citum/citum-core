---
# csl26-3yk1
title: Interactive server API — Tier 2 session layer
status: completed
type: feature
priority: normal
created_at: 2026-05-09T00:21:07Z
updated_at: 2026-06-04T12:24:39Z
blocked_by:
    - csl26-isrv
---

Implement the Tier 2 session API specified in `docs/specs/SERVER_INTERACTIVE_API.md`,
on top of the Tier 1 `format_document` work landed in csl26-isrv.

## Scope

- `DocumentSession` struct + lifecycle methods in `citum-engine`:
  `open_session`, `put_references`, `insert_citation`, `update_citation`,
  `delete_citation`, `insert_citations_batch`, `preview_citation`,
  `get_citations`, `get_bibliography`, `close_session`.
- `CitationInsertPosition` neighbour-ID position type.
- Session mutation envelope: `version`, `affected_citations`,
  `renumbering_occurred`, `warnings`.
- Transport-neutral server support:
  - stdio mode uses one implicit process-local session and returns fixed
    `session_id: "default"`;
  - HTTP mode uses a multi-session store with generated session IDs.
- New default-on `session` feature flag in `citum-server/Cargo.toml`; it must not
  imply `http`, `async`, or `tokio`.
- HTTP session TTL eviction (default 30 min) + `session_expired` error.
- WASM exposure as a wasm-bindgen class in `citum-bindings`.
- Acceptance criteria from the spec covering each method.

Reuses Tier 1 types unchanged: `CitationOccurrence`, `CitationOccurrenceItem`,
`FormattedCitation`, `FormattedBibliography`, `BibliographyEntry`,
`EntryMetadata`, `Warning`, `DocumentOptions`.

## Transport Scope

This bean includes stdio RPC support.  The LibreOffice adapter (citum-office) uses
a stdio subprocess per document, so the subprocess is the session: one process =
one document = one implicit session.  HTTP remains the multi-session deployment
mode with TTL eviction.

Method names, parameter shapes, and return types are identical in stdio and HTTP.
Only storage differs.  This keeps the adapter self-contained without a daemon
lifecycle or port allocation.

`affected_citations` must be complete: after every committed mutation it includes
every current citation whose rendered text or referenced ID set changed.
`renumbering_occurred` is diff-derived and true only when existing note numbers or
numeric/label citation output actually shifted.

## Acceptance Criteria

See `docs/specs/SERVER_INTERACTIVE_API.md` Tier 2 section.
