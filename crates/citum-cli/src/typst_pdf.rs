/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Typst PDF compilation shim — delegates to the `citum-pdf` crate when the
//! `typst-pdf` feature is enabled.

use std::error::Error;
use std::path::Path;

#[cfg(feature = "typst-pdf")]
/// Compile a generated Typst document to PDF.
pub fn compile_document_to_pdf(
    source: &str,
    output: &Path,
    keep_source: bool,
) -> Result<(), Box<dyn Error>> {
    citum_pdf::compile_document_to_pdf(source, output, keep_source)
}

#[cfg(not(feature = "typst-pdf"))]
/// Compile a generated Typst document to PDF.
pub fn compile_document_to_pdf(
    _source: &str,
    _output: &Path,
    _keep_source: bool,
) -> Result<(), Box<dyn Error>> {
    Err("Typst PDF compilation requires building `citum` with the `typst-pdf` feature.".into())
}
