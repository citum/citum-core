/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use thiserror::Error;

/// Errors produced while resolving or rendering citations and bibliographies.
#[derive(Error, Debug)]
pub enum ProcessorError {
    /// A citation referenced an item ID that is not present in the bibliography.
    #[error("Reference not found: {0}")]
    ReferenceNotFound(String),

    /// Date parsing or normalization failed for a reference field.
    #[error("Date parse error: {0}")]
    DateParseError(String),

    /// Locale data could not be loaded or did not contain a required term.
    #[error("Locale error: {0}")]
    LocaleError(String),

    /// Contributor or title substitution logic failed.
    #[error("Substitution error: {0}")]
    SubstitutionError(String),

    /// Reading an input file from disk failed.
    #[error("File I/O error: {0}")]
    FileIO(#[from] std::io::Error),

    /// Parsing a named input failed with a message describing the problem.
    #[error("Parse error ({0}): {1}")]
    ParseError(String, String),
}
