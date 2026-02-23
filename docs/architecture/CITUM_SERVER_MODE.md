# Citum Server Mode: Architecture Plan

**Status:** Approved for Phase 2 implementation
**Last updated:** 2026-02-23
**Related:** [CITUM_MODULARIZATION.md](./CITUM_MODULARIZATION.md)

## Problem Statement

The Citum engine needs a server mode to support real-time citation formatting
in word processors (Word, LibreOffice) and live preview in the citum-hub web
app. The batch CLI (`citum-cli`) is synchronous and stdin/stdout-driven; adding
an HTTP server there conflates two fundamentally different runtime models.

---

## Decision: Dedicated `citum-server` Binary Crate

The server mode belongs in a new `citum-server` binary crate within the
existing workspace. Not as a subcommand on `citum-cli`, and not in a separate
repository.

**Rationale:**
- CLI is a batch tool with synchronous I/O; a server has a different lifecycle
  (long-running process, connection management, graceful shutdown)
- A dedicated crate allows independent binary packaging and versioning
- Maps directly to the `citum-bindings` layer planned in Phase 2 of the
  modularization plan — server and bindings both depend only on `citum-engine`
- Async (tokio) can be an opt-in feature flag; sync builds remain possible for
  embedding contexts that don't want a runtime

### Updated Dependency Graph

```
citum-schema    (no legacy deps)
     |
citum-engine  ---> citum-schema
     |                   |
citum-server ------------|   [new; engine + schema only, no migrate/legacy deps]
     |
citum-migrate ---> citum-schema, csl-legacy
     |
citum-cli     ---> citum-engine, citum-migrate
     |
citum-bindings --> citum-engine  [cdylib/wasm, Phase 2]
```

### Crate Map Update

| Crate            | Published? | Notes                                      |
|-----------------|------------|--------------------------------------------|
| `citum-schema`   | Yes        | Schema source of truth                     |
| `citum-engine`   | Yes        | Rendering engine                           |
| `citum-server`   | Yes (bin)  | JSON-RPC + optional HTTP server            |
| `citum-migrate`  | No         | Internal tooling                           |
| `csl-legacy`     | No         | Internal tooling                           |
| `csln-edtf`      | Yes        | Potentially standalone                     |
| `citum-analyze`  | No         | Internal tooling                           |
| `citum-cli`      | Yes (bin)  | CLI binary                                 |

---

## Transport: JSON-RPC over stdin/stdout (Default)

Primary transport is newline-delimited JSON objects on stdin/stdout, following
the same pattern as citeproc-rs and Haskell citeproc. This transport:

- Requires no port management or OS-level permissions
- Works cleanly inside word processor plugins (Zotero, Pandoc pipelines)
- Is trivially testable with `echo '...' | citum-server`
- Has established prior art in the citation processing ecosystem

HTTP/REST is available behind an opt-in `http` feature flag (see below).

### Request/Response Envelope

```json
{ "id": 1, "method": "render_citation", "params": { ... } }
{ "id": 1, "result": "Smith (2024)" }

{ "id": 2, "method": "render_bibliography", "params": { ... } }
{ "id": 2, "result": ["Smith, J. (2024). Title. Publisher."] }

{ "id": 3, "method": "validate_style", "params": { ... } }
{ "id": 3, "result": { "valid": true, "warnings": [] } }

{ "id": 4, "error": "style not found: apa-7th" }
```

Three methods, matching the planned `citum-bindings` public API surface
(`render_citation`, `render_bibliography`, `validate_style`). No internal
types leak through.

---

## Feature Flags

### `async` (opt-in, default: off)

When enabled, wraps the synchronous `Processor` in
`tokio::task::spawn_blocking` to avoid blocking the async runtime. Required
for the `http` feature. Without this flag the server runs a simple sync
read-loop with no runtime overhead.

### `http` (opt-in, implies `async`)

Exposes the same three methods over HTTP/REST using `axum`. Useful for:
- The citum-hub web app preview panel (eliminates process-per-request overhead)
- Browser-based style editors requiring a local engine proxy

Enabling `http` automatically enables `async`. The HTTP handler reuses the
same dispatcher logic as the stdin/stdout handler — no duplication.

---

## Files to Create/Modify

| Operation | Path | Purpose |
|-----------|------|---------|
| CREATE | `crates/citum-server/Cargo.toml` | Crate manifest; deps on engine + schema only; optional async/http features |
| CREATE | `crates/citum-server/src/main.rs` | Entry point; arg parsing (mode flag: `--http`, `--port`) |
| CREATE | `crates/citum-server/src/rpc.rs` | JSON-RPC stdin/stdout handler |
| CREATE | `crates/citum-server/src/http.rs` | HTTP handler (feature-gated behind `http`) |
| MODIFY | `Cargo.toml` | Add `crates/citum-server` to workspace members |
| MODIFY | `docs/architecture/CITUM_MODULARIZATION.md` | Add `citum-server` to crate map and dependency graph |

---

## Implementation Notes for @builder

- Use `serde_json` for request/response types (already a workspace dep)
- Protocol: newline-delimited JSON on stdin/stdout (same as citeproc-rs)
- No `clap` in library modules; minimal arg parsing in `main.rs` only
- The `Processor` from `csln_processor` (future: `citum-engine`) is sync;
  the `async` feature wraps it in `tokio::task::spawn_blocking`
- Error responses use `{ "id": N, "error": "message" }` envelope
- HTTP feature uses `axum`; add to workspace deps only when feature is enabled
- No authentication or multi-tenant isolation in this iteration
- The server crate sits *above* `citum-bindings` in the stack — both call the
  same engine functions independently; they do not depend on each other

---

## Relationship to citum-bindings (Phase 2)

`citum-server` and `citum-bindings` share the same public API surface
(`render_citation`, `render_bibliography`, `validate_style`) but serve
different deployment targets:

| | `citum-server` | `citum-bindings` |
|---|---|---|
| Target | Process (long-running) | Library (embedded, WASM) |
| Callers | Word processors, hub app | Web apps, WASM runtimes |
| Transport | JSON-RPC / HTTP | Rust FFI / wasm-bindgen |
| Async | Opt-in feature | Opt-in feature |

Implement `citum-server` first (simpler, no FFI complexity). Use it to
validate the API surface before stabilizing `citum-bindings`.

---

## Persona Fit

| Persona         | Impact                                                          |
|----------------|------------------------------------------------------------------|
| Style Author    | None: YAML style files are unaffected                           |
| Web Developer   | Direct beneficiary: live preview via HTTP mode in hub app       |
| Systems Architect | Clean boundary: server has no legacy/migrate deps             |
| Domain Expert   | Enables real-time formatting in word processors                 |

---

## Related Beans

| Bean ID        | Title                                              | Phase |
|---------------|----------------------------------------------------|-------|
| `csl26-srvr`  | citum-server mode (epic)                           | 2     |
| `csl26-srpc`  | Implement JSON-RPC stdin/stdout handler            | 2     |
| `csl26-shtp`  | Implement HTTP feature (axum, feature-gated)       | 2     |
