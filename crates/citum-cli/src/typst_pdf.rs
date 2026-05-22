/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Typst PDF compilation shim. Bundled-Typst PDF output isn't available in
//! the crates.io-published `citum` binary because the `citum-pdf` helper
//! crate isn't yet on crates.io. Users on the published binary should run
//! `citum render doc … -f typst -o paper.typ && typst compile paper.typ`.

use std::error::Error;
use std::path::Path;

/// Compile a generated Typst document to PDF.
///
/// Returns an error in the v1 published build: bundled PDF compilation
/// requires the `citum-pdf` crate, which is not yet on crates.io. Users
/// can install Typst separately (https://typst.app/docs/) and run
/// `typst compile <source.typ>` on the output of
/// `citum render doc … -f typst`.
pub fn compile_document_to_pdf(
    _source: &str,
    _output: &Path,
    _keep_source: bool,
) -> Result<(), Box<dyn Error>> {
    Err(
        "Direct PDF output is not available in this build. Generate Typst with `-f typst -o <file>.typ` and compile with `typst compile <file>.typ`. See https://typst.app for installation.".into(),
    )
}
