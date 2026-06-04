---
# csl26-j7ah
title: Document session API in citum-server, docs.rs, and jsr.io
status: completed
type: task
priority: normal
created_at: 2026-06-04T22:16:44Z
updated_at: 2026-06-04T22:19:14Z
---

Update README.md, lib.rs crate docs, and README-JSR.md to document the new session feature. Also fix cargo run -p references.

## Summary of Changes

- crates/citum-server/README.md: added Install section, replaced `cargo run -p citum-server` with `citum-server` binary in all user-facing examples (kept one for workspace dev), added session methods to Method Summary table, added session feature to Features table, added Session API section with lifecycle, stdio and HTTP examples
- crates/citum-server/src/lib.rs: updated crate-level doc Methods table to include 10 session methods, updated Features list to include session
- crates/citum-bindings/README-JSR.md: added DocumentSession to import block, added Stateful Session API section with TypeScript lifecycle example and return-type documentation
