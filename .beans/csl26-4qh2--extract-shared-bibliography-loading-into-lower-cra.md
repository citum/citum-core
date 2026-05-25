---
# csl26-4qh2
title: Extract shared bibliography loading into lower crate
status: todo
type: task
priority: normal
tags:
    - architecture
    - io
    - engine
created_at: 2026-05-25T14:49:00Z
updated_at: 2026-05-25T14:49:00Z
---

citum-io owns the broad bibliography loading pipeline, but it depends on citum-engine, so engine APIs such as RefsInput cannot reuse it without a dependency cycle. Extract the shared native/legacy bibliography parsing surface into a lower crate used by citum-io, citum-engine, citum-server, and bindings. Preserve CLI behavior and avoid changing public wire formats.
