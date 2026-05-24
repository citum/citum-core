/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
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
//! - **stdio (default)**: Newline-delimited JSON-RPC on stdin/stdout. No HTTP listener or Tokio runtime is created.
//! - **HTTP (default feature)**: Exposes `/rpc` endpoint via axum.
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
//! - `async`: Enable the Tokio runtime dependency used by HTTP transport
//! - `http`: Enable HTTP server (implies async; enabled by default)

/// Server error types and conversions.
pub mod error;
/// JSON-RPC request handling and stdio transport.
pub mod rpc;

#[cfg(feature = "http")]
/// Optional HTTP transport built on axum.
pub mod http;

pub use error::ServerError;
pub use rpc::dispatch;
