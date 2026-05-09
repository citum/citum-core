---
# csl26-isrv
title: Interactive server API — document-batch and session modes
status: completed
type: feature
priority: normal
created_at: 2026-05-04T00:00:00Z
updated_at: 2026-05-09T00:21:52Z
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


## Summary of Changes

Tier 1 (`format_document`) landed; Tier 2 (session API) deferred to follow-ups.

### Engine — `crates/citum-engine/src/api/`
- `StyleInput` enum: `Id`, `Uri`, `Path`, `Yaml` variants. `resolve_local()`
  handles `Path` and `Yaml`; `Id`/`Uri` return `UnresolvedInput` for adapters
  with a resolver chain to handle.
- Request/result types: `FormatDocumentRequest`, `FormatDocumentResult`,
  `CitationOccurrence`, `CitationOccurrenceItem`, `FormattedCitation`,
  `FormattedBibliography`, `BibliographyEntry`, `EntryMetadata`, `Warning`,
  `WarningLevel`, `OutputFormatKind`, `DocumentOptions`.
- `From<CitationOccurrence> for Citation` / `From<CitationOccurrenceItem>
  for CitationItem` so the wire shape is a thin layer over the existing
  schema.
- `format_document(request)` — convenience entry point; resolves
  `Path`/`Yaml` locally.
- `format_document_with_style(style, request)` — adapter entry point taking
  an already-resolved `Style`.
- Missing-ref handling: items whose IDs are absent from the bibliography are
  dropped with a `Warning { code: "missing_ref" }`; citation count and order
  are preserved (placeholder for fully-missing citations).
- Output format dispatch (`Plain`, `Html`, `Djot`, `Latex`, `Typst`).
- 4 integration tests pass (empty citations, missing-ref warning, inline
  YAML, URI unresolved).

### Server — `crates/citum-server/src/rpc.rs`
- New `"format_document"` arm in `dispatch`. Pre-resolves `Id`/`Uri`/`Path`
  styles via the existing `load_style` chain; passes `Yaml` directly to the
  engine.

### Bindings — `crates/citum-bindings/src/lib.rs`
- New `format_document(request_json: &str) -> Result<String, String>`
  exposed via `wasm_bindgen`. JSON-in / JSON-out. `Id`/`Uri` inputs error
  in WASM (no resolver chain); callers pass `StyleInput::Yaml`.

### DocumentOptions Coverage
Wired (engine-confirmed):
- `output_format`, `locale`, `annotations`, `annotation_format`,
  `show_semantics`, `inject_ast_indices`.

Deferred (Pandoc-equivalent TBDs, csl26-ukpz):
- `suppress_bibliography`, `link_citations`, `link_bibliography`,
  `notes_after_punctuation`.

`bibliography_groups`, `sort_partitioning`, `integral_names` are accepted
in the request shape but processor-level overrides for these still need
wiring through (existing engine setters live but aren't applied here).

### Commits

- `65b2d71b feat(engine): add format_document batch API`
- `8593a487 feat(server): wire format_document arm + wasm`

### Follow-ups

- **csl26-3yk1** — Tier 2 session layer (blocked-by csl26-isrv)
- **csl26-ukpz** — Pandoc-equivalent `DocumentOptions` fields
- **csl26-wq0y** — Extended integration tests (ibid, integral mode,
  annotations, HTTP dispatch)
