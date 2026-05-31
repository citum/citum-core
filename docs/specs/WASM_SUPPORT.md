# WebAssembly Support Specification

**Status:** Active
**Date:** 2026-05-31
**Supersedes:** previous "Design Phase (Deferred)" architecture document
**Related:** `crates/citum-bindings/`, `scripts/build-jsr-package.sh`,
  `.github/workflows/release.yml`, `RELEASING.md`

## Purpose

Document Citum's shipped WebAssembly and TypeScript publication strategy. Citum
exposes its citation engine to JavaScript/TypeScript consumers via a wasm-bindgen
build of `crates/citum-bindings`, published as `@citum/engine` on JSR. This spec
is the design authority for the public JS/TS API surface, the build pipeline, and
the downstream integration point in citum-hub.

## Scope

**In scope:** the `citum-bindings` wasm feature set, exported JS API, build
pipeline (`build-jsr-package.sh`), JSR publication, and the citum-hub wasm-bridge
as a downstream reference consumer.

**Out of scope:** native FFI bindings (citum-labs), WASM runtimes other than the
web target, bundle-size optimizations not yet implemented, and browser integration
testing infrastructure.

## Design

### Feature flags

`crates/citum-bindings/Cargo.toml` exposes three composable WASM feature flags:

| Feature | Contents |
|---|---|
| `wasm` | wasm-bindgen bindings only; minimal bundle |
| `small-wasm` | alias for `wasm` |
| `full-wasm` | `wasm` + `icu` (locale-aware collation via ICU4X) |

The JSR release build uses `full-wasm` for correct Unicode collation. Consumers
wanting a smaller bundle can use `small-wasm` and accept ASCII-only collation
fallback.

### Exported JavaScript API

All public functions are feature-gated with
`#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "..."))]`
in `crates/citum-bindings/src/lib.rs`:

| JS name | Rust fn | Description |
|---|---|---|
| `getStyleMetadata` | `get_style_metadata` | Parse a YAML style and return metadata as JSON |
| `materializeStyle` | `materialize_style` | Resolve inheritance and return a fully-materialized style JSON |
| `renderCitation` | `render_citation` | Render a citation cluster from style YAML + refs JSON |
| `renderBibliography` | `render_bibliography` | Render a bibliography from style YAML + refs JSON |
| `validateStyle` | `validate_style` | Validate a YAML style; returns `Ok(())` or an error string |
| `formatDocument` | `format_document` | Full document-batch rendering from a JSON request |

`export_typescript` is native-only (used by the schema generation toolchain).

### Build pipeline

`scripts/build-jsr-package.sh` is the authoritative build script:

1. Runs `wasm-pack build crates/citum-bindings --target web --features full-wasm`.
2. Stages output under `target/jsr/citum/` alongside `README-JSR.md` (renamed
   to `README.md`) and a generated `jsr.json`.
3. `jsr.json` sets `"name": "@citum/engine"` and lists the four wasm-bindgen
   artefacts: `citum_bindings.js`, `citum_bindings.d.ts`,
   `citum_bindings_bg.wasm`, `citum_bindings_bg.wasm.d.ts`.

The `target/jsr/` directory is gitignored; the package is built fresh on every
release tag.

### Publication

`@citum/engine` is published to `jsr.io/@citum/engine` via GitHub OIDC trusted
publishing in the `publish-jsr` job of `.github/workflows/release.yml`. No JSR
token is stored in CI secrets; publication is gated on a successful `build` job.

```bash
# Install
deno add jsr:@citum/engine
```

### citum-hub wasm-bridge (downstream reference)

`citum-hub/server/crates/wasm-bridge` is a Hub-specific adapter crate that
depends on `citum-bindings` (with `wasm` + `legacy-convert` features) and the
Hub's `intent-engine`. It is built with `wasm-pack --target nodejs` for the Hub
server and exposes three additional Hub-specific functions: `decide`,
`generate_style`, `render_intent_citation`. It is **not** part of the public
`@citum/engine` API.

## Implementation Notes

- `wasm-opt` is disabled in the citum-hub wasm-bridge release profile but
  enabled with size flags in `citum-bindings`
  (`-Oz --enable-bulk-memory --enable-simd --strip-debug`).
- TypeScript definitions are generated automatically by wasm-bindgen and
  included in the JSR package.
- `full-wasm` includes ICU4X for locale-aware bibliography sorting; `small-wasm`
  falls back to bytewise comparison.

## Acceptance Criteria

- [x] `crates/citum-bindings` compiles to `wasm32-unknown-unknown` with
      `full-wasm` feature.
- [x] All six JS API functions exported with correct camelCase JS names.
- [x] TypeScript definitions generated and included in the JSR package.
- [x] `@citum/engine` published to JSR via trusted publishing (no stored token).
- [x] `deno add jsr:@citum/engine` installs successfully.
- [ ] End-to-end integration test: `formatDocument` called from Deno with a real
      style + bibliography returns correct output.
- [ ] Browser integration tests (Chrome, Firefox, Safari) via web target.

## Changelog

- 2026-05-31: Rewrite. Supersedes the "Design Phase (Deferred)" document that
  proposed `csln_wasm` crate, `@csln/processor-wasm` on npm, and a three-tier
  future roadmap. Documents the shipped `@citum/engine` on JSR, the
  `citum-bindings` feature-flag model, and citum-hub wasm-bridge as a downstream
  consumer.
