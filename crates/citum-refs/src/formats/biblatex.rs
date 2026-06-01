/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! BibLaTeX bibliography loading helpers.

use std::fs;
use std::path::Path;

use citum_schema::InputBibliography;

use crate::RefsError;

/// Parse BibLaTeX bibliography content from an in-memory string.
///
/// This is the core parsing primitive; [`load_biblatex`] delegates to it after
/// reading the file.
///
/// # Errors
///
/// Returns an error when `src` cannot be parsed as valid BibLaTeX.
pub fn parse_biblatex_str(src: &str) -> Result<InputBibliography, RefsError> {
    let bibliography = ::biblatex::Bibliography::parse(src)
        .map_err(|e| RefsError::ParseError("BibLaTeX".to_string(), e.to_string()))?;
    let references = bibliography
        .iter()
        .map(crate::biblatex::input_reference_from_biblatex)
        .collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

/// Load a BibLaTeX bibliography from a file path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed as BibLaTeX.
pub fn load_biblatex(path: &Path) -> Result<InputBibliography, RefsError> {
    let src = fs::read_to_string(path)?;
    parse_biblatex_str(&src)
}
