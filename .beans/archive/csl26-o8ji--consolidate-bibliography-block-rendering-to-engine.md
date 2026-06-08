---
# csl26-o8ji
title: Consolidate bibliography block rendering to engine core
status: completed
type: task
priority: high
created_at: 2026-06-08T12:57:00Z
updated_at: 2026-06-08T13:39:54Z
---

Two separate implementations of sectional bibliography rendering currently exist in the engine:

1. **Document pipeline path** (`processor/document/pipeline.rs`): parses `:::bibliography{...}` markers from Djot/Markdown content and replaces them inline. Entry point: `process_document_with_bibliography_blocks`.

2. **API path** (`api/document.rs`): `format_bibliography_blocks` accepts a `Vec<BibliographyBlockRequest>` and returns `Vec<FormattedBibliographyBlock>`. Introduced in feat(server): add bibliography blocks (PR #884).

Both ultimately call `render_document_bibliography_block` on the `Processor`, but the surrounding orchestration is duplicated and the two entry points are not composable.

## Goal

Move the sectional bibliography concept fully into the engine core as a single, well-defined primitive. The CLI and server should be thin wrappers over that primitive — not independent implementations.

## Work items

- [x] Audit the two paths and define the canonical engine-level API for sectional bibliography rendering
- [x] Consolidate into a single code path in the engine (document pipeline and API path should share the same rendering logic)
- [x] Update CLI (`render doc`) to expose sectional bibliographies via the engine primitive
- [x] Update server to confirm it uses the same primitive (should be a no-op or minor wiring change)
- [x] Remove any duplicated orchestration code

## Context

Discovered during review of PR #884. The server-side `bibliography_blocks` field was added without wiring the CLI to the same mechanism, revealing the DRY violation. See PR #884 discussion.

## Related

- [[csl26-k9y0]] fixed the assigned-dedup problem in the fenced-div pipeline path — the same mechanism this PR added to the API path. Consolidation should carry that fix through to the unified primitive.

## Summary of Changes

- Added `render_document_bibliography_blocks` primitive to `grouping.rs` — owns the ordered-block loop and single `assigned` dedup set; both wrappers delegate to it.
- Extended `RenderedBibliographyGroup` with `entries: Vec<ProcEntry>` so the API path can expose per-entry metadata without a second render pass.
- Refactored pipeline `replace_document_bibliography_blocks` and API `format_bibliography_blocks` to be thin wrappers over the primitive, removing the two duplicated loops.
- Added `BibliographyBlockRequest` / `FormattedBibliographyBlock` types to `api/types.rs` with JsonSchema derives; wired to `FormatDocumentRequest` / `FormatDocumentResult`.
- Added `process_document_with_caller_blocks` public method to the pipeline, enabling callers to supply `BibliographyGroup` slices directly instead of in-document fenced divs.
- Added `--bibliography-blocks <json>` flag to `citum render doc` using the same type the server uses.
- Updated server `FormatDocumentParams.bibliography_blocks` from `serde_json::Value` to `Vec<BibliographyBlockRequest>` for correct schema generation.
- Regenerated `docs/schemas/server.json`; updated `docs/specs/SERVER_INTERACTIVE_API.md`.
