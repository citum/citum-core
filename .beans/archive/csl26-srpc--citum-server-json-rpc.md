---
# csl26-srpc
title: Implement citum-server JSON-RPC stdin/stdout handler
status: completed
type: task
priority: normal
created_at: 2026-02-23T00:00:00Z
updated_at: 2026-02-27T18:13:47Z
---

Create the `crates/citum-server/` crate and implement the core JSON-RPC
handler over stdin/stdout (newline-delimited JSON, same pattern as
citeproc-rs).

## Acceptance Criteria

- New `crates/citum-server/` crate added to workspace Cargo.toml
- Cargo.toml deps: citum_engine (citum-engine), citum_schema (citum-schema),
  serde_json; optional `async` feature adds tokio
- `src/main.rs`: entry point, minimal arg parsing (no clap in lib code)
- `src/rpc.rs`: sync stdin/stdout loop; reads newline-delimited JSON requests,
  dispatches to Processor, writes JSON responses
- Three methods handled: render_citation, render_bibliography, validate_style
- Error envelope: `{ "id": N, "error": "message" }`
- All pre-commit checks pass (cargo fmt, clippy, tests)

## Protocol

Request: `{ "id": 1, "method": "render_citation", "params": { ... } }`
Success: `{ "id": 1, "result": "Smith (2024)" }`
Error:   `{ "id": 1, "error": "style not found: apa-7th" }`

See docs/architecture/CITUM_SERVER_MODE.md for full spec.
