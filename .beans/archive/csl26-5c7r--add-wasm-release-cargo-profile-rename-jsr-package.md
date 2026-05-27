---
# csl26-5c7r
title: Add wasm-release Cargo profile + rename JSR package to @citum/engine
status: completed
type: task
priority: normal
created_at: 2026-05-27T18:33:17Z
updated_at: 2026-05-27T18:37:04Z
---

Add a dedicated [profile.wasm-release] (opt-level=z, codegen-units=1) to reduce WASM binary size from ~6 MB. Update citum-bindings wasm-pack metadata, build-jsr-package.sh, and README-JSR.md. Rename JSR package from @citum/citum to @citum/engine.

## Summary of Changes

- Added `[profile.wasm-release]` to workspace Cargo.toml with `opt-level = "z"` and `codegen-units = 1` (inherits `lto = "fat"`, `panic = "abort"`, `strip = true` from release).
- Added `[package.metadata.wasm-pack.profile.wasm-release]` to citum-bindings with the same `-Oz` wasm-opt flags.
- Updated `scripts/build-jsr-package.sh` to use `--profile wasm-release` instead of `--release`.
- Renamed JSR package from `@citum/citum` to `@citum/engine` in build script and README-JSR.md.
