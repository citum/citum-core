/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citum JSON-RPC Server
//!
//! This crate provides a JSON-RPC server for citation and bibliography processing.
//! It wraps the `citum-engine` functionality in a request-response protocol suitable
//! for use with word processors, web applications, and other clients.
//!
//! ## Modes
//!
//! The server supports two transport modes:
//!
//! - **stdio (default)**: Newline-delimited JSON-RPC on stdin/stdout. No runtime overhead.
//! - **HTTP (optional, requires `http` feature)**: Exposes `/rpc` endpoint via axum.
//!
//! ## Example (stdio mode)
//!
//! ```bash
//! echo '{"id": 1, "method": "validate_style", "params": {"style_path": "path/to/style.yaml"}}' \
//!   | citum-server
//! ```
//!
//! ## Features
//!
//! - `async`: Enable tokio runtime (required for HTTP)
//! - `http`: Enable HTTP server (implies async)

pub mod error;
pub mod rpc;

#[cfg(feature = "http")]
pub mod http;

pub use error::ServerError;
pub use rpc::dispatch;
