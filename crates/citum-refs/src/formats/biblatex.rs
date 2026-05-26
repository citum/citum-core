/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! BibLaTeX bibliography loading helpers.

use std::path::Path;

use citum_schema::InputBibliography;

use crate::RefsError;

/// Load a BibLaTeX bibliography from a file path.
///
/// # Errors
///
/// Returns an error; BibLaTeX loading is not available in citum-refs due to circular
/// dependency constraints. Use `citum-io::load_input_bibliography()` with
/// `RefsFormat::Biblatex` instead.
pub fn load_biblatex(_path: &Path) -> Result<InputBibliography, RefsError> {
    Err(RefsError::ParseError(
        "BibLaTeX".to_string(),
        "BibLaTeX loading is not available in citum-refs. Use citum-io::load_input_bibliography() with RefsFormat::Biblatex instead.".to_string(),
    ))
}
