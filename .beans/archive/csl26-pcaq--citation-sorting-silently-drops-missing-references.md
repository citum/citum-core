---
# csl26-pcaq
title: Citation sorting silently drops missing references
status: completed
type: bug
priority: high
tags:
    - engine
    - correctness
created_at: 2026-07-10T21:50:45Z
updated_at: 2026-07-10T21:50:45Z
parent: csl26-8m2p
---

When `citation.sort` is configured, `sort_citation_items` used `filter_map`
and removed `CitationItem`s whose IDs were absent from the bibliography. The
low-level `process_citation` API could therefore return successful partial
output instead of `ProcessorError::ReferenceNotFound`.

## Checklist

- [x] Preserve unresolved `CitationItem`s through sorting
- [x] Add a regression test asserting `ReferenceNotFound`
- [x] Run the targeted regression test

## Summary of Changes

Citation sorting now retains unresolved items and orders them after resolved
items, allowing the renderer's documented missing-reference error path to run.
The regression test fails on the old `filter_map` behavior and passes with the
fix.
