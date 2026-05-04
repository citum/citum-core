# Interactive Server API Specification

**Status:** Draft
**Date:** 2026-05-04
**Related:** `.beans/csl26-isrv--server-interactive-api.md`, `.beans/csl26-stat--consider-stateful-mode-for-citum-server.md`

## Purpose

Define a document-level batch API and an optional session API for `citum-server` and `citum-bindings`, enabling correct whole-document citation rendering and efficient interactive editing use cases. The core types and logic live in `citum-engine` so both adapters share them without duplication.

The current server API renders citations one at a time (`render_citation`) without knowledge of the full document context. This precludes correct note-position inference, ibid detection, and back-reference disambiguation — all of which require seeing the ordered citation sequence. This spec resolves that gap.

## Scope

**In scope:**
- Tier 1: `format_document` — stateless, document-shaped batch endpoint (primary)
- Tier 2: Session lifecycle API — stateful, `session` feature flag, HTTP only
- Placement of shared types and logic in `citum-engine`
- WASM exposure of both tiers via `citum-bindings`
- Server exposure via `citum-server` RPC dispatch

**Out of scope:**
- Streaming/incremental progress events (Tier 3) — deferred, requires SSE or chunked HTTP
- Replacement or removal of existing `render_citation`, `render_bibliography`, `validate_style` methods
- Changes to `Processor` internals (thread-safety refactor is a separate concern)
- `validate_document` validation-only endpoint — useful future work, not in this spec

## Design

### Architecture

Core types and the `format_document` function belong in `citum-engine`. Both `citum-bindings` and `citum-server` are thin transport adapters; adding logic to only one would require duplicating it in the other.

```
citum-engine
  ├── StyleInput                                     (union: path | yaml | embedded [+ url, csl26-r8d2])
  ├── DocumentOptions                                (doc-level config: Pandoc equivalents + Citum extensions)
  ├── CitationOccurrence, CitationOccurrenceItem     (input types, mirror Citation/CitationItem)
  ├── FormattedCitation, FormattedBibliography       (output types)
  ├── Warning                                        (structured diagnostic)
  ├── FormatDocumentRequest / FormatDocumentResult   (batch API envelope)
  ├── format_document(request) → Result              (pure function)
  ├── CitationInsertPosition                         (neighbour-ID position context for sessions)
  └── DocumentSession                               (stateful facade over Processor)
        ├── new(style_input, options) → Self
        ├── put_references(&mut self, refs)
        ├── insert_citation(citation, position) → SessionResult
        ├── insert_citations_batch(citations) → SessionResult
        ├── update_citation(id, citation, position) → SessionResult
        ├── delete_citation(id) → SessionResult
        ├── preview_citation(items, position) → String   (no state mutation)
        ├── get_citations(&self) → Vec<FormattedCitation>
        └── get_bibliography(&self) → FormattedBibliography

citum-bindings (WASM adapter)
  ├── format_document(request_json: &str) → Result<String, String>
  └── DocumentSession exposed as a wasm-bindgen class
        (all methods take/return JSON strings; JS holds the object reference)

citum-server (RPC adapter)
  ├── format_document dispatch arm (works in stdio + HTTP)
  └── session lifecycle methods (HTTP + `session` feature only)
        (session store: Arc<Mutex<HashMap<String, DocumentSession>>>)
```

### `StyleInput` — cross-transport style reference

All methods that accept a style use a `StyleInput` union to avoid adapter-specific leakage. The current variants cover all transports:

```json
// Path — server resolves from filesystem or embedded registry
{"kind": "path", "value": "styles/embedded/apa-7th.yaml"}

// Embedded ID — resolved from the embedded style registry by name
{"kind": "embedded", "value": "apa-7th"}

// Inline YAML — WASM and HTTP callers may supply the style body directly
{"kind": "yaml", "value": "---\ninfo:\n  title: ..."}
```

Adapter mapping: `path` and `embedded` are valid in all transports; `yaml` is valid everywhere but most useful in WASM where filesystem access is unavailable.

> **Note (csl26-r8d2):** The resolver architecture is under active reconsideration. That bean proposes a `StyleResolver` trait with a `ChainResolver` (`EmbeddedResolver` → `FileResolver` → `StoreResolver` → future `HttpResolver`) and URI-based `extends` values. When that work lands, `StyleInput` will grow a `url` variant (`{"kind": "url", "value": "https://..."}`) and resolution of `path`/`embedded` variants will delegate to the resolver chain. This spec's `StyleInput` union is intentionally open for that extension; no API shape change is required at Tier 1 or Tier 2 entry points.

### `DocumentOptions` — document-level configuration

> **Draft — options require resolution before implementation.** The fields below are candidate examples drawn from the existing engine API surface and Pandoc/citeproc precedent. Which of these belong here vs. on the request top level, how they map to Rust types, and which Pandoc options Citum should adopt at all are open questions. Do not treat this table as final.

Controls rendering behaviour that belongs to the document rather than the style. Passed as an optional `document_options` field on `FormatDocumentRequest` and on `open_session`.

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `bibliography_groups` | `BibliographyGroup[]` | — | Override or replace style-defined grouping. Reuses `BibliographyGroup` from `citum-schema-style` (has `id`, `heading`, `selector`, `sort`, `template` — see csl26-extg, commit `86dca5f`). Takes precedence over style-defined groups. |
| `annotations` | `Record<string, string>` | — | Ref ID → annotation text map. When present, the processor appends each entry's annotation after its bibliography entry. Annotation text is parsed as Djot inline markup by default. |
| `annotation_format` | `"djot"` \| `"plain"` \| `"org"` | `"djot"` | Controls how annotation text is interpreted. Maps to `AnnotationFormat` in `citum-engine/src/io.rs`. |
| `suppress_bibliography` | boolean | false | *Pandoc precedent — not yet in engine. TBD whether to add.* |
| `link_citations` | boolean | false | *Pandoc precedent — not yet in engine. TBD.* |
| `link_bibliography` | boolean | false | *Pandoc precedent — not yet in engine. TBD.* |
| `notes_after_punctuation` | boolean | false | *Pandoc precedent — not yet in engine. TBD.* |

**What is confirmed in the current engine:**

`render_bibliography_with_format_and_annotations` accepts `annotations: Option<&HashMap<String, String>>` and `annotation_style: Option<&AnnotationStyle>` explicitly — the caller supplies the annotation map; the engine does not auto-extract from any reference field. The CLI exposes this via `--annotations <path>` (JSON/YAML file). The API surface here needs to mirror that contract.

**`bibliography_groups` shape** (abbreviated — see `citum-schema-style/src/grouping.rs` for full definition):
```json
{
  "id": "cases",
  "heading": {"literal": "Table of Cases"},
  "selector": {"type": ["legal-case"]},
  "sort": null
}
```

> **Scope note:** Style identity (`style`, `locale`, `output_format`) stays at the request top level. `document_options` is for per-document rendering overrides that don't belong in the style itself.

### `CitationOccurrence` and `CitationOccurrenceItem`

These mirror the existing `Citation` and `CitationItem` types from `citum-schema-data` to ensure `citum-engine` can pass them through without an impedance-mismatch transform.

**`CitationOccurrence` fields** (maps to `Citation`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Stable identifier for this occurrence in the document |
| `items` | array | yes | References cited together |
| `mode` | `"integral"` \| `"non-integral"` | no | Default `non-integral`. `integral` = narrative author-in-text ("Smith (2020) argues..."); `non-integral` = parenthetical "(Smith, 2020)". Only affects author-date styles. |
| `note_number` | integer | no | Footnote/endnote number. Omit (null) for in-text citations. Matches `Citation.note_number: Option<u32>`. |
| `suppress_author` | boolean | no | Suppress author across all items. Use when author is already named in surrounding prose. Per-item suppression is not supported. |
| `grouped` | boolean | no | Treat as a compound-numeric group; suppresses internal sorting and merges bibliography entries. |
| `prefix` | string | no | Text before all formatted items |
| `suffix` | string | no | Text after all formatted items |

**`CitationOccurrenceItem` fields** (maps to `CitationItem`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Reference ID (must exist in `refs`) |
| `locator` | `LocatorSegment` \| `{segments: LocatorSegment[]}` | no | Pinpoint locator. See below. |
| `prefix` | string | no | Text before this item |
| `suffix` | string | no | Text after this item |
| `integral_name_state` | `"first"` \| `"subsequent"` | no | Explicit override for integral name rendering. `first` = full name form; `subsequent` = short form. Omit to let the processor infer from document position. |

**`LocatorSegment`** shape (maps to `LocatorSegment { label: LocatorType, value: LocatorValue }`):

```json
// Single locator — most common
{"label": "page", "value": "23"}

// Plural-aware value
{"label": "page", "value": {"value": "23-25", "plural": true}}

// Custom locator type
{"label": {"custom": "surah"}, "value": "3"}
```

For compound locators (e.g. chapter + page), use the compound form:
```json
{"segments": [{"label": "chapter", "value": "3"}, {"label": "page", "value": "42"}]}
```

Standard `label` values (kebab-case): `page`, `chapter`, `section`, `paragraph`, `volume`, `issue`, `note`, `figure`, `line`, `verse`, `column`, and others per `LocatorType`.

### `Warning` — structured diagnostic

Both `format_document` and session mutation results carry a `warnings` array with structured entries:

```json
{
  "level": "warning",
  "code": "missing_ref",
  "citation_id": "cite1",
  "message": "Reference 'smith2020' not found in refs"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `level` | `"warning"` \| `"error"` | Severity |
| `code` | string | Machine-readable code (e.g. `missing_ref`, `invalid_label`, `style_not_found`) |
| `citation_id` | string | Which citation occurrence triggered the diagnostic (omitted for document-level issues) |
| `message` | string | Human-readable description |

---

### Tier 1 — `format_document` (stateless)

Modelled on Pandoc citeproc: one call, all inputs, all outputs. Works in stdio, HTTP, and WASM.

**RPC method name:** `format_document`

**Request fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `style` | `StyleInput` | yes | Style reference (see above) |
| `locale` | string | no | BCP 47 locale override (e.g. `"en-US"`) |
| `output_format` | string | no | One of `plain` (default), `html`, `djot`, `latex`, `typst` |
| `refs` | object | yes | Bibliography: reference ID → reference object |
| `citations` | array | yes | Ordered `CitationOccurrence` items (document order) |
| `document_options` | `DocumentOptions` | no | Document-level rendering config (see above) |

**Example request:**
```json
{
  "style": {"kind": "embedded", "value": "apa-7th"},
  "locale": "en-US",
  "output_format": "plain",
  "refs": {
    "smith2020": {
      "id": "smith2020",
      "class": "monograph",
      "type": "book",
      "title": "Example Book",
      "author": [{"family": "Smith", "given": "Jane"}],
      "issued": "2020"
    }
  },
  "citations": [
    {
      "id": "cite1",
      "items": [{"id": "smith2020", "locator": {"label": "page", "value": "23"}}]
    },
    {
      "id": "cite2",
      "mode": "integral",
      "items": [{"id": "smith2020"}]
    }
  ],
  "document_options": {
    "annotations": {
      "smith2020": "A foundational text for the field."
    },
    "annotation_format": "djot"
  }
}
```

**Example result:**
```json
{
  "formatted_citations": [
    {"id": "cite1", "text": "(Smith, 2020, p. 23)"},
    {"id": "cite2", "text": "Smith (2020)"}
  ],
  "bibliography": {
    "format": "plain",
    "content": "Smith, J. (2020). Example Book.",
    "entries": ["Smith, J. (2020). Example Book."]
  },
  "warnings": []
}
```

The ordered `citations` array gives the processor full document context in a single pass, enabling correct note-position inference, ibid, and disambiguation — none of which are possible with the current per-citation `render_citation` method. The second citation above uses `mode: "integral"` to render as a narrative citation.

---

### Tier 2 — Session API (stateful)

For word processors with large bibliographies where re-sending the full `refs` library on each edit is too slow. Amortizes style parsing and deserialization across many edits. Available only with the `session` feature flag (which implies `http`).

**Session store (server-side):** `Arc<Mutex<HashMap<String, DocumentSession>>>`

**Session mutation result envelope** (all mutation methods share this shape):

```json
{
  "version": 5,
  "affected_citations": [{"id": "c14", "text": "..."}],
  "bibliography": {"format": "plain", "content": "...", "entries": [...]},
  "renumbering_occurred": false,
  "warnings": []
}
```

`affected_citations` contains only citations whose formatted text changed. `renumbering_occurred` is `true` when note numbers or numeric labels shifted — GUIs should call `get_citations` to refresh all visible citations when this is set. `version` increments on each mutation; clients can use it to detect stale state.

**Session eviction error** — when a session has been evicted after TTL expiry:
```json
{"error": "session_expired", "session_id": "s-a1b2c3d4", "expired_at": "2026-05-04T12:34:56Z"}
```

Clients must handle this by opening a new session and re-uploading references.

#### `open_session`

```json
{
  "method": "open_session",
  "params": {
    "style": {"kind": "embedded", "value": "chicago-17th"},
    "locale": "en-US",
    "output_format": "plain",
    "document_options": {
      "notes_after_punctuation": true
    }
  }
}
```

Returns: `{"session_id": "s-a1b2c3d4"}`

#### `put_references`

```json
{
  "method": "put_references",
  "params": {
    "session_id": "s-a1b2c3d4",
    "refs": {"doe2021": {...}}
  }
}
```

Returns: `{}`

Replaces the full reference set. For incremental delta updates, a future `patch_references` method may add/remove individual entries; not specced here.

#### `insert_citations_batch`

For initializing a session from an existing document — avoids N sequential round-trips:

```json
{
  "method": "insert_citations_batch",
  "params": {
    "session_id": "s-a1b2c3d4",
    "citations": [
      {"id": "c1", "items": [{"id": "doe2021"}]},
      {"id": "c2", "mode": "integral", "items": [{"id": "doe2021"}]}
    ]
  }
}
```

`citations` is the full ordered list in document order. Returns the standard session mutation envelope.

#### `insert_citation`

```json
{
  "method": "insert_citation",
  "params": {
    "session_id": "s-a1b2c3d4",
    "citation": {
      "id": "c14",
      "note_number": 4,
      "items": [{"id": "doe2021", "locator": {"label": "page", "value": "45"}}]
    },
    "position": {
      "after_citation_id": "c12",
      "before_citation_id": "c20"
    }
  }
}
```

`position` uses neighbour citation IDs rather than citeproc-js `citationsPre`/`citationsPost` arrays. `after_citation_id` and `before_citation_id` are both optional; omitting both places the citation at the end of the document.

Returns the standard session mutation envelope.

#### `update_citation`

Same shape as `insert_citation` but targets an existing citation by `citation_id`. Returns the standard session mutation envelope.

#### `delete_citation`

```json
{
  "method": "delete_citation",
  "params": {"session_id": "s-a1b2c3d4", "citation_id": "c14"}
}
```

Returns the standard session mutation envelope.

#### `preview_citation`

Returns a formatted preview without mutating session state. Useful for real-time UI previews while the user selects references before inserting.

```json
{
  "method": "preview_citation",
  "params": {
    "session_id": "s-a1b2c3d4",
    "items": [{"id": "doe2021", "locator": {"label": "page", "value": "45"}}],
    "position": {"after_citation_id": "c12"}
  }
}
```

Returns: `{"preview": "(Doe, 2021, p. 45)"}` — no version bump, no bibliography.

#### `get_citations`

```json
{"method": "get_citations", "params": {"session_id": "s-a1b2c3d4"}}
```

Returns: `{"formatted_citations": [{"id": "c12", "text": "..."}, ...]}`

#### `get_bibliography`

```json
{"method": "get_bibliography", "params": {"session_id": "s-a1b2c3d4"}}
```

Returns: `{"bibliography": {"format": "plain", "content": "...", "entries": [...]}}`

#### `close_session`

```json
{"method": "close_session", "params": {"session_id": "s-a1b2c3d4"}}
```

Returns: `{}`

Frees the processor from the session store immediately. Clients must call this when a document is closed to avoid memory growth.

---

### WASM Session Variant

In WASM, the JS runtime holds the `DocumentSession` object directly — no session IDs needed. `wasm-bindgen` exposes it as a JS class:

```typescript
// TypeScript signature (generated via specta)
class DocumentSession {
  constructor(style_yaml: string, refs_json?: string): DocumentSession;
  put_references(refs_json: string): void;
  insert_citations_batch(citations_json: string): string; // JSON → SessionResult
  insert_citation(citation_json: string, position_json: string): string;
  update_citation(citation_id: string, citation_json: string, position_json: string): string;
  delete_citation(citation_id: string): string;
  preview_citation(items_json: string, position_json: string): string; // → {"preview": "..."}
  get_citations(): string; // JSON → {formatted_citations: [...]}
  get_bibliography(): string; // JSON → {bibliography: {...}}
  free(): void;    // wasm-bindgen destructor — call in cleanup/unmount
  dispose(): void; // alias for free()
}
```

`refs_json` is optional in the constructor; callers may omit it and call `put_references()` separately.

### Terminology

Use **"citations"** throughout — not "citation clusters" (citeproc-js terminology). Use **"citation occurrence"** when distinguishing a specific in-text or in-note appearance from the abstract citation concept. Use **"integral"** for narrative (author-in-text) citations; this maps to `CitationMode::Integral` in the Rust schema.

---

## Implementation Notes

### `Processor` is `!Sync`

`Processor` uses `RefCell<...>` fields (`citation_numbers`, `cited_ids`, etc.), making it `!Sync` — it cannot be shared across concurrent requests. `RefCell<T>` does not affect `Send`, so `Processor` (and `DocumentSession`) can be moved between threads.

The `Arc<Mutex<HashMap<String, DocumentSession>>>` session store addresses this: each request acquires the mutex, gets exclusive access to the `DocumentSession`, does its work, and releases the lock. No concurrent access occurs, so `!Sync` is never violated.

**Alternative (actor pattern):** Confine each `DocumentSession` to a dedicated Tokio task that owns it forever. HTTP handlers communicate via channels (`mpsc::Sender<SessionCommand>`). The session never leaves its owning task, so `!Sync` is not a concern and there is no mutex contention. Preferred for high-throughput deployments; the mutex approach is simpler for initial implementation.

### Feature Flags

`session` implies `http` implies `async`. Add to `citum-server/Cargo.toml`:

```toml
[features]
session = ["http"]
```

Session methods are unreachable without HTTP (sessions require server-side state). `format_document` works in both stdio and HTTP modes.

### Session TTL

Sessions idle beyond a configurable timeout (default: 30 minutes) should be evicted to bound memory growth. When evicted, any subsequent method call for that session ID returns the `session_expired` error (see above). Implementation detail left to the implementing task.

---

## Acceptance Criteria

- [ ] `format_document` dispatches correctly in both stdio and HTTP modes.
- [ ] `format_document` returns one `FormattedCitation` per input `CitationOccurrence`, in the same order.
- [ ] A document with multiple citations to the same work in a note style produces correct ibid where the style requires it.
- [ ] An integral (`mode: "integral"`) citation occurrence renders as a narrative citation.
- [ ] Session lifecycle (`open_session` → `put_references` → `insert_citations_batch` → `close_session`) works end-to-end over HTTP.
- [ ] `preview_citation` returns a `{"preview": "..."}` response without mutating the session's citation list.
- [ ] `DocumentSession` is exposed as a wasm-bindgen class in `citum-bindings` with optional `refs_json` constructor and `dispose()` alias.
- [ ] Expired session requests return `{"error": "session_expired", ...}`.
- [ ] Existing `render_citation`, `render_bibliography`, `validate_style` methods continue to pass all current tests unchanged.
- [ ] `cargo nextest run` passes.

## Changelog

- 2026-05-04: Initial draft.
- 2026-05-04: Added `DocumentOptions` covering Pandoc-equivalent metadata fields (`suppress_bibliography`, `link_citations`, `link_bibliography`, `notes_after_punctuation`), document-level `bibliography_groups` override (reusing `BibliographyGroup` from csl26-extg), and `annotated_bibliography` mode. Added `csl26-r8d2` forward-reference for `StyleInput` URL variant.
- 2026-05-04: Revised per reviewer feedback. `StyleInput` union replaces bare `style` string; `output_format` promoted to top-level; `note_number` made optional (null = in-text); `CitationOccurrence` fields aligned with `Citation`/`CitationItem` schema (`CitationLocator` shape, `mode`, `suppress_author`, `integral_name_state`, `grouped`); `author_only` removed; `!Send` corrected to `!Sync`; session results unified into versioned envelope; `insert_citations_batch` added; `warnings` structured; WASM constructor makes `refs_json` optional; `affected_citations` semantics clarified with `renumbering_occurred`; session eviction error documented.
