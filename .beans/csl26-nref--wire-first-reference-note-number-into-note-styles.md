---
# csl26-nref
title: 'Wire first-reference-note-number into note-based styles for cross-references'
status: todo
type: enhancement
priority: low
created_at: 2026-07-01T00:00:00Z
updated_at: 2026-07-01T00:00:00Z
---

CMOS18's classic note-cross-reference behavior ("see note 5 above") is not
emitted by any shipped note-based style (e.g. `styles/embedded/chicago-notes-18th.yaml`),
even though `citum-engine` already fully implements the underlying mechanism.

## Context

- `Processor::normalize_note_context` (`crates/citum-engine/src/processor/note_context.rs:116-152`)
  tracks `first_note_by_id: HashMap<String, u32>` — the note number where each
  reference first appeared.
- Exposed to templates via `ProcHints.first_reference_note_number` →
  `NumberVariable::FirstReferenceNoteNumber`
  (`crates/citum-engine/src/values/number.rs:84-86`), and as the schema type
  `FirstReferenceNoteNumber` (`crates/citum-schema-style/src/template.rs:959,987,1016`,
  serializes as `first-reference-note-number`).
- `chicago-notes-18th.yaml`'s `subsequent`/`ibid` templates only use shortened
  author/title forms or bare `term.ibid` — none reference this variable.

## Approach

- [ ] Confirm which CMOS18-family styles (and any other note-based styles,
      e.g. legal/Bluebook variants) are expected to render note back-references
      rather than (or in addition to) shortened-form subsequent citations.
- [ ] Add `number: first-reference-note-number` to the relevant `subsequent`
      template(s), with appropriate label wording (e.g. "(see note N above)").
- [ ] Verify against oracle/fidelity corpus that this doesn't regress existing
      shortened-citation behavior where CMOS18 doesn't call for cross-references.

This is a style-authoring change, not an engine change — no schema or
processor work is required.
