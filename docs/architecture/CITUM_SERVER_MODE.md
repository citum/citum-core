# Citum Server Mode: Architecture Plan

**Status:** Implemented
**Last updated:** 2026-05-03
**Related:** [CITUM_MODULARIZATION.md](./CITUM_MODULARIZATION.md)

## Problem Statement

The Citum engine provides a server mode to support real-time citation formatting
in word processors (Word, LibreOffice) and live preview in the citum-hub web
app. The batch CLI (`citum`) is synchronous and stdin/stdout-driven; the
dedicated server provides a persistent runtime model for low-latency updates.

---

## Decision: Dedicated `citum-server` Binary Crate (In `citum-core`)

The server mode is implemented as a dedicated `citum-server` binary crate in the
`citum-core` workspace. It is not a subcommand on `citum`.

**Rationale:**
- CLI is a batch tool with synchronous I/O; the server has a different lifecycle
  (long-running process, connection management, graceful shutdown)
- A dedicated crate keeps boundaries clear while iteration is still fast in one repo
- Maps directly to the `citum-bindings` layer — server and bindings both depend only on `citum-engine`
- Async (tokio) is an opt-in feature flag; sync builds remain possible for
  embedding contexts that don't want a runtime

### Implementation Status

| Component | Status | Purpose |
|-----------|--------|---------|
| `crates/citum-server/Cargo.toml` | DONE | Crate manifest; deps on engine + schema only; optional async/http features |
| `crates/citum-server/src/main.rs` | DONE | Entry point; arg parsing (mode flag: `--http`, `--port`) |
| `crates/citum-server/src/rpc.rs` | DONE | JSON-RPC stdin/stdout handler |
| `crates/citum-server/src/http.rs` | DONE | HTTP handler (feature-gated behind `http`) |

---

## Transport: JSON-RPC over stdin/stdout (Default)

Primary transport is newline-delimited JSON objects on stdin/stdout, following
the same pattern as citeproc-rs and Haskell citeproc. This transport:

- Requires no port management or OS-level permissions
- Works cleanly inside word processor plugins (Zotero, Pandoc pipelines)
- Is trivially testable with `echo '...' | citum-server`
- Has established prior art in the citation processing ecosystem

HTTP/REST is available behind an opt-in `http` feature flag.

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

Methods match the `citum-bindings` public API surface
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
same dispatcher logic as the stdin/stdout handler.

---

## Scalability and Performance

As of Phase 2, the server operates in a **stateless** mode. On every request, the
client must provide the full reference library and style path. 

Benchmarking (May 2026) confirmed:
- **500 Refs:** ~21ms latency for full bibliography refresh (Interval=1)
- **5000 Refs:** ~210ms latency for full bibliography refresh (Interval=1)

While the stateless model is highly robust, research is ongoing into an optional
stateful mode for sub-millisecond updates in massive documents (see `csl26-stat`).

---

## Relationship to citum-bindings

`citum-server` and `citum-bindings` share the same public API surface
(`render_citation`, `render_bibliography`, `validate_style`) but serve
different deployment targets:

| | `citum-server` | `citum-bindings` |
|---|---|---|
| Target | Process (long-running) | Library (embedded, WASM) |
| Callers | Word processors, hub app | Web apps, WASM runtimes |
| Transport | JSON-RPC / HTTP | Rust FFI / wasm-bindgen |
| Async | Opt-in feature | Opt-in feature |

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
| `csl26-stat`  | Consider optional stateful mode for citum-server   | 3     |

