# Annotated Bibliography Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-09
**Supersedes:** docs/architecture/ANNOTATED_BIBLIOGRAPHY.md
**Related:** citum_engine, `citum render refs --annotations`

## Purpose

Defines how Citum supports annotated bibliographies: a document-scoped
annotation overlay passed at render time, separate from the reference
schema. All styles support annotations by default with no opt-in required.

## Scope

**In scope:**
- Annotation data model (`AnnotationStyle`, `ParagraphBreak`)
- Rendering pipeline — how annotations attach to bibliography entries
- Input formats (YAML, JSON annotation maps)
- CLI interface (`--annotations`, `--annotation-italic`, `--annotation-indent`)
- Optional per-style `annotation_style` defaults

**Out of scope:**
- Inline citations with annotation text
- Note-taking or annotation authoring workflows
- Storage of annotations on `InputReference` objects

## Design

### Data Model

Annotations are not stored on reference objects. They are passed as a flat
map from reference ID to annotation text alongside references at render time.

Two concepts coexist deliberately:

| Field | Location | Authored by | Purpose |
|-------|----------|-------------|---------|
| `abstract` | `InputReference` | Work's original author | Summary of the work itself |
| annotation | `RenderInput` overlay | Bibliography author | Reader's evaluative note for this document |

Rendering options:

```rust
pub struct AnnotationStyle {
    pub italic: bool,
    pub indent: bool,
    pub paragraph_break: ParagraphBreak,
}

pub enum ParagraphBreak {
    SingleLine,
    BlankLine,  // default
}
```

### Rendering

Annotations are appended after the rendered bibliography entry as a
post-render step. The processor checks: does an annotation exist for this
reference ID in the overlay map? If yes, append it per `AnnotationStyle`.
If no annotation map is supplied, output is identical to a standard bibliography.

Default rendering (no `AnnotationStyle` supplied):
- Blank line before annotation
- Flush left paragraph (no indentation)
- Plain text

### Style Formatting Defaults (Optional)

Styles may optionally declare `annotation_style` defaults for their context
(e.g. a journal style that mandates italic annotations). This is never
required. It does not gate annotation support — all styles render annotations
when an annotation map is supplied.

This avoids style proliferation: there are no `apa-7th-annotated.yaml` variants.

### Input Formats

Annotations are passed as a flat map from reference ID to annotation text.
Format is detected by file extension.

**YAML** (recommended for hand-authoring):
```yaml
smith2019: >
  A foundational treatment of X. Particularly useful for its comparative
  methodology, which complements the quantitative approach in jones2021.

jones2021: >
  Useful for its comparative methodology. Best read alongside smith2019.
```

**JSON**:
```json
{
  "smith2019": "A foundational treatment of X...",
  "jones2021": "Useful for its comparative methodology..."
}
```

### CLI Interface

```bash
citum render refs \
  -b references.json \
  -s styles/apa-7th.yaml \
  --annotations annotations.yaml

# With formatting options:
citum render refs \
  -b references.json \
  -s styles/apa-7th.yaml \
  --annotations annotations.yaml \
  --annotation-italic \
  --annotation-indent
```

## Implementation Notes

- Annotation rendering is a post-render step in the engine — not a template
  concern — so no style changes are needed to support it.
- The overlay map is keyed by the same reference IDs used in `RenderInput`.
- `AnnotationStyle` defaults: `italic: false`, `indent: false`,
  `paragraph_break: BlankLine`.

## Acceptance Criteria

- [ ] `citum render refs --annotations <file>` appends annotation text after each entry
- [ ] Missing annotation for a reference ID produces no output (not an error)
- [ ] Omitting `--annotations` produces output identical to a standard bibliography
- [ ] `--annotation-italic` and `--no-annotation-indent` flags work independently
- [ ] YAML and JSON annotation files both parse correctly
- [ ] No style file modification is required to enable annotation output

## Changelog

- v1.0 (2026-03-09): Migrated from docs/architecture/ and formatted as spec.
