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
Evolve `citum-core`'s resolution logic to support a truly distributed ecosystem:

1. **Universal Resource Identifiers (URIs):** Transition `extends` references to a URI-based system (e.g., `@hub/apa`, `did:web:university.edu/styles/thesis`).
2. **Pluggable Resolvers:** Implement a resolver trait that allows the engine to fetch style parents from multiple backends (local file system, Hub API, Git).
3. **Immutability and Content Addressing:** Integrate content-addressable hashes (CIDs) or strict semantic versioning to prevent "left-pad" style breakages.
4. **Caching Layer:** Design a robust local caching mechanism for remote styles to ensure resilient, offline-first rendering.

# Key Considerations
- **Trust & Security:** Evaluating the implications of fetching and executing remote style/locale definitions.
- **Conflict Resolution:** Defining a clear hierarchy when multiple registries (builtin, local, and multiple remotes) provide the same style ID.
- **Schema & Versioning:** How to ensure remote registries remain compatible with the engine version.
- **Caching Strategy:** Balancing offline availability with the need for updates.

# Goals
- Eliminate the single-point-of-failure bottleneck for style publication.
- Enable institutional privacy and institutional "ownership" of style variants.
- Maintain the "Zero-Config" advantage for standard users through a strong primary Hub default.
