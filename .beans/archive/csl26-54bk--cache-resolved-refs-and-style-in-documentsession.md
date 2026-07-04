---
# csl26-54bk
title: Cache resolved refs and style in DocumentSession
status: completed
type: task
priority: normal
tags:
    - cache
    - session
created_at: 2026-07-04T02:42:25Z
updated_at: 2026-07-04T23:38:15Z
parent: csl26-8m2p
---

Every session mutation re-parses RefsInput (full YAML/JSON parse), re-runs warning scans, rebuilds the Processor (disambiguation hints), and re-renders the whole document. Cache the resolved Bibliography (invalidate on put_references) and consider caching the resolved style. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 5.

## Summary of Changes

Eager resolution: `DocumentSession::put_references` now parses the
`RefsInput` once (returning `Result<(), DocumentSessionError>`) and caches
the resolved `Bibliography` plus the bibliography-derived warning scans
(`unknown_reference_class/field_warnings`). `render_citations` reuses the
cache — mutations no longer re-parse YAML/JSON per edit. The resolved style
was already cached (session stores `Style`, not `StyleInput`); the remaining
per-mutation cost (style/bibliography clone + Processor rebuild incl.
disambiguation hints + full re-render) is documented on the struct per the
audit. Callers updated: citum-bindings wasm session, citum-server rpc
(`put_references` now surfaces parse errors at put time; protocol table
updated). New tests: malformed-input error at put time, cache replacement on
re-put, rpc error surface.
