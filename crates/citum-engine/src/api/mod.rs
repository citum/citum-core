/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Interactive document-level API for batch citation formatting.
//!
//! This module provides the Tier 1 `format_document` API, enabling whole-document
//! citation and bibliography rendering with proper context (note positions, ibid
//! detection, disambiguation).

mod document;
mod style_input;
mod types;

pub use document::{
    FormatDocumentError, FormatDocumentRequest, FormatDocumentResult, format_document,
    format_document_with_style,
};
pub use style_input::StyleInput;
pub use types::{
    AbbreviationMap, AnnotationFormat, AnnotationStyle, BibliographyEntry, CitationOccurrence,
    CitationOccurrenceItem, DocumentOptions, EntryMetadata, FormattedBibliography,
    FormattedCitation, OutputFormatKind, Warning, WarningLevel,
};
