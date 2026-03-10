# Document Processor

This module provides the infrastructure for document-level citation processing. It allows the CSLN processor to scan entire documents (currently Djot plus Markdown with Pandoc-style citations), identify citation markers, and replace them with rendered citations or generated footnotes depending on the active style.

## Architecture

The system is designed around the `CitationParser` trait, allowing format-specific parsing logic while sharing the core processing workflow.

### Core Trait: `CitationParser`

Any new document format must implement the `CitationParser` trait:

```rust
pub trait CitationParser {
    /// Parse the document into citation placements and note metadata.
    fn parse_document(&self, content: &str, locale: &Locale) -> ParsedDocument;

    /// Finalize rendered document markup as HTML.
    fn finalize_html_output(&self, rendered: &str) -> String {
        djot::djot_to_html(rendered)
    }
}
```

- **Input**: The raw document content as a string plus the active locale.
- **Output**: A `ParsedDocument` containing:
  - parsed citations with byte ranges
  - citation placement (`InlineProse` vs `ManualFootnote`)
  - manual footnote reference order

`parse_citations()` remains available as a compatibility helper for callers that only need the flat citation list. `finalize_html_output()` has a default implementation so existing parser implementations remain source-compatible unless they need custom HTML post-processing.

## Note Styles

When the active style uses `options.processing: note`, `Processor::process_document()` behaves differently from inline styles:

1. Citations in prose are replaced with generated Djot footnote references such as `[^citum-auto-2]`.
2. Generated footnote definitions are emitted before the bibliography heading.
3. Citations inside authored Djot footnote definitions are rendered in place and keep the note number of that manual footnote.
4. Manual and generated notes share one note-number sequence based on body reference order, not source-definition order.
5. Punctuation and note-marker placement are configurable through `options.notes`. When omitted, the processor falls back to locale-based defaults modeled on org-cite note rules. In particular, `punctuation: adaptive` means punctuation stays inside a closing quote when it is already flush with that quote, and otherwise moves outside.

Generated/manual note handling is currently Djot-only. Markdown support in this first pass is limited to prose citations written with Pandoc-style citation markers.

## Adding a New Format

To add support for a new document format (e.g., Markdown):

1.  **Create a new file**: `src/processor/document/markdown.rs`.
2.  **Implement the parser**: Use a parsing library to identify citation markers, note references, and note definitions.
3.  **Register the module**: Add `pub mod markdown;` to `src/processor/document/mod.rs`.
4.  **Update `DocumentFormat`**: Add your format to the `DocumentFormat` enum in `mod.rs`.
5.  **Override `finalize_html_output()` if needed**: Formats that require custom HTML post-processing can override the default Djot-compatible conversion hook.

## Existing Implementations

- **Djot (`djot.rs`)**: Uses `winnow` to parse citation markers, resolves locator labels with locale-aware normalization, tracks manual footnote references/definitions via `jotdown`, and relies on the default Djot HTML finalization hook.
- **Markdown (`markdown.rs`)**: Parses Pandoc-style citation markers in prose and reuses the shared rendering pipeline, while leaving front matter, bibliography blocks, and Markdown footnote parsing for future work.

## Workflow

The `Processor::process_document` method follows these steps:
1.  Parse the document into `ParsedDocument`.
2.  For non-note styles, render each parsed citation inline.
3.  For note styles, assign note numbers from body note-reference order, annotate citation positions in that note order, replace prose citations with generated footnote references, and render citations inside manual footnotes in place.
4.  Emit any generated footnote definitions.
5.  Append the bibliography only when rendered bibliography content is non-empty.
6.  Optionally finalize the rendered markup as HTML.
