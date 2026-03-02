# Djot Inline Markup for Reference Fields

## Status

Implemented (Phase 1) — bean `csl26-suz3`, branch `feat/djot-rich-text`, commit `df059ae`.

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

Implemented in `crates/citum-engine/src/render/rich_text.rs`:

```rust
pub fn render_djot_inline<F: OutputFormat<Output = String>>(src: &str, fmt: &F) -> String
```

Uses a stack-based approach over `jotdown::Parser::new(src)`. Each `Event::Start` pushes a new scope; `Event::End` pops the scope, applies the format method, and pushes the result to the parent scope.

| jotdown event | OutputFormat method |
|---|---|
| `Container::Emphasis` | `fmt.emph(…)` |
| `Container::Strong` | `fmt.strong(…)` |
| `Container::Link(…)` | `fmt.text(…)` (link URL not available at End; degrades to text — known limitation) |
| `Container::Verbatim` | `fmt.text(…)` (preserve as-is) |
| `Container::Span` with class `smallcaps` | `fmt.small_caps(…)` |
| Block-level containers (`Heading`, `Paragraph`, `CodeBlock`, etc.) | Text content collected, block structure dropped |
| `Event::Str(s)` | `fmt.text(s)` |
| `Event::Softbreak` / `Hardbreak` | `fmt.text(" ")` |

**Known limitation:** Link URL is only accessible at `Event::Start(Container::Link(url, …))` but the formatted inner content is only available at `Event::End`. The current implementation degrades links to plain text. Fix requires stashing the URL on the stack alongside the content buffer.

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

`AnnotationStyle` gets `format: AnnotationFormat` defaulting to `Djot`. CLI (`citum-cli/src/main.rs`) also updated to default to `Djot`.

### Annotation render path

`refs_to_string_with_format` in `bibliography.rs`: when `AnnotationFormat::Djot`, calls `render_djot_inline::<F>(annotation_text, &fmt)` before applying indent and italic. `AnnotationFormat::Plain` passes through unchanged.

## Test Coverage (implemented)

Unit tests in `rich_text.rs`:

- `test_djot_emphasis_plain` — `_foo_` with PlainText renders as `_foo_` (PlainText wraps emph in underscores)
- `test_djot_strong_single_asterisk` — `*bar*` with PlainText renders as `**bar**`
- `test_djot_unicode_math` — `H₂O` passes through unchanged
- `test_djot_plain_no_markup` — plain string passes through unchanged
- `test_djot_combined_formatting` — nested emphasis + strong renders correctly

## Non-Goals

- LaTeX or MathML interpretation
- Full djot block-level rendering for field values
- AST-level title markup (deferred to a follow-up)
