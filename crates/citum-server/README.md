# citum-server

Long-running server process for real-time citation formatting. Wraps
`citum-engine` behind a newline-delimited JSON-RPC interface, with an optional
HTTP mode for web app integrations.

## Transports

### stdin/stdout (default)

The default mode reads JSON-RPC requests from stdin and writes responses to
stdout, one object per line. This matches the protocol used by citeproc-rs and
works cleanly inside word-processor plugins (Zotero, Pandoc pipelines) with no
port management.

```sh
echo '{"id":1,"method":"render_citation","params":{"style_path":"styles/apa-7th.yaml","refs":{"hawking1988":{"id":"hawking1988","class":"monograph","type":"book","title":"A Brief History of Time","author":[{"family":"Hawking","given":"Stephen"}],"issued":"1988"}},"citation":{"id":"cite-1","items":[{"id":"hawking1988"}]}}}' \
  | citum-server
# {"id":1,"result":"(Hawking, 1988)"}
```

### HTTP (feature-gated)

Build with `--features http` to expose the same three methods over HTTP via
`axum`. Useful for the citum-hub live preview panel.

```sh
cargo run -p citum-server --features http -- --http --port 8080
```

```sh
curl -s http://localhost:8080/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "id": 1,
    "method": "render_bibliography",
    "params": {
      "style_path": "styles/apa-7th.yaml",
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
# {"id":1,"result":["Hawking, S. (1988). A Brief History of Time."]}
```

> **Note:** `refs` uses native Citum schema format. `issued` is an EDTF string
> (`"1988"`), not a CSL-JSON `{"date-parts": [[1988]]}` object.

## Methods

| Method | Params | Result |
|---|---|---|
| `render_citation` | `style_path`, `refs`, `citation` | `String` |
| `render_bibliography` | `style_path`, `refs` | `[String]` |
| `validate_style` | `style_path` | `{valid, warnings}` |

### Request / response envelope

```json
{"id": 1, "method": "render_citation", "params": {"style_path": "styles/apa-7th.yaml", "refs": [...], "citation": {...}}}
{"id": 1, "result": "Smith (2024)"}

{"id": 2, "error": "style not found: missing.yaml"}
```

## Features

| Feature | Default | Description |
|---|---|---|
| `async` | off | Wraps `Processor` in `tokio::task::spawn_blocking` |
| `http` | off | Enables axum HTTP server; implies `async` |

## Usage

```sh
# stdio mode (default)
citum-server

# HTTP mode
cargo build --features http && citum-server --http --port 9000

# Options
citum-server --help
citum-server --version
```

## Dependencies

Depends only on `citum-engine` and `citum-schema`. No legacy or migrate
crates are pulled in.
