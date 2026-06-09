---
# csl26-f9ri
title: 'feat(engine): nocite @* wildcard and allrefs flag'
status: todo
type: feature
priority: normal
created_at: 2026-06-09T12:08:17Z
updated_at: 2026-06-09T12:08:58Z
---

## Background

The `feat(engine): nocite bibliography-only entries` PR established that the bibliography
is always restricted to `cited_ids` (cited + registered nocite IDs). Two common use cases
are not yet covered:

1. **`@*` wildcard** — Pandoc syntax meaning "cite every loaded reference"; equivalent to
   calling `nocite` with all ref IDs. Useful for reference lists with no in-text citations.
2. **`allrefs` flag** — an explicit opt-in to include all loaded refs, bypassing the
   `cited_ids` restriction. Mirrors the existing `render_bibliography_with_format` behavior
   (the standalone / FFI path) but from within a document context.

## Proposed API shape

### `FormatDocumentRequest` (batch)
```json
{
  "nocite": ["@*"]
}
```
If `"@*"` is present in `nocite`, expand it to all ref IDs before registration (drop the
literal `"@*"` from the ID set). This matches Pandoc semantics exactly and requires no new
field — just special-case handling inside the existing nocite registration block in
`format_document_with_style` (`crates/citum-engine/src/api/document.rs`, just before
`processor.register_nocite_ids(…)`).

### `DocumentSession` (interactive)
Same: `session.set_nocite(vec!["@*".to_string()])` expands to all loaded ref IDs.
Handle in `render_citations` in `crates/citum-engine/src/api/session.rs` at the
nocite registration block (same pattern as batch path).

### `FormatDocumentRequest` — `allrefs` flag (optional, lower priority)
```json
{ "allrefs": true }
```
Shorthand for `nocite: ["@*"]`. Implement as a pre-processing step that populates
`nocite` from `bibliography.keys()` when `allrefs` is true, then follows the normal
nocite path. May be deferred if `@*` alone is sufficient.

## Key files

- `crates/citum-engine/src/api/document.rs` — expand `@*` in nocite registration block
- `crates/citum-engine/src/api/session.rs` — same expansion in `render_citations`
- `crates/citum-engine/src/api/mod.rs` (or `document.rs`) — add `allrefs: bool` field
  to `FormatDocumentRequest` if `allrefs` flag is implemented
- `crates/citum-server/src/rpc.rs` — add `allrefs` to `FormatDocumentParams` mirror if implemented
- `docs/specs/NOCITE_BIBLIOGRAPHY_ONLY_ENTRIES.md` — update to document `@*` and `allrefs`

## Tests needed

- `nocite: ["@*"]` with 3 loaded refs → all 3 appear in bibliography
- `nocite: ["@*", "extra"]` — `@*` expands and `"extra"` is treated as a normal ID
  (already covered by expansion; no special handling needed)
- Session: `set_nocite(["@*"])` → all session refs appear in bibliography

## Spec cross-ref

`docs/specs/NOCITE_BIBLIOGRAPHY_ONLY_ENTRIES.md` § Out of Scope notes this explicitly.
