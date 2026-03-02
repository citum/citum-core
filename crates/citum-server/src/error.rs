/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use citum_engine::ProcessorError;
use std::io;
use thiserror::Error;

/// Server-level errors.
#[derive(Error, Debug)]
pub enum ServerError {
    /// The style file loaded successfully but failed schema validation.
    #[error("style validation failed: {0}")]
    StyleValidation(String),

    /// The requested style file could not be found on disk.
    #[error("style not found: {0}")]
    StyleNotFound(String),

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
    MissingField(String),

    /// The request asked for an unsupported output format.
    #[error("unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
}
