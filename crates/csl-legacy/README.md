# csl-legacy

Parser for the legacy **Citation Style Language 1.0** XML format (`.csl`)
and CSL-JSON references. Produces a typed, in-memory representation of the
CSL document that downstream tools can analyze, transform, or convert.

The primary consumer is [`citum-migrate`](https://crates.io/crates/citum-migrate),
which uses this crate to read CSL 1.0 styles and convert them to equivalent
Citum styles. Migration tooling is exposed through the standalone
`citum-migrate` crate and binary, not through the primary `citum` CLI.

## Scope

This crate is **read-only and migration-focused**. It is not a full CSL 1.0
*processor*; it parses the XML and JSON formats into Rust types but does
not execute citation rendering. For rendering, see
[`citum-engine`](https://crates.io/crates/citum-engine).

## Usage

The XML entry point takes a [`roxmltree::Node`](https://docs.rs/roxmltree)
representing the root `<style>` element:

```rust
use csl_legacy::parser::parse_style;
use roxmltree::Document;

let xml = std::fs::read_to_string("apa.csl")?;
let doc = Document::parse(&xml)?;
let style = parse_style(doc.root_element())?;

println!("Style: {}", style.info.title);
println!("Citation format: {:?}", style.citation);
```

For CSL-JSON input, see [`csl_legacy::csl_json`].

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust. CSL 1.0 is the legacy format Citum is migrating from.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
