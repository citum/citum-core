/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! JSON-RPC server for Citum citation rendering.
//!
//! `citum-server` wraps `citum-engine` in a process boundary for clients that
//! should not link the Rust engine directly. It supports lightweight
//! single-citation workflows and full-document rendering through the
//! `format_document` RPC method.
//!
//! ## Transports
//!
//! - **stdio**: newline-delimited JSON-RPC on stdin/stdout. This transport is
//!   available in every build and is the default runtime mode.
//! - **HTTP**: `POST /rpc` using axum. This transport is enabled by the default
//!   `http` feature and is selected with `citum-server --http`.
//!
//! ## JSON-RPC envelope
//!
//! Requests use `{ "id", "method", "params" }`. Successful responses echo the
//! request ID and include `result`; failures echo the request ID when available
//! and include `error`.
//!
//! ## Methods
//!
//! | Method | Required params | Optional params | Result |
//! |---|---|---|---|
//! | `render_citation` | `style_path`, `refs`, `citation` | `output_format`, `inject_ast_indices` | rendered citation string |
//! | `render_bibliography` | `style_path`, `refs` | `output_format`, `inject_ast_indices` | rendered bibliography object |
//! | `validate_style` | `style_path` | none | validation object |
//! | `format_document` | `style`, `refs`, `citations` | `output_format`, `locale`, `document_options` | `{formatted_citations, bibliography, warnings}` |
//!
//! `refs` uses native Citum reference data. Dates are EDTF strings such as
//! `"1988"`, not CSL-JSON `date-parts` objects.
//!
//! `render_citation`, `render_bibliography`, and `validate_style` accept
//! `style_path`, a string path to a local Citum YAML style. `format_document`
//! accepts the richer `style` object, for example
//! `{ "kind": "path", "value": "styles/embedded/apa-7th.yaml" }`.
//!
//! ## `format_document` result
//!
//! `format_document` returns one document-level result object with three
//! top-level fields:
//!
//! - `formatted_citations`: one rendered citation object per input citation.
//! - `bibliography`: the rendered bibliography object, including `format`,
//!   `content`, and `entries`.
//! - `warnings`: non-fatal diagnostics produced while evaluating the style.
//!
//! Clients should read bibliography output from `result.bibliography`, not from
//! `result.formatted_citations`.
//!
//! ## Stdio example
//!
//! From the Citum repository root:
//!
//! ```text
//! printf '%s\n' '{"id":1,"method":"render_citation","params":{"style_path":"styles/embedded/apa-7th.yaml","refs":{"hawking1988":{"id":"hawking1988","class":"monograph","type":"book","title":"A Brief History of Time","author":[{"family":"Hawking","given":"Stephen"}],"issued":"1988"}},"citation":{"id":"cite-1","items":[{"id":"hawking1988"}]}}}' \
//!   | cargo run -q -p citum-server
//! ```
//!
//! ## HTTP example
//!
//! Start the server:
//!
//! ```text
//! cargo run -q -p citum-server -- --http --port 9000
//! ```
//!
//! Then send a request:
//!
//! ```text
//! curl -s http://localhost:9000/rpc \
//!   -H 'Content-Type: application/json' \
//!   -d '{"id":2,"method":"format_document","params":{"style":{"kind":"path","value":"styles/embedded/apa-7th.yaml"},"output_format":"html","refs":{"smith2010":{"id":"smith2010","class":"monograph","type":"book","title":"Nationalism: Theory, Ideology, History","author":[{"family":"Smith","given":"Anthony D."}],"issued":"2010","publisher":{"name":"Polity"}}},"citations":[{"id":"cite-1","items":[{"id":"smith2010","locator":{"label":"page","value":"10"}}]}],"document_options":{"show_semantics":true}}}'
//! ```
//!
//! `format_document` returns a document-level result object. Read formatted
//! citations from `result.formatted_citations`, bibliography output from
//! `result.bibliography`, and diagnostics from `result.warnings`.
//!
//! ```json
//! {
//!   "id": 2,
//!   "result": {
//!     "formatted_citations": [
//!       { "id": "cite-1", "text": "...", "ref_ids": ["smith2010"] }
//!     ],
//!     "bibliography": {
//!       "format": "html",
//!       "content": "<div class=\"citum-bibliography\">...</div>",
//!       "entries": [
//!         { "id": "smith2010", "text": "...", "metadata": { "author": "Smith", "year": "2010", "title": "Nationalism: Theory, Ideology, History" } }
//!       ]
//!     },
//!     "warnings": []
//!   }
//! }
//! ```
//!
//! ## Features
//!
//! - `async`: enable the Tokio runtime dependency used by HTTP transport.
//! - `http`: enable the axum HTTP server; implies `async` and is enabled by
//!   default.
//! - `schema`: enable the `/rpc/schema` endpoint and schema derivations.
//! - `schema-types`: enable schema derivations without the HTTP schema endpoint.

/// Server error types and conversions.
pub mod error;
/// JSON-RPC request handling and stdio transport.
pub mod rpc;

#[cfg(feature = "http")]
/// Optional HTTP transport built on axum.
pub mod http;

pub use error::ServerError;
pub use rpc::dispatch;
