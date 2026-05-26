---
# csl26-4qh2
title: Extract shared bibliography loading into lower crate
status: in-progress
type: task
priority: normal
tags:
    - architecture
    - io
    - engine
created_at: 2026-05-25T14:49:00Z
updated_at: 2026-05-26T19:04:55Z
---

citum-io owns the broad bibliography loading pipeline, but it depends on citum-engine, so engine APIs such as RefsInput cannot reuse it without a dependency cycle. Extract the shared native/legacy bibliography parsing surface into a lower crate used by citum-io, citum-engine, citum-server, and bindings. Preserve CLI behavior and avoid changing public wire formats.

## Naming decision\n\nCrate: . Functions: , , , , , etc. Types: , .  and  already correctly named.  type alias rename deferred — add TODO comment at definition site.
