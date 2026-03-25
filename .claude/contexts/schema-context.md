# Schema Context

Working on the **Citum data model** — types, schemas, and the specification itself.

## Philosophy
- **Code-First**: Rust structs and enums are the source of truth for the schema.
- **Strict Validation**: All types use `deny_unknown_fields` to catch typos and invalid fields at parse time.
- **Extension via Defaults**: New features use `Option<T>` with `#[serde(default)]`.
- **Explicit Extensions**: User-defined metadata uses explicit `custom` fields.

## Key Crate: `citum_schema`
| Module | Responsibility |
|--------|----------------|
| `style.rs` | Top-level `Style` struct (with `version` field) |
| `template.rs` | Template components, `type-variants`, contributor/date/title types |
| `options.rs` | Three-tier options: global → context (citation/bibliography) → template |
| `locale.rs` | Locale terms, months, contributor roles |
| `reference.rs` | Reference types and metadata fields |

## Serde Conventions
- `#[serde(rename_all = "kebab-case")]` — YAML/JSON field naming
- `#[non_exhaustive]` — extensible enums
- `#[serde(deny_unknown_fields)]` — strict validation on all types
- `Option<T>` + `skip_serializing_if` — optional fields
- `#[serde(flatten)]` — inline rendering options (NOT for unknown field capture)
- `custom: Option<HashMap<String, serde_json::Value>>` — explicit extension fields

## Two-Tier Options + Type Variants
```
Global options:        style.options
Context options:       style.citation.options / style.bibliography.options
type-variants:         spec-level type-specific templates (citation/bibliography)
```
Context-specific options override global for their context.
`type-variants` are full template replacements for structural outliers — not option patches.

## Schema Generation
```bash
cargo run --bin citum -- schema > csln.schema.json
```

## Reference Docs
- [STYLE_ALIASING.md](../../docs/architecture/design/STYLE_ALIASING.md)
- [PUNCTUATION_NORMALIZATION.md](../../docs/architecture/design/PUNCTUATION_NORMALIZATION.md)
- [PRIOR_ART.md](../../docs/architecture/PRIOR_ART.md)
