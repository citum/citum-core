---
# csl26-suz3
title: Djot as default markup for annotations and reference fields
status: in-progress
type: feature
priority: normal
created_at: 2026-03-02T20:10:22Z
updated_at: 2026-03-02T20:26:13Z
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
- [ ] Fix link rendering (URL lost at End event — degrade to text currently)
- [ ] Phase 2: `RichText` type for `note`/`abstract` fields
- [ ] Phase 3: `title` field (requires AST-aware case transformation)
