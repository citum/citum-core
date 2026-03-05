/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citation input model for the CSLN processor.
//!
//! This module defines the structures for representing citations as input
//! to the processor. Citations reference entries in the bibliography and
//! can include locators, prefixes, suffixes, and mode information.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A list of citations to process.
pub type Citations = Vec<Citation>;

/// Citation mode for author-date styles.
///
/// Determines how the author name is rendered relative to the citation.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum CitationMode {
    /// Author inline in text: "Smith (2020) argues..."
    /// Also known as "narrative" or "in-text" citations.
    Integral,
    /// Author in parentheses: "(Smith, 2020)"
    /// The default mode for most citations.
    #[default]
    NonIntegral,
}

/// Position of a citation in the document flow.
///
/// Indicates where this citation appears relative to previous citations
/// of the same item(s). Used for note-based styles to detect ibid and
/// subsequent citations, and for author-date styles to apply position-specific
/// formatting rules (e.g., short forms after first citation).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum Position {
    /// First citation of an item.
    First,
    /// Subsequent citation of an item (non-consecutive).
    Subsequent,
    /// Same item cited immediately before, no locator on either.
    Ibid,
    /// Same item cited immediately before, with different locator.
    IbidWithLocator,
}

/// A citation containing one or more references.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Citation {
    /// The citation ID (optional, for tracking).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Note number for footnote/endnote styles.
    /// Assigned by the document processor, not the citation processor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_number: Option<u32>,
    /// Citation mode: integral (narrative) vs non-integral (parenthetical).
    /// Only relevant for author-date styles.
    #[serde(default, skip_serializing_if = "is_default_mode")]
    pub mode: CitationMode,
    /// Position of this citation in the document flow.
    /// Detected automatically by the processor or set explicitly by the caller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Suppress the author name across all items in this citation.
    /// Used when the author is already named in the prose: "Smith argues (2020)".
    /// Applies uniformly to all items — per-item suppression is not supported
    /// because mixed-visibility citations are typographically incoherent.
    #[serde(default, skip_serializing_if = "is_false")]
    pub suppress_author: bool,
    /// Prefix text before all citation items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Suffix text after all citation items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// The citation items (references being cited).
    pub items: Vec<CitationItem>,
}

impl Citation {
    /// Create a simple single-item citation.
    ///
    /// Convenience constructor for a citation with a single reference ID and default settings.
    pub fn simple(id: &str) -> Self {
        Self {
            items: vec![CitationItem {
                id: id.to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

/// Helper for skip_serializing_if on mode field.
fn is_default_mode(mode: &CitationMode) -> bool {
    *mode == CitationMode::NonIntegral
}

/// Helper for skip_serializing_if on bool fields that default to false.
fn is_false(b: &bool) -> bool {
    !b
}

/// Locator types for pinpoint citations.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LocatorType {
    /// Locator refers to a book within a larger work.
    Book,
    /// Locator refers to a chapter.
    Chapter,
    /// Locator refers to a clause.
    Clause,
    /// Locator refers to a column.
    Column,
    /// Locator refers to a corollary.
    Corollary,
    /// Locator refers to a definition.
    Definition,
    /// Locator refers to a division.
    Division,
    /// Locator refers to a figure.
    Figure,
    /// Locator refers to a folio.
    Folio,
    /// Locator refers to a numbered line.
    Line,
    /// Locator refers to a lemma.
    Lemma,
    /// Locator refers to a note.
    Note,
    /// Locator refers to a numbered unit.
    Number,
    /// Locator refers to an opus number.
    Opus,
    #[default]
    /// Locator refers to a page.
    Page,
    /// Locator refers to a paragraph.
    Paragraph,
    /// Locator refers to a sub-paragraph.
    Subparagraph,
    /// Locator refers to a sub-clause.
    Subclause,
    /// Locator refers to a sub-division.
    Subdivision,
    /// Locator refers to a sub-section.
    Subsection,
    /// Locator refers to a part or division.
    Part,
    /// Locator refers to a problem.
    Problem,
    /// Locator refers to a proposition.
    Proposition,
    /// Locator refers to a recital.
    Recital,
    /// Locator refers to a schedule.
    Schedule,
    /// Locator refers to a section.
    Section,
    /// Locator refers to a surah.
    Surah,
    /// Locator refers to a theorem.
    Theorem,
    /// Locator refers to an entry under a headword.
    SubVerbo,
    /// Locator refers to a supplement.
    Supplement,
    /// Locator refers to a verse.
    Verse,
    /// Locator refers to a volume.
    Volume,
    /// Locator refers to a periodical volume.
    VolumePeriodical,
    /// Locator refers to a monograph volume.
    VolumeBook,
    /// Locator refers to an issue.
    Issue,
    /// Locator refers to an algorithm.
    Algorithm,
}

/// A single segment of a compound locator.
///
/// Pairs a locator type with its value, e.g. `{ label: chapter, value: "3" }`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct LocatorSegment {
    /// The locator type for this segment.
    pub label: LocatorType,
    /// The locator value (e.g., "3", "42-45").
    pub value: String,
}

/// A resolved locator that abstracts over flat and compound forms.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedLocator<'a> {
    /// A single label + value pair (the traditional flat form).
    Flat {
        /// The locator type.
        label: LocatorType,
        /// The locator value.
        value: &'a str,
    },
    /// Multiple label + value segments (compound locator).
    Compound(&'a [LocatorSegment]),
}

/// A single citation item referencing a bibliography entry.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct CitationItem {
    /// The reference ID (citekey).
    pub id: String,
    /// Locator type (page, chapter, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<LocatorType>,
    /// Locator value (e.g., "42-45" for pages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
    /// Compound locator segments for multi-part references.
    ///
    /// When present, takes priority over `label`/`locator`.
    /// Example: `[{ label: chapter, value: "3" }, { label: section, value: "42" }]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locators: Option<Vec<LocatorSegment>>,
    /// Prefix text before this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Suffix text after this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

impl CitationItem {
    /// Resolve the locator, preferring compound `locators` over flat `label`/`locator`.
    pub fn resolved_locator(&self) -> Option<ResolvedLocator<'_>> {
        if let Some(segments) = &self.locators
            && !segments.is_empty()
        {
            return Some(ResolvedLocator::Compound(segments));
        }
        match (&self.label, &self.locator) {
            (Some(label), Some(value)) => Some(ResolvedLocator::Flat {
                label: label.clone(),
                value,
            }),
            (None, Some(value)) => Some(ResolvedLocator::Flat {
                label: LocatorType::default(),
                value,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_deserialization() {
        let json = r#"
        {
            "items": [
                {
                    "id": "kuhn1962"
                }
            ],
            "mode": "integral"
        }
        "#;
        let citation: Citation = serde_json::from_str(json).unwrap();
        assert_eq!(citation.items.len(), 1);
        assert_eq!(citation.items[0].id, "kuhn1962");
        assert_eq!(citation.mode, CitationMode::Integral);
    }

    #[test]
    fn test_citation_item_with_locator() {
        let json = r#"
        {
            "id": "kuhn1962",
            "label": "page",
            "locator": "42-45"
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "kuhn1962");
        assert_eq!(item.label, Some(LocatorType::Page));
        assert_eq!(item.locator, Some("42-45".to_string()));
    }

    #[test]
    fn test_compound_locator_serde_roundtrip() {
        let json = r#"
        {
            "id": "smith2020",
            "locators": [
                { "label": "chapter", "value": "3" },
                { "label": "section", "value": "42" }
            ]
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.locators.as_ref().unwrap().len(), 2);
        assert_eq!(
            item.locators.as_ref().unwrap()[0].label,
            LocatorType::Chapter
        );
        assert_eq!(item.locators.as_ref().unwrap()[0].value, "3");
        assert_eq!(
            item.locators.as_ref().unwrap()[1].label,
            LocatorType::Section
        );
        assert_eq!(item.locators.as_ref().unwrap()[1].value, "42");

        // Round-trip
        let serialized = serde_json::to_string(&item).unwrap();
        let deserialized: CitationItem = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.locators, item.locators);
    }

    #[test]
    fn test_resolved_locator_compound_priority() {
        let item = CitationItem {
            id: "test".to_string(),
            label: Some(LocatorType::Page),
            locator: Some("99".to_string()),
            locators: Some(vec![
                LocatorSegment {
                    label: LocatorType::Chapter,
                    value: "3".to_string(),
                },
                LocatorSegment {
                    label: LocatorType::Section,
                    value: "42".to_string(),
                },
            ]),
            ..Default::default()
        };
        let resolved = item.resolved_locator().unwrap();
        assert!(matches!(resolved, ResolvedLocator::Compound(_)));
    }

    #[test]
    fn test_resolved_locator_flat_fallback() {
        let item = CitationItem {
            id: "test".to_string(),
            label: Some(LocatorType::Page),
            locator: Some("42".to_string()),
            ..Default::default()
        };
        let resolved = item.resolved_locator().unwrap();
        assert!(matches!(resolved, ResolvedLocator::Flat { .. }));
    }

    #[test]
    fn test_resolved_locator_none() {
        let item = CitationItem {
            id: "test".to_string(),
            ..Default::default()
        };
        assert!(item.resolved_locator().is_none());
    }

    #[test]
    fn test_flat_locator_skips_serializing_locators() {
        let item = CitationItem {
            id: "test".to_string(),
            label: Some(LocatorType::Page),
            locator: Some("42".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_value(&item).unwrap();
        assert!(!json.as_object().unwrap().contains_key("locators"));
    }
}
