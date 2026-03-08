---
# csl26-p0dc
title: 'Phase 0: Decouple csln_core from csl_legacy and biblatex'
status: completed
type: refactor
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-02-22T00:00:00Z
blocking:
    - csl26-modz
---

`csln_core` now has zero legacy deps in its default configuration.

## What was done

* `csl_legacy` made optional, gated behind new `legacy-convert` feature in
  `csln_core/Cargo.toml`. The `From<csl_legacy::...>` trait impls remain in
  `csln_core` (Rust orphan rule requires this — `InputReference` is defined
  there), but only compile in when the feature is active.
* `biblatex` removed from `csln_core` entirely. `InputReference::from_biblatex`
  and the `from_biblatex_persons` helper moved to `csln_processor/src/ffi.rs`
  as free functions — the only call site at the time of the change.
* `csln_processor` activates `legacy-convert` on `csln_core`; all existing
  call sites in `io.rs`, `ffi.rs`, and test files continue to work.

## Follow-on work

`from_biblatex` in `ffi.rs` is an interim placement. The right long-term home
is a dedicated IO or reference-conversion module within `csln_processor` (not
the FFI layer). This can move when the processor gains a cleaner `io/` or
`reference/conversion.rs` structure.

Refs: csl26-modz, docs/architecture/CITUM_MODULARIZATION.md
