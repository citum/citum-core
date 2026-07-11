---
# csl26-u2kb
title: Cache sorted-ID spine and reduce Reference cloning in bibliography group rendering
status: todo
type: task
priority: normal
created_at: 2026-07-11T09:33:54Z
updated_at: 2026-07-11T09:33:59Z
parent: csl26-8m2p
blocked_by:
    - csl26-plaz
---

Follow-up to csl26-plaz (P2 / \"Additional Audit Scope\" from the 2026-07-10
engine follow-up audit, deferred to keep csl26-plaz focused on the P1
duplicate-render fix).

`entries_for_bibliography_group` (processor/bibliography/grouping.rs) calls
`sorted_id_stubs`, which sorts the *entire* bibliography for every
independently rendered group/block — for `g` blocks over `n` references this
approaches `g * O(n log n)`. This is the primary target: cache the sorted ID
spine once per document-facade call (e.g. per
`render_document_bibliography_blocks` invocation) instead of re-sorting per
block.

`build_group_local_hints` (same file) clones every selected `Reference` into
a temporary `Bibliography` before running group-local disambiguation, to give
`Disambiguator` a simple, safe lifetime boundary. Only widen `Disambiguator`
to accept a borrowed subset if profiling after the spine-caching fix still
shows this cloning as a measurable cost — the audit explicitly flags the
owned-bibliography approach as a defensible trade-off, not a bug.

## Checklist

- [ ] Benchmark `render_document_bibliography_blocks` with many groups over a
      large library to quantify the repeated-sort cost
- [ ] Cache the sorted ID spine per document-facade call, threaded through
      `entries_for_bibliography_group`
- [ ] Re-benchmark; only pursue a borrowed-subset `Disambiguator` API if
      `build_group_local_hints` cloning still shows up as a measurable cost
- [ ] Preserve existing group-local disambiguation/sort/heading behavior
      (regression tests already covering `render_document_bibliography_blocks`
      and `render_document_bibliography_block` must stay green)

Audit: docs/architecture/audits/2026-07-10_CITUM_ENGINE_FOLLOW_UP_REVIEW.md (P2)
