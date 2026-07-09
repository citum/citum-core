---
# csl26-dog9
title: 'Design: explicit render-run state for Processor'
status: completed
type: task
priority: normal
tags:
    - state
    - rendering
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-09T13:00:15Z
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

## Implementation Checklist (2026-07-09)

- [x] (0) Spec: docs/specs/EXPLICIT_RENDER_RUN_STATE.md (Active)
- [x] (a) Introduce RunState wrapping existing RefCells, callers unchanged
- [x] (b)+(c) combined: Thread &mut RunState/&FinalizedRun through registration and rendering; RunState fully external to Processor (see spec's Implementation Notes for why (b)/(c) could not split — Rust borrow-checker rejects self.method(&mut self.field))
- [x] (d) FFI/WASM session plumbing (FfiSession opaque handle, ABI-preserving)
- [x] Idempotency test: test_processor_is_reusable_across_independent_runs (two begin_run calls -> identical output)
- [x] just pre-commit green (fmt + clippy -D warnings + 1854 tests, --all-features, whole workspace)
- [x] workflow-test fidelity check: apa.csl + apa-numeric-superscript.csl match byte-for-byte vs git-stash pre-refactor baseline
- [x] Spec Draft -> Active
- [x] File follow-up bean for Rc->Arc/rayon: csl26-nitz

**Spec:** docs/specs/EXPLICIT_RENDER_RUN_STATE.md (Draft, 2026-07-09)

## Summary of Changes

Implemented the full explicit render-run state refactor:

- `crates/citum-engine/src/processor/run_state.rs` (new): `RunState` (registration-phase, `&mut`) and `FinalizedRun` (render-phase, `&`) typestate. `citation_numbers`/`first_note_by_id` stay `RefCell`-wrapped inside `RunState` (lazy numeric assignment during render); the other five fields (`cited_ids`, `compound_groups`, three dynamic compound maps) are plain, mutated only via `&mut RunState`.
- `Processor` no longer owns any of the seven former `RefCell` fields. `Processor::begin_run(&self) -> RunState` is the new entry point.
- Registration (citation.rs, note_context.rs, setup.rs) takes `&mut RunState`; bibliography rendering (bibliography/{mod,grouping,compound}.rs) takes `&FinalizedRun`.
- One-shot convenience wrappers preserve the pre-refactor zero-argument call shape where no cross-call continuity is needed: `process_citation`/`process_citations` (pre-existing names, now throwaway-run), plus a new `*_standalone` family for the bibliography methods (`render_bibliography_with_format_standalone`, `render_selected_bibliography_with_format_and_annotations_standalone`, etc.) — begins a throwaway run internally.
- FFI (`ffi/mod.rs`): opaque `FfiSession { processor, run }` replaces the bare `*mut Processor` pointee; C ABI signatures unchanged. Bibliography renders clone-and-finalize a snapshot of the session's run so citation registration can continue afterward.
- `api/document.rs`, `api/session.rs`: thread one `begin_run`/`finalize` per document/render instead of relying on implicit per-`Processor` state; `DocumentSession::render_citations` no longer needs the per-render style/bibliography clone workaround.
- Updated every downstream consumer to compile against the new signatures: `citum-cli`, `citum-server`, `citum-bindings`, `citum-migrate`, plus the engine's own unit/integration tests, benches, and examples.
- Fixed two latent test bugs the refactor surfaced: `test_dynamic_group_first_occurrence_wins` and `test_dynamic_group_ungrouped_first_occurrence_wins" relied on `process_citation`'s (now-throwaway-run) cross-call accumulation for dynamic compound-group state; both now thread one explicit `RunState` across their two citations.
- Added `test_processor_is_reusable_across_independent_runs`: the headline idempotency property this refactor makes both expressible and true.

**Verification:** `just pre-commit` green (fmt + clippy `-D warnings` + 1854 tests) across the whole workspace with `--all-features`. `just workflow-test` on an author-date style (apa.csl) and a numeric/compound style (apa-numeric-superscript.csl) both match byte-for-byte against a `git stash`-isolated pre-refactor baseline — no fidelity regression.

**Follow-up filed:** csl26-nitz (Rc→Arc + rayon parallelization, deferred, gated on a real workload).
