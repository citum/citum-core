# citum-server

Long-running server process for real-time citation formatting. Wraps
`citum-engine` behind a newline-delimited JSON-RPC interface, with HTTP mode
for web app integrations.

## Transports

### stdin/stdout (default)

The default mode reads JSON-RPC requests from stdin and writes responses to
stdout, one object per line. This matches the protocol used by citeproc-rs and
works cleanly inside word-processor plugins (Zotero, Pandoc pipelines) with no
port management.

```sh
echo '{"id":1,"method":"render_citation","params":{"style_path":"styles/embedded/apa-7th.yaml","refs":{"hawking1988":{"id":"hawking1988","class":"monograph","type":"book","title":"A Brief History of Time","author":[{"family":"Hawking","given":"Stephen"}],"issued":"1988"}},"citation":{"id":"cite-1","items":[{"id":"hawking1988"}]}}}' \
  | citum-server
# {"id":1,"result":"(Hawking, 1988)"}
```

### HTTP (default-enabled feature)

Default builds expose the same methods over HTTP via `axum`. Useful for the
citum-hub live preview panel. Install with
`cargo install citum-server --no-default-features` only when you need a
stdio-only binary.

```sh
cargo run -p citum-server -- --http --port 9000
```

```sh
curl -s http://localhost:9000/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "id": 1,
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
          "author": [{"family": "Hawking", "given": "Stephen"}],
          "issued": "1988"
        }
      }
    }
  }'
# {"id":1,"result":{"format":"html","content":"<div class=\"csln-bibliography\">...","entries":null}}
```

> **Note:** `refs` uses native Citum schema format. `issued` is an EDTF string
> (`"1988"`), not a CSL-JSON `{"date-parts": [[1988]]}` object.

## Performance

A simulation script is available to benchmark the server under a simulated word processor workflow (large bibliography, sequential citation insertion, and periodic bibliography refreshes).

```sh
# Ensure the server is built in release mode first
cargo build --release -p citum-server

# Run the simulation
node scripts/benchmark-rpc-workflow.js
```

## Methods

| Method | Params | Result |
|---|---|---|
| `render_citation` | `style_path`, `refs`, `citation`, `output_format?`, `inject_ast_indices?` | `String` |
| `render_bibliography` | `style_path`, `refs`, `output_format?`, `inject_ast_indices?` | `{format, content, entries?}` |
| `validate_style` | `style_path` | `{valid, warnings}` |
| `format_document` | `style`, `refs`, `citations`, `document_options?`, `output_format?`, `locale?` | `{formatted_citations, bibliography, warnings}` |

Supported `output_format` values:

- `plain` (default)
- `html`
- `djot`
- `latex`
- `typst`

**Debug Parameters:** The `inject_ast_indices` parameter (optional, default `false`) is accepted by `render_citation` and `render_bibliography` for debug use. When enabled, AST node indices are embedded in the output.

### Request / response envelope

```json
{"id": 1, "method": "render_citation", "params": {"style_path": "styles/embedded/apa-7th.yaml", "refs": [...], "citation": {...}, "output_format": "html"}}
{"id": 1, "result": "Smith (2024)"}

{"id": 2, "error": "style not found: missing.yaml"}
```

## Discovery

When running in HTTP mode, two read-only discovery endpoints are available:

```sh
# List all supported methods
curl http://localhost:9000/rpc/methods

# JSON Schema for method parameters (requires --features schema build)
curl http://localhost:9000/rpc/schema
```

Sending a `GET` request to `/rpc` returns a `405 Method Not Allowed` response with a JSON hint explaining POST usage.

## Batch Document Formatting

The `format_document` method is designed for processing a full document in one request. For practical workflows where data is stored in local files, you can use `jq` to assemble the JSON-RPC payload.

```sh
# Assemble and send a document formatting request using local JSON files
jq -n \
  --arg style_path "styles/embedded/apa-7th.yaml" \
  --slurpfile refs examples/document-refs-native.json \
  --slurpfile citations examples/document-citations.json \
  --slurpfile options examples/document-options.json \
  '{
    id: 1,
    method: "format_document",
    params: {
      style: { kind: "path", value: $style_path },
      refs: $refs[0],
      citations: $citations[0],
      document_options: $options[0],
      output_format: "html"
    }
  }' | curl -s http://localhost:9000/rpc \
    -H 'Content-Type: application/json' \
    -d @-
```

## Working with Local Files

When running `citum-server` locally, the `style_path` (or `style`) parameter accepts relative or absolute paths to Citum YAML files.

However, the `refs` and `citations` parameters always expect data objects, not paths. This allows the server to remain transport-agnostic and simplifies its security model (preventing arbitrary file reads by the server process). To use local files for these parameters, load them into the request payload on the client side as shown in the example above.

## Features

| Feature | Default | Description |
|---|---|---|
| `async` | on | Enables the Tokio runtime dependency used by HTTP transport |
| `http` | on | Enables axum HTTP server; **requires and automatically enables `async`** |
| `schema` | off | Enables `/rpc/schema` endpoint; **requires and automatically enables `http` and `async`** |

## Usage

```sh
# stdio mode (default)
citum-server

# HTTP mode
citum-server --http --port 9000

# stdio-only binary
cargo build -p citum-server --no-default-features
cargo install citum-server --no-default-features

# Options
citum-server --help
citum-server --version
```

## Dependencies

Depends on `citum-engine`, `citum-schema`, and `citum_store` for the standard
resolver chain. No migrate crate is pulled in.
