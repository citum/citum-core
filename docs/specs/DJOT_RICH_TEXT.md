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

## Implementation Notes

- `OutputFormat` satisfies `Default`, so `F::default()` is safe in `values<F: OutputFormat>`.
- `looks_like_djot_markup` guard is used in the title path. For note/abstract, apply djot unconditionally — notes commonly contain markup, and the overhead is negligible for plain text.

## Acceptance Criteria

- [x] `RichText` enum defined with `Plain(String)` and `Djot { djot: String }` variants.
- [x] `abstract_text: Option<RichText>` field on Monograph, CollectionComponent, SerialComponent (and Deser + From impls).
- [x] `note: Option<RichText>` field on all reference types with note support (and Deser + From impls).
- [x] `InputReference::note()` and `abstract_text()` return `Option<RichText>`.
- [x] Legacy CSL conversion wraps plain strings as `RichText::Plain`.
- [x] `TemplateVariable::values()` dispatches on RichText variant — plain text vs djot rendering.
- [x] Integration test: note with `_italic_` content renders as `<i>italic</i>` in HTML output.
- [x] Schema regenerated and staged (`docs/schemas/bib.json` and `docs/schemas/style.json`).

## Changelog
- 2026-04-26: Initial version (Phase 2 implementation).
