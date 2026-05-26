/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! BibLaTeX bibliography loading helpers.

use std::fs;
use std::path::Path;

use citum_engine::ProcessorError;
use citum_schema::InputBibliography;

/// Load a BibLaTeX bibliography from a file path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed as BibLaTeX.
///
/// Note: BibLaTeX loading is kept in citum-io to avoid circular dependencies with citum-refs.
/// The conversion logic uses `crate::biblatex::input_reference_from_biblatex`.
pub(crate) fn load_biblatex_bibliography(path: &Path) -> Result<InputBibliography, ProcessorError> {
    let src = fs::read_to_string(path)?;
    let bibliography = ::biblatex::Bibliography::parse(&src)
        .map_err(|e| ProcessorError::ParseError("BibLaTeX".to_string(), e.to_string()))?;
    let references = bibliography
        .iter()
        .map(crate::biblatex::input_reference_from_biblatex)
        .collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}
