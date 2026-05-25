/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Citum JSON-RPC server.
//!
//! `citum-server` runs `citum-engine` behind a process boundary for clients
//! that need a standalone renderer instead of linking the engine directly.
//! The crate supports stdio JSON-RPC in every build and an optional axum HTTP
//! transport behind the `http` feature.
//!
//! See [`rpc`] for the request envelope, method surface, and stdio transport
//! details.
#![cfg_attr(
    feature = "http",
    doc = "See [`http`] for the feature-gated HTTP transport."
)]
#![cfg_attr(
    not(feature = "http"),
    doc = "The `http` module documents the optional HTTP transport when the feature is enabled."
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
/// Optional HTTP transport built on axum.
pub mod http;

pub use error::ServerError;
pub use rpc::dispatch;
