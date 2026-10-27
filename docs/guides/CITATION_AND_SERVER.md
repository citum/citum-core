# Citations and JSON-RPC integration

Citum separates citation occurrences from the bibliography entries they cite.
Use the published [`citation.json`](../schemas/citation.json) schema for
ordered citation data, and the [`server.json`](../schemas/server.json) schema
for JSON-RPC method parameters. This guide connects those generated contracts
to the common application tasks they support.

For the fields inside `refs`, start with [Authoring bibliography data](BIBLIOGRAPHY_DATA.md).
The [generated data-model field reference](../reference/generated/DATA_MODEL_FIELDS.md)
and [native-format examples](../reference/NATIVE_FORMAT.md) are the exhaustive
lookup pages; this guide focuses on citation order, transport, and responses.

## Start with a citation list

A citation file is an ordered YAML or JSON array. Each item in `items` names a
reference ID from the bibliography supplied by the host application. Add the
schema hint when your editor supports YAML language-server schemas:

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/citation.json
# schema: citation
- id: kuhn-page-10
  mode: non-integral
  items:
    - id: kuhn1962
      locator:
        label: page
        value: "10"
      prefix: "see "
- id: kuhn-narrative
  mode: integral
  items:
    - id: kuhn1962
```

The citation schema validates the citation shape, not whether `kuhn1962`
exists in a particular bibliography. The host must provide matching reference
IDs when it renders the document. Use `mode: integral` for a narrative citation
such as “Kuhn (1962)” and `mode: non-integral` for a parenthetical citation
such as “(Kuhn, 1962)”; styles that do not distinguish those forms ignore the
mode.

Useful citation fields are:

- `items`: the references cited by the cluster, in input order;
- `locator`: a pinpoint such as a page, chapter, or section;
- `prefix` and `suffix`: text attached to the whole cluster or one item; and
- `position`: an explicit note-style position when the document processor has
  already determined it. Otherwise the document-level API infers position from
  citation order.

Dates, names, titles, and identifiers belong in the bibliography data rather
than in a citation occurrence. The generated [field reference](../reference/generated/DATA_MODEL_FIELDS.md)
and [native data-model reference](../reference/NATIVE_FORMAT.md) describe those
reference fields.

## Choose a JSON-RPC method

The server accepts the same request envelope over newline-delimited stdio and
HTTP `POST /rpc`:

```json
{
  "id": 1,
  "method": "format_document",
  "params": {}
}
```

Use `format_document` when the host has the complete document. It is the
preferred integration path because the engine sees citation order, can infer
note positions and disambiguation, and returns citations and the bibliography
from one render. Use `render_citation` for an isolated preview or a one-off
citation where document context is not needed. `render_bibliography` renders
the cited reference map without citation occurrences, and `validate_style`
checks a local style file.

The generated server schema describes each method's `params` object. It does
not replace the JSON-RPC envelope or describe rendered response text; the
response examples below show the stable result shapes clients should consume.

## Render one citation

This request uses the local APA style and an inline native reference map. The
`citation` value has the same shape as one element of the citation list above.
The `style_path` form is available to `render_citation`,
`render_bibliography`, and `validate_style`.

```json server:render_citation
{
  "id": 1,
  "method": "render_citation",
  "params": {
    "style_path": "styles/embedded/apa-7th.yaml",
    "refs": {
      "hawking1988": {
        "id": "hawking1988",
        "class": "monograph",
        "type": "book",
        "title": "A Brief History of Time",
        "author": [
          { "family": "Hawking", "given": "Stephen" }
        ],
        "issued": "1988"
      }
    },
    "citation": {
      "id": "cite-1",
      "items": [
        { "id": "hawking1988" }
      ]
    },
    "output_format": "plain"
  }
}
```

The server echoes the request ID and puts the rendered citation string in
`result`:

```json response
{
  "id": 1,
  "result": "(Hawking, 1988)"
}
```

For HTTP, send the same JSON object to `/rpc`:

```console
curl -s http://localhost:9000/rpc \
  -H 'Content-Type: application/json' \
  -d @render-citation.json
```

For stdio, send one compact JSON object per line to `citum-server`:

```console
printf '%s\n' "$(jq -c . render-citation.json)" | citum-server
```

## Format a complete document

This request uses the tagged `style` and `refs` forms accepted by
`format_document`. A `path` style is resolved on the server's filesystem; a
`json` reference input keeps the bibliography inline. Replace either with
`kind: yaml` for inline YAML, or use a shared file path when the client and
server run on the same machine.

```json server:format_document
{
  "id": 2,
  "method": "format_document",
  "params": {
    "style": {
      "kind": "path",
      "value": "styles/embedded/apa-7th.yaml"
    },
    "refs": {
      "kind": "json",
      "value": {
        "smith2010": {
          "id": "smith2010",
          "class": "monograph",
          "type": "book",
          "title": "Nationalism: Theory, Ideology, History",
          "author": [
            { "family": "Smith", "given": "Anthony D." }
          ],
          "issued": "2010",
          "publisher": { "name": "Polity" }
        }
      }
    },
    "citations": [
      {
        "id": "cite-1",
        "items": [
          {
            "id": "smith2010",
            "locator": { "label": "page", "value": "10" }
          }
        ]
      }
    ],
    "output_format": "plain"
  }
}
```

The response preserves citation order and returns the bibliography as a
structured object. `bibliography.entries` is useful for editors that need to
attach metadata to individual entries; `bibliography.content` is the complete
rendered bibliography body.

```json response
{
  "id": 2,
  "result": {
    "formatted_citations": [
      {
        "id": "cite-1",
        "text": "(Smith, 2010, p. 10)",
        "ref_ids": ["smith2010"]
      }
    ],
    "bibliography": {
      "format": "plain",
      "content": "Smith, A. D. (2010). _Nationalism: Theory, Ideology, History_. Polity.",
      "entries": [
        {
          "id": "smith2010",
          "text": "Smith, A. D. (2010). _Nationalism: Theory, Ideology, History_. Polity.",
          "metadata": {
            "author": "Smith",
            "year": "2010",
            "title": "Nationalism: Theory, Ideology, History"
          }
        }
      ]
    },
    "bibliography_blocks": [],
    "warnings": [
      {
        "level": "warning",
        "code": "unknown_role_label_term",
        "message": "Style bibliography layout uses unrecognized role-label term 'interviewer'; falling back to the contributor's own role term instead of the requested one."
      }
    ]
  }
}
```

If the response includes warnings, treat them as diagnostics rather than
assuming that the rendered strings are invalid. Unknown reference IDs and
unsupported input fields are reported through the warning list where the
request can still be processed.

## Validate and integrate

Use the published schemas before sending requests to a server or saving input
files:

```console
citum schema citation
citum schema server
```

For editor completion, reference the hosted schema URLs directly:

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/citation.json
```

The [JSON Schemas page](../schemas/) links the machine-readable artifacts and
the [generated field reference](../reference/generated/DATA_MODEL_FIELDS.md).
The [server crate README](https://github.com/citum/citum-core/blob/main/crates/citum-server/README.md)
covers session methods, HTTP discovery endpoints, and feature flags; use the stateless
`format_document` path above unless the client needs incremental session state.
