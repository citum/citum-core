/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Request and response types for the interactive document API.

use citum_schema::data::citation::{CitationLocator, IntegralNameState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use citum_schema_data::AbbreviationMap;

/// Severity level for a structured diagnostic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WarningLevel {
    /// Non-fatal diagnostic message.
    Warning,
    /// Error that may affect output quality or correctness.
    Error,
}

/// A structured diagnostic warning returned alongside formatted output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    /// The severity level.
    pub level: WarningLevel,
    /// A machine-readable error code.
    pub code: String,
    /// The citation ID this warning pertains to, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_id: Option<String>,
    /// The reference ID this warning pertains to, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<String>,
    /// A human-readable diagnostic message.
    pub message: String,
}

/// Supported output format kinds.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormatKind {
    /// Plain text (default).
    #[default]
    Plain,
    /// HTML markup.
    Html,
    /// Djot inline markup.
    Djot,
    /// LaTeX markup.
    Latex,
    /// Typst markup.
    Typst,
}

/// Controls how annotation text is rendered in an annotated bibliography.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationStyle {
    /// Markup format for annotation text. Default: Djot.
    #[serde(default)]
    pub format: AnnotationFormat,
}

impl Default for AnnotationStyle {
    fn default() -> Self {
        Self {
            format: AnnotationFormat::Djot,
        }
    }
}

/// Markup format for annotation text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AnnotationFormat {
    /// Parse annotation as djot inline markup (default).
    #[default]
    Djot,
    /// Treat annotation as plain text with no markup interpretation.
    Plain,
    /// Parse annotation as org-mode markup.
    Org,
}

/// A single item within a citation occurrence.
///
/// Maps to `CitationItem` from `citum-schema-data`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationOccurrenceItem {
    /// The reference ID (citekey) being cited.
    pub id: String,
    /// Optional locator (pinpoint citation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<CitationLocator>,
    /// Optional prefix text before this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Optional suffix text after this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Explicit integral (narrative) name state override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integral_name_state: Option<IntegralNameState>,
}

impl From<CitationOccurrenceItem> for citum_schema::data::citation::CitationItem {
    fn from(item: CitationOccurrenceItem) -> Self {
        Self {
            id: item.id,
            locator: item.locator,
            prefix: item.prefix,
            suffix: item.suffix,
            integral_name_state: item.integral_name_state,
        }
    }
}

/// A citation occurrence in the document.
///
/// Maps to `Citation` from `citum-schema-data`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationOccurrence {
    /// Stable identifier for this citation in the document.
    pub id: String,
    /// The citation items (references cited together).
    pub items: Vec<CitationOccurrenceItem>,
    /// The citation mode (integral vs non-integral).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<citum_schema::data::citation::CitationMode>,
    /// Footnote/endnote number for note-based styles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_number: Option<u32>,
    /// Whether to suppress the author across all items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_author: Option<bool>,
    /// Whether this is a compound-numeric group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouped: Option<bool>,
    /// Optional prefix text before all formatted items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Optional suffix text after all formatted items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

impl From<CitationOccurrence> for citum_schema::data::citation::Citation {
    fn from(occ: CitationOccurrence) -> Self {
        Self {
            id: Some(occ.id),
            items: occ.items.into_iter().map(|item| item.into()).collect(),
            mode: occ.mode.unwrap_or_default(),
            note_number: occ.note_number,
            suppress_author: occ.suppress_author.unwrap_or(false),
            grouped: occ.grouped.unwrap_or(false),
            prefix: occ.prefix,
            suffix: occ.suffix,
            position: None, // Assigned by processor
        }
    }
}

/// Document-level configuration for rendering.
///
/// Controls rendering behavior that belongs to the document rather than the style.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
pub struct DocumentOptions {
    /// Override or replace style-defined bibliography grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography_groups: Option<Vec<citum_schema::grouping::BibliographyGroup>>,
    /// Automatic bibliography partitioning by script or language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_partitioning: Option<citum_schema::options::BibliographySortPartitioning>,
    /// Document-level narrative citation rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integral_name_memory: Option<crate::processor::document::DocumentIntegralNameOverride>,
    /// Whether to output semantic markup (HTML spans, Djot attributes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_semantics: Option<bool>,
    /// Whether to annotate semantic wrappers with source template indices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inject_ast_indices: Option<bool>,
    /// Reference ID to annotation text mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
    /// Format for annotation text (djot, plain, org).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_format: Option<AnnotationFormat>,
    /// Map from full rendered strings to their abbreviations.
    #[serde(alias = "abbreviation-map", skip_serializing_if = "Option::is_none")]
    pub abbreviation_map: Option<AbbreviationMap>,
}

/// A single formatted citation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedCitation {
    /// The citation identifier from the input.
    pub id: String,
    /// The formatted citation text.
    pub text: String,
    /// Referenced entry IDs from this citation.
    pub ref_ids: Vec<String>,
}

/// Metadata extracted from a bibliography entry for interactivity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntryMetadata {
    /// Rendered author(s) string.
    pub author: String,
    /// Rendered year string.
    pub year: String,
    /// Rendered title string.
    pub title: String,
}

/// A single bibliography entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibliographyEntry {
    /// The reference ID.
    pub id: String,
    /// The formatted entry text.
    pub text: String,
    /// Extracted metadata for interactivity.
    pub metadata: EntryMetadata,
}

/// A formatted bibliography.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedBibliography {
    /// The output format used.
    pub format: OutputFormatKind,
    /// The complete formatted bibliography content.
    pub content: String,
    /// Individual entries with metadata.
    pub entries: Vec<BibliographyEntry>,
}
