/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! CSL-JSON bibliography loading helpers.

use std::fs;
use std::path::Path;

use citum_schema::InputBibliography;
use citum_schema::reference::InputReference;
use csl_legacy::csl_json::Reference as LegacyReference;

use crate::RefsError;

/// Load a CSL-JSON bibliography file.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed as CSL-JSON.
pub fn load_csl_json(path: &Path) -> Result<InputBibliography, RefsError> {
    let bytes = fs::read(path)?;
    let refs: Vec<LegacyReference> = serde_json::from_slice(&bytes)
        .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?;
    let references = refs.into_iter().map(InputReference::from).collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}
