---
# csl26-erwz
title: User style & locale store (citum_store)
status: completed
type: epic
priority: normal
tags:
    - cli
created_at: 2026-03-04T18:11:14Z
updated_at: 2026-05-23T14:37:53Z
parent: csl26-li63
---

Platform-aware local store for user-owned styles and locales. Resolver crate shared across CLI, desktop, web, and mobile. Hub sync deferred to Phase 2.

Architecture plan: `docs/architecture/CITUM_STORE_PLAN.md`

## Phase 1 tasks
- [x] `citum_store` crate: StoreResolver, platform_data_dir, YAML/JSON/CBOR loading
- [x] Format config: global `config.{yaml,toml}` works end-to-end. Per-invocation flag intentionally not added — install preserves source extension, resolve detects from extension, list is format-agnostic; the global setting is a tiebreaker. See `docs/architecture/CITUM_STORE_PLAN.md` §Format transparency.
- [x] Locale CLI integration: `-L <id>` consults the user store for both builtin-alias and file-based styles via the shared chain resolver (closed by csl26-hqy5).
- [x] CLI integration: `-s` checks user store before builtins
- [x] `citum store list` + `citum store install <path|slug>`
- [x] `citum store remove <name>` with confirmation

## Summary of Changes

Phase 1 complete. Styles and locales both resolve through `citum_store::ChainResolver`:
- Styles: `file path → user store → http/git → registries → embedded` (already in place).
- Locales (file-based style): `sibling locales/ → user store → embedded` (new in csl26-hqy5).
- Locales (builtin-alias style): `user store → embedded` (new in csl26-hqy5).

Format transparency confirmed across YAML, JSON, and CBOR: `install` preserves source ext, `resolve` detects from ext, `list` is format-agnostic. Architecture plan updated to reflect this and to drop the never-needed `--store-format` per-invocation flag.

Phase 2 (Hub sync) remains scheduled for pre-1.0.
