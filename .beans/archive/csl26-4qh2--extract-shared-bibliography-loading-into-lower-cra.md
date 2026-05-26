---
# csl26-4qh2
title: Extract shared bibliography loading into lower crate
status: completed
type: task
priority: normal
tags:
    - architecture
    - io
    - engine
created_at: 2026-05-25T14:49:00Z
updated_at: 2026-05-26T19:25:12Z
---

citum-io owns the broad bibliography loading pipeline, but it depends on citum-engine, so engine APIs such as RefsInput cannot reuse it without a dependency cycle. Extract the shared native/legacy bibliography parsing surface into a lower crate used by citum-io, citum-engine, citum-server, and bindings. Preserve CLI behavior and avoid changing public wire formats.

## Naming decision\n\nCrate: . Functions: , , , , , etc. Types: , .  and  already correctly named.  type alias rename deferred — add TODO comment at definition site.

## Summary of Changes

Created  crate (no engine dependency) with:
- , , , 
- , , , 
-  (YAML/JSON/CBOR), , ,  (stub)

Wired into:
- : ;  delegates to citum-refs parsers
- : all load functions delegate to citum-refs; serialize/output stays local
-  + : dependency added for future use

Naming follows Refs* convention; TODO comment at  marks the deferred  type-alias rename.

PR: https://github.com/citum/citum-core/pull/815
