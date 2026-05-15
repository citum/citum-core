# csl-legacy

The CSL 1.0 XML parser. Complete and frozen — consumed by `citum-migrate` for conversion to Citum schema. Treat as a stable dependency, not active development.

## Layout

| Path | Purpose |
|---|---|
| `src/lib.rs` | Public re-exports |
| `src/parser.rs` | CSL 1.0 XML → AST (~35K, navigate via jcodemunch) |
| `src/model.rs` | AST types (~21K) |
| `src/csl_json.rs` | CSL-JSON support (~39K) |
| `src/bin/` | Standalone parser binaries |

## Gotchas

- **Commit scope is `migrate`, not `csl-legacy`.** The allowed-scope list excludes `csl-legacy`. Bundle changes here with related `citum-migrate` work under `migrate`.
- **Frozen.** Add behavior only when CSL 1.0 spec compliance is genuinely missing or `citum-migrate` exposes a real parser gap. Most fidelity issues live downstream in `citum-engine` rendering.
- **No `unwrap()` / `unsafe`** still applies. Parser errors flow through the crate's error type.

## Symbol queries

Large files — use **jcodemunch**: `get_file_outline` to map symbols in a single file, `get_symbol` to read one symbol's body. For trait/type resolution, **rust-analyzer**.
