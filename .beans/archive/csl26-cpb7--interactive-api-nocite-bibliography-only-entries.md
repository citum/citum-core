---
# csl26-cpb7
title: 'Interactive API: nocite (bibliography-only entries)'
status: completed
type: feature
priority: normal
created_at: 2026-06-04T14:21:41Z
updated_at: 2026-06-09T11:50:48Z
---

The interactive server API (`format_document` + session methods) has no `nocite` parameter. Hosts cannot include a reference in the bibliography without creating an in-text citation.

Word-processor hosts (citum-office) need no-cite / bibliography-only entries: the user adds a work to the reference list (e.g. "further reading") without citing it in text. This is standard citeproc/Pandoc `nocite` behaviour.

## Scope

- Add an optional `nocite: [ref_id, ...]` field to `FormatDocumentRequest` and to the session API (e.g. a `set_nocite` method or a field on `put_references`).
- Included refs appear in the bibliography output but produce no `formatted_citations` entry.
- Interaction with bibliography sort/grouping: nocite entries sort alongside cited entries.
- Tests: a ref present only in `nocite` appears in `bibliography.entries` but not in `formatted_citations`.

## Origin

Required by citum-office P2 no-cite UX. Tracked gap in
citum-office docs/specs/01-cdip-protocol.md § Known Engine Gaps.

## Summary of Changes

- Added `register_nocite_ids` to `Processor` (engine); inserts IDs into `cited_ids`.
- Added `nocite: Vec<String>` field to `FormatDocumentRequest`; unknown IDs emit `nocite_missing_ref` warnings.
- Unified `format_bibliography` to restrict to `cited_ids` (cited + nocite) on both the batch and session paths — breaking fix for the longstanding inconsistency where the interactive API returned all loaded refs.
- Added `DocumentSession::set_nocite` method and `nocite` field; wired nocite registration into `render_citations`.
- Added `SetNociteParams` to the JSON-RPC server; wired `set_nocite` dispatch arm and handler; updated HTTP schema map.
- Regenerated `docs/schemas/server.json`.
- Added spec `docs/specs/NOCITE_BIBLIOGRAPHY_ONLY_ENTRIES.md`.
- 1536/1536 tests passing.
