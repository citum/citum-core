# citum-schema-data

Data-input schema types for Citum: the Rust types representing references,
contributor lists, dates, titles, and other bibliographic data that gets
fed *into* a citation engine.

Pair this crate with [`citum-schema-style`](https://crates.io/crates/citum-schema-style)
(style definitions) for the full schema surface, or use
[`citum-schema`](https://crates.io/crates/citum-schema) which re-exports both.

## Usage

References are discriminated by the kebab-case `class:` field
(`monograph`, `collection`, `serial`, `legal-case`, `patent`, …).
Use the `class()` accessor and the `as_<class>()` downcasts to reach
class-specific fields:

```rust
use citum_schema_data::reference::{InputReference, ReferenceClass};

let yaml = r#"
id: smith2026
class: monograph
title: A Book
issued: "2026"
monograph-type: book
volume: "2"
"#;

let reference: InputReference = serde_yaml::from_str(yaml)?;

assert!(matches!(reference.class(), ReferenceClass::Monograph));
let monograph = reference.as_monograph().unwrap();
assert_eq!(monograph.monograph_type.as_deref(), Some("book"));
```

Supports serde-driven (de)serialization to/from YAML, JSON, and CBOR.

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust. See the workspace README for the architecture overview.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
