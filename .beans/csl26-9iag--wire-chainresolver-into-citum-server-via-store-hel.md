---
# csl26-9iag
title: Wire ChainResolver into citum-server via store helper
status: in-progress
type: feature
created_at: 2026-05-08T20:51:53Z
updated_at: 2026-05-08T20:51:53Z
---

Extract build_standard_chain() into citum_store so both CLI and server share the same resolver construction. Three commits: (A) refactor(store): extract standard chain builder into chain.rs module; (B) refactor(cli): delegate load_any_style to store helper; (C) feat(server): wire ChainResolver into style loading. Closes the deferred follow-up in docs/specs/DISTRIBUTED_RESOLVER.md lines 671-682.
