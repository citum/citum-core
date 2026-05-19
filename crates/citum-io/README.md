# citum-io

I/O and format conversion library for Citum. Loads bibliographies and
citations into Citum's internal types, with explicit conversion helpers for
legacy and interchange formats:

| Format        | Read | Notes                                            |
|---------------|:----:|--------------------------------------------------|
| Citum YAML    |  ✓   | Native, lossless                                 |
| Citum JSON    |  ✓   | Native, lossless                                 |
| Citum CBOR    |  ✓   | Native, binary; smallest payload                 |
| CSL-JSON      |  ✓   | Maps CSL-JSON references → Citum input model     |
| BibLaTeX      |  ✓   | Parses `.bib` files; richer than BibTeX          |
| RIS           |  ✓   | Reference Manager / Zotero export format         |

## Usage

The high-level rendering entry point loads native Citum bibliographies and
CSL-JSON files, inferring the format from the file extension and JSON content:

```rust
use citum_io::load_bibliography;
use std::path::Path;

let bibliography = load_bibliography(Path::new("library.json"))?;
let bibliography = load_bibliography(Path::new("library.yaml"))?;
```

For BibLaTeX, RIS, and other explicit conversion flows, use
`infer_refs_input_format(path)` and `load_input_bibliography(path, format)`:

```rust
use citum_io::{infer_refs_input_format, load_input_bibliography};
use std::path::Path;

let path = Path::new("library.bib");
let format = infer_refs_input_format(path)?;
let bibliography = load_input_bibliography(path, format)?;
```

Companion helpers exist for citations (`load_citations`) and combined
bibliography/citation-set loading (`load_bibliography_with_sets`).

## Project

Part of [Citum](https://github.com/citum/citum-core), a modern citation
engine in Rust. The `citum` CLI uses this crate for `--bibliography` input
parsing.

## License

Dual-licensed under [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE-APACHE)
at your option.
