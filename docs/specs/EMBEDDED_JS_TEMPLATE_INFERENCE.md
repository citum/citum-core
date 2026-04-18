# Embedded JS Template Inference Specification

**Status:** Active
**Date:** 2026-04-18
**Related:** `crates/citum-migrate/src/template_resolver.rs`, `docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md`

## Purpose
Define a migrator-only embedded JavaScript runtime for live template inference so
`citum-migrate` can execute the output-driven inference path without spawning a
Node subprocess, while preserving the existing inferred-fragment cache contract
and leaving oracle/report tooling unchanged.

## Scope
In scope:
- embedded JS runtime for `citum-migrate` live inference only
- host-neutral inference core extracted from the Node wrapper
- committed generated JS bundle for Cargo builds
- backend selection via `--live-infer-backend auto|embedded|node`

Out of scope:
- replacing `oracle.js`, `report-core.js`, or other Node verification scripts
- implementing a general Node compatibility layer inside `deno_core`
- changing inferred cache filenames or fragment JSON shape
- reworking downstream fixups or XML fallback semantics

## Design
- Keep template resolution order unchanged: hand-authored, cached inferred, live inferred, XML fallback.
- Treat the live inference result as the same fragment JSON contract already consumed by `parse_fragment()`.
- Extract a pure JS inference core that accepts `styleXml`, `testItems`, `localeXml`, and `section` as inputs and performs no filesystem or process I/O.
- Keep `scripts/infer-template.js` as a thin Node wrapper that reads files, delegates to the host-neutral core, and preserves CLI behavior.
- Generate and commit `crates/citum-migrate/js/embedded-template-runtime.js` from the host-neutral core plus `citeproc-js`.
- Initialize a single `deno_core::JsRuntime` per `citum-migrate` invocation and reuse it for both bibliography and citation live inference.
- Load fixture JSON and locale XML in Rust and inject them into the embedded runtime; JS must not read from disk.
- `--live-infer-backend auto` tries embedded JS first, then Node fallback. `embedded` does not fall back to Node. `node` preserves the legacy subprocess path.

## Implementation Notes
- The embedded runtime bundle is regenerated with `node scripts/build-embedded-template-runtime.js`.
- Cache-only `--template-source inferred` remains unchanged and never triggers live inference.
- Existing Node-based style workflows remain the canonical verification surface.

## Acceptance Criteria
- [x] `citum-migrate` exposes `--live-infer-backend auto|embedded|node`.
- [x] Embedded live inference preserves existing fragment cache paths and JSON shape.
- [x] The JS inference core can run without direct filesystem access.
- [x] `auto` mode falls back from embedded JS to Node subprocess when embedded initialization or execution fails.
- [x] A committed generated bundle exists for Cargo builds.

## Changelog
- 2026-04-18: Initial version.
