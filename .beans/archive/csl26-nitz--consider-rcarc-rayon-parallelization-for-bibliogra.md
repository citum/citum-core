---
# csl26-nitz
title: Consider Rc→Arc + rayon parallelization for bibliography rendering
status: completed
type: task
priority: deferred
created_at: 2026-07-09T12:59:36Z
updated_at: 2026-07-10T11:56:10Z
parent: csl26-8m2p
---

Once csl26-dog9's FinalizedRun typestate lands (done), Rc<Config>/Rc<BibliographyConfig> (introduced in csl26-qi7l) could become Arc, and Renderer::process_bibliography_entry/render_group_entries could parallelize with rayon, since FinalizedRun makes the read-only-before-render contract typed rather than comment-documented. Out of scope for csl26-dog9 itself. Only worth doing if a real workload shows single-threaded rendering as the bottleneck — the O(n×m) clone cost fixed by csl26-qi7l was the actual hot-path issue, not thread count. See docs/specs/EXPLICIT_RENDER_RUN_STATE.md and the csl26-qi7l follow-up note recorded in csl26-dog9's body.

## Implementation Checklist (2026-07-09)

- [x] Phase 1: Rc→Arc for Config/BibliographyConfig across citum-engine
- [x] Phase 2: RunState RefCell→RwLock; Send+Sync static assertions
- [x] Phase 3: feature-gated rayon parallel bibliography rendering (opt-in feature after measurement, threshold)
- [x] Phase 4: 200-item bench case + spec doc + EXPLICIT_RENDER_RUN_STATE.md update
- [x] Review pass (Fable) + full gates (pre-commit, no-default-features build, workflow-test x2)
- [x] Bench evidence (feature off vs on, back-to-back — loaded runs inconclusive; quiet-machine runs conclusive, recorded in spec)
- [x] PR opened (#1034), CI green (no merge — user reviews)

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

## Post-review updates (2026-07-10)

Morning follow-up after user review (quiet machine, load < 1, so benches finally meaningful):

- **Copilot review fix:** number_sorted_refs took a fresh RwLock read-guard per reference; now one guard for the whole pass (also dropped a per-ref String allocation).
- **Quiet-machine benches exposed real parallel overhead:** 400 items parallel 9.69 ms vs sequential 7.95 ms (+22%). Cause: the flat path rebuilt its Renderer — full config merge + deep clone + two Arc::new — per entry; 8 threads doing that contend on the allocator.
- **Fix: hoisted per-pass EntryRenderContext** (closes the csl26-qi7l deferred item), unifying the flat and grouped render paths and deleting the process_fn closure plumbing. Result: parallel penalty gone AND sequential faster (200 items 4.99→4.36 ms).
- **Post-fix verdict: parallel is performance-neutral at 10–400 entries** (575.5 µs vs 575.8 µs / 4.42 vs 4.36 ms / 8.46 vs 8.54 ms). Entry rendering is allocation-bound, not compute-bound. Per user decision, `parallel` demoted from default feature to opt-in; equality tests run in CI via nextest --all-features. Durable wins: Send+Sync Processor (server can share one Processor across request threads without rayon) + the hoist.
- **Incidental fix:** cargo test --all-features stopped linking locally (lld duplicate-symbol on the ffi no_mangle exports via the citum-io dev-dep cycle; CI's mold tolerated it, main+lld did not extract the archive member). FFI exports now use cfg_attr(not(test), unsafe(no_mangle)).

Final verification: pre-commit 1855/1855; nextest --all-features 1872/1872; workflow-test apa.csl 20/20+45/46 and apa-numeric-superscript.csl 20/20+17/51 (both = main baselines); measurements recorded in docs/specs/PARALLEL_BIBLIOGRAPHY_RENDERING.md.
