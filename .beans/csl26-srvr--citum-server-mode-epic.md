---
# csl26-srvr
title: citum-server mode (epic)
status: completed
type: milestone
priority: normal
created_at: 2026-02-23T00:00:00Z
updated_at: 2026-02-27T18:13:46Z
---

Epic tracking the creation of a dedicated `citum-server` binary crate for
real-time citation formatting. Supports word processor integrations (Word,
LibreOffice via Zotero) and live preview in the citum-hub web app.

See docs/architecture/CITUM_SERVER_MODE.md for the full plan.

## Sub-tasks

- csl26-srpc: Implement JSON-RPC stdin/stdout handler (core transport)
- csl26-shtp: HTTP feature flag (axum, implies async/tokio)

## Design Decisions

- Dedicated binary crate, NOT a subcommand on citum-cli
- Primary transport: newline-delimited JSON-RPC on stdin/stdout
- Deps: citum-engine + citum-schema only (no legacy/migrate)
- async (tokio) is opt-in behind a feature flag
- HTTP (axum) is opt-in behind a second feature flag
- Three methods: render_citation, render_bibliography, validate_style
- Same API surface as the planned citum-bindings (but different deployment)

## Summary of Changes

- Created  binary crate with JSON-RPC stdin/stdout transport
- Feature-gated  (tokio) and  (axum) modes
- Three methods: render_citation, render_bibliography, validate_style
- Fixed two pre-existing test bugs in citum-engine (unwrap, #[ignore])
- Added README for the crate
- PR #246 merged
