---
# csl26-54bk
title: Cache resolved refs and style in DocumentSession
status: todo
type: task
priority: normal
tags:
    - cache
    - session
parent: csl26-8m2p
created_at: 2026-07-04T02:42:25Z
updated_at: 2026-07-04T17:49:02Z
---

Every session mutation re-parses RefsInput (full YAML/JSON parse), re-runs warning scans, rebuilds the Processor (disambiguation hints), and re-renders the whole document. Cache the resolved Bibliography (invalidate on put_references) and consider caching the resolved style. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 5.
