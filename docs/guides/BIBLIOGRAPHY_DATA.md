# Authoring bibliography data

Citum's native bibliography format is YAML or JSON built around a top-level
`references` list. The published
[`bib.json`](https://docs.citum.org/schemas/bib.json) schema supplies editor
completion and validation. This guide explains the choices that the schema
cannot make for you.

For exhaustive lookup, use the
[data-model field reference](../reference/generated/DATA_MODEL_FIELDS.md).
The [native-format examples](../reference/NATIVE_FORMAT.md) provide one
test-backed example for each reference class, and the
[data-model overview](../reference/DATA_MODEL.md) explains the underlying
design.

## Start with a complete bibliography

Add the schema comment at the top of a YAML file when your editor supports
YAML language-server schemas:

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/bib.json
references:
  - id: kuhn1962
    class: monograph
    type: book
    title: The Structure of Scientific Revolutions
    author:
      - family: Kuhn
        given: Thomas S.
    issued: "1962"
    publisher:
      name: University of Chicago Press
      place: Chicago
    isbn: "9780226458113"
```

Every reference needs a `class`. Most structural classes also need a `type`.
For example, books and reports use `class: monograph`, while journal articles
use `class: serial-component`. The field reference lists the allowed types for
each class.

Quote year-only dates and identifiers in YAML. Without quotes, a YAML parser
may treat them as numbers instead of text. Citum dates use EDTF strings, from a
year such as `"1962"` through a full date such as `"2026-10-27"`.

## Names and contributor roles

Use `author`, `editor`, and `translator` for the common roles. Each accepts a
structured personal name or an organization name. Use `contributors` when the
role is more specific, such as `introduction-author`, `director`, or
`performer`.

The following editor-only book also records a DOI. A DOI should contain the
identifier itself, not a `https://doi.org/` prefix.

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/bib.json
references:
  - id: wilke2017
    class: monograph
    type: book
    title:
      main: Readings in the Anthropocene
      sub: The Environmental Humanities, German Studies, and Beyond
    editor:
      - family: Wilke
        given: Sabine
      - family: Johnstone
        given: Japhet
    issued: "2017"
    publisher:
      name: Bloomsbury
      place: New York
    doi: 10.5040/9781501307782
```

Styles decide how a contributor role is labeled and where it appears. The data
records the person's role; it does not hard-code wording such as `ed. by` or
`intro. by`.

## Related works

`container`, `original`, `reviewed`, and `event` are work relations. A relation
can be either:

- the `id` of another reference in the same bibliography; or
- an embedded reference object with its own `class` and fields.

Use an ID when the related work should also exist as a standalone bibliography
entry. Embedding is convenient when the related work only supplies context for
the reference being cited.

This reprint embeds its original publication. The current edition keeps its
own `issued`, `publisher`, and `edition`; the first publication belongs under
`original`.

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/bib.json
references:
  - id: peary1985
    class: monograph
    type: book
    title: The North Pole
    author:
      - family: Peary
        given: Robert E.
    contributors:
      - roles: [introduction-author]
        contributor:
          family: Roosevelt
          given: Theodore
    issued: "1985"
    edition: facsimile repr.
    publisher:
      name: Time Life
      place: New York
    original:
      class: monograph
      type: book
      issued: "1910"
      publisher:
        name: Frederick A. Stokes
        place: New York
```

This article refers to a serial by ID instead of embedding it:

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/bib.json
references:
  - id: nature-journal
    class: serial
    type: academic-journal
    title: Nature
    issn: "0028-0836"
  - id: watson1953
    class: serial-component
    type: article
    title: "Molecular Structure of Nucleic Acids"
    author:
      - family: Watson
        given: James D.
      - family: Crick
        given: Francis H. C.
    issued: "1953-04-25"
    pages: "737-738"
    container: nature-journal
```

The same two forms apply to `reviewed`, for the work discussed by a review,
and `event`, for an associated conference, performance, or other event. The
available relation fields depend on the reference class; consult the field
reference before adding one.

## Validate, then render

Validate a native bibliography before rendering it:

```console
citum check --bibliography references.yaml
```

Schema validation answers whether the file has a supported data shape. It does
not guarantee that every style renders every valid field. Styles choose which
fields apply to each reference type. If a valid field is missing from rendered
output, check another style or a minimal fixture before reporting the problem;
the result may expose a style coverage gap rather than an input error.

When reporting an output problem, include the style name, a minimal valid
bibliography, the expected output, and the actual output. That information
separates data-model questions from style behavior quickly.
