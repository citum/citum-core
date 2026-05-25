# citum-server

`citum-server` exposes Citum citation rendering through JSON-RPC. Use it when a
client should talk to a long-running process instead of linking
`citum-engine` directly.

The same dispatcher is available over two transports:

- stdin/stdout: newline-delimited JSON-RPC, enabled in every build
- HTTP: `POST /rpc`, enabled by the default `http` feature

The document-level RPC method is `format_document`. The single-citation and
bibliography methods are convenience methods for smaller workflows.

## Run It

From the workspace root:

```sh
cargo run -p citum-server
```

That starts stdio mode. It reads one JSON object per line from stdin and writes
one JSON object per line to stdout.

For HTTP mode:

```sh
cargo run -p citum-server -- --http --port 9000
```

The examples below assume they are run from the Citum repository root so the
relative style path `styles/embedded/apa-7th.yaml` resolves. Outside the
repository, replace that value with an absolute path to a Citum YAML style.

## JSON-RPC Envelope

Every request has this shape:

```json
{
  "id": 1,
  "method": "render_citation",
  "params": {}
}
```

Successful responses echo the request ID and include `result`:

```json
{
  "id": 1,
  "result": "..."
}
```

Errors echo the request ID when available and include `error`:

```json
{
  "id": 1,
  "error": "missing required field: style_path"
}
```

`refs` in `render_citation` and `render_bibliography` is an inline JSON map of
reference objects. `refs` in `format_document` accepts a tagged input object —
see the `refs` input section below. Reference data uses native Citum schema.
Dates are EDTF strings such as `"1988"`, not CSL-JSON `date-parts` objects.

## Method Summary

| Method | Required params | Optional params | Result |
|---|---|---|---|
| `render_citation` | `style_path`, `refs`, `citation` | `output_format`, `inject_ast_indices` | rendered citation string |
| `render_bibliography` | `style_path`, `refs` | `output_format`, `inject_ast_indices` | `{format, content, entries}` |
| `validate_style` | `style_path` | none | `{valid, warnings}` |
| `format_document` | `style`, `refs`, `citations` | `output_format`, `locale`, `document_options` | `{formatted_citations, bibliography, warnings}` |

Supported `output_format` values are `plain` (default), `html`, `djot`,
`latex`, and `typst`.

`render_citation`, `render_bibliography`, and `validate_style` use
`style_path`, a string path to a local Citum YAML style. `format_document` uses
the richer `style` object:

```json
{ "kind": "path", "value": "styles/embedded/apa-7th.yaml" }
```

Other `style.kind` values are `id`, `uri`, and `yaml`.

## Stdio Example: `render_citation`

The stdio transport expects each request on a single line:

```sh
printf '%s\n' '{"id":1,"method":"render_citation","params":{"style_path":"styles/embedded/apa-7th.yaml","refs":{"hawking1988":{"id":"hawking1988","class":"monograph","type":"book","title":"A Brief History of Time","author":[{"family":"Hawking","given":"Stephen"}],"issued":"1988"}},"citation":{"id":"cite-1","items":[{"id":"hawking1988"}]}}}' \
  | cargo run -q -p citum-server
```

The response is a JSON object whose `result` is the rendered citation string.

## HTTP Example: `render_bibliography`

Start the server in another terminal:

```sh
cargo run -q -p citum-server -- --http --port 9000
```

Then send a request:

```sh
curl -s http://localhost:9000/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "id": 2,
    "method": "render_bibliography",
    "params": {
      "style_path": "styles/embedded/apa-7th.yaml",
      "output_format": "html",
      "refs": {
        "hawking1988": {
          "id": "hawking1988",
          "class": "monograph",
          "type": "book",
          "title": "A Brief History of Time",
          "author": [{ "family": "Hawking", "given": "Stephen" }],
          "issued": "1988"
        }
      }
    }
  }'
```

The response `result` contains:

- `format`: the selected output format
- `content`: the rendered bibliography
- `entries`: optional plain-text entry strings when the renderer supplies them

## Full Document Example: `format_document`

Use `format_document` when the client has the whole document citation order.
This is the preferred editor and word-processor integration method because it
lets the engine render document-sensitive citation state and bibliography output
in one call.

### HTTP `format_document`

Use this form when the server is running with `--http`:

```sh
curl -s http://localhost:9000/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "id": 3,
    "method": "format_document",
    "params": {
      "style": { "kind": "path", "value": "styles/embedded/apa-7th.yaml" },
      "output_format": "html",
      "refs": {
        "smith2010": {
          "id": "smith2010",
          "class": "monograph",
          "type": "book",
          "title": "Nationalism: Theory, Ideology, History",
          "author": [{ "family": "Smith", "given": "Anthony D." }],
          "issued": "2010",
          "publisher": { "name": "Polity" }
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
      "document_options": {
        "show_semantics": true
      }
    }
  }'
```

### Stdio JSON-RPC `format_document`

Use this form when the server is running in its default stdin/stdout mode. The
request must be one JSON object on one line:

```sh
printf '%s\n' '{"id":3,"method":"format_document","params":{"style":{"kind":"path","value":"styles/embedded/apa-7th.yaml"},"output_format":"html","refs":{"smith2010":{"id":"smith2010","class":"monograph","type":"book","title":"Nationalism: Theory, Ideology, History","author":[{"family":"Smith","given":"Anthony D."}],"issued":"2010","publisher":{"name":"Polity"}}},"citations":[{"id":"cite-1","items":[{"id":"smith2010","locator":{"label":"page","value":"10"}}]}],"document_options":{"show_semantics":true}}}' \
  | cargo run -q -p citum-server
```

The response `result` has the same top-level shape over HTTP and stdio:

```json
{
  "id": 3,
  "result": {
    "formatted_citations": [
      {
        "id": "cite-1",
        "text": "(<span class=\"citum-citation\" data-ref=\"smith2010\">Smith, <span class=\"citum-issued\">2010</span>, <span class=\"citum-variable\">p. 10</span></span>)",
        "ref_ids": ["smith2010"]
      }
    ],
    "bibliography": {
      "format": "html",
      "content": "<div class=\"citum-bibliography\">...</div>",
      "entries": [
        {
          "id": "smith2010",
          "text": "<div class=\"citum-bibliography\">...</div>",
          "metadata": {
            "author": "Smith",
            "year": "2010",
            "title": "Nationalism: Theory, Ideology, History"
          }
        }
      ]
    },
    "warnings": []
  }
}
```

Clients should read bibliography output from `result.bibliography`, not from
`result.formatted_citations`.

## `refs` Input for `format_document`

The `refs` field in `format_document` accepts a tagged input object, mirroring
the `style` field. These are alternative shapes, not one combined JSON value:

```json
{ "kind": "path",  "value": "/abs/path/to/refs.yaml" }
```

```json
{ "kind": "yaml",  "value": "hawking1988:\n  id: hawking1988\n  ..." }
```

```json
{ "kind": "json",  "value": { "hawking1988": { ... } } }
```

A bare JSON object (no `kind` key) is also accepted as `{"kind": "json"}` for
backward compatibility.

`path` and `yaml` have the server read and parse the bibliography file; `json`
passes inline reference data directly. Use `path` when the server and client
share a filesystem (e.g., when called via pipe from a LuaLaTeX document).

## Loading Local Files

Style and bibliography files on the same filesystem can be referenced by path
in both `style` and `refs`:

```sh
printf '%s\n' '{
  "id": 4,
  "method": "format_document",
  "params": {
    "style":        { "kind": "path", "value": "styles/embedded/apa-7th.yaml" },
    "refs":         { "kind": "path", "value": "examples/document-refs-native.json" },
    "output_format": "html",
    "citations":    [{ "id": "cite-1", "items": [{ "id": "kuhn1962" }] }]
  }
}' | cargo run -q -p citum-server
```

For HTTP clients that prefer inline data, assemble the request with `jq`:

```sh
jq -n \
  --arg style_path "styles/embedded/apa-7th.yaml" \
  --slurpfile refs examples/document-refs-native.json \
  --slurpfile citations examples/document-citations.json \
  '{
    id: 4,
    method: "format_document",
    params: {
      style: { kind: "path", value: $style_path },
      refs:  { kind: "json", value: $refs[0] },
      citations: $citations[0],
      output_format: "html"
    }
  }' | curl -s http://localhost:9000/rpc \
    -H 'Content-Type: application/json' \
    -d @-
```

## Discovery

HTTP mode exposes read-only discovery endpoints:

```sh
curl http://localhost:9000/rpc/methods
```

`GET /rpc/methods` returns supported methods, required fields, and optional
fields.

```sh
curl http://localhost:9000/rpc/schema
```

`GET /rpc/schema` returns JSON Schemas for method parameters when the binary is
built with the `schema` feature. Sending `GET /rpc` returns `405 Method Not
Allowed` with a JSON hint explaining that `/rpc` expects `POST`.

## Features

| Feature | Default | Description |
|---|---|---|
| `async` | on through `http` | Enables the Tokio runtime dependency used by HTTP transport |
| `http` | on | Enables the axum HTTP server and implies `async` |
| `schema` | off | Enables `/rpc/schema` and implies `http` plus schema type support |
| `schema-types` | off | Enables schema derivations without the HTTP schema endpoint |

Build a stdio-only binary with:

```sh
cargo build -p citum-server --no-default-features
cargo install citum-server --no-default-features
```

## Performance

A simulation script benchmarks a server-backed word-processor workflow:

```sh
cargo build --release -p citum-server
node scripts/benchmark-rpc-workflow.js
```

## Dependencies

`citum-server` depends on `citum-engine`, `citum-schema`, and `citum_store` for
rendering and the standard resolver chain. It does not depend on
`citum-migrate`.
