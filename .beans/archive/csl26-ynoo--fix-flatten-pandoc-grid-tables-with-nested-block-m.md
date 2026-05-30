---
# csl26-ynoo
title: 'fix: flatten Pandoc grid tables with nested block markup (#845)'
status: completed
type: bug
priority: high
created_at: 2026-05-30T18:51:32Z
updated_at: 2026-05-30T19:13:01Z
---

Block quotes inside Pandoc grid tables produce invalid Typst/LaTeX output. Fix by preprocessing grid tables into flat sequential block markup before any parser runs. Covers Typst + LaTeX + HTML output. Closes #845.

## Summary of Changes

- New module `crates/citum-engine/src/processor/document/grid_table.rs`: `flatten_grid_tables()` preprocessor + 9 unit tests
- Wired into `process_document` in `pipeline.rs` (single call site, zero allocation on non-table documents)
- 5 integration tests in `document.rs` covering Markdown+Djot × Typst/LaTeX/HTML
- Spec note added to `docs/specs/DJOT_RICH_TEXT.md`

All 1448 tests pass. Gate: cargo fmt --check + clippy -D warnings + cargo nextest run.
