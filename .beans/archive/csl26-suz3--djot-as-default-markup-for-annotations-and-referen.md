---
# csl26-suz3
title: Djot as default markup for annotations and reference fields
status: completed
type: feature
priority: normal
tags:
    - schema
    - engine
created_at: 2026-03-02T20:10:22Z
updated_at: 2026-04-26T23:45:00Z
parent: csl26-li63
---

Implement djot inline markup support for annotation rendering and reference fields (note, abstract). Document math policy (Unicode-only) and title markup cases. See dplan output for full architecture.

## Implementation Summary

Phase 1 (annotations) complete — commit `df059ae` on `feat/djot-rich-text`.

- [x] `render_djot_inline` helper in `citum-engine/src/render/rich_text.rs`
- [x] `AnnotationFormat` enum added to `io.rs`
- [x] `AnnotationStyle.format` field, default `Djot`
- [x] Annotation render path in `bibliography.rs` uses djot
- [x] CLI default updated
- [x] Architecture doc at `docs/architecture/DJOT_RICH_TEXT.md`
- [x] Fix link rendering (URL preserved through Djot inline rendering)
- [x] Phase 2: `RichText` type for `note`/`abstract` fields
- [x] Phase 3: `title` field under the current rendering model

2026-03-09 residual gap: this bean remains open only for `note`/`abstract`
rich-text support. Broader title/text-case semantics were split out to
`csl26-wv5o`.

## Phase 3 — Title Markup (Current Rendering Model)

**Goal:** Allow djot inline markup in title fields without introducing a new
title/sentence-case subsystem.

**Implemented behavior:**

1. `TemplateTitle::values()` now routes resolved title strings through Djot
   inline rendering before returning them to the component renderer.
2. Smart apostrophe handling is applied to Djot `Event::Str` leaf text, so
   inner markup remains intact.
3. Title values are returned with `pre_formatted: true`, allowing outer title
   rendering (quotes, italics, prefixes, suffixes) to wrap already-rendered
   inline markup.
4. Explicit inline Djot links inside titles suppress whole-title auto-linking,
   so the authored inline link wins.
5. This phase does **not** implement `.nocase`, sentence-case, title-case, or
   other general text-case semantics.

**Key files:**
- `crates/citum-engine/src/values/title.rs` — main logic insertion point
- `crates/citum-engine/src/render/rich_text.rs` — Djot inline renderer with
  leaf-text transforms and explicit-link metadata
- `crates/citum-engine/src/processor/tests.rs` — preset and autolink regressions

**Definition of done:**
- Djot inline markup in title strings survives through bibliography rendering.
- Outer title rendering from presets/config still applies around inner Djot
  markup.
- Explicit inline title links take precedence over whole-title auto-linking.
- General title/text-case semantics are tracked separately in `csl26-wv5o`.

## Phase 2 Summary (2026-04-26)

Phase 2 implementation complete:
- Added `abstract_text` field to `Monograph`, `CollectionComponent`, and `SerialComponent` structs
- Implemented `InputReference::abstract_text()` dispatch to the three types above
- Applied `render_djot_inline` in `TemplateVariable::values()` for `Note` and `Abstract` variables
- Set `pre_formatted: true` for these variables to prevent double-encoding
- Created spec at `docs/specs/DJOT_RICH_TEXT.md` (Active)
- Schema regenerated and staged
- All tests pass (1093/1093)

## Summary of Changes (All Phases)

- refactored Djot inline rendering to preserve nested formatted child output and
  use frame-local span metadata
- added title-path Djot rendering with leaf-level smart apostrophe handling
- suppressed outer title autolinks when the title already contains an explicit
  inline link
- added regressions for title value pre-formatting, inline-link precedence, and
  title preset wrapping around Djot markup
- Phase 2: added abstract_text field and applied djot rendering for note/abstract
