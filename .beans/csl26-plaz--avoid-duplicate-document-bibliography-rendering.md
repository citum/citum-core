---
# csl26-plaz
title: Avoid duplicate document bibliography rendering
status: todo
type: task
priority: normal
tags:
    - engine
    - performance
created_at: 2026-07-10T21:50:45Z
updated_at: 2026-07-10T21:54:28Z
parent: csl26-8m2p
---

`Processor::render_document_bibliography` builds content and entries in
separate render passes. In the flat `restrict_to_cited` path, content first
renders every loaded reference and `render_with_legacy_grouping` then clones
and keeps only cited entries; entries are rendered again from the cited subset.
Custom-group and partition paths likewise render content separately from flat
per-entry output.

## Checklist

- [ ] Benchmark document rendering with a large library and small cited subset
- [ ] Produce content and per-entry data from one cited, correctly ordered `ProcEntry` pass where semantics allow
- [ ] Preserve group-local templates, disambiguation, annotations, and subsequent-author substitution
- [ ] Add public-output equivalence tests for flat, custom-group, and partitioned paths

Audit: `docs/architecture/audits/2026-07-10_CITUM_ENGINE_FOLLOW_UP_REVIEW.md`

## Additional Audit Scope

Measure and, if warranted, eliminate repeated whole-bibliography sorting in
`entries_for_bibliography_group` and deep `Reference` cloning in
`build_group_local_hints`. Prefer caching the sorted ID spine per
document-facade call before widening `Disambiguator` lifetimes.
