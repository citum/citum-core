# Document Processor

This module provides the infrastructure for document-level citation processing. It allows the Citum processor to scan entire documents (currently Djot), identify citation markers, and replace them with rendered citations or generated footnotes depending on the active style.

## Architecture

The system is designed around the `CitationParser` trait, allowing format-specific parsing logic while sharing the core processing workflow.

### Core Trait: `CitationParser`

Any new document format must implement the `CitationParser` trait:

```rust
pub trait CitationParser {
    /// Parse the document into citation placements and note metadata.
    fn parse_document(&self, content: &str) -> ParsedDocument;
}
```

- **Input**: The raw document content as a string.
- **Output**: A `ParsedDocument` containing:
  - parsed citations with byte ranges
  - citation placement (`InlineProse` vs `ManualFootnote`)
  - manual footnote reference order

`parse_citations()` remains available as a compatibility helper for callers that only need the flat citation list.

## Note Styles

When the active style uses `options.processing: note`, `Processor::process_document()` behaves differently from inline styles:

1. Citations in prose are replaced with generated Djot footnote references such as `[^citum-auto-2]`.
2. Generated footnote definitions are emitted before the bibliography heading.
3. Citations inside authored Djot footnote definitions are rendered in place and keep the note number of that manual footnote.
4. Manual and generated notes share one note-number sequence based on body reference order, not source-definition order.
5. Punctuation and note-marker placement are configurable through `options.notes`. When omitted, the processor falls back to locale-based defaults modeled on org-cite note rules. In particular, `punctuation: adaptive` means punctuation stays inside a closing quote when it is already flush with that quote, and otherwise moves outside.

This support is currently Djot-only.

## Adding a New Format

To add support for a new document format (e.g., Markdown):

1.  **Create a new file**: `src/processor/document/markdown.rs`.
2.  **Implement the parser**: Use a parsing library to identify citation markers, note references, and note definitions.
3.  **Register the module**: Add `pub mod markdown;` to `src/processor/document/mod.rs`.
4.  **Update `DocumentFormat`**: Add your format to the `DocumentFormat` enum in `mod.rs`.
5.  **Update `Processor::process_document`**: If your format requires specific post-processing (like Djot's HTML conversion), update the final conversion branch.

## Existing Implementations

- **Djot (`djot.rs`)**: Uses `winnow` to parse citation markers and `jotdown` offsets to track manual footnote references/definitions. It also converts final Djot output to HTML using `jotdown`.

## Workflow

The `Processor::process_document` method follows these steps:
1.  Parse the document into `ParsedDocument`.
2.  For non-note styles, render each parsed citation inline.
3.  For note styles, assign note numbers from body note-reference order, annotate citation positions in that note order, replace prose citations with generated footnote references, and render citations inside manual footnotes in place.
4.  Emit any generated footnote definitions.
5.  Append the bibliography.
6.  Optionally convert Djot output to HTML.
