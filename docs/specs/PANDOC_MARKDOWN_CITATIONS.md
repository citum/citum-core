# Pandoc Markdown Citations Specification

**Status:** Completed
**Version:** 1.2
**Date:** 2026-05-30
**Supersedes:** None
**Related:** `crates/citum-cli/src/main.rs`, `crates/citum-engine/src/processor/document/README.md`

## Purpose
Add real support for `citum render doc --input-format markdown` by parsing
Pandoc-style citation markers in Markdown documents and feeding them into the
existing document rendering pipeline. This replaces the current CLI stub that
advertises Markdown input but exits with a not-implemented error.

## Scope
In scope:
- Parsing Pandoc-style citation markers in Markdown prose
- Reusing existing document rendering after citation extraction
- Introducing a dedicated Markdown document parser module
- Tests covering inline citations, locators, suppressed-author citations, and
  multi-cite clusters

Out of scope for v1:
- Markdown front matter overrides
- Markdown inline bibliography blocks or placement controls
- Markdown footnote/manual-note parity with the Djot document parser
- Full Pandoc document AST parity

## Design
`render doc --input-format markdown` must use a real Markdown parser module,
not a regex pre-pass embedded in the CLI. The new module must implement the
existing `CitationParser` trait and return a standard `ParsedDocument`.

The Markdown parser should support Pandoc citation markers that map cleanly
onto the existing citation model, including:
- single citations
- suppressed-author citations
- locators
- multi-cite clusters
- prefixes and suffixes that the current citation item model already supports

For v1, citations found in Markdown are treated as inline prose citations.
The parser should still populate the normal `ParsedDocument` structure so that
future work can add note placement and richer Markdown document metadata
without redesigning the document-processing API.

The CLI must stop rejecting `--input-format markdown` and instead route to the
new parser. User-facing help and docs should describe Markdown support as
Pandoc citation syntax support, not as feature parity with the Djot path.

## Implementation Notes
The current Djot parser remains the reference for how parsed citations flow
through the processor, but Markdown support should only copy the reusable
parts of that pipeline. Djot-specific behaviors such as front matter,
bibliography blocks, and manual footnote handling must remain isolated to the
Djot parser.

Prefer tests that assert rendered plain-text document output for both
author-date and note-insensitive inline rendering paths. Add unit coverage for
the Markdown parser itself where syntax normalization is non-obvious.

## Acceptance Criteria
- [x] `citum render doc --input-format markdown` no longer errors for supported
      Pandoc citation syntax
- [x] Inline Markdown citations render correctly for single, suppressed-author,
      locator, and multi-cite scenarios
- [x] The implementation introduces a dedicated Markdown parser module that
      implements `CitationParser`
- [x] Existing Djot document behavior remains unchanged
- [x] Tests cover the new Markdown citation path

## Passthrough Output Formats and Pandoc Interop

Citum's document pipeline has two distinct classes of output format:

**Converted formats** (`html`, `typst`, `latex`) — citations are spliced in,
then a second pass converts surrounding block markup to the target format
(jotdown for Djot→HTML, `render_body_markup` for Djot/Markdown→Typst/LaTeX).

**Passthrough formats** (`plain`, `djot`, `markdown`) — citations are rendered
and spliced in, but block-level document markup is **emitted verbatim**. The
output is ready to pipe to any downstream formatter.

### Pandoc interop workflow

The correct direction is always **citum → pandoc**: Citum processes citations
first, replacing `[@key]` markers with rendered text, then Pandoc receives the
output and handles block-level formatting and final output conversion.

```bash
citum render doc input.md --input-format markdown --format markdown \
  -s apa -b refs.yaml | pandoc --to html
```

Omitting `--from` lets Pandoc use its native Markdown reader, which handles
pipe tables, fenced code blocks, and footnote syntax (`[^n]`) — all of which
citum passes through verbatim. Do not use `--from commonmark`: bare CommonMark
cannot parse pipe tables or `[^n]` footnotes.

**Do not pre-process with `pandoc --to commonmark`** if the document contains
Citum citation markers. Pandoc escapes `[` when converting to CommonMark
(`[@key]` → `\[@key\]`), which Citum's Markdown parser cannot recognise.
Citations would silently go unrendered (verified with pandoc 3.9.0.2).

### Supported CommonMark surface

Citum supports **CommonMark + GFM extensions** (pipe tables, strikethrough).
Pandoc-only block syntax (grid tables, definition lists, etc.) is **not**
handled — documents using such syntax must avoid mixing it with Citum citation
markers, or convert the block syntax separately by a tool that preserves
`[@key]` verbatim.

### Note styles and footnote syntax

Note-based styles emit `[^label]` anchors in prose and `[^label]: …`
definitions at the document end — the Pandoc/GFM footnote extension, not core
CommonMark. Use `--from gfm+footnotes` or `--from markdown` downstream:

```bash
citum render doc input.md --input-format markdown --format markdown \
  -s chicago-notes -b refs.yaml | pandoc --to html
```

`--format djot` is unaffected — Djot supports footnotes natively.

## Changelog
- v1.0 (2026-03-09): Initial version.
- v1.1 (2026-03-16): All criteria met. `BibliographyBlock` moved to shared
  `types.rs`; `CitationParser::finalize_html_output` default changed to
  pass-through; `DocumentFormat::Markdown` variant added; engine README
  restructured to distinguish document input, output, and field markup concerns.
- v1.2 (2026-05-30): Added `OutputFormat::Markdown` CLI variant and
  `render::markdown::Markdown` renderer, making `--format markdown` reachable.
  Documented passthrough vs. converted output formats, Pandoc interop workflow,
  and footnote-extension caveat for note-based styles. Grid-table preprocessor
  (PR #846) reverted — Pandoc-only syntax is Pandoc's responsibility. Corrected
  workflow direction to `citum → pandoc` (primary); removed `pandoc → citum`
  pre-processing chain (Pandoc escapes `[@key]` to `\[@key\]`, breaking citation
  parsing). Added `text()` escaping for CommonMark-active punctuation.
