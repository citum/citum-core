/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Shared document-processing types and parser contracts.

use crate::Citation;
use citum_schema::grouping::BibliographyGroup;
use citum_schema::locale::Locale;
use citum_schema::options::{IntegralNameMemoryConfig, OrgAbbreviationMemoryConfig};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DocumentIntegralNameOverride {
    /// Whether the integral-name policy is enabled for this document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Where name-memory resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<citum_schema::options::IntegralNameScope>,
    /// Which document contexts participate in the policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<citum_schema::options::IntegralNameContexts>,
    /// The contributor form used after the first mention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_form: Option<citum_schema::options::SubsequentNameForm>,
}

impl DocumentIntegralNameOverride {
    /// Apply this frontmatter override to a base integral-name configuration.
    ///
    /// Returns `None` if the override explicitly disables the policy. Otherwise,
    /// returns a new configuration where non-null fields from the override
    /// supersede fields from the base.
    pub(super) fn apply_to(
        &self,
        base: Option<&IntegralNameMemoryConfig>,
    ) -> Option<IntegralNameMemoryConfig> {
        if self.enabled == Some(false) {
            return None;
        }

        let mut result = base.cloned().unwrap_or_default();
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

/// Document-level org-abbreviation override parsed from frontmatter.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DocumentOrgAbbreviationOverride {
    /// Whether org-abbreviation is enabled for this document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Where the first-mention memory resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<citum_schema::options::IntegralNameScope>,
    /// Which document contexts participate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<citum_schema::options::IntegralNameContexts>,
    /// How to display a short name on the first integral mention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name_display: Option<citum_schema::options::ShortNameDisplay>,
}

impl DocumentOrgAbbreviationOverride {
    /// Apply this frontmatter override to a base org-abbreviation configuration.
    #[allow(dead_code, reason = "Infrastructure for org-abbreviation support")]
    pub(super) fn apply_to(
        &self,
        base: Option<&OrgAbbreviationMemoryConfig>,
    ) -> Option<OrgAbbreviationMemoryConfig> {
        if self.enabled == Some(false) {
            return None;
        }
        let mut result = base.cloned().unwrap_or_default();
        if self.scope.is_some() {
            result.scope = self.scope;
        }
        if self.contexts.is_some() {
            result.contexts = self.contexts;
        }
        if self.short_name_display.is_some() {
            result.short_name_display = self.short_name_display;
        }
        Some(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use super::{DocumentIntegralNameOverride, DocumentOrgAbbreviationOverride};
    use citum_schema::options::{
        IntegralNameMemoryConfig, OrgAbbreviationMemoryConfig, ShortNameDisplay,
    };

    #[test]
    fn test_document_org_abbreviation_override_deserializes_short_name_display() {
        assert_eq!(
            serde_yaml::from_str::<DocumentOrgAbbreviationOverride>(
                "short-name-display: short-then-bracketed"
            )
            .ok()
            .map(|config| config.short_name_display),
            Some(Some(ShortNameDisplay::ShortThenBracketed))
        );
    }

    #[test]
    fn test_document_org_abbreviation_override_applies_short_name_display() {
        let base = OrgAbbreviationMemoryConfig {
            short_name_display: Some(ShortNameDisplay::FullThenParenthetical),
            ..Default::default()
        };
        let override_config = DocumentOrgAbbreviationOverride {
            short_name_display: Some(ShortNameDisplay::ShortThenBracketed),
            ..Default::default()
        };

        assert_eq!(
            override_config
                .apply_to(Some(&base))
                .map(|config| config.short_name_display),
            Some(Some(ShortNameDisplay::ShortThenBracketed))
        );
    }

    #[test]
    fn test_document_integral_name_override_enabled_false_disables_block() {
        let base = IntegralNameMemoryConfig::default();
        let override_config = DocumentIntegralNameOverride {
            enabled: Some(false),
            ..Default::default()
        };

        assert!(override_config.apply_to(Some(&base)).is_none());
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

/// A bibliography block found in a document by a format-specific adapter.
///
/// Stores the byte range of the block in the source so the pipeline can
/// replace it with rendered output, plus the group descriptor that controls
/// which entries are selected and how they are headed.
#[derive(Debug, Clone)]
pub struct BibliographyBlock {
    /// Byte offset of the block's opening marker in source.
    pub start: usize,
    /// Byte offset past the block's closing marker in source.
    pub end: usize,
    /// The bibliography group for this block.
    pub group: BibliographyGroup,
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
    /// Bibliography blocks found in the document, in source order.
    pub bibliography_blocks: Vec<BibliographyBlock>,
    /// Bibliography groups from YAML frontmatter.
    pub frontmatter_groups: Option<Vec<citum_schema::grouping::BibliographyGroup>>,
    /// Integral-name-memory override from YAML frontmatter (legacy top-level field).
    pub frontmatter_integral_name_memory: Option<DocumentIntegralNameOverride>,
    /// Org-abbreviation-memory override from YAML frontmatter.
    pub frontmatter_org_abbreviation_memory: Option<DocumentOrgAbbreviationOverride>,
    /// Byte offset where the document body starts (past any frontmatter).
    pub body_start: usize,
}

/// A trait for document parsers that can identify citations.
///
/// Each implementation is a format-specific adapter (Djot, Markdown, etc.).
/// Adapters are responsible for source-syntax citation parsing, note discovery,
/// frontmatter extraction, bibliography block detection, and HTML finalization.
/// The shared pipeline consumes the resulting [`ParsedDocument`] without
/// inspecting format-specific internals.
pub trait CitationParser {
    /// Parse the document into citation placements and note metadata.
    fn parse_document(&self, content: &str, locale: &Locale) -> ParsedDocument;

    /// Finalize rendered document markup as HTML.
    ///
    /// Called after citation strings have been spliced back into the source
    /// markup and the result must be converted to HTML. The default
    /// implementation is a pass-through: it returns the markup unchanged.
    /// Format-specific adapters that require a markup-to-HTML conversion step
    /// (e.g. Djot via `jotdown`) must override this method.
    fn finalize_html_output(&self, rendered: &str) -> String {
        rendered.to_owned()
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
///
/// This governs the render target for citation strings and the generated
/// bibliography. It is independent of the document *input* format (Djot,
/// Markdown) and of in-field reference markup (Djot inline via
/// `render_djot_inline`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    /// Plain text (raw markup).
    Plain,
    /// Djot markup.
    Djot,
    /// Markdown markup.
    Markdown,
    /// HTML output.
    Html,
    /// LaTeX output.
    Latex,
    /// Typst output.
    Typst,
}
