---
# csl26-8gb2
title: Add local WASM build recipe and optional JSR download to benchmark-wasm-workflow.js
status: completed
type: task
priority: normal
tags:
    - tooling
    - wasm
created_at: 2026-07-20T21:25:26Z
updated_at: 2026-07-20T21:36:37Z
---

scripts/benchmark-wasm-workflow.js requires a manually-built nodejs-target WASM artifact with no just recipe to build it, and can't fall back to the prebuilt @citum/engine package already published on JSR. Add: (1) just build-wasm-nodejs recipe wrapping wasm-pack build --target nodejs --features full-wasm, matching what the script already expects at crates/citum-bindings/pkg/. (2) An opt-in --source=jsr flag on the benchmark script that downloads @citum/engine from JSR's npm-compatible registry into a scratch dir and runs the benchmark against it (web-target init() called with the wasm bytes directly, bypassing fetch). Default --source=local behavior (and its error message) stays offline; --source=jsr is the only path that touches the network.

## Todo
- [x] Add build-wasm-nodejs recipe to justfile
- [x] Update missing-artifact error message in benchmark-wasm-workflow.js
- [x] Add --source arg parsing (local default / jsr)
- [x] Implement JSR download + web-target init path
- [x] Verify: just build-wasm-nodejs works
- [x] Verify: default (local) missing-artifact path stays offline
- [x] Verify: --source jsr path runs end to end

## Summary of Changes

- Added `just build-wasm-nodejs` recipe (wraps `wasm-pack build crates/citum-bindings --target nodejs --features full-wasm`).
- Added `--source local|jsr` to benchmark-wasm-workflow.js. `jsr` downloads `@jsr/citum__engine` from npm.jsr.io into a disposable tmp prefix (no repo package.json/lockfile touched) and loads the web-target ESM build via dynamic import, initializing wasm-bindgen with the raw bytes (bypassing fetch). Default stays fully offline with a clearer error pointing at both options.
- Extra (raised mid-task, not in original scope): added `--mode stateless|session`. `session` drives the script through the stateful `DocumentSession` API (parse style/refs once, `insert_citation` incrementally) instead of re-parsing style+refs JSON on every call — a more realistic word-processor simulation. Note: `DocumentSession.insert_citation` always re-renders the bibliography internally (no incremental re-render support yet), so in session mode `get_bibliography()` timings reflect a cached read, not a fresh render.
- Verified all four source/mode combinations locally (local/jsr x stateless/session) plus the offline missing-artifact error path.
