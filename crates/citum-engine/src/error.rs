/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
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

    /// Frontmatter parsing failed for a document input.
    #[error("Frontmatter parse error: {0}")]
    FrontmatterParse(String),

    /// Compound-set validation failed for a bibliography input.
    #[error("Compound set validation error: {0}")]
    CompoundSetValidation(String),

    /// Parsing a named reference input failed with a message describing the problem.
    #[error("Parse error ({name}): {message}")]
    RefsParse {
        /// Name of the input format or source that failed to parse.
        name: String,
        /// Human-readable parser message.
        message: String,
    },
}

impl From<citum_refs::RefsError> for ProcessorError {
    fn from(e: citum_refs::RefsError) -> Self {
        match e {
            citum_refs::RefsError::FileIO(io) => ProcessorError::FileIO(io),
            citum_refs::RefsError::ParseError(name, message) => {
                ProcessorError::RefsParse { name, message }
            }
        }
    }
}
