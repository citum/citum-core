---
# csl26-xrc5
title: Short-title-for-disambiguation + first-reference-note-number (CSL schema#452 part B)
status: completed
type: task
priority: normal
created_at: 2026-05-29T11:15:03Z
updated_at: 2026-05-29T13:03:40Z
---

Design and implement a disambiguation strategy that can render a short title *only* when needed to disambiguate, compatible with `first-reference-note-number` cross-references.

Citum has note-number support (`processor/document/note_support.rs`) but no short-title-for-disambiguation strategy. This is larger than the `disambiguate.ignore` work (csl26-zrz5); treat as a separate feature.

## The problem

Two works by the same author, same year, cited in a note style (e.g. Chicago):

- Smith, *A History of Rome* (2020)
- Smith, *A History of Greece* (2020)

A style might add a short title *only* when disambiguation is needed:

| Context | Current CSL | Desired |
|---|---|---|
| First cite of *Rome* | Smith, *Rome*, 2020, 45. ✓ | same |
| First cite of *Greece* | Smith, *Greece*, 2020, 67. ✓ | same |
| Later short cite of *Rome* | Smith, *Rome*, see n. 1. | Smith, see n. 1. |

The short title leaks into the `first-reference-note-number` form because CSL has
no way to say "show title only when disambiguating AND not in cross-ref position."
The note number already identifies the work; the short title is redundant there.

Spec context: docs/specs/DISAMBIGUATION.md §6 (open question).

Refs:
- CSL schema#452: https://github.com/citation-style-language/schema/issues/452
- CSL styles#7667: https://github.com/citation-style-language/styles/issues/7667
- Zotero forum discussion: https://forums.zotero.org/discussion/124486/apa7-add-letter-to-publication-date-even-if-originally-published-date-is-different

## Summary of Changes

- Added `NumberVariable::FirstReferenceNoteNumber` variant (key: `first-reference-note-number`) to template.rs. Renders from `ProcHints.first_reference_note_number`.
- Added `first_reference_note_number: Option<u32>` and `suppress_disambiguation_title: bool` to `ProcHints`.
- Added `first_note_by_id: RefCell<HashMap<String,u32>>` to `Processor`; populated in `normalize_note_context` (first occurrence wins per reference id).
- Threaded `first_reference_note_number` through `TemplateRenderRequest` → `build_template_render_hint`. Set to `Some(n)` for `Subsequent` position only.
- Extended `title.rs` `disambiguate_only` guard: suppress when `suppress_disambiguation_title` is true (note number supersedes title as identifier).
- Regenerated `docs/schemas/style.json`.
- Added test `disambiguate_only_title_suppressed_when_first_ref_note_number_is_present` in citations.rs.
