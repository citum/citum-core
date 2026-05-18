# citum-schema

Compatibility facade that re-exports the Citum schema surface from its
two underlying crates:

- [`citum-schema-data`](https://crates.io/crates/citum-schema-data) — input
  reference and bibliography data types
- [`citum-schema-style`](https://crates.io/crates/citum-schema-style) — style
  models, embedded styles, locale schemas

Most consumers should depend on **this** crate rather than the two underlying
ones — `citum-schema` provides a single import surface that matches what
`citum-engine` and `citum_store` use internally, while still letting you
reach for the underlying crates when you need a smaller dependency.

## Usage

Style types come from the crate root (re-exported from
[`citum-schema-style`](https://crates.io/crates/citum-schema-style));
data-input types live under [`data`] (re-exported from
[`citum-schema-data`](https://crates.io/crates/citum-schema-data)):

```rust
use citum_schema::Style;
use citum_schema::data::reference::InputReference;

let style: Style = serde_yaml::from_str(&style_yaml)?;
let reference: InputReference = serde_yaml::from_str(&ref_yaml)?;
```

Exposes `SCHEMA_VERSION` (also `STYLE_SCHEMA_VERSION` via the re-export)
for compatibility checks.

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
