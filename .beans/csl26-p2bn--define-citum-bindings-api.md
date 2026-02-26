---
# csl26-p2bn
title: 'Phase 2: Define citum-bindings public API surface'
status: todo
type: feature
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-02-22T00:00:00Z
blocking:
    - csl26-modz
    - csl26-p1rn
---

Create a `citum-bindings` crate as a thin wrapper over `citum-engine` with
a stable public API for cross-language interop.

## API Surface (minimal)

Expose only:
* `render_citation(style, refs, cite_keys) -> String`
* `render_bibliography(style, refs) -> Vec<String>`
* `validate_style(style_yaml) -> Result<(), Vec<String>>`

No internal types should leak through the public boundary.

## Feature Flags

* `wasm` feature flag with `wasm-bindgen` gated behind it
* Existing `cdylib` setup in `citum_engine` serves as the interim until
  this crate stabilises

## Defer

FFI binding generation (boltffi or similar) is deferred until the engine
API is stable. Do not introduce until Phase 4 at earliest.

Refs: csl26-modz, docs/architecture/CITUM_MODULARIZATION.md
