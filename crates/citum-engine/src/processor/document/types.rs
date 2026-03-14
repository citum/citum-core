/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Shared document-processing types and parser contracts.

use crate::Citation;
use citum_schema::locale::Locale;
use citum_schema::options::IntegralNameConfig;
use serde::Deserialize;
use std::collections::HashSet;

/// Describes where a parsed citation appears in the source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationPlacement {
    /// The citation marker appears in prose and should become a generated note
    /// reference for note styles.
    InlineProse,
    /// The citation marker appears inside a manually authored footnote
    /// definition and should render in place.
    ManualFootnote {
        /// The source footnote label that identifies the manual note block.
        label: String,
    },
}

/// Structural citation scope metadata derived from the source document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CitationStructure {
    /// The resolved chapter-level scope key for this citation location.
    pub chapter_scope: String,
    /// The resolved section-level scope key for this citation location.
    pub section_scope: String,
}

/// Document-level integral-name override parsed from frontmatter.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DocumentIntegralNameOverride {
    /// Whether the integral-name policy is enabled for this document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// The name-memory rule to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<citum_schema::options::IntegralNameRule>,
    /// Where name-memory resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<citum_schema::options::IntegralNameScope>,
    /// Which document contexts participate in the policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<citum_schema::options::IntegralNameContexts>,
    /// The contributor form used after the first mention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_form: Option<citum_schema::options::IntegralNameForm>,
}

impl DocumentIntegralNameOverride {
    pub(super) fn apply_to(&self, base: Option<&IntegralNameConfig>) -> Option<IntegralNameConfig> {
        if self.enabled == Some(false) {
            return None;
        }

        let mut result = base.cloned().unwrap_or_default();
        if self.rule.is_some() {
            result.rule = self.rule;
        }
        if self.scope.is_some() {
            result.scope = self.scope;
        }
        if self.contexts.is_some() {
            result.contexts = self.contexts;
        }
        if self.subsequent_form.is_some() {
            result.subsequent_form = self.subsequent_form;
        }
        Some(result)
    }
}

/// A citation marker parsed from a document.
#[derive(Debug, Clone)]
pub struct ParsedCitation {
    /// Byte offset where the citation marker starts in the source document.
    pub start: usize,
    /// Byte offset immediately after the citation marker in the source document.
    pub end: usize,
    /// The parsed citation payload and its items.
    pub citation: Citation,
    /// Where the citation was found in the source document.
    pub placement: CitationPlacement,
    /// Structural scope metadata for this citation location.
    pub structure: CitationStructure,
}

#[derive(Debug, Clone)]
pub(crate) struct ManualNoteReference {
    pub label: String,
    pub start: usize,
}

/// Structured output from a document parser.
#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    /// Citation markers discovered in source order.
    pub citations: Vec<ParsedCitation>,
    /// Manual footnote labels in the order they appear in the document.
    pub manual_note_order: Vec<String>,
    pub(crate) manual_note_references: Vec<ManualNoteReference>,
    pub(crate) manual_note_labels: HashSet<String>,
    /// Bibliography blocks found in the document.
    pub bibliography_blocks: Vec<super::djot::BibliographyBlock>,
    /// Bibliography groups from YAML frontmatter.
    pub frontmatter_groups: Option<Vec<citum_schema::grouping::BibliographyGroup>>,
    /// Integral-name override from YAML frontmatter.
    pub frontmatter_integral_names: Option<DocumentIntegralNameOverride>,
    /// Byte offset where the document body starts (past any frontmatter).
    pub body_start: usize,
}

/// A trait for document parsers that can identify citations.
pub trait CitationParser {
    /// Parse the document into citation placements and note metadata.
    fn parse_document(&self, content: &str, locale: &Locale) -> ParsedDocument;

    /// Finalize rendered document markup as HTML.
    ///
    /// The default implementation treats the rendered markup as Djot-compatible
    /// content and converts it with the existing Djot HTML renderer.
    fn finalize_html_output(&self, rendered: &str) -> String {
        super::djot::djot_to_html(rendered)
    }

    /// Find and extract citations from a document string.
    ///
    /// Returns a list of `(start_index, end_index, citation_model)` tuples.
    fn parse_citations(&self, content: &str, locale: &Locale) -> Vec<(usize, usize, Citation)> {
        self.parse_document(content, locale)
            .citations
            .into_iter()
            .map(|parsed| (parsed.start, parsed.end, parsed.citation))
            .collect()
    }
}

/// Document output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    /// Plain text (raw markup).
    Plain,
    /// Djot markup.
    Djot,
    /// HTML output.
    Html,
    /// LaTeX output.
    Latex,
    /// Typst output.
    Typst,
}
