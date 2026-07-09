# Explicit Render-Run State for `Processor` Specification

**Status:** Active
**Date:** 2026-07-09
**Supersedes:** (none)
**Related:** bean `csl26-dog9`; `docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md` finding 6; follow-up note in bean `csl26-qi7l`

## Purpose

`Processor` (`crates/citum-engine/src/processor/mod.rs`) currently holds seven
interior-mutable (`RefCell`) fields — `citation_numbers`, `cited_ids`,
`compound_groups`, `dynamic_compound_set_by_ref`,
`dynamic_compound_member_index`, `dynamic_compound_sets`, and
`first_note_by_id` — that are mutated from `&self` render methods. Rendering
therefore becomes an implicit state machine: bibliography output depends on
which citations were processed first, and processing the same citation list
twice through one `Processor` can produce different output (dynamic compound
groups are first-occurrence-wins). This is intentional citeproc semantics, but
today the ordering contract lives only in comments (e.g.
`processor/citation.rs:178-180`, "must be called before
`track_cited_ids_and_init_numbers`"). It also makes `Processor` effectively
single-threaded, since `RefCell` is not `Sync`.

This spec defines an explicit per-run state object, `RunState`, plus a
`FinalizedRun` typestate, so that:

- `Processor` becomes an immutable, reusable, shareable description of a style
  + bibliography + locale.
- The register-before-render ordering contract is enforced by the type system
  rather than by doc comments.
- A `Processor` can be rendered against fresh state repeatedly without cloning
  the whole style/bibliography per render (the workaround
  `DocumentSession::render_citations` currently uses,
  `api/session.rs:367`, specifically to reset state).

## Scope

**In scope:**

- Introducing `RunState` to own the seven currently-`RefCell` fields.
- A `FinalizedRun` typestate gating bibliography/citation-number-dependent
  rendering.
- Updating all internal call sites (`processor/citation.rs`,
  `processor/note_context.rs`, `processor/setup.rs`,
  `processor/bibliography/{mod,grouping,compound}.rs`,
  `processor/rendering/{mod,collapse,grouped/core}.rs`) to the new shape.
- Updating `api/session.rs` and `api/document.rs` to the
  begin-run/register/finalize flow.
- Updating the FFI (`ffi/mod.rs`) and WASM (`citum-bindings`) surfaces to an
  opaque session handle with no C ABI signature changes.

**Explicitly out of scope:**

- Changing `Rc<Config>` / `Rc<BibliographyConfig>` to `Arc` (tracked as a
  follow-up in bean `csl26-qi7l`; only worth doing once a real workload shows
  single-threaded rendering as a bottleneck).
- Parallelizing bibliography entry rendering with `rayon` (same follow-up;
  `FinalizedRun` is a prerequisite, not a trigger, for that work).
- Changing the citeproc semantics themselves (first-occurrence-wins dynamic
  grouping, lazy numeric assignment) — this spec makes the *existing*
  semantics typed, not different.

## Design

### `RunState` and `FinalizedRun`

```rust
/// Mutable per-render-run state: citation numbering, cite-order tracking,
/// and dynamic (cite-time) compound-group membership.
///
/// Constructed via `Processor::begin_run`. Registration methods
/// (`&self, &mut RunState`) populate it in citation-processing order;
/// `finalize` then produces a `FinalizedRun` for rendering.
pub struct RunState {
    cited_ids: HashSet<String>,
    compound_groups: IndexMap<usize, Vec<String>>,
    dynamic_compound_set_by_ref: HashMap<String, String>,
    dynamic_compound_member_index: HashMap<String, usize>,
    dynamic_compound_sets: IndexMap<String, Vec<String>>,
    // Kept RefCell internally: the render layer (`Renderer`) lazily assigns
    // citation numbers and first-note numbers *during* rendering (see
    // "Idempotency" below). The typestate boundary still enforces that
    // registration is complete before a `FinalizedRun` can exist.
    citation_numbers: RefCell<HashMap<String, usize>>,
    first_note_by_id: RefCell<HashMap<String, u32>>,
}

/// A `RunState` that has completed the registration phase. Rendering
/// methods take `&FinalizedRun` instead of `&RunState` so "render before
/// registration is complete" is a compile error.
pub struct FinalizedRun(RunState);
```

`Processor` keeps only immutable, construction-time data: `style`,
`bibliography`, `locale`, `default_config`, `hints`, and the three *static*
compound maps (`compound_sets`, `compound_set_by_ref`,
`compound_member_index` — derived once from style-declared compound sets, not
cite-time state).

`Processor::begin_run(&self) -> RunState` constructs a fresh, empty run and
performs numeric pre-initialization (`initialize_numeric_citation_numbers`,
`initialize_numeric_bibliography_numbers`) using the processor's immutable
data.

### Registration phase

Methods that populate per-run state take `(&self, run: &mut RunState)`:
`track_cited_ids_and_init_numbers`, `resolve_dynamic_group`,
`register_nocite_ids`, `normalize_note_context`. `annotate_positions` is
unaffected — it only mutates its `&mut [Citation]` argument, not processor or
run fields.

One-shot convenience methods (`process_citation`, `process_citations`) remain
available on `&self` for simple callers: internally they call `begin_run`,
register, finalize, and render against a throwaway run, preserving today's
API surface for callers that don't need cross-call state.

### Finalization and rendering

`RunState::finalize(self) -> FinalizedRun` is a plain newtype wrap — no
additional computation; it exists purely as a compile-time marker that
registration for this run is considered complete.

Render methods (`render_citation_content`, and everything under
`processor/bibliography/`) take `(&self, run: &FinalizedRun)`. `Renderer::new`
sources `citation_numbers: &run.0.citation_numbers` and
`first_note_by_id: Some(&run.0.first_note_by_id)` — `Renderer`'s existing
`&'a RefCell<...>` field types are unchanged, only their origin moves from
`Processor` to `RunState`.

### Idempotency, precisely stated

The render phase is **idempotent under repeated calls with the same
`FinalizedRun`**, and **pure with respect to the five non-`RefCell` run
fields** (`cited_ids`, `compound_groups`, and the three dynamic compound
maps). It is *not* strictly read-only for `citation_numbers`:
`Renderer::get_or_assign_citation_number` (`rendering/grouped/core.rs:799`)
lazily assigns a citation number the first time a reference is rendered, if
one is not already present. This assignment is monotonic and
assign-once-per-id — rendering the same `FinalizedRun` twice, or rendering
citations and then the bibliography from the same run, produces stable,
consistent numbers. It is *not* safe to construct two `FinalizedRun`s
concurrently over shared mutable numbering state; that is why
`citation_numbers` stays behind a `RefCell` inside `RunState` rather than
becoming a plain field read via `&FinalizedRun` — the compile-time contract
this spec adds is "registration precedes rendering," not "rendering never
touches interior state."

### FFI / WASM

The C ABI's opaque `*mut Processor`-shaped handle is preserved; only its
pointee changes:

```rust
struct FfiSession {
    processor: Processor,
    run: RunState,
}
```

`citum_processor_new*` allocates an `FfiSession` (processor +
`processor.begin_run()`) and returns it cast through the existing opaque
pointer type. `citum_render_citation_*` registers into `session.run` before
rendering (via a throwaway `FinalizedRun` snapshot, or a `&FinalizedRun`
view — implementation detail, not an ABI change).
`citum_render_bibliography_*` finalizes and renders. No C header signature
changes. `citum-bindings` (WASM) mirrors the same handle shape.

### `api/session.rs` and `api/document.rs`

`DocumentSession::render_citations` currently builds a *fresh*
`Processor::new(style.clone(), bibliography_cache.clone())` per render call
specifically to get clean run state (`api/session.rs:367`, comment: "each
render still clones the style and bibliography into a fresh processor").
After this change, the session can hold one `Processor` across renders and
call `begin_run()` per render instead — removing the per-render style/
bibliography clone. `api/document.rs`'s single-pass pipeline
(`Processor::new` → `register_nocite_ids` → `process_citations_with_format`
→ `render_document_bibliography`) converts to
`begin_run` → register → `finalize` → render in the same order it already
runs in.

## Implementation Notes

Migration proceeds in four phases, each independently green
(`just pre-commit`):

- **(a)** Introduce `RunState` (new `processor/run_state.rs`) wrapping the
  existing seven fields; `Processor` holds a transitional `RunState` (or
  `RefCell<RunState>`) so no `&self` signatures change yet. Pure relocation.
- **(b)** Thread `&mut RunState` through registration methods; delete the
  transitional field from `Processor`; add `begin_run`.
- **(c)** Add the `FinalizedRun` typestate; convert bibliography and citation
  render entry points to `&FinalizedRun`. This is where the ordering contract
  becomes a compile error.
- **(d)** FFI/WASM session plumbing (`FfiSession`, ABI-preserving).

Steps (a)-(b) are mechanical (mostly signature threading). Steps (c)-(d)
touch cross-crate boundaries (FFI/WASM) and want a human review pass.

**Deviation from the phase split above (discovered during implementation):**
Rust's borrow checker rejects `self.method(&mut self.run_state)` — a method
receiver (`&self`) and a mutable borrow of one of its own fields cannot be
passed as two arguments to the same call, even though they are logically
disjoint. This means `Processor` cannot hold `RunState` as a field *and* have
its methods take `&mut RunState` as an explicit parameter; the two are
mutually exclusive. Phases (b) and (c) were therefore implemented together as
one change: `RunState` was removed from `Processor` entirely (not left as a
"transitional field") in the same step that threaded `&mut RunState` through
registration methods and introduced `FinalizedRun`. Every caller (FFI,
`api/session.rs`, `api/document.rs`, CLI, server, `citum-migrate`, and both
the in-crate and integration test suites) had to be updated in that same
change to keep compiling, so the four-phase commit split became one
comprehensive pass instead. Convenience wrappers (`process_citation`,
`process_citations`, and a `*_standalone` family added for the bibliography
methods) preserve the pre-refactor zero-argument call shape for the ~150
call sites that don't need cross-call continuity.

## Acceptance Criteria

- [x] `Processor` no longer declares any of the seven `RefCell` fields;
      they live on `RunState`.
- [x] `RunState::finalize(self) -> FinalizedRun` exists; render methods that
      depend on cite order or citation numbers take `&FinalizedRun`, not
      `&self`-only.
- [x] It is a compile error to call a bibliography/citation-number-dependent
      render method before `finalize()` — i.e. there is no path to construct
      a `FinalizedRun` other than through `RunState::finalize`.
- [x] A test demonstrates running the same citation list twice through one
      reused `Processor` with two independent `begin_run()` calls and
      asserts identical output (the property that was previously
      impossible to state, let alone test) —
      `test_processor_is_reusable_across_independent_runs` in
      `crates/citum-engine/src/processor/tests.rs`.
- [x] `cargo nextest run` and `just pre-commit` pass (fmt + clippy
      `-D warnings` + 1854 tests, `--workspace --all-features`).
- [x] No oracle/fidelity regression: `just workflow-test styles-legacy/apa.csl`
      (author-date, 20/20 citations, 45/46 bibliography) and
      `styles-legacy/apa-numeric-superscript.csl` (numeric/compound, 20/20
      citations, 17/51 bibliography) both match byte-for-byte against a
      `git stash`-isolated pre-refactor baseline — the shortfalls are
      pre-existing migration-fidelity gaps, unchanged by this refactor.
- [x] FFI/WASM C ABI signatures are unchanged: `ffi/mod.rs` now boxes an
      opaque `FfiSession { processor, run }` behind the same pointer shape;
      `citum_render_citation_*` mutates the session's `run`,
      `citum_render_bibliography_*` renders from a cloned, finalized
      snapshot so accumulation across calls on one handle is preserved.

## Changelog

- 2026-07-09: Initial draft.
- 2026-07-09: Implemented (phases a-d, see Implementation Notes for the
  phase-split deviation); Status → Active.
