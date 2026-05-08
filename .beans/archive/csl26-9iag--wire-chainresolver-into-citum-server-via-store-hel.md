---
# csl26-9iag
title: Wire ChainResolver into citum-server via store helper
status: completed
type: feature
created_at: 2026-05-08T20:51:53Z
updated_at: 2026-05-08T20:51:53Z
---

Extract build_standard_chain() into citum_store so both CLI and server share the same resolver construction. Three commits: (A) refactor(store): extract standard chain builder into chain.rs module; (B) refactor(cli): delegate load_any_style to store helper; (C) feat(server): wire ChainResolver into style loading. Closes the deferred follow-up in docs/specs/DISTRIBUTED_RESOLVER.md lines 671-682.

## Summary of Changes

- Added chain.rs to citum_store (http feature) with build_standard_chain() and registry_resolvers()
- CLI load_any_style() now delegates chain construction to store; keeps strsim suggestion logic
- citum-server gains citum_store http dep; load_style() uses build_standard_chain()
- Server can now resolve styles by name, path, or URI (not just absolute file paths)
- All 1225 workspace tests pass
