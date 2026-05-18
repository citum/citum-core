# citum_store

Platform-aware storage layer for user-installed Citum styles and locales.
Provides a `StoreResolver` that searches XDG data directories for custom
styles, falling back to embedded builtins when nothing local matches.

Supports YAML, JSON, and CBOR formats. With the `http` feature, the
resolver chain can also fetch styles from remote registries on demand
(using `rustls` only — no system OpenSSL dependency).

## Usage

A `StoreResolver` resolves a style URI against a single data directory
and returns a parsed `Style` (the resolver handles format detection
and YAML/JSON/CBOR parsing under the hood):

```rust
use citum_store::{StoreFormat, StoreResolver};
use std::path::PathBuf;

let store_dir = PathBuf::from(std::env::var("HOME")? + "/.local/share/citum");
let resolver = StoreResolver::new(store_dir, StoreFormat::Yaml);

let style = resolver.resolve_style("my-custom-style")?;
```

For the standard XDG + embedded-fallback chain (and HTTP fetches when
the `http` feature is on), use `build_standard_chain`, which composes
the platform-appropriate resolvers automatically:

```rust
use citum_store::build_standard_chain;

let chain = build_standard_chain()?;
let style = chain.resolve_style("apa-7th")?;
```

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
