---
# csl26-shtp
title: citum-server HTTP feature (axum, feature-gated)
status: completed
type: task
priority: low
created_at: 2026-02-23T00:00:00Z
updated_at: 2026-02-27T18:13:47Z
blocked_by:
    - csl26-srpc
---

Add an optional `http` feature to `citum-server` that exposes the same three
methods over HTTP/REST using axum. Required for the citum-hub live preview
panel and browser-based style editor.

## Acceptance Criteria

- `src/http.rs` behind `#[cfg(feature = "http")]`
- `http` feature in Cargo.toml adds: tokio (full), axum; implies `async`
- Routes: POST /render_citation, POST /render_bibliography, POST /validate_style
- Reuses the same dispatcher logic as rpc.rs (no duplication)
- `--http` and `--port` flags added to main.rs arg parsing
- No authentication in this iteration
- All pre-commit checks pass

## Notes

Implement only after csl26-srpc is complete and the API surface is validated.
See docs/architecture/CITUM_SERVER_MODE.md for design rationale.
