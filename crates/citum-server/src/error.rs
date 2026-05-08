/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use citum_engine::ProcessorError;
use std::borrow::Cow;
use std::io;
use thiserror::Error;

/// Server-level errors.
#[derive(Error, Debug)]
pub enum ServerError {
    /// The style YAML was found but failed to parse or validate against the schema.
    #[error("style validation failed: {0}")]
    StyleValidation(String),

    /// No resolver in the chain could locate the requested style name, path, or URI.
    #[error("style not found: {0}")]
    StyleNotFound(String),

    /// The style was found but inheritance or template resolution failed.
    #[error("style resolution failed: {0}")]
    StyleResolution(String),

    /// The chain resolver itself failed to initialize or run.
    #[error("resolver error: {0}")]
    ResolverError(String),

    /// Bibliography input could not be deserialized or rendered.
    #[error("bibliography processing failed: {0}")]
    BibliographyError(String),

    /// Citation input could not be deserialized or rendered.
    #[error("citation processing failed: {0}")]
    CitationError(String),

    /// An I/O operation failed while reading input or writing output.
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// JSON input or output failed to deserialize or serialize.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// YAML style parsing failed.
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// The underlying citation engine returned an error.
    #[error("engine error: {0}")]
    EngineError(#[from] ProcessorError),

    /// A required JSON-RPC parameter was missing from the request.
    #[error("missing required field: {0}")]
    MissingField(Cow<'static, str>),

    /// The request asked for an unsupported output format.
    #[error("unsupported output format: {0}")]
    UnsupportedOutputFormat(Cow<'static, str>),
}
