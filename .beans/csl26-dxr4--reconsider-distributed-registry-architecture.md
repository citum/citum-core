---
# csl26-dxr4
title: Reconsider distributed registry architecture
status: draft
type: task
priority: low
tags:
    - registry
    - architecture
    - style
    - research
created_at: 2026-04-23T00:00:00Z
updated_at: 2026-04-25T20:20:08Z
---

# Objective
Evaluate the necessity and architectural design of a distributed registry system for Citum styles.

## Context
Currently, Citum relies on a centralized builtin registry and a local user store. While this ensures stability and ease of use for standard styles, it creates friction for organizations or publishers who wish to maintain and distribute their own collection of styles without upstreaming them or requiring users to manually install files.

## Rationale for Consideration
- **Decentralization:** Allows publishers (e.g., journals, universities) to host their own "canonical" style registries.
- **Ease of Discovery:** Users could subscribe to a remote registry URL to automatically resolve and update styles from that source.
- **Workflow Automation:** Enables CI/CD pipelines to pull specific style sets from a managed remote source.

## Key Considerations
- **Trust & Security:** Evaluating the implications of fetching and executing remote style/locale definitions.
- **Conflict Resolution:** Defining a clear hierarchy when multiple registries (builtin, local, and multiple remotes) provide the same style ID.
- **Schema & Versioning:** How to ensure remote registries remain compatible with the engine version.
- **Caching Strategy:** Balancing offline availability with the need for updates.
