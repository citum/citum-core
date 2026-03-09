# Djot Inline Markup for Reference Fields

## Status

Partially implemented (Phases 1 and 3) — bean `csl26-suz3`.

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

### Scope B — `note` and `abstract` fields (Phase 2, still open)

Reference fields `note` and `abstract` are plain `Option<String>` today. A new `RichText` wrapper type will be introduced in `citum-schema` that serialises transparently from YAML strings but carries a format tag. At render time in the engine, field values pass through `render_djot_inline` before entering template rendering.

`abstract` and `note` are safe to migrate first because neither undergoes case transformation. See the title section below.

### Scope C — `title` fields under the current rendering model (Phase 3)

Implemented for the current engine model. Resolved title strings are parsed as
Djot inline, smart apostrophes are applied to `Event::Str` leaf text, and the
result is returned as `pre_formatted` so outer title rendering can still apply
quotes, italics, and affixes around the already-rendered inline markup.

If an authored title contains an explicit inline Djot link, that link takes
precedence over whole-title auto-linking.

This does **not** implement general title-case or sentence-case semantics.
`.nocase` and the broader text-case question remain deferred to follow-up bean
`csl26-wv5o`.

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

Uses a stack-based approach over `jotdown::Parser::new(src)`. Each frame stores
its collected child output, any explicit link target, and frame-local span
classes so nested markup can be rendered without escaping already-formatted
children.

| jotdown event | OutputFormat method |
|---|---|
| `Container::Emphasis` | `fmt.emph(…)` |
| `Container::Strong` | `fmt.strong(…)` |
| `Container::Link(…)` | `fmt.link(…)` |
| `Container::Verbatim` | `fmt.text(…)` (preserve as-is) |
| `Container::Span` with class `smallcaps` / `small-caps` | `fmt.small_caps(…)` |
| Block-level containers (`Heading`, `Paragraph`, `CodeBlock`, etc.) | Text content collected, block structure dropped |
| `Event::Str(s)` | `fmt.text(s)` or transformed text for title rendering |
| `Event::Softbreak` / `Hardbreak` | `fmt.text(" ")` |

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
- `test_djot_nested_formatting_preserves_typst_markup` — nested Djot markup is
  not re-escaped when rendered to Typst
- `test_djot_nested_link_preserves_inner_markup_html` — explicit links preserve
  formatted child content in HTML output

Title regressions in `values/tests.rs` and `processor/tests.rs` cover:

- pre-formatted Djot title values
- smart apostrophes applied to Djot title leaf text
- inline title links suppressing whole-title auto-links
- title preset wrapping around inner Djot markup

## Non-Goals

- LaTeX or MathML interpretation
- Full djot block-level rendering for field values
- General title/text-case semantics such as `.nocase`
