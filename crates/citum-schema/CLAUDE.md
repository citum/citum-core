# citum-schema (facade)

This crate is a **facade**: it re-exports `citum-schema-style` and `citum-schema-data`. Real schema types live in **`crates/citum-schema-style/`** — that's where edits usually go.

## Where to edit

| Concern | Crate |
|---|---|
| Style model (`Style`, `Template`, `Options`, `Locale`, `Renderer`, `Presets`, `Registry`) | `citum-schema-style/src/` |
| Reference / data model accessors | `citum-schema-data/src/` |
| Public re-export surface | `citum-schema/src/lib.rs` (rarely changes) |

`citum-schema-style/src/lib.rs` is ~120K — use **jcodemunch** (`get_symbol`, `get_repo_outline`) to navigate it. Do not `cat` it.

## Gotchas

- **Serde-driven truth.** YAML round-trips define correctness. New fields need `#[serde(default)]` or explicit handling for back-compat.
- **Do not bump `STYLE_SCHEMA_VERSION`** in feature commits. The release workflow handles schema version bumps based on conventional commits.
- **Regenerate JSON schema** when public schema types change: `cargo run --bin citum --features schema -- schema --out-dir docs/schemas` — stage in the same commit.
- **Grouped struct pattern.** 3+ related schema fields should become a grouped struct rather than parallel scalars (see `WrapConfig` in `citum-schema-style`, `BibRefContext` in `citum-engine/src/ffi/biblatex.rs`). Keeps signatures stable and avoids `clippy::too_many_arguments`.
- **`info.source` is CSL-only.** Biblatex-derived and citum-native styles must omit `info.source` and `info.id` (no Zotero URL). Biblatex-derived: only `info.title` and `info.default-locale`.

## Type addition

Adding a new reference type or variant requires the policy in `docs/policies/TYPE_ADDITION_POLICY.md`. Read it before proposing new types.

## Symbol queries

For "what does this trait resolve to?" → **rust-analyzer**. For symbol bodies, callers, the API surface of `citum-schema-style` → **jcodemunch**.
