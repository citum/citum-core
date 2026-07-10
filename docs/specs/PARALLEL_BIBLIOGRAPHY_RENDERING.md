# Parallel Bibliography Rendering Specification

**Status:** Active
**Date:** 2026-07-09
**Supersedes:** (none)
**Related:** bean `csl26-nitz`; `docs/specs/EXPLICIT_RENDER_RUN_STATE.md` (bean `csl26-dog9`)

## Purpose

`docs/specs/EXPLICIT_RENDER_RUN_STATE.md` made the register-before-render
ordering contract for bibliography and citation rendering a compile-time
property (`FinalizedRun`), but explicitly left `Processor` single-threaded:
`Config`/`BibliographyConfig` were shared via `Rc`, and `RunState`'s
`citation_numbers`/`first_note_by_id` were `RefCell`. Both are `!Sync`, so
`Processor` and `FinalizedRun` could not cross a thread boundary.

This spec covers the follow-up: `Rc` → `Arc` for config sharing, `RefCell` →
`RwLock` for the two run-state maps, and — now that `Processor: Send + Sync`
and `FinalizedRun: Send + Sync` hold — rendering bibliography entries across
the `rayon` thread pool once a bibliography is large enough to make it worth
the dispatch overhead.

## Scope

**In scope:**

- `Arc` instead of `Rc` for `Config`/`BibliographyConfig` sharing throughout
  `citum-engine` (`Renderer`, `RendererResources`, `RenderOptions`,
  `ProcTemplateComponent`).
- `RwLock` instead of `RefCell` for `RunState::citation_numbers` and
  `RunState::first_note_by_id`, with `PoisonError::into_inner` recovery at
  every lock site (the repo forbids `unwrap`/`expect`).
- A new `parallel` cargo feature on `citum-engine`, on by default, gating a
  `rayon`-backed parallel map over bibliography entries in both rendering
  paths: `Processor::process_sorted_refs` (flat bibliographies) and
  `GroupedBibliography`'s `render_group_entries` (custom groups).
- A size threshold (`PARALLEL_MIN_ENTRIES`) below which entries render
  sequentially even with the feature on, since thread-pool dispatch has a
  fixed cost that a small bibliography can't amortize.
- Compile-time `Send + Sync` assertions for `Processor` and `FinalizedRun`.

**Explicitly out of scope:**

- Citation rendering. `render_citation_content` and the citation-cluster
  render paths stay sequential: citations are registered and rendered
  interleaved per citation (see `processor/citation.rs`'s module docs), which
  is inherently a sequential, order-dependent process. Only bibliography
  entry rendering — which reads a *closed* set of already-registered
  citation numbers — parallelizes.
- Changing citeproc semantics (subsequent-author substitution, lazy numeric
  assignment, dynamic compound grouping). This spec restructures *how*
  bibliography entries are produced, not what they produce; see the
  determinism argument below and the equality tests in
  `processor/bibliography/tests.rs`.
- Parallelizing anything in `citum-migrate`, `citum-cli`, or `citum-bindings`
  directly — those crates benefit transitively (via `citum-engine`) or, for
  `citum-bindings` (WASM, no thread pool), not at all, by design (see
  Design → Feature gate).

## Design

### Feature gate

```toml
# crates/citum-engine/Cargo.toml
[dependencies]
rayon = { version = "1", optional = true }

[features]
default = ["icu", "parallel"]
parallel = ["dep:rayon"]
```

`parallel` is on by default for native builds (CLI, server, migrate). WASM is
unaffected: `citum-bindings` already builds `citum-engine` with
`default-features = false` (there is no thread pool in a WASM target), so it
never pulls in `rayon`. `cargo check -p citum-engine --no-default-features
--features icu` and `-p citum-bindings` are part of this change's
verification precisely to keep that boundary honest — the sequential code
path is not `#[cfg]`-gated at all (it's the only path when `parallel` is
off, or below the threshold), so a build without the feature is exercised
by the same tests as one with it, just without the parallel branch.

### `PARALLEL_MIN_ENTRIES`

```rust
// processor/bibliography/mod.rs
#[cfg(feature = "parallel")]
pub(crate) const PARALLEL_MIN_ENTRIES: usize = 32;
```

Below this many entries, rendering stays sequential even with `parallel`
enabled — rayon's thread-pool dispatch has a fixed cost that isn't worth
paying for a short bibliography. The value is a conservative starting point;
`cargo bench --bench rendering` compares 10-item and 200-item bibliographies
so the crossover point can be tuned from real measurements without changing
the shape of the code.

Initial measurements (2026-07-09, 8-core desktop under sustained ~5.0 load
average from ambient desktop processes) were inconclusive: run-to-run
variance on identical binaries reached ±75% (10-item case) and the
sequential/parallel bands at 200 items overlapped entirely (5.2–6.3 ms
sequential vs 5.6–7.4 ms parallel, medians across repeated runs). Neither a
speedup nor a regression is demonstrable under that noise; tuning the
threshold — and validating the feature's value — needs a quiet machine or
citum-server's concurrent production context.

### Restructuring the two render loops

Both `Processor::process_sorted_refs` (flat bibliographies) and
`GroupedBibliography::render_group_entries` (custom groups) previously ran
one sequential loop that rendered an entry *and* applied subsequent-author
substitution in the same iteration. Both now split into three steps:

1. **Number** (`number_sorted_refs`): a sequential pass that resolves
   `(reference, entry_number)` pairs, reading `run`'s shared
   `citation_numbers` map once per reference. Always sequential — it's a
   single read pass over a shared map, not the expensive part.
2. **Render** (`render_entries` / `render_group_numbered_refs`): an
   order-preserving map from `(reference, entry_number)` to
   `Option<ProcTemplate>`. This is the parallel step: `rayon::par_iter`
   when `parallel` is enabled and `numbered_refs.len() >=
   PARALLEL_MIN_ENTRIES`, a plain iterator otherwise. `par_iter().collect()`
   over a slice preserves input order, so the sequential and parallel
   branches produce identically-ordered results.
3. **Substitution post-pass** (`apply_substitution_post_pass`): a sequential
   walk over the (order-preserved) rendered results that applies
   subsequent-author substitution and assembles `ProcEntry`s. This step
   *cannot* run in parallel: substitution depends on whether the
   **previous successfully rendered** reference's contributors match the
   current one, which is inherently a left-to-right fold. `None` results
   (entries the renderer skipped) do not advance "previous reference" — this
   matches the pre-refactor behavior exactly.

`render_group_entries`'s parallel branch additionally builds a **fresh
`Renderer` per task** (`GroupRenderContext` bundles the `Arc`-wrapped
config plus style/hints/run so each task's `Renderer::new` call is cheap)
rather than sharing one `Renderer` across threads. `Renderer` has a
per-render scratch field, `filtered_to_original_index: RefCell<...>`, that
is intentionally *not* `Sync` — it's mutated during a single entry's render
and reset per call; sharing one `Renderer` across parallel tasks would be
either a compile error or (if worked around) a race. The flat path
(`process_sorted_refs`) doesn't need this care explicitly: its `process_fn`
closures already go through `with_bibliography_renderer`, which constructs
a new `Renderer` per call.

The sequential fallback in `render_group_entries` keeps the original single
shared `Renderer` — there's no correctness reason to change it, and
avoiding per-entry `Renderer` construction below the parallel threshold
keeps the common (small-bibliography) case exactly as fast as before this
change.

### Determinism argument

Parallel bibliography rendering is deterministic because bibliography
rendering only **reads** `citation_numbers` — it never assigns a number
during bibliography rendering:

- Numeric styles pre-assign all citation numbers at `begin_run`
  (`setup.rs::initialize_numeric_bibliography_numbers`), before any
  rendering (parallel or not) begins.
- The lazy assignment path (`Renderer::get_or_assign_citation_number`,
  `rendering/grouped/core.rs:799`) only fires from *citation* rendering,
  which — per Scope — stays sequential and out of this change.

So every task in the parallel render step observes a `citation_numbers` map
that is already stable for the duration of that render. The `RwLock` exists
for `Sync`-ness and to make the *type* honest about interior mutability, not
because bibliography rendering races on writes to it.

The one property parallel rendering must preserve that the *type system*
doesn't enforce on its own is *order*: `par_iter().collect()` over a slice
is order-preserving, and the substitution post-pass consumes results
strictly in that order — see the equality tests below.

## Acceptance Criteria

- [x] `Processor: Send + Sync` and `FinalizedRun: Send + Sync`, asserted at
      compile time
      (`given_arc_config_and_rwlock_run_state_when_checked_then_processor_and_finalized_run_are_send_sync`
      in `processor/tests.rs`).
- [x] `cargo check -p citum-engine --no-default-features --features icu`
      and `-p citum-bindings` both succeed (WASM-shape build, no `rayon`).
- [x] Parallel-vs-sequential equality: `processor/bibliography/tests.rs`
      (feature-gated on `parallel`) asserts that
      `render_entries_sequential`/`render_entries_parallel` and
      `render_group_numbered_refs_sequential`/`render_group_numbered_refs_parallel`,
      followed by the same `apply_substitution_post_pass`, produce identical
      `Vec<ProcEntry>` for bibliographies above `PARALLEL_MIN_ENTRIES`, for
      both an author-date style (subsequent-author substitution exercised
      via three-entry author runs) and a numeric style (pre-assigned
      citation numbers).
- [x] `just pre-commit` passes (fmt + clippy `-D warnings` + `cargo nextest
      run`, `--workspace --all-features`).
- [x] `cargo bench --bench rendering` includes a 200-item bibliography case
      (`Process Bibliography (APA, 200 items)`) alongside the existing
      10-item one, so the parallel crossover is measurable, not just
      asserted.

## Changelog

- 2026-07-09: Initial draft; implemented in the same change (Rc→Arc,
  RunState RwLock, feature-gated rayon parallel bibliography rendering).
  Status → Active.
