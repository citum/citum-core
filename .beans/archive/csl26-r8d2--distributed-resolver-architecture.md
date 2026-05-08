---
# csl26-r8d2
title: Reconsider Distributed Registry and Resolver Architecture
status: completed
type: task
priority: high
tags:
    - registry
    - architecture
    - style
    - research
created_at: 2026-05-03T00:00:00Z
updated_at: 2026-05-08T00:00:00Z
---

# csl26-r8d2

# Objective
Evaluate the necessity and architectural design of a distributed registry system for Citum styles, evolving the resolution logic to support a truly distributed ecosystem.

# Context
The current Citum model, while technically supporting registries and inheritance, still defaults to a centralized resolution pattern. As identified in the Citum Hub visioning process, this risks creating a new "SaaS Silo" that mirrors the bottlenecks of the monolithic CSL repository.

Currently, Citum relies on a centralized builtin registry and a local user store. While this ensures stability and ease of use for standard styles, it creates friction for organizations or publishers who wish to maintain and distribute their own collection of styles without upstreaming them or requiring users to manually install files.

To support over 1M+ users while enabling institutional autonomy, Citum needs a federated, GitOps-friendly resolution model inspired by modern package managers (Cargo, npm) and decentralized protocols (AT Protocol).

# Proposal
Evolve `citum-core`'s resolution logic to support a truly distributed ecosystem via a phased approach:

## Phase 1: Trait Boundary & URI Foundation (completed)
- **StyleReference Enum:** `extends` supports both `StyleBase` enums and URI strings.
- **StyleResolver Trait:** Resolver interface in `citum_store` (with a minimal twin in `citum-schema-style`).
- **Resolver Chain:** Concrete resolvers (`FileResolver`, `StoreResolver`, `EmbeddedResolver`) plus `ChainResolver`.
- **CLI Refactor:** `citum-cli` resolves all styles through the chain.

## Phase 2: Remote Fetching & Caching (completed)
- **Remote Resolvers:** `HttpResolver` and `GitResolver`.
- **Caching Middleware:** Per-resolver disk cache under the platform cache dir.
- **Institutional Autonomy:** Static-HTTP and Git-backed registries via `StoreConfig.registries`.

## Phase 3: Content Addressing & Hub Federation (completed)
- **CIDs:** CIDv1 raw + sha-256 + base32 (`bafkrei…`).
- **Verification:** `extends-pin` integrity middleware.
- **Hub Protocol:** Federated registry index format with `citum-version` per style.

# Key Considerations
- **Trust & Security:** YAML data structures only; no executable style code.
- **Conflict Resolution:** First-match chain order, no merge semantics.
- **Schema & Versioning:** `info.citum-version` required-range check.
- **Caching Strategy:** Offline-first; CID entries immutable.

# Goals
- Eliminate the single-point-of-failure bottleneck for style publication.
- Enable institutional privacy and institutional ownership of style variants.
- Maintain the zero-config advantage for standard users through a strong primary Hub default.

## Stage 1 Implementation (completed)

Implemented `file://` URI resolution in `try_into_resolved_recursive`:
- Added `UriResolutionFailed { uri, reason }` to `ResolutionError`
- Handles `.yaml`/`.yml`, `.json`, and `.cbor` format detection by file extension
- Loop protection via existing `visited` set
- Only `file://` URIs accepted; other schemes and bare paths return `UriResolutionFailed`

## Phase 2 Implementation (completed)

- **GitResolver:** Clones shallow repos via `git clone --depth=1`, caches by URI hash
- **HttpResolver:** Checks host allowlist (empty = allow all), serves stale cache on network error
- **StoreConfig:** Extends with `registries: Vec<RegistryConfig>` field
- **RegistryConfig:** Name, URL, priority, ttl_secs, trusted flag
- **Config Loading:** Supports `~/.config/citum/config.yaml` (YAML preferred) and `config.toml` (fallback)
- **RegistryResolver:** Routes `git+https://` URIs to GitResolver
- **Depth Cap:** MAX_DEPTH=5 check in `try_into_resolved_recursive_with_depth`

## Phase 3 Implementation (completed 2026-05-08)

Stacked branch `feat/distributed-resolver-phase-3`:

- [x] feat(schema): add cid + integrity fields
- [x] feat(store): add resolver error variants
- [x] feat(store): cid resolver and verifying middleware
- [x] feat(schema): wire integrity + version into resolution
- [x] feat(cli): add style cid/pin/validate commands (also enriches `style info`)
- [x] docs(specs): update distributed resolver to phase 3
- [x] docs(guides): distributed registry walkthrough

## Summary of Changes (Phase 3)

- **Schema:** `StyleReference` accepts `cid:` URIs (via the existing `Uri` variant); `Style.extends_pin` and `StyleInfo.citum_version` added; `ResolutionError` gains `IntegrityFailure` and `VersionMismatch`.
- **Store:** `ResolverError` gains `Denied`/`NetworkError`/`VersionMismatch`/`IntegrityFailure` variants; new `citum_store::cid` module computes/verifies canonical CIDv1 (raw codec, sha-256, base32 lower); new `CidResolver` routes `cid:` URIs through a configurable IPFS gateway; new `VerifyingResolver` middleware and `fetch_and_verify_bytes` helper enforce integrity. `StoreConfig` adds `cid_gateway` override.
- **Resolution:** `try_into_resolved_recursive_with_depth` verifies `extends-pin` against the resolved parent and rejects builtin `StyleBase` parents with pins; `check_citum_version` runs at the root and on every URI-resolved parent.
- **CLI:** new `citum style cid`, `citum style pin`, `citum style validate` subcommands; `citum style info` now prints CID and `citum-version`.
- **Docs:** `docs/specs/DISTRIBUTED_RESOLVER.md` flipped to Active (Phase 3) with nine spec amendments and full acceptance check-offs; new `docs/guides/DISTRIBUTED_REGISTRIES.md` walks a non-Rust user through the whole UX with copy-pasteable commands.

Deferred follow-ups (tracked in spec): citum-server wiring, locale CID/integrity, registry discovery, IPFS gateway redundancy.

## Spec

[docs/specs/DISTRIBUTED_RESOLVER.md](../../docs/specs/DISTRIBUTED_RESOLVER.md) — Status: Active (Phase 3)
