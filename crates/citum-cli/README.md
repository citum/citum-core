# citum

The **Citum** command-line tool: render citations and bibliographies,
validate Citum-format styles and references, convert between bibliographic
formats, and migrate legacy CSL 1.0 styles to Citum.

## Installation

```sh
cargo install citum
```

Pre-built binaries for macOS, Linux, and Windows are also published on each
release; see [the GitHub releases page](https://github.com/citum/citum-core/releases).

## Usage

Render a document with citations:

```sh
citum render document.dj --style apa --bibliography refs.bib
```

Convert a bibliography between formats:

```sh
citum convert refs.bib --from biblatex --to citum-yaml > refs.yaml
```

Migrate a legacy CSL 1.0 style to Citum:

```sh
citum migrate apa.csl --output apa.yaml
```

Validate a Citum style or reference file:

```sh
citum validate refs.yaml
citum validate my-style.yaml
```

Use `citum --help` for the full subcommand list, and `citum <subcommand> --help`
for per-subcommand options.

## Builtin styles

The binary ships with curated styles (APA 7th, Chicago 18th, IEEE, MLA,
AMA, and more) — use `--style apa` rather than supplying a file path to
pick a builtin.

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust. The CLI wraps the underlying
[`citum-engine`](https://crates.io/crates/citum-engine).

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
