# Annotated Bibliography Support

## Overview

Citum supports annotated bibliographies through a document-scoped annotation overlay — a separate input layer passed alongside references at render time. Annotations are not stored on reference objects.

## Design Rationale

Annotations in an annotated bibliography are reader-authored prose composed *for a specific document* — a course reading list, a grant proposal, a research project. The same reference may carry different annotations in different contexts. This places annotations outside the reference schema entirely.

Two distinct concepts coexist deliberately:

| Field | Location | Authored by | Purpose |
|-------|----------|-------------|---------|
| `abstract` | `InputReference` | Work's original author | Summary of the work itself |
| annotation | `RenderInput` overlay | Bibliography author | Reader's evaluative note for this document |

## Data Model

Annotations and their rendering options are passed alongside references at render time:

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

## Rendering

Annotations are appended after the rendered bibliography entry as a post-render step. Styles require no modification. The processor checks: does an annotation exist for this reference ID? If yes, append it according to `AnnotationStyle`.

Default rendering (no `AnnotationStyle` supplied):
- Blank line before annotation
- Indented paragraph
- Plain text

## Style Escape Hatch

For styles that mandate specific annotation formatting (rare), a future style-level `annotation_style` override can be added. This avoids style proliferation — no separate `apa-7th-annotated.yaml` variants are needed.

## Input Formats

Annotations are passed as a flat map from reference ID to annotation text. Both JSON and YAML are supported; format is detected by file extension.

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

## CLI Usage

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
  --no-annotation-indent
```

## What This Is Not

- Not a note-taking system. Annotations are composed outside Citum.
- Not a replacement for `abstract` on the reference.
- Not a style concern for standard cases.
