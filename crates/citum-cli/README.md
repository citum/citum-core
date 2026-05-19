# citum

The **Citum** command-line tool: render citations and bibliographies,
check Citum-format styles and references, convert typed data files, and manage
styles, locales, and registries.

## Installation

```sh
cargo install citum
```

Pre-built binaries for macOS, Linux, and Windows are also published on each
release; see [the GitHub releases page](https://github.com/citum/citum-core/releases).

## Usage

Render a document with citations:

```sh
citum render doc manuscript.djot --style apa-7th --bibliography refs.yaml
```

Convert a bibliography between reference formats:

```sh
citum convert refs refs.bib --from biblatex --to citum-yaml -o refs.yaml
```

Validate a Citum style and reference file:

```sh
citum check --style apa-7th --bibliography refs.yaml
citum check --style my-style.yaml --bibliography refs.yaml --json
```

Search or inspect bundled and installed styles:

```sh
citum style search chicago
citum style info apa-7th
```

Use `citum --help` for the full subcommand list, and `citum <subcommand> --help`
for per-subcommand options.

CSL 1.0 style migration is not a `citum` subcommand. Developer migration
tooling lives in the separate
[`citum-migrate`](https://crates.io/crates/citum-migrate) crate.

## Builtin styles

The binary ships with curated styles (APA 7th, Chicago 18th, IEEE, MLA,
AMA, and more) — use `--style apa` rather than supplying a file path to
pick a builtin.

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust. The CLI wraps the underlying
[`citum-engine`](https://crates.io/crates/citum-engine),
[`citum-io`](https://crates.io/crates/citum-io), and
[`citum_store`](https://crates.io/crates/citum_store) crates.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
