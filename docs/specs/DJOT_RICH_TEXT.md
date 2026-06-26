# Djot Rich Text Specification

**Status:** Active
**Date:** 2026-06-26
**Related:** bean `csl26-suz3`, bean `csl26-fdzc`

## Purpose

Specifies djot inline markup processing for title, annotation, note, and abstract fields in the Citum engine.

## Math Policy

Mathematical notation in citation fields should use **Unicode only**. This follows John MacFarlane's analysis ([CSL schema issue #278](https://github.com/citation-style-language/schema/issues/278#issuecomment-650402841)): practical cases (H₂O, p53, isotopes) are representable in Unicode; full math (integrals, matrices) does not belong in citation fields by design. LaTeX or MathML markup in reference fields is out of scope. Authors should use Unicode precomposed characters; styles and processors must not interpret TeX fragments.

## Title Markup: Known Cases

### Title-within-title

A book title containing an embedded article title, or vice versa: `_Some Embedded Book_`. The embedded title italicises if context is not already italic, de-italicises inside an italic context (Chicago rule). Quote nesting uses locale-dependent quotation marks; this is a locale/style config concern, not a field markup concern.

### Case protection (`.nocase`)

The primary driver for title markup is preventing sentence-case transformation from destroying content that must stay uppercase:

| Case | Example |
|---|---|
| Acronyms | `DNA`, `NASA`, `pH`, `mRNA` |
| Proper nouns | `Paris`, `Alzheimer`, `Google` |
| Chemical names | `NaCl`, `HCl` |
| Embedded titles | Any proper noun acting as a title |

Citum's equivalent of CSL `<span class="nocase">` uses djot span attributes: `{.nocase}[DNA]`. This is the **highest-value** markup use case for titles.

### Smallcaps in titles

Rare; mostly style-driven (some styles render `BCE`/`CE` in smallcaps). Djot span: `{.smallcaps}[BCE]`.

## Scope

In scope: inline djot markup (bold, italic, links, code) in title strings, annotation strings, note fields, and abstract fields.
Out of scope: block-level djot, math markup, sentence-case and title-case text transformations (tracked in csl26-wv5o).

Locale message bodies are a separate surface. This spec covers rich text that
comes from bibliographic fields and template-rendered values; it does not enable
Djot or any other inline markup inside locale-authored `messages:` strings.
Locale-owned literal styling, such as italicizing an `In` supplied by a locale
message, is deferred to the fragment-output design documented in
[`LOCALE_MESSAGES.md`](./LOCALE_MESSAGES.md), not handled by Djot field markup.

## Design

### Phase 1 — Annotation rendering (complete)
`render_djot_inline` helper in `citum-engine/src/render/rich_text.rs`. `AnnotationFormat` enum and `AnnotationStyle.format` field default to `Djot`. Annotation render path in `bibliography.rs` routes through djot.

### Phase 2 — Note and abstract fields (complete)
Note (`SimpleVariable::Note`) and abstract (`SimpleVariable::Abstract`) fields use the `RichText` enum: plain strings deserialize as `RichText::Plain(String)`; `{ djot: "..." }` objects as `RichText::Djot { djot }`. Rendering in `TemplateVariable::values()` dispatches on variant — plain text returns `pre_formatted: false`; djot applies `render_djot_inline::<F>(&djot, &F::default())` and returns `pre_formatted: true`.

`note: Option<RichText>` and `abstract_text: Option<RichText>` added to `Monograph`, `CollectionComponent`, and `SerialComponent` structs (and their `*Deser` counterparts and `From` impls). Legacy CSL conversion wraps plain strings as `RichText::Plain`. `InputReference::abstract_text()` dispatches from those three types; all other types return `None`.

### Phase 3 — Title markup (complete)
`TemplateTitle::values()` routes resolved title strings through djot inline rendering before returning them to the component renderer. Smart apostrophe handling applied to djot `Event::Str` leaf text. Titles returned with `pre_formatted: true`.

### Phase 4 — Inline rendering context (complete)
Inline rich text rendering carries an explicit `InlineRenderContext`. The initial context field is `quote_depth`, which records quotation nesting inherited from an outer template wrapper.

This is required because title values may be rendered before component-level wrappers are applied. When a component has effective `quote: true`, the title value renderer receives `quote_depth = 1` so straight or Djot inline quotes inside the field render as inner quotes before the component renderer applies the outer quote pair.

Inline emphasis toggling remains target-renderer-owned:
- HTML emits semantic `<em>` and relies on Citum-scoped stylesheet rules such as `.citum-citation em em` and `.citum-bibliography em em`.
- LaTeX emits `\emph{...}` so TeX handles italic/roman toggling.
- Typst emits `#emph[...]` so Typst handles the emphasis state.

Quote toggling is likewise renderer-owned. `OutputFormat` exposes depth-aware quote marks and wrappers. Depth `0` is an outer double quote pair; depth `1` is an inner single quote pair; deeper depths alternate. LaTeX maps the same depths to TeX double and single quote delimiters.

## Implementation Notes

- `OutputFormat` satisfies `Default`, so `F::default()` is safe in `values<F: OutputFormat>`.
- `looks_like_djot_markup` guard is used in the title path. For note/abstract, apply djot unconditionally — notes commonly contain markup, and the overhead is negligible for plain text.
- Plain title smart-quote conversion is also depth-aware, because quotes alone do not trip the djot-markup guard.

## Acceptance Criteria

- [x] `RichText` enum defined with `Plain(String)` and `Djot { djot: String }` variants.
- [x] `abstract_text: Option<RichText>` field on Monograph, CollectionComponent, SerialComponent (and Deser + From impls).
- [x] `note: Option<RichText>` field on all reference types with note support (and Deser + From impls).
- [x] `InputReference::note()` and `abstract_text()` return `Option<RichText>`.
- [x] Legacy CSL conversion wraps plain strings as `RichText::Plain`.
- [x] `TemplateVariable::values()` dispatches on RichText variant — plain text vs djot rendering.
- [x] Integration test: note with `_italic_` content renders as `<em>italic</em>` in HTML output.
- [x] Schema regenerated and staged (`docs/schemas/bib.json` and `docs/schemas/style.json`).
- [x] Component-level quoted titles alternate inner quote marks for normal and grouped bibliography rendering.
- [x] Djot inline rendering alternates nested quote marks using ambient quote depth.

## Non-Goals

- LaTeX or MathML interpretation in reference fields
- Full djot block-level rendering for field values
- General title/text-case semantics (`.nocase` transformation engine — tracked in csl26-wv5o)

## Changelog
- 2026-06-26: Clarified that locale message body rich text is a separate
  deferred design.
- 2026-05-21: Added inline rendering context for nested emphasis and quote toggling.
- 2026-04-26: Initial version (Phase 2 implementation).
