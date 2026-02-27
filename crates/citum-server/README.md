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
echo '{"id":1,"method":"render_citation","params":{"style_path":"styles/apa-7th.yaml","refs":[...],"citation":{...}}}' \
  | citum-server
# {"id":1,"result":"Smith (2024)"}
```

### HTTP (feature-gated)

Build with `--features http` to expose the same three methods over HTTP via
`axum`. Useful for the citum-hub live preview panel.

```sh
cargo build --features http
citum-server --http --port 8080
# POST http://localhost:8080/rpc  (same JSON-RPC envelope)
```

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
