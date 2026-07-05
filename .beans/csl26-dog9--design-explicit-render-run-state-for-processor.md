---
# csl26-dog9
title: 'Design: explicit render-run state for Processor'
status: todo
type: task
priority: normal
tags:
    - state
    - rendering
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-05T12:22:47Z
parent: csl26-8m2p
---

Processor uses RefCell interior mutability (citation_numbers, cited_ids, first_note_by_id, dynamic compound maps), making &self render methods order-dependent and non-idempotent, with invariants recorded only in comments. Design an explicit per-run state object so ordering contracts are typed and Processor becomes reusable/shareable. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 6.

## Follow-up note (from csl26-qi7l, 2026-07-05)

While fixing the per-component config clones (Rc<Config>/Rc<BibliographyConfig>
in RenderOptions/Renderer/RendererResources/ProcTemplateComponent), we
deliberately chose `Rc` over `Arc` since the engine has zero threading today.
If this bean's explicit-state redesign makes `Processor` shareable across
threads, note two concrete downstream opportunities worth reconsidering then:

- The `Rc` wrapping introduced by csl26-qi7l would need to become `Arc`
  (mechanical, but touches the same call sites again).
- Bibliography entry rendering (`Renderer::process_bibliography_entry` /
  `render_group_entries`) is a natural candidate for parallelization (e.g.
  `rayon`) once per-run state (citation numbers, disambiguation hints,
  first-note tracking) is resolved read-only *before* the render phase — which
  is largely already true, but is only safe to rely on once this bean makes
  the ordering/idempotency contract explicit and typed rather than
  comment-documented.

Not planned as part of this bean's scope — recorded so it isn't lost. Only
worth doing if a real workload shows single-threaded rendering as the
bottleneck; the O(n×m) clone cost fixed by csl26-qi7l was the actual hot-path
issue, not thread count.
