/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Typst PDF compilation for Citum.

use std::error::Error;
use std::fs;
use std::path::Path;

use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::ecow::EcoVec;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World, compile};
use typst_kit::fonts::{FontStore, embedded, system};

struct TypstWorld {
    library: LazyHash<Library>,
    main: FileId,
    source: Source,
    fonts: FontStore,
}

impl TypstWorld {
    fn new(source: &str) -> Result<Self, Box<dyn Error>> {
        let mut fonts = FontStore::new();
        fonts.extend(system());
        fonts.extend(embedded());
        let main = FileId::unique(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("/main.typ")?,
        ));
        let source = Source::new(main, source.to_string());

        Ok(Self {
            library: LazyHash::new(Library::default()),
            main,
            source,
            fonts,
        })
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            Ok(Bytes::new(self.source.text().as_bytes().to_vec()))
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

fn format_diagnostics(
    warnings: EcoVec<SourceDiagnostic>,
    errors: Option<EcoVec<SourceDiagnostic>>,
) -> String {
    let mut lines = Vec::new();

    for warning in warnings {
        lines.push(format!("warning: {}", warning.message));
        for hint in warning.hints {
            lines.push(format!("hint: {}", hint.v));
        }
    }

    if let Some(errors) = errors {
        for error in errors {
            lines.push(format!("error: {}", error.message));
            for hint in error.hints {
                lines.push(format!("hint: {}", hint.v));
            }
        }
    }

    lines.join("\n")
}

fn typst_source_path(output: &Path) -> std::path::PathBuf {
    output.with_extension("typ")
}

/// Compile a generated Typst document to PDF.
///
/// # Errors
///
/// Returns an error if the output path does not end with `.pdf`, if directory
/// creation fails, if Typst compilation or PDF export fails, or if file writing fails.
pub fn compile_document_to_pdf(
    source: &str,
    output: &Path,
    keep_source: bool,
) -> Result<(), Box<dyn Error>> {
    if output.extension().and_then(|ext| ext.to_str()) != Some("pdf") {
        return Err("Typst PDF output path must end with `.pdf`.".into());
    }

    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    if keep_source {
        fs::write(typst_source_path(output), source)?;
    }

    let world = TypstWorld::new(source)?;
    let warned = compile(&world);
    let warnings = warned.warnings.clone();
    let document = match warned.output {
        Ok(document) => document,
        Err(errors) => {
            let diagnostics = format_diagnostics(warnings, Some(errors));
            return Err(format!("Typst compilation failed.\n{diagnostics}").into());
        }
    };

    let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).map_err(|errors| {
        let rendered = errors
            .into_iter()
            .map(|error| {
                let mut lines = vec![format!("error: {}", error.message)];
                lines.extend(
                    error
                        .hints
                        .into_iter()
                        .map(|hint| format!("hint: {}", hint.v)),
                );
                lines.join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("Typst PDF export failed.\n{rendered}")
    })?;

    fs::write(output, pdf)?;
    Ok(())
}
