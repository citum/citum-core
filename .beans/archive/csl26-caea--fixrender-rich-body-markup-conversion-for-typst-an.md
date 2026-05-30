---
# csl26-caea
title: 'fix(render): rich body markup conversion for Typst and LaTeX (#824)'
status: completed
type: bug
priority: high
created_at: 2026-05-30T00:53:29Z
updated_at: 2026-05-30T01:37:38Z
---

First external bug report. Pandoc block quotes convert incorrectly to Typst. Root cause: no body-markup conversion for --format typst/latex. Fix: add block-aware renderer using jotdown (Djot) + pulldown-cmark (Markdown), extend OutputFormat with block methods, generalize placeholder pipeline. Scope: Djot+Markdown inputs, Typst+LaTeX outputs, comprehensive conversion. Ref: https://github.com/citum/citum-core/issues/824

## Todo

- [x] Add pulldown-cmark dep, measure binary size
- [x] Create render/markup/ module (MarkupEvent + adapters + renderer)
- [x] Extend OutputFormat with block methods (default impls)
- [x] Override block methods in typst.rs and latex.rs
- [x] Add render_body_markup to CitationParser trait
- [x] Generalize placeholder pipeline for Typst/LaTeX
- [x] Wire pipeline.rs to route Typst/LaTeX through new path
- [x] Add tests (document.rs)
- [x] Verify with typst compile (repro from #824)
- [x] Repo gate: cargo fmt/clippy/nextest

## Summary of Changes

- Added `pulldown-cmark = "0.13"` to `citum-engine` (CommonMark parser for Markdown body).
- New `crates/citum-engine/src/render/markup/` module: `renderer.rs` (frame-stack renderer), `djot.rs` (jotdown adapter), `markdown.rs` (pulldown-cmark adapter).
- Extended `OutputFormat` trait with block-level default methods: `paragraph`, `block_quote`, `bullet_list`, `ordered_list`, `list_item`, `heading`, `code_block`, `inline_code`, `strikeout`, `hard_break`.
- Overrode block methods in `Typst` and `Latex` renderers (e.g. `block_quote` → `#quote(block: true)[…]` / `\begin{quote}…`).
- Added `render_body_markup` to `CitationParser` trait; implemented in `DjotParser` (jotdown) and `MarkdownParser` (pulldown-cmark).
- Extended pipeline: Typst/LaTeX documents go through the placeholder path so citations are token-protected before body markup conversion; bibliography is deferred to `trailing` so it is never run through the markup parser.
- Non-note styles (e.g. APA): full conversion. Note styles (e.g. Chicago): passthrough (footnote syntax is format-specific; separate follow-up).
- 4 new BDD tests in `document.rs`: Markdown→Typst block quote, Djot→Typst block quote, Markdown→LaTeX block quote, Markdown→Typst citation + emphasis.
- 1432/1432 tests passing.
