---
# csl26-qi7l
title: Reduce per-component config clones in render path
status: completed
type: task
priority: normal
tags:
    - performance
    - rendering
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-05T12:02:43Z
parent: csl26-8m2p
---

Each ProcTemplateComponent carries an owned Config clone plus BibliographyConfig clone; RenderOptions/Renderer construction clones BibliographyConfig too. O(entries x components) deep clones per render pass. Borrow or Rc/Arc the configs. Watch cargo bench --bench rendering. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 9.

## Summary of Changes

Wrapped the render-hot-path config payloads in `Rc` so the per-component cost
drops from a full deep clone of `Config`/`BibliographyConfig` to a refcount
bump:

- `RenderOptions.config` / `.bibliography_config`, `Renderer.config` /
  `.bibliography_config`, `RendererResources.config` / `.bibliography_config`,
  and `ProcTemplateComponent.config` / `.bibliography_config` are now
  `Rc<Config>` / `Option<Rc<BibliographyConfig>>` instead of owned values.
- The one remaining allocation per render pass happens once at each of the 5
  `RendererResources` build sites (`processor/citation.rs`,
  `processor/bibliography/mod.rs`, `processor/bibliography/grouping.rs` (x2),
  `processor/document/notes.rs`), replacing the previous n×m deep clones
  across `ProcTemplateComponent` construction.
- `Rc`, not `Arc` — the engine is single-threaded by design (interior
  `RefCell` state per review finding 6); no `Send`/`Sync` bounds existed
  before this change either.

**Verification:**
- `just pre-commit` (fmt, clippy `-D warnings`, `cargo nextest run`): clean,
  1754/1754 workspace tests pass.
- `cargo bench --bench rendering`, fresh back-to-back baseline vs branch on
  the `Process Bibliography (APA, 10 items)` benchmark (the n×m
  entries-by-components case this bean targets): **17.7% faster**
  (777.37 µs → 613.82 µs median). Note: this machine has background load
  (~4.7 load avg on 8 cores) that produced noisy, contradictory deltas on
  unrelated benchmarks (`Disambiguator`, `GroupSorter` — files untouched by
  this diff) across separate runs; only back-to-back same-session
  comparisons on the target benchmark are treated as signal.
- `./scripts/workflow-test.sh styles-legacy/apa.csl`: identical output vs
  `main` (20/20 citations, 45/46 bibliography — the 1 failure is a
  pre-existing, unrelated oracle mismatch confirmed present on `main` too).

**Deferred (not done here, out of scope):** caching the resolved
`Rc<Config>`/`Rc<BibliographyConfig>` on `Processor` itself so the one
per-pass merge-and-wrap is also amortized across repeated renders. Overlaps
review finding 5 (session/style caching) and would add processor state.
