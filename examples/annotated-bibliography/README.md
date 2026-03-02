# Annotated Bibliography Example

This example shows how to render an annotated bibliography with Citum.

## Usage

```bash
# Render with APA style
citum render refs \
  -b references.yaml \
  -s apa-7th \
  --annotations annotations.yaml

# With italic annotations
citum render refs \
  -b references.yaml \
  -s apa-7th \
  --annotations annotations.yaml \
  --annotation-italic

# HTML output
citum render refs \
  -b references.yaml \
  -s apa-7th \
  --annotations annotations.yaml \
  --format html
```

## How it works

The `annotations.yaml` file is a plain mapping from reference ID to annotation text. It is separate from the reference data — the same references can carry different annotations in different documents (a course syllabus vs. a grant proposal vs. a personal reading list).

Citum appends annotations as a post-render step. The citation style itself is unchanged — any style works with `--annotations`.
