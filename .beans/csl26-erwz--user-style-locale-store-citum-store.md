---
# csl26-erwz
title: User style & locale store (citum_store)
status: todo
type: epic
priority: normal
tags:
    - cli
created_at: 2026-03-04T18:11:14Z
updated_at: 2026-04-25T20:20:07Z
parent: csl26-li63
---

Platform-aware local store for user-owned styles and locales. Resolver crate shared across CLI, desktop, web, and mobile. Hub sync deferred to Phase 2.

Architecture plan: `docs/architecture/CITUM_STORE_PLAN.md`

## Phase 1 tasks
- [x] `citum_store` crate: StoreResolver, platform_data_dir, YAML/JSON/CBOR loading
- [ ] Format config: global config.toml + per-invocation flag
- [x] CLI integration: `-s` checks user store before builtins
- [x] `citum store list` + `citum store install <path|slug>`
- [x] `citum store remove <name>` with confirmation
