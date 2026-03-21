# Style Registry Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-20
**Related:** bean `csl26-j5rz`, `docs/architecture/design/STYLE_ALIASING.md`

## Purpose

Replace the hardcoded `EMBEDDED_STYLE_ALIASES` and `EMBEDDED_STYLE_NAMES`
slices in `citum-schema-style` with a serde-driven `StyleRegistry` type backed
by a YAML data file. The registry is the discovery and alias-resolution layer
that sits in front of the embedded style loader — it does not replace `StylePreset`
(the inheritance/composition mechanism).

## Scope

**In scope:**
- `StyleRegistry` and `RegistryEntry` types in `citum-schema-style`
- `registry/default.yaml` — the embedded default registry shipped with the crate
- Three-layer runtime resolution: local file → embedded default → filesystem
- `citum registry list` and `citum registry resolve <name>` CLI subcommands
- JSON Schema output via `citum schema registry`

**Out of scope:**
- Remote/URL-based style fetching (citum-hub concern)
- `StylePreset` preset-inheritance mechanism (unchanged)
- Per-style metadata beyond what is needed for resolution and display
- Bulk registration of CSL 1.0 dependent styles (citum-hub concern)

## Design

### Data Model

```yaml
# registry/default.yaml
version: "1"
styles:
  - id: apa-7th
    aliases: [apa]
    builtin: apa-7th
    description: "APA 7th edition"
    fields: [psychology, social-science]
  - id: modern-language-association
    aliases: [mla]
    builtin: modern-language-association
    description: "Modern Language Association"
    fields: [humanities]
```

A `RegistryEntry` has:
- `id` (String, required) — canonical style name, matches the key used in `get_embedded_style`
- `aliases` (Vec<String>, default empty) — short names that resolve to this entry
- `builtin` (Option<String>) — refers to an embedded style; present for the default registry
- `path` (Option<PathBuf>) — relative path to a YAML file; used in local registries
- `description` (Option<String>)
- `fields` (Vec<String>, default empty) — subject/domain classification

Exactly one of `builtin` or `path` must be present per entry (validated at load time).
A future `url` variant is reserved for hub registries.

A `StyleRegistry` contains:
- `version` (String)
- `styles` (Vec<RegistryEntry>)
- Methods: `resolve(name) -> Option<&RegistryEntry>`, `all_ids() -> impl Iterator<Item=&str>`,
  `merge_over(other: &StyleRegistry) -> StyleRegistry` (self wins on conflict)

### Three-Layer Resolution

```
[1] Local registry   — $CITUM_REGISTRY | ./citum-registry.yaml | ~/.config/citum/registry.yaml
       ↓ merge (local wins)
[2] Embedded default — include_bytes!("../../../../styles/registry.yaml")
       ↓ fallback
[3] Filesystem       — styles/<name>.yaml relative to cwd (unchanged existing behaviour)
```

The resolver merges layer 1 over layer 2 at startup (if a local file exists). Layer 3
is unchanged — it applies only when neither registry layer resolves the name.

### CLI Subcommands

`citum registry list` — tabular output mirroring the existing `citum styles` command, but
sourced from the merged registry. Includes `--format json` flag.

`citum registry resolve <name>` — print the resolved `id` and source (`builtin` or `path`)
for a given name or alias. Exit 1 if unresolvable.

The existing `citum styles` command remains a distinct command that lists
embedded styles directly. It does not act as an alias for `citum registry list`.

## Implementation Notes

- Keep `get_embedded_style(name)` signature unchanged; update its implementation to call
  `resolve()` on the embedded default registry instead of the hardcoded slices.
- `EMBEDDED_STYLE_NAMES` and `EMBEDDED_STYLE_ALIASES` are kept as deprecated `pub const`
  re-exports generated from the registry at startup to avoid breaking downstream callers.
- The `StyleRegistry` type lives in a new `crates/citum-schema-style/src/registry.rs` module,
  re-exported from `citum_schema::embedded` and also from `citum_schema` root.
- Serialisation: serde + schemars; CBOR and JSON are trivially supported via the same derives.

## Acceptance Criteria

- [ ] `registry/default.yaml` is present and contains all 12 embedded styles + 8 aliases
- [ ] `StyleRegistry::resolve("apa")` returns the `apa-7th` entry
- [ ] `StyleRegistry::resolve("apa-7th")` also returns the `apa-7th` entry
- [ ] A local `citum-registry.yaml` with one custom entry is loaded and wins over the default
- [ ] `citum registry list` prints all styles with aliases
- [ ] `citum registry resolve apa` prints `apa-7th (builtin)`
- [ ] `citum schema --target registry` emits a valid JSON Schema
- [ ] All existing tests pass (no regression in `get_embedded_style`)

## Changelog

- v1.0 (2026-03-20): Initial version.
