# citum-engine

The citation/bibliography rendering engine. Consumes Citum-schema styles + InputReferences, emits rendered citations and bibliographies. Target: byte-for-byte match with citeproc-js for CSL-derived styles and biblatex for biblatex-derived styles.

## Layout

| Path | Purpose |
|---|---|
| `src/lib.rs` | Crate roots and public API |
| `src/api/` | Public engine entry points (call from CLI, FFI, WASM) |
| `src/processor/` | Style-driven processing core |
| `src/render/` | Render passes — `citations.rs`, bibliography, sort, layout |
| `src/grouping/` | Cite-group resolution (integral multi-cite, et al.) |
| `src/values/` | Resolved value types (`MaybeGendered`, dates, names) |
| `src/ffi/` | C ABI surface: `mod.rs` (~570 lines) + `biblatex.rs` (uses `BibRefContext<'a>` to avoid `too_many_arguments`) |
| `src/sort_partitioning.rs` | Bibliography sort & partition logic |

## Gotchas

- **Integral multi-cites** join via locale-aware prose ("Smith (2020) and Jones (2021)"). Grouped integral locator punctuation must not duplicate delimiters — see commit `88a6f62`. File: `src/render/citations.rs`.
- **Oracle routing:** check the style's `originKey` before picking an oracle. CSL-derived → CSL oracle (`node scripts/oracle.js`); biblatex-derived → biblatex oracle. Never run a CSL-shape oracle against a biblatex-derived style — the diff is meaningless.
- **Locale MF2 is live** for locator/role term resolution — `messages:` consulted first, falls back to legacy `terms:` / `roles:` / `locators:`. Gendered role labels still flow through `roles:` (multi-selector MF2 follow-up pending).
- **No `unwrap()` / `unsafe`.** Result/Option must surface as engine errors via `src/error.rs`.

## Symbol & type queries

For "what does this generic resolve to?" or LSP-grade hover, use **rust-analyzer**. For symbol bodies and call hierarchies, use **jcodemunch** (`get_symbol`, `get_call_hierarchy`). See root `CLAUDE.md` for the priority table.

## Hot paths

`cargo bench --bench rendering` — keep regressions out. Workflow test: `./scripts/workflow-test.sh styles-legacy/apa.csl`.
