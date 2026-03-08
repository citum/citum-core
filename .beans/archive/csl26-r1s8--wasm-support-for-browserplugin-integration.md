---
# csl26-r1s8
title: WASM support for browser/plugin integration
status: completed
type: feature
priority: normal
created_at: 2026-02-15T18:35:23Z
updated_at: 2026-03-06T16:11:53Z
---

WebAssembly bindings for Citum processor enabling browser, desktop plugins, and serverless deployment. See docs/architecture/design/WASM_SUPPORT.md for full architecture. Deferred until API stable (10+ parent styles working).

## Summary of Changes

Implemented via citum-bindings crate (c149b68). WASM/FFI architecture documented in docs/architecture/design/WASM_SUPPORT.md (8fcf6f4). LuaLaTeX and LuaJIT FFI bindings also added.
