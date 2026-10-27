---
# csl26-upkg
title: Improve public registry and auxiliary schema docs
status: completed
type: task
priority: normal
tags:
    - docs
    - schema
    - dx
created_at: 2026-10-27T12:54:57Z
updated_at: 2026-10-27T14:08:08Z
parent: csl26-e0av
---

## Goal

Registry, abbreviation-map, and type-map artifacts need a clear explanation of audience, lifecycle, and when integrators should use each one.

## Acceptance criteria

- [x] Add public entry points for registry and auxiliary schemas.
- [x] Provide validated configuration examples where applicable.
- [x] Explain generated type-map purpose and stability expectations.
- [x] Build, inspect, and verify the deployed pages.

## Summary of Changes

Added a public guide for current style registries, document-level abbreviation maps, and generated type-map metadata; linked it from the reference and schema landing pages; added schema-validated examples, effective-description checks, generated-reference links, and a type-map stability contract. Regenerated schemas and references with no wire-shape change, passed the full repository gate, and verified clean scratch builds, links, and desktop/mobile rendering.
