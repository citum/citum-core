/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Citum JSON-RPC server.
//!
//! `citum-server` runs `citum-engine` behind a process boundary for clients
//! that need a standalone renderer instead of linking the engine directly.
//! The crate supports stdio JSON-RPC in every build and an axum HTTP transport
//! through the default-on `http` feature. Both transports expose the same
//! JSON-RPC method surface; only the framing changes.
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
//! `refs` in `render_citation` and `render_bibliography` is an inline JSON map
//! of native Citum reference objects. `refs` in `format_document` accepts
//! `citum-engine`'s tagged [`citum_engine::RefsInput`] shape, including local
//! bibliography paths, inline YAML, inline JSON, and legacy bare JSON maps.
//! Dates are EDTF strings such as `"1988"`, not CSL-JSON `date-parts` objects.
//!
//! `render_citation`, `render_bibliography`, and `validate_style` accept
//! `style_path`, a string path to a local Citum YAML style. `format_document`
//! accepts the richer `style` object, for example
//! `{ "kind": "path", "value": "styles/embedded/apa-7th.yaml" }`.
//!
//! See [`rpc`] for the request envelope and stdio transport details.
#![cfg_attr(feature = "http", doc = "See [`http`] for the default HTTP transport.")]
#![cfg_attr(
    not(feature = "http"),
    doc = "The HTTP transport is unavailable in this build because the `http` feature is disabled."
)]
//!
//! ## Features
//!
//! - `async`: Tokio runtime support used by HTTP transport.
//! - `http`: axum HTTP transport; enabled by default and implies `async`.
//! - `schema`: `/rpc/schema` plus schema derivations.
//! - `schema-types`: schema derivations without the HTTP schema endpoint.

/// Server error types and conversions.
pub mod error;
/// JSON-RPC request handling and stdio transport.
pub mod rpc;

#[cfg(feature = "http")]
/// Default HTTP transport built on axum.
pub mod http;

pub use error::ServerError;
pub use rpc::dispatch;
