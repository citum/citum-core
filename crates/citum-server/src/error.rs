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
    #[error("style validation failed: {0}")]
    StyleValidation(String),

    #[error("style not found: {0}")]
    StyleNotFound(String),

    #[error("bibliography processing failed: {0}")]
    BibliographyError(String),

    #[error("citation processing failed: {0}")]
    CitationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("engine error: {0}")]
    EngineError(#[from] ProcessorError),

    #[error("missing required field: {0}")]
    MissingField(String),
}
