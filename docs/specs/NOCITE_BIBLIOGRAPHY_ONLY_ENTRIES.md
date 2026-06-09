# Nocite: Bibliography-Only Entries

Status: Active

## Overview

Word-processor hosts and document authors sometimes need to include a reference in the
bibliography without citing it in text — a "further reading" or "background" entry.
This is the standard `nocite` semantics as defined by citeproc and Pandoc.

This spec describes how Citum exposes nocite across its three API surfaces: the
batch (`format_document`) path, the interactive session (`DocumentSession`) path, and
the JSON-RPC server.

## Motivation

Prior to this feature the interactive API bibliography rendered *all* loaded references,
whereas the document-string path (via `process_document_*`) correctly restricted the
bibliography to cited refs (`restrict_to_cited = true`). This inconsistency was a
historical accident from when grouping was added later. Nocite is the mechanism that
bridges the two: a "flat" bibliography is conceptually one group with no cited filter,
so both paths now restrict to the *document reference set* — cited IDs plus nocite IDs.

## Behaviour

1. The caller supplies a list of reference IDs to register as nocite.
2. Each ID is validated against the loaded bibliography. Unknown IDs produce a
   `nocite_missing_ref` warning and are otherwise ignored.
3. Valid nocite IDs are inserted into the processor's `cited_ids` set — the same set
   that gates bibliography inclusion.
4. Bibliography rendering (both `content` and `entries`) includes nocite refs alongside
   cited refs, sorted and grouped by the active style.
5. Nocite refs produce **no** `formatted_citations` entry.

## API Surface

### `format_document` (batch)

```json
{
  "style": { "kind": "path", "value": "styles/apa-7th.yaml" },
  "refs": { ... },
  "citations": [ ... ],
  "nocite": ["background-ref-1", "background-ref-2"]
}
```

The `nocite` field is optional (defaults to empty). Unknown IDs produce a warning in
the `warnings` array of the response.

### `DocumentSession` (interactive)

```rust
session.set_nocite(vec!["background-ref-1".to_string()])?;
```

`set_nocite` replaces the session's nocite list atomically and re-renders, returning a
`SessionMutationResult` with the updated bibliography and any affected citations.

### JSON-RPC (`set_nocite`)

```json
{
  "method": "set_nocite",
  "params": {
    "session_id": "s-0000000000000001",
    "nocite": ["background-ref-1"]
  }
}
```

The response is the standard `SessionMutationResult` envelope
(`version`, `affected_citations`, `bibliography`, `warnings`).

## Warning Code

| Code | Level | Meaning |
|------|-------|---------|
| `nocite_missing_ref` | `warning` | The supplied nocite ID is not present in the loaded bibliography. |

## See Also

- [BIBLIOGRAPHY_RENDERING_PIPELINE.md](BIBLIOGRAPHY_RENDERING_PIPELINE.md) — consolidated
  pipeline diagram and routing rules; explains how `cited_ids` (cited + nocite IDs) gates
  bibliography inclusion across all three document-context API surfaces.

## Out of Scope

- Numeric-label assignment for nocite-only entries (no in-text number, but a bibliography
  number). Tracked as a follow-up.
- `nocite` in the document-string (`process_document`) Markdown/front-matter path.
- Pandoc `@*` "cite everything" wildcard expansion.
