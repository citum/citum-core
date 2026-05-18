# citum-io

I/O and format conversion library for Citum. Loads bibliographies and
citations from external formats into Citum's internal types:

| Format        | Read | Notes                                            |
|---------------|:----:|--------------------------------------------------|
| Citum YAML    |  ✓   | Native, lossless                                 |
| Citum JSON    |  ✓   | Native, lossless                                 |
| Citum CBOR    |  ✓   | Native, binary; smallest payload                 |
| CSL-JSON      |  ✓   | Maps CSL-JSON references → Citum input model     |
| BibLaTeX      |  ✓   | Parses `.bib` files; richer than BibTeX          |
| RIS           |  ✓   | Reference Manager / Zotero export format         |

## Usage

The high-level entry point loads a bibliography from any supported
format, inferring the format from the file extension:

```rust
use citum_io::load_bibliography;
use std::path::Path;

let bibliography = load_bibliography(Path::new("library.bib"))?;
let bibliography = load_bibliography(Path::new("library.json"))?;
let bibliography = load_bibliography(Path::new("library.yaml"))?;
```

For explicit format dispatch (or to check what format a path is), use
`infer_refs_input_format(path)` to obtain a `RefsFormat`. Companion
helpers exist for citations (`load_citations`) and combined
bibliography/citation-set loading (`load_bibliography_with_sets`,
`load_input_bibliography`).

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust. The `citum` CLI uses this crate for `--bibliography` input
parsing.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
