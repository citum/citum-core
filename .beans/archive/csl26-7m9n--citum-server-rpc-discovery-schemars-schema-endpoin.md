---
# csl26-7m9n
title: 'citum-server: RPC discovery + schemars schema endpoint'
status: completed
type: task
priority: normal
created_at: 2026-05-14T10:33:40Z
updated_at: 2026-05-14T10:39:21Z
---

Add GET /rpc (405 hint), GET /rpc/methods discovery, GET /rpc/schema (schemars, feature-gated). Create typed param structs for schemars derives. Fix stale README (add format_document, inject_ast_indices). Branch: feat/server-rpc-discovery.

## Summary of Changes

- Added typed param structs (RenderCitationParams, RenderBibliographyParams, ValidateStyleParams) with schemars::JsonSchema derives gated on schema feature
- Added GET /rpc → 405 hint, GET /rpc/methods → static descriptor list, GET /rpc/schema → schemars-generated (schema feature)
- Added schema feature to Cargo.toml (implies http)
- Fixed README: added format_document row, inject_ast_indices note, Discovery section, schema feature row
- All 21 tests pass
