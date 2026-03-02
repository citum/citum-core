# Djot Inline Markup for Reference Fields

## Status

Planned — bean `csl26-suz3`, branch `feat/djot-rich-text`.

## Summary

Djot inline markup becomes the default format for annotation text, with future support for `note` and other free-text reference fields. Math in citation fields is handled by Unicode, not markup.

## Motivation

Annotations and reference fields like `note` and `abstract` can contain rich text: emphasis, smallcaps, hyperlinks, inline code. Today these are rendered as plain strings. `jotdown` (the djot parser) is already a dependency; the `OutputFormat` trait already has `emph`, `strong`, `small_caps`, and `link` methods. The plumbing exists — it just needs to be connected.

## Math Policy

Mathematical notation in citation fields should use **Unicode only**. This position follows John MacFarlane's analysis ([CSL schema issue #278](https://github.com/citation-style-language/schema/issues/278#issuecomment-650402841)):

> The practical cases — chemical formulas (H₂O), gene names (p53), isotopes, basic super/subscript — are all representable in Unicode. Full math (integrals, matrices) does not belong in citation fields by design.

LaTeX or MathML markup in reference fields is out of scope. Styles and processors must not attempt to interpret TeX fragments. Authors should use Unicode precomposed characters or combining characters where Unicode covers the case, and accept plain ASCII fallback otherwise.

## Scope

### Scope A — Annotations (Phase 1, this PR)

Annotation values (`HashMap<String, String>`) are parsed as djot inline at render time. A new `format: AnnotationFormat` field on `AnnotationStyle` controls this, defaulting to `Djot`.

```yaml
# style option
annotation:
  format: djot   # default; also accepts "plain"
  indent: true
  paragraph_break: blank_line
```

### Scope B — `note` and `abstract` fields (Phase 2)

Reference fields `note` and `abstract` are plain `Option<String>` today. A new `RichText` wrapper type will be introduced in `citum-schema` that serialises transparently from YAML strings but carries a format tag. At render time in the engine, field values pass through `render_djot_inline` before entering template rendering.

`abstract` and `note` are safe to migrate first because neither undergoes case transformation. See the title section below.

### Scope C — `title` and other case-transformed fields (future)

**Deferred.** `title` has a custom `Title` type and is subject to sentence-case and title-case transformation in the engine. Djot parsing must happen *after* case transformation is applied to leaf text nodes — not the raw string, where markers like `_foo_` would be uppercased. This requires making the case-transformation code AST-aware, which is a separate architectural change. See the title markup section below.

## Title Markup: Known Cases

Even before djot support is implemented for `title`, the following markup needs are well-established:

### Title-within-title

A book title containing an embedded article title, or vice versa:

- *A Review of* Some Embedded Book — the embedded title italicises if context is not already italic, de-italicises inside an italic context (Chicago rule)
- "A review of 'Some Article'" — nested quotes, locale-dependent quotation marks

Djot syntax: `_Some Embedded Book_`. Quote nesting is a locale/style config concern, not a field markup concern.

### Case protection (`.nocase`)

The primary driver for title markup is preventing sentence-case transformation from destroying content that must stay uppercase:

| Case | Example |
|---|---|
| Acronyms | `DNA`, `NASA`, `pH`, `mRNA` |
| Proper nouns | `Paris`, `Alzheimer`, `Google` |
| Chemical names | `NaCl`, `HCl` |
| Embedded titles | Any proper noun acting as a title |

CSL uses `<span class="nocase">` for this. Citum's equivalent will use djot span attributes: `{.nocase}[DNA]`. The case-transformation engine must recognise `.nocase` spans and skip them.

This is the **highest-value** markup use case for titles — more common than italics.

### Smallcaps in titles

Rare; mostly style-driven (some styles render `BCE`/`CE` in smallcaps). Djot span: `{.smallcaps}[BCE]`. Not a priority for initial implementation.

## Architecture

### `render_djot_inline<F: OutputFormat>`

New function in `crates/citum-engine/src/render/rich_text.rs`:

```rust
pub fn render_djot_inline<F: OutputFormat<Output = String>>(src: &str, fmt: &F) -> String
```

Uses `jotdown::Parser` over the input string, maps inline `Event` variants to `OutputFormat` methods:

| jotdown event | OutputFormat method |
|---|---|
| `Container::Emphasis` | `fmt.emph(…)` |
| `Container::Strong` | `fmt.strong(…)` |
| `Container::Link(url, …)` | `fmt.link(url, …)` |
| `Container::Verbatim` | `fmt.text(…)` (preserve as-is) |
| `Container::Span` with `.smallcaps` | `fmt.small_caps(…)` |
| `Container::Span` with `.nocase` | content passed through, case flag set |
| Block-level containers | Ignored; fall back to plain text |
| `Event::Str(s)` | `fmt.text(s)` |

Block-level djot constructs (headings, bullet lists, fenced code) are invalid in a field context and are silently degraded to their text content.

### `AnnotationFormat` enum

Added to `crates/citum-engine/src/io.rs`:

```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationFormat {
    #[default]
    Djot,
    Plain,
}
```

`AnnotationStyle` gets a new field `format: AnnotationFormat` defaulting to `Djot`.

### Annotation render path

`refs_to_string_with_format` in `bibliography.rs`: when `AnnotationFormat::Djot`, replace the current string concatenation with `render_djot_inline::<F>(annotation_text, &fmt)`.

## Test Coverage

- Djot annotation with emphasis renders correctly in HTML, PlainText, and Djot output formats
- Djot annotation with a link renders correctly in HTML only (plain text degrades to label)
- Block-level djot in an annotation field degrades gracefully to plain text
- `AnnotationFormat::Plain` passes the string through unmodified
- Unicode math characters in annotation text pass through unmodified

## Non-Goals

- LaTeX or MathML interpretation
- Full djot block-level rendering for field values
- AST-level title markup (deferred to a follow-up)
