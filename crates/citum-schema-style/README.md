# citum-schema-style

Schema types for **Citum styles** — declarative, type-safe successors to
CSL 1.0. Defines the Rust models for style metadata, citation templates,
bibliography layouts, locale terms, sort keys, and the embedded asset
catalog that ships with the binary.

This crate also bundles a curated set of **embedded styles and locales**
(APA 7th, Chicago 18th, IEEE, Modern Language Association, etc.) baked
into the binary via `include_bytes!`, so the CLI can render citations
without an external style file. Embedded files live under
[`embedded/`](./embedded).

## Usage

```rust
use citum_schema_style::Style;

let yaml = std::fs::read_to_string("my-style.yaml")?;
let style: Style = serde_yaml::from_str(&yaml)?;

println!("Style: {}", style.info.title);
```

Resolve a builtin by name:

```rust
use citum_schema_style::embedded::get_embedded_style;

let apa = get_embedded_style("apa-7th").expect("known builtin")?;
```

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option. Embedded styles retain their original CC BY-SA 3.0 license.
