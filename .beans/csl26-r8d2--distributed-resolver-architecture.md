---
title: Reconsider Distributed Registry and Resolver Architecture
status: draft
type: task
priority: high
tags:
    - registry
    - architecture
    - style
    - research
created_at: 2026-05-03T00:00:00Z
updated_at: 2026-05-03T00:00:00Z
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

## Phase 1: Trait Boundary & URI Foundation (Current Focus)
Establish a flexible, trait-based resolution boundary and generalize the `extends` mechanism to support URIs, without adding network dependencies yet.
- **StyleReference Enum:** Update `extends` to support both `StyleBase` enums and URI strings (e.g., `file://...`, `@hub/...`, `https://...`).
- **StyleResolver Trait:** Introduce a trait in `citum_store` for resolving styles and locales.
- **Resolver Chain:** Implement concrete resolvers (`FileResolver`, `StoreResolver`, `EmbeddedResolver`) and a `ChainResolver` for sequential fallback.
- **CLI Refactor:** Transition `citum-cli` to use the `ChainResolver` for all style loading.

## Phase 2: Remote Fetching & Caching
Add networking capabilities and a local caching layer to ensure offline-first resilience.
- **Remote Resolvers:** Implement `HttpResolver` and `GitResolver` to fetch styles from standard endpoints.
- **Caching Middleware:** Wrap remote resolvers in a local cache that stores fetched styles in the user data directory.
- **Institutional Autonomy:** Enable organizations to host their own style registries via static HTTP or Git.

## Phase 3: Content Addressing & Hub Federation
Scale the architecture for massive distribution and trust.
- **CIDs:** Integrate content-addressable hashes (CIDs) for immutable style references.
- **Verification:** Implement hashing middleware to verify remote style integrity against URIs.
- **Hub Protocol:** Design and implement decentralized registry protocols for Citum Hub federation.

# Key Considerations
- **Trust & Security:** Evaluating the implications of fetching and executing remote style/locale definitions.
- **Conflict Resolution:** Defining a clear hierarchy when multiple registries (builtin, local, and multiple remotes) provide the same style ID.
- **Schema & Versioning:** How to ensure remote registries remain compatible with the engine version.
- **Caching Strategy:** Balancing offline availability with the need for updates.

# Goals
- Eliminate the single-point-of-failure bottleneck for style publication.
- Enable institutional privacy and institutional "ownership" of style variants.
- Maintain the "Zero-Config" advantage for standard users through a strong primary Hub default.

## Stage 1 Implementation (completed)

Implemented `file://` URI resolution in `try_into_resolved_recursive`:
- Added `UriResolutionFailed { uri, reason }` to `ResolutionError`
- Handles `.yaml`/`.yml`, `.json`, and `.cbor` format detection by file extension
- Loop protection via existing `visited` set
- Only `file://` URIs accepted; other schemes and bare paths return `UriResolutionFailed`

## Remaining Work (Stage 2+)

Phase 2 (remote fetching, caching) and Phase 3 (content addressing, hub federation)
remain. The `StyleResolver` trait and resolver chain in `citum_store` are already
in place for when HTTP/Git resolution is added.

Key decision deferred: whether to thread a `&dyn StyleResolver` into
`try_into_resolved_recursive` — enables pluggable resolution at the schema
level and is required before Phase 2 can handle non-file URIs.
