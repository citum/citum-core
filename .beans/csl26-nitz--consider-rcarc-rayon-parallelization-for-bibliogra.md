---
# csl26-nitz
title: Consider Rc→Arc + rayon parallelization for bibliography rendering
status: in-progress
type: task
priority: deferred
created_at: 2026-07-09T12:59:36Z
updated_at: 2026-07-10T10:25:38Z
parent: csl26-8m2p
---

Once csl26-dog9's FinalizedRun typestate lands (done), Rc<Config>/Rc<BibliographyConfig> (introduced in csl26-qi7l) could become Arc, and Renderer::process_bibliography_entry/render_group_entries could parallelize with rayon, since FinalizedRun makes the read-only-before-render contract typed rather than comment-documented. Out of scope for csl26-dog9 itself. Only worth doing if a real workload shows single-threaded rendering as the bottleneck — the O(n×m) clone cost fixed by csl26-qi7l was the actual hot-path issue, not thread count. See docs/specs/EXPLICIT_RENDER_RUN_STATE.md and the csl26-qi7l follow-up note recorded in csl26-dog9's body.

## Implementation Checklist (2026-07-09)

- [x] Phase 1: Rc→Arc for Config/BibliographyConfig across citum-engine
- [x] Phase 2: RunState RefCell→RwLock; Send+Sync static assertions
- [x] Phase 3: feature-gated rayon parallel bibliography rendering (default feature, threshold)
- [x] Phase 4: 200-item bench case + spec doc + EXPLICIT_RENDER_RUN_STATE.md update
- [x] Review pass (Fable) + full gates (pre-commit, no-default-features build, workflow-test x2)
- [x] Bench evidence (feature off vs on, back-to-back — inconclusive under load, recorded in spec)
- [ ] PR opened, CI green (no merge — user reviews)

Plan: ~/.claude/plans/complete-bean-csl26-nitz-by-mossy-wombat.md

## Summary of Changes

Implemented on branch perf/csl26-nitz-parallel-bibliography (Sonnet subagent implementation, Fable review):

- Rc→Arc for Config/BibliographyConfig sharing throughout citum-engine (Renderer, RendererResources, RenderOptions, ProcTemplateComponent + all construction sites and tests).
- RunState.citation_numbers / .first_note_by_id: RefCell→RwLock with PoisonError::into_inner recovery at every lock site; hand-written RunState Clone (RwLock is not Clone); Processor and FinalizedRun are now Send+Sync, enforced by a compile-time assertion test.
- New default-on `parallel` feature (rayon, optional dep). Both bibliography render loops (process_sorted_refs flat path, render_group_entries grouped path) restructured into: sequential entry-numbering pass → order-preserving render map (parallel above PARALLEL_MIN_ENTRIES = 32) → sequential subsequent-author-substitution post-pass preserving previous-successful-reference semantics. Grouped parallel path builds a fresh Renderer per task (its filtered_to_original_index scratch RefCell stays intentionally non-Sync). WASM unaffected (citum-bindings builds with default-features = false; verified).
- Spec: docs/specs/PARALLEL_BIBLIOGRAPHY_RENDERING.md (Active); EXPLICIT_RENDER_RUN_STATE.md updated (RefCell→RwLock + changelog).
- Tests: 3 parallel-vs-sequential equality tests (author-date flat, numeric flat, grouped) + Send/Sync assertion; suite 1854→1858, all green.

**Verification:** just pre-commit green (fmt, clippy -D warnings, 1858 nextest); cargo check --no-default-features + citum-bindings green; workflow-test apa.csl (20/20, 45/46) and apa-numeric-superscript.csl (20/20, 17/51) both identical to documented main baselines.

**Bench evidence (the bean's gating question):** inconclusive on this machine. Sustained ~5.0 load avg (desktop apps) produced ±75% run-to-run variance on identical binaries; 200-item sequential (5.2–6.3 ms) and parallel (5.6–7.4 ms) bands overlap entirely. No speedup demonstrable, no regression demonstrable. Recorded in the spec; threshold tuning deferred to a quiet machine or citum-server production profiling. Landed per user decision (2026-07-09): default-on feature + threshold protect the common case.
