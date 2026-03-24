# Language Bindings Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-24
**Related:** `citum-bindings` crate, `docs/specs/STYLE_REGISTRY.md`

## Purpose

Add feature-gated, multi-language type bindings for citum-core's canonical
data shapes. TypeScript is the first target (for `citum-hub`'s WASM bridge and
SvelteKit frontend); Swift and others follow via the same annotation path.

The orphan rule prevents downstream crates from implementing foreign traits on
citum-core types. Bindings must be derived in the owning crate, gated behind
optional features so native Rust consumers pay zero compilation cost.

## Scope

**In scope:**
- `bindings` feature on `citum-schema-data` and `citum-schema-style` activating
  `specta::Type` derive on all public types
- `typescript` feature on `citum-bindings` activating the specta TS exporter
- `citum bindings --typescript --out-dir <dir>` CLI subcommand for dev-time type
  generation
- Extensibility contract: adding a second language requires only a new feature
  in `citum-bindings`, no changes to schema crates

**Out of scope:**
- Engine output types (`String`-based; no typed structs cross the boundary yet)
- Runtime WASM type marshalling — wasm-bindgen handles that separately
- Distributing generated `.d.ts` files as build artifacts in CI

## Design

### Library choice: specta

Use [`specta`](https://github.com/oscartbeaumont/specta) v2.x, not `ts-rs`.

Rationale:
- `specta::Type` is language-agnostic — one derive annotation covers all
  exporters (TypeScript, Swift, and future targets)
- TypeScript and Swift exporters are both stable in specta 2.x
- `ts-rs` is TypeScript-only; adopting it would require replacing annotations
  later when Swift support is needed
- Compile overhead is zero when the `bindings` feature is not enabled

### Feature naming contract

```
citum-schema-data/bindings     → activates specta::Type derive only
citum-schema-style/bindings    → activates specta::Type derive only
                                  (also enables citum-schema-data/bindings)

citum-bindings/typescript      → activates specta + specta-typescript exporters
                                  + enables citum-schema-data/bindings
                                  + enables citum-schema-style/bindings
citum-bindings/swift           → (future) activates specta-swift exporter
                                  + enables same schema bindings features
```

The schema crates expose a single `bindings` feature regardless of target
language. `citum-bindings` is the only crate that knows about specific
exporters.

### Cargo changes

```toml
# citum-schema-data/Cargo.toml
[dependencies]
specta = { version = "2.0.0-rc.23", features = ["derive"], optional = true }

[features]
bindings = ["dep:specta"]

# citum-schema-style/Cargo.toml
[dependencies]
specta = { version = "2.0.0-rc.23", features = ["derive"], optional = true }

[features]
bindings = ["dep:specta", "citum_schema_data/bindings"]

# citum-bindings/Cargo.toml
[dependencies]
specta = { version = "2.0.0-rc.23", features = ["collect"], optional = true }
specta-typescript = { version = "0.0.10", optional = true }

[features]
typescript = [
  "dep:specta",
  "dep:specta-typescript",
  "citum-schema/bindings",   # propagates to schema-data + schema-style
]

# citum-cli/Cargo.toml
[dependencies]
citum-bindings = { path = "../citum-bindings", optional = true }

[features]
typescript = ["dep:citum-bindings", "citum-bindings/typescript"]
```

### Source annotation pattern

Mirror the existing `schema`/`schemars` pattern already in both schema crates:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub struct InputReference { ... }
```

### Types requiring annotation

**`citum-schema-data`** (~12 types):
`InputReference`, `Citation`, `CitationItem`, `CitationLocator`,
`CitationMode`, `LocatorType`, `LocatorValue`, `LocatorSegment`,
`Contributor`, `StructuredName`, `SimpleName`, `FlatName`

**`citum-schema-style`** (~25–30 types):
All public `Style`, `Template`, and options structs exposed through
`citum-schema`.

### Export function

In `citum-bindings/src/lib.rs` under `#[cfg(feature = "typescript")]`:

```rust
pub fn export_typescript(out_path: &std::path::Path)
    -> Result<(), specta_typescript::Error>
{
    let types = TypeCollection::default()
        .register::<InputReference>()
        .register::<InputBibliography>()
        .register::<Citation>()
        // ... all annotated schema types
    Typescript::default().export_to(out_path, &types)
}
```

### Output path

`citum bindings --out-dir <dir>` writes to `<dir>/citum.d.ts`. The file is
intended for developer use (copy into your project's `src/lib/` or check in to
your own repo) and is **not** committed to `citum-core`.

### CLI subcommand

```
citum bindings --out-dir <dir>
```

Implemented in `citum-cli`, feature-gated behind the `typescript` feature
(`citum-bindings/typescript` transitively).

## Implementation Notes

- Pin specta to a specific minor version to avoid churn from its active
  development cycle.
- The `citum-bindings` crate has `crate-type = ["cdylib", "rlib"]`. The
  `generate_typescript` function is `rlib`-only (not exported via cdylib); add
  `#[cfg(not(target_arch = "wasm32"))]` guard if needed.
- specta's `specta-typescript` crate name on crates.io should be verified
  before adding to `Cargo.toml`.
- Generated `.d.ts` output is for developer use, not committed to the repo.

## Acceptance Criteria

- [x] `cargo check -p citum-schema-data --features bindings` passes
- [x] `cargo check -p citum-schema-style --features bindings` passes
- [x] `cargo check -p citum-bindings --features typescript` passes
- [x] `cargo check -p citum --features typescript` passes
- [x] `cargo build --workspace` (no extra features) is unchanged — zero cost
- [x] Adding a second language later requires only a new feature in
      `citum-bindings`, no changes to schema crates

## Changelog

- v1.0 (2026-03-24): Initial draft.
