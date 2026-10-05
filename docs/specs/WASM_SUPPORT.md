# WebAssembly Support Specification

**Status:** Active
**Date:** 2026-10-05
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
web target, and bundle-size optimizations not yet implemented.

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
| `getStyleMetadata` | `get_style_metadata` | Resolve a style input and return metadata as JSON |
| `materializeStyle` | `materialize_style` | Resolve inheritance and template references and return canonical YAML |
| `renderCitation` | `render_citation` | Render a citation cluster from style YAML + refs JSON |
| `renderBibliography` | `render_bibliography` | Render a bibliography from style YAML + refs JSON |
| `validateStyle` | `validate_style` | Validate a YAML style; returns `Ok(())` or an error string |
| `formatDocument` | `format_document` | Full document-batch rendering from a JSON request |
| `DocumentSession` | `WasmDocumentSession` | Stateful citation and bibliography rendering |

`export_typescript` is native-only (used by the schema generation toolchain).

Every style-taking export accepts full Citum YAML or an exact selector document
with one top-level field, such as `id: apa-7th`. Selectors and aliases resolve
only when their registry entry names a compiled-in style. Remote catalog IDs
fail with instructions to pass fetched, version-pinned YAML. A top-level `id`
mixed with style fields is invalid; authored style metadata belongs at
`info.id`.

The loader resolves inheritance, presets, variants, scoped options, engine
requirements, and collapse constraints before an export uses the style.
`materializeStyle` serializes that resolved form and removes inheritance and
template references. Citation and bibliography renders fail if the authored
style lacks the requested section.

`formatDocument` accepts `StyleInput::Id` for bundled IDs and aliases and
`StyleInput::Yaml` for supplied content. URI and path inputs fail explicitly in
WASM. No binding export performs network I/O.

Locale selection is shared by one-shot formatting, sessions, and direct
renders: explicit request, style default, then `en-US`. The bundled set includes
a sparse `en-GB` locale for MHRA. Structured APIs preserve fallback warnings;
direct string-returning renders turn unresolved locale warnings into errors.

### Build pipeline

`scripts/build-jsr-package.sh` is the authoritative build script:

1. Runs `wasm-pack build crates/citum-bindings --target web --features full-wasm`.
2. Stages output under `target/jsr/citum/` alongside `README-JSR.md` (renamed
   to `README.md`) and a generated `jsr.json`.
3. `jsr.json` sets `"name": "@citum/engine"` and lists the four wasm-bindgen
   artefacts: `citum_bindings.js`, `citum_bindings.d.ts`,
   `citum_bindings_bg.wasm`, `citum_bindings_bg.wasm.d.ts`.
4. The staged artifact runs the documented APA and MHRA workflow under Node 24,
   Deno, and headless Chromium before a publish or dry run.

The `target/jsr/` directory is gitignored; the package is built fresh on every
release tag.

### Publication

`@citum/engine` is published to `jsr.io/@citum/engine` via GitHub OIDC trusted
publishing in the `publish-jsr` job of `.github/workflows/release.yml`. No JSR
token is stored in CI secrets. The publication job builds and executes the
staged artifact before it invokes JSR.

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
- [x] All public functions and `DocumentSession` export with their documented JS names.
- [x] TypeScript definitions generated and included in the JSR package.
- [x] `@citum/engine` published to JSR via trusted publishing (no stored token).
- [x] `deno add jsr:@citum/engine` installs successfully.
- [x] The documented APA and MHRA workflows run under Node 24 and Deno.
- [x] The same workflow runs in headless Chromium using the staged web target.

## Changelog

- 2026-10-05: Defined bundled selector resolution, locale selection, canonical
  materialization, missing-surface errors, and release smoke tests for Node 24,
  Deno, and Chromium.
- 2026-05-31: Rewrite. Supersedes the "Design Phase (Deferred)" document that
  proposed `csln_wasm` crate, `@csln/processor-wasm` on npm, and a three-tier
  future roadmap. Documents the shipped `@citum/engine` on JSR, the
  `citum-bindings` feature-flag model, and citum-hub wasm-bridge as a downstream
  consumer.
