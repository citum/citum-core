/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! BibLaTeX bibliography loading helpers.

use std::path::Path;

use citum_engine::ProcessorError;
use citum_schema::InputBibliography;

/// Load a BibLaTeX bibliography from a file path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed as BibLaTeX.
pub(crate) fn load_biblatex_bibliography(path: &Path) -> Result<InputBibliography, ProcessorError> {
    citum_refs::formats::biblatex::load_biblatex(path).map_err(|e| ProcessorError::RefsParse {
        name: "BibLaTeX".to_string(),
        message: e.to_string(),
    })
}
