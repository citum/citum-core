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

pub(crate) use types::ManualNoteReference;
pub use types::{
    BibliographyBlock, CitationParser, CitationPlacement, CitationStructure, DocumentFormat,
    DocumentIntegralNameOverride, ParsedCitation, ParsedDocument,
};

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests;
