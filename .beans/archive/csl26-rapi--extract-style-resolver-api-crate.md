---
# csl26-rapi
title: Extract StyleResolver and related types to a dedicated API crate
status: completed
type: task
priority: normal
created_at: 2026-05-09T00:00:00Z
updated_at: 2026-05-09T00:00:00Z
---

Currently, style resolution uses a "two-trait bridge" to avoid cyclic dependencies 
between `citum-schema-style` and `citum_store`. This adds mental overhead and 
boilerplate as both crates define and implement their own version of a 
`StyleResolver` trait.

## Scope

- Create a new, lightweight `citum-resolver-api` (or `citum-resolve`) crate 
  with minimal dependencies.
- Move the `StyleResolver` trait, `ResolverError`, `ResolutionError`, and 
  related types (like `ResolverCache` interfaces) into this crate.
- Update `citum-schema`, `citum-engine`, and `citum_store` to import the 
  canonical traits from the new API crate.
- Ensure the API crate is zero-dependency (or near-zero) to facilitate 
  integration by third-party tool authors who don't need the full store logic.

## Rationale

A dedicated API crate provides a single source of truth for resolution 
interfaces. It simplifies the implementation of remote resolvers (HTTP, Git, CID) 
and makes the engine more extensible for external integrators who need to 
provide custom style resolution logic without pulling in the heavy dependency 
tree of `citum_store`.
