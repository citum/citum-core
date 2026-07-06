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
updated_at: 2026-07-06T18:47:31Z
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

## Design (2026-07-06)

Verified current state: `processor/mod.rs` holds seven `RefCell` fields (citation_numbers, cited_ids, compound_groups, dynamic_compound_set_by_ref, dynamic_compound_member_index, dynamic_compound_sets, first_note_by_id) mutated from `&self` render methods.

**Shape: typestate run object, three mechanical phases.**

1. `struct RunState` owns the seven maps (plain, no RefCell). `Processor` keeps only immutable style+references+locale and gains `fn begin_run(&self) -> RunState`.
2. Registration phase: citation processing takes `&self, &mut RunState` — position assignment, citation numbers, first-note numbers, compound-set membership all happen here. Existing one-shot conveniences (`process_citation` on `&self`) remain as wrappers that create a throwaway run, preserving API compatibility for simple callers.
3. Finalization typestate: `RunState::finalize(self) -> FinalizedRun`; bibliography rendering and any output that depends on cite order take `&FinalizedRun`. This makes the ordering contract a compile error instead of a comment: you cannot render a bibliography before registration is complete, and rendering is pure/idempotent over `&FinalizedRun`.
4. FFI/WASM: the C handle becomes a boxed `(Processor, Option<RunState>, Option<FinalizedRun>)` session; existing entry points map onto begin/register/finalize internally, no ABI signature changes required in the first pass.
5. Document/session pipeline (`api/document`, `api/session`) is the primary consumer and already runs in register-then-render order — it converts first.

**Migration plan (each step green on its own):** (a) introduce RunState wrapping the existing RefCells and move the fields, callers unchanged; (b) change internal call chains to `&mut RunState` and delete the RefCells; (c) add the FinalizedRun typestate and convert bibliography entry points; (d) update FFI session plumbing. Step (a)-(b) are Sonnet-executable; (c)-(d) want a review pass.

Post-finalize, the csl26-qi7l note applies: `FinalizedRun` is where Rc→Arc + rayon over `render_group_entries` becomes safe — out of scope here, gated on a real workload.
