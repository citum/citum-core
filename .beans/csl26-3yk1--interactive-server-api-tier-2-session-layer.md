---
# csl26-3yk1
title: Interactive server API — Tier 2 session layer
status: todo
type: feature
priority: normal
created_at: 2026-05-09T00:21:07Z
updated_at: 2026-05-09T00:21:07Z
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
- Server-side session store: `Arc<Mutex<HashMap<String, DocumentSession>>>`.
- New `session` feature flag (implies `http`) in `citum-server/Cargo.toml`.
- Session TTL eviction (default 30 min) + `session_expired` error.
- WASM exposure as a wasm-bindgen class in `citum-bindings`.
- Acceptance criteria from the spec covering each method.

Reuses Tier 1 types unchanged: `CitationOccurrence`, `CitationOccurrenceItem`,
`FormattedCitation`, `FormattedBibliography`, `BibliographyEntry`,
`EntryMetadata`, `Warning`, `DocumentOptions`.

## Acceptance Criteria

See `docs/specs/SERVER_INTERACTIVE_API.md` Tier 2 section.
