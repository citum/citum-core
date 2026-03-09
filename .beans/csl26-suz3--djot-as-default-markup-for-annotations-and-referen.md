---
# csl26-suz3
title: Djot as default markup for annotations and reference fields
status: in-progress
type: feature
priority: normal
created_at: 2026-03-02T20:10:22Z
updated_at: 2026-03-09T20:25:41Z
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
- [x] Fix link rendering (URL lost at End event — degrade to text currently)
- [x] Phase 2: `RichText` type for `note`/`abstract` fields
- [ ] Phase 3: `title` field (requires AST-aware case transformation)

## Phase 3 — Title Markup (AST-aware case transformation)

**Goal:** Allow djot inline markup in title fields so authors can protect spans from
title-casing (e.g. `*Homo sapiens*` stays italicised and is not upper-cased by
an APA/Chicago title-case pass).

**Current rendering pipeline** (`crates/citum-engine/src/values/title.rs`):
1. `TemplateTitle::values()` calls `title_text()` — returns a plain `String`.
2. `smarten_apostrophes()` is applied to that string.
3. Result stored in `ProcValues { value, pre_formatted: false, ... }`.
4. Downstream, the engine applies title-case / sentence-case to `value` based on
   `TitleRendering` config.

**Problem:** Because casing is applied to the raw string, djot markup syntax
would either survive into output verbatim or be cased incorrectly.

**Required changes:**

1. AST-split approach in `title_text` / `TemplateTitle::values`:
   - Parse title string with jotdown into an inline event stream.
   - Walk the AST; apply title-case / sentence-case only to `Event::Str` leaf
     nodes (aware of first-word, after-colon, stop-word rules).
   - Re-emit the event stream using `render_djot_inline` with the
     case-transformed leaf strings.
   - Return result with `pre_formatted: true` to skip the second casing pass.

2. Case transformation helper needed:
   - An `apply_title_case_to_word(word, position: WordPosition) -> String`
     that knows about stop-words, first/last-word rules, etc.
   - Currently casing lives outside the values layer; needs to be threaded in
     or exposed as a shared helper.

3. `rich_text.rs` extension:
   - May need a `render_djot_inline_with_case_fn` variant that accepts a
     `Fn(&str) -> String` applied to each `Str` leaf event.

4. Casing bypass:
   - Wherever the casing pass currently runs, it must skip when
     `pre_formatted: true`.

**Key files:**
- `crates/citum-engine/src/values/title.rs` — main logic insertion point
- `crates/citum-engine/src/render/rich_text.rs` — case-aware djot renderer
- Wherever `TextCase` / `apply_text_case` runs in citum-engine/src/render/

**Definition of done:**
- Fixture reference with `title: "Homo sapiens and the modern world"` (italics
  on Homo sapiens via djot markup).
- APA title-case config renders: "Homo sapiens and the Modern World"
  (italicised span intact; stop-word "and" stays lower; first word capitalised).
- All existing title oracle tests pass.
