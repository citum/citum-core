# Djot Rich Text Specification

**Status:** Active
**Date:** 2026-04-26
**Related:** bean `csl26-suz3`

## Purpose

Specifies djot inline markup processing for title, annotation, note, and abstract fields in the Citum engine.

## Scope

In scope: inline djot markup (bold, italic, links, code) in title strings, annotation strings, note fields, and abstract fields.
Out of scope: block-level djot, math markup, sentence-case and title-case text transformations (tracked in csl26-wv5o).

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

## Changelog
- 2026-05-21: Added inline rendering context for nested emphasis and quote toggling.
- 2026-04-26: Initial version (Phase 2 implementation).
