# citum-resolver-api

Lightweight interface crate that defines how the Citum schema layer talks to
the storage and resolution layer (filesystem, HTTP, git, etc.). Keeps the
schema crate dependency-free of network/IO concerns while letting consumers
swap resolver implementations.

This crate is mostly traits and error types. End users typically use
[`citum_store`](https://crates.io/crates/citum_store) or higher-level engine
APIs; resolver implementors should depend on this crate directly.

## Usage

Implementors define a `StyleResolver` and choose their own concrete
`Style` and `Locale` types (typically `citum-schema-style`'s `Style`
and `Locale`):

```rust
use citum_resolver_api::{ResolverError, StyleResolver};

struct MyResolver;

impl StyleResolver for MyResolver {
    type Style = Vec<u8>;
    type Locale = Vec<u8>;

    fn resolve_style(&self, uri: &str) -> Result<Self::Style, ResolverError> {
        // load and parse the style identified by `uri`
        todo!()
    }

    fn resolve_locale(&self, id: &str) -> Result<Self::Locale, ResolverError> {
        // load and parse the locale by BCP 47 id
        todo!()
    }
}
```

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust. See the workspace README for the full architecture.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
