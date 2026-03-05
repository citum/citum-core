/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citation input model for the CSLN processor.
//!
//! This module defines the structures for representing citations as input
//! to the processor. Citations reference entries in the bibliography and
//! can include locators, prefixes, suffixes, and mode information.

use indexmap::IndexMap;
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

/// A locator value that supports both plain strings and explicit plurality.
///
/// Plain strings use heuristic plural detection (checking for `-`, `–`, `,`, `&`).
/// Use the explicit form to override when the heuristic fails (e.g., "figure A-3"
/// should be singular despite containing a hyphen).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum LocatorValue {
    /// Plain string value with heuristic plural detection.
    Text(String),
    /// Explicit value with manual plural override.
    Explicit {
        /// The locator value string.
        value: String,
        /// Whether this locator is plural.
        plural: bool,
    },
}

impl LocatorValue {
    /// Returns the raw value string.
    pub fn value_str(&self) -> &str {
        match self {
            LocatorValue::Text(s) => s,
            LocatorValue::Explicit { value, .. } => value,
        }
    }

    /// Returns whether this locator value is plural.
    ///
    /// For `Text`, uses the heuristic (contains `-`, `–`, `,`, or `&`).
    /// For `Explicit`, returns the specified `plural` field.
    pub fn is_plural(&self) -> bool {
        match self {
            LocatorValue::Text(s) => {
                s.contains('\u{2013}') || s.contains('-') || s.contains(',') || s.contains('&')
            }
            LocatorValue::Explicit { plural, .. } => *plural,
        }
    }
}

impl Default for LocatorValue {
    fn default() -> Self {
        LocatorValue::Text(String::new())
    }
}

impl From<String> for LocatorValue {
    fn from(s: String) -> Self {
        LocatorValue::Text(s)
    }
}

impl From<&str> for LocatorValue {
    fn from(s: &str) -> Self {
        LocatorValue::Text(s.to_string())
    }
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
    pub value: LocatorValue,
}

/// Input form for compound locators, supporting both verbose and compact syntax.
///
/// The verbose form uses an explicit list of segments:
/// ```yaml
/// locators:
///   - label: page
///     value: "23"
///   - label: line
///     value: "13"
/// ```
///
/// The compact form uses a map (order preserved via `IndexMap`):
/// ```yaml
/// locators:
///   page: "23"
///   line: "13"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum LocatorsInput {
    /// Verbose list of labeled segments.
    List(Vec<LocatorSegment>),
    /// Compact map form (locator type → value).
    Map(IndexMap<LocatorType, LocatorValue>),
}

impl LocatorsInput {
    /// Normalize to a list of segments regardless of input form.
    pub fn segments(&self) -> Vec<LocatorSegment> {
        match self {
            LocatorsInput::List(list) => list.clone(),
            LocatorsInput::Map(map) => map
                .iter()
                .map(|(label, value)| LocatorSegment {
                    label: label.clone(),
                    value: value.clone(),
                })
                .collect(),
        }
    }

    /// Returns true if this contains no segments.
    pub fn is_empty(&self) -> bool {
        match self {
            LocatorsInput::List(list) => list.is_empty(),
            LocatorsInput::Map(map) => map.is_empty(),
        }
    }
}

#[cfg(feature = "schema")]
impl JsonSchema for LocatorsInput {
    fn schema_name() -> String {
        "LocatorsInput".to_string()
    }

    fn json_schema(
        schema_generator: &mut schemars::r#gen::SchemaGenerator,
    ) -> schemars::schema::Schema {
        use schemars::schema::SchemaObject;

        let list_schema = schema_generator.subschema_for::<Vec<LocatorSegment>>();
        let map_schema =
            schema_generator.subschema_for::<std::collections::BTreeMap<String, LocatorValue>>();

        let mut schema = SchemaObject::default();
        schema.subschemas().one_of = Some(vec![list_schema, map_schema]);
        schema.into()
    }
}

/// A resolved locator that abstracts over flat and compound forms.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedLocator {
    /// A single label + value pair (the traditional flat form).
    Flat {
        /// The locator type.
        label: LocatorType,
        /// The locator value.
        value: String,
    },
    /// Multiple label + value segments (compound locator).
    Compound(Vec<LocatorSegment>),
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
    /// Supports both verbose list form and compact map form.
    /// Example list: `[{ label: chapter, value: "3" }, { label: section, value: "42" }]`
    /// Example map: `{ page: "23", line: "13" }`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locators: Option<LocatorsInput>,
    /// Prefix text before this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Suffix text after this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

impl CitationItem {
    /// Resolve the locator, preferring compound `locators` over flat `label`/`locator`.
    pub fn resolved_locator(&self) -> Option<ResolvedLocator> {
        if let Some(input) = &self.locators
            && !input.is_empty()
        {
            return Some(ResolvedLocator::Compound(input.segments()));
        }
        match (&self.label, &self.locator) {
            (Some(label), Some(value)) => Some(ResolvedLocator::Flat {
                label: label.clone(),
                value: value.clone(),
            }),
            (None, Some(value)) => Some(ResolvedLocator::Flat {
                label: LocatorType::default(),
                value: value.clone(),
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
        let segs = item.locators.as_ref().unwrap().segments();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].label, LocatorType::Chapter);
        assert_eq!(segs[0].value.value_str(), "3");
        assert_eq!(segs[1].label, LocatorType::Section);
        assert_eq!(segs[1].value.value_str(), "42");

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
            locators: Some(LocatorsInput::List(vec![
                LocatorSegment {
                    label: LocatorType::Chapter,
                    value: LocatorValue::from("3"),
                },
                LocatorSegment {
                    label: LocatorType::Section,
                    value: LocatorValue::from("42"),
                },
            ])),
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

    #[test]
    fn test_compact_map_form_deserialization() {
        let json = r#"
        {
            "id": "smith2020",
            "locators": {
                "page": "23",
                "line": "13"
            }
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        let segs = item.locators.as_ref().unwrap().segments();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].label, LocatorType::Page);
        assert_eq!(segs[0].value.value_str(), "23");
        assert_eq!(segs[1].label, LocatorType::Line);
        assert_eq!(segs[1].value.value_str(), "13");
    }

    #[test]
    fn test_locator_value_explicit_plural_override() {
        let json = r#"
        {
            "id": "test",
            "locators": [
                {
                    "label": "figure",
                    "value": {
                        "value": "A-3",
                        "plural": false
                    }
                }
            ]
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        let segs = item.locators.as_ref().unwrap().segments();
        assert_eq!(segs[0].value.value_str(), "A-3");
        assert!(!segs[0].value.is_plural());
    }

    #[test]
    fn test_locator_value_heuristic_plural() {
        let lv_range = LocatorValue::from("42-45");
        assert!(lv_range.is_plural());

        let lv_single = LocatorValue::from("42");
        assert!(!lv_single.is_plural());

        let lv_en_dash = LocatorValue::from("42–45");
        assert!(lv_en_dash.is_plural());

        let lv_comma = LocatorValue::from("1, 3, 5");
        assert!(lv_comma.is_plural());

        let lv_ampersand = LocatorValue::from("A & B");
        assert!(lv_ampersand.is_plural());
    }
}
