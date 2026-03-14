/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Document-level citation processing.

pub mod djot;
pub mod markdown;

mod integral_names;
mod note_support;
mod notes;
mod output;
mod pipeline;
mod types;

pub use djot::BibliographyBlock;
pub(crate) use types::ManualNoteReference;
pub use types::{
    CitationParser, CitationPlacement, CitationStructure, DocumentFormat,
    DocumentIntegralNameOverride, ParsedCitation, ParsedDocument,
};

#[cfg(test)]
mod tests;
