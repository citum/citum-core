/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citation input model for the Citum processor.
//!
//! This module defines the structures for representing citations as input
//! to the processor. Citations reference entries in the bibliography and
//! can include locators, prefixes, suffixes, and mode information.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "bindings")]
use specta::Type;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};

/// A list of citations to process.
pub type Citations = Vec<Citation>;

/// Citation mode for author-date styles.
///
/// Determines how the author name is rendered relative to the citation.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
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

/// Explicit integral citation name-memory state for one citation item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub enum IntegralNameState {
    /// Render this item as the first integral mention in scope.
    First,
    /// Render this item as a subsequent integral mention in scope.
    Subsequent,
}

/// Position of a citation in the document flow.
///
/// Indicates where this citation appears relative to previous citations
/// of the same item(s). Used for note-based styles to detect ibid and
/// subsequent citations, and for author-date styles to apply position-specific
/// formatting rules (e.g., short forms after first citation).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
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
#[cfg_attr(feature = "bindings", derive(Type))]
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
    /// If true, the entire citation is a single dynamic compound set.
    ///
    /// The first item acts as the head and subsequent items are merged as tails
    /// in the bibliography. Ignored for non-numeric (compound-numeric) styles.
    /// Item order is preserved and sorting is suppressed when this flag is set.
    #[serde(default, skip_serializing_if = "is_false")]
    pub grouped: bool,
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
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "bindings", derive(Type))]
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
    /// Locator refers to a custom pinpoint label.
    Custom(String),
}

impl LocatorType {
    /// Return the canonical kebab-case key for this locator label.
    #[must_use]
    pub fn as_key(&self) -> Cow<'_, str> {
        match self {
            Self::Book => Cow::Borrowed("book"),
            Self::Chapter => Cow::Borrowed("chapter"),
            Self::Clause => Cow::Borrowed("clause"),
            Self::Column => Cow::Borrowed("column"),
            Self::Corollary => Cow::Borrowed("corollary"),
            Self::Definition => Cow::Borrowed("definition"),
            Self::Division => Cow::Borrowed("division"),
            Self::Figure => Cow::Borrowed("figure"),
            Self::Folio => Cow::Borrowed("folio"),
            Self::Line => Cow::Borrowed("line"),
            Self::Lemma => Cow::Borrowed("lemma"),
            Self::Note => Cow::Borrowed("note"),
            Self::Number => Cow::Borrowed("number"),
            Self::Opus => Cow::Borrowed("opus"),
            Self::Page => Cow::Borrowed("page"),
            Self::Paragraph => Cow::Borrowed("paragraph"),
            Self::Subparagraph => Cow::Borrowed("subparagraph"),
            Self::Subclause => Cow::Borrowed("subclause"),
            Self::Subdivision => Cow::Borrowed("subdivision"),
            Self::Subsection => Cow::Borrowed("subsection"),
            Self::Part => Cow::Borrowed("part"),
            Self::Problem => Cow::Borrowed("problem"),
            Self::Proposition => Cow::Borrowed("proposition"),
            Self::Recital => Cow::Borrowed("recital"),
            Self::Schedule => Cow::Borrowed("schedule"),
            Self::Section => Cow::Borrowed("section"),
            Self::Surah => Cow::Borrowed("surah"),
            Self::Theorem => Cow::Borrowed("theorem"),
            Self::SubVerbo => Cow::Borrowed("sub-verbo"),
            Self::Supplement => Cow::Borrowed("supplement"),
            Self::Verse => Cow::Borrowed("verse"),
            Self::Volume => Cow::Borrowed("volume"),
            Self::VolumePeriodical => Cow::Borrowed("volume-periodical"),
            Self::VolumeBook => Cow::Borrowed("volume-book"),
            Self::Issue => Cow::Borrowed("issue"),
            Self::Algorithm => Cow::Borrowed("algorithm"),
            Self::Custom(value) => normalize_kind_key(value)
                .map(Cow::Owned)
                .unwrap_or_else(|| Cow::Borrowed(value.as_str())),
        }
    }

    /// Parse a locator label from a known keyword or custom identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is empty or normalizes to an empty key.
    pub fn from_key(value: &str) -> Result<Self, String> {
        let canonical = normalize_kind_key(value)
            .ok_or_else(|| "locator label must not be empty".to_string())?;
        Ok(match canonical.as_str() {
            "algorithm" => Self::Algorithm,
            "book" => Self::Book,
            "chapter" => Self::Chapter,
            "clause" => Self::Clause,
            "column" => Self::Column,
            "corollary" => Self::Corollary,
            "definition" => Self::Definition,
            "division" => Self::Division,
            "figure" => Self::Figure,
            "folio" => Self::Folio,
            "line" => Self::Line,
            "lemma" => Self::Lemma,
            "note" => Self::Note,
            "number" => Self::Number,
            "opus" => Self::Opus,
            "page" => Self::Page,
            "paragraph" => Self::Paragraph,
            "part" => Self::Part,
            "problem" => Self::Problem,
            "proposition" => Self::Proposition,
            "recital" => Self::Recital,
            "schedule" => Self::Schedule,
            "section" => Self::Section,
            "subclause" => Self::Subclause,
            "subdivision" => Self::Subdivision,
            "subparagraph" => Self::Subparagraph,
            "subsection" => Self::Subsection,
            "sub-verbo" => Self::SubVerbo,
            "supplement" => Self::Supplement,
            "surah" => Self::Surah,
            "theorem" => Self::Theorem,
            "verse" => Self::Verse,
            "volume" => Self::Volume,
            "volume-book" => Self::VolumeBook,
            "volume-periodical" => Self::VolumePeriodical,
            "issue" => Self::Issue,
            _ => Self::Custom(canonical),
        })
    }
}

impl PartialEq for LocatorType {
    fn eq(&self, other: &Self) -> bool {
        self.as_key().as_ref() == other.as_key().as_ref()
    }
}

impl Eq for LocatorType {}

impl Hash for LocatorType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_key().as_ref().hash(state);
    }
}

impl Serialize for LocatorType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_key().as_ref())
    }
}

impl<'de> Deserialize<'de> for LocatorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_key(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl JsonSchema for LocatorType {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "LocatorType".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "description": "Known locator label keyword or custom kebab-case identifier."
        })
    }
}

/// A locator value that supports both plain strings and explicit plurality.
///
/// Plain strings use heuristic plural detection (checking for `-`, `–`, `,`, `&`).
/// Use the explicit form to override when the heuristic fails (e.g., "figure A-3"
/// should be singular despite containing a hyphen).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
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
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct LocatorSegment {
    /// The locator type for this segment.
    pub label: LocatorType,
    /// The locator value (e.g., "3", "42-45").
    pub value: LocatorValue,
}

impl LocatorSegment {
    /// Create a locator segment from a canonical label and value.
    pub fn new(label: LocatorType, value: impl Into<LocatorValue>) -> Self {
        Self {
            label,
            value: value.into(),
        }
    }
}

/// A canonical citation locator.
///
/// Simple locators use the single-segment form, while compound locators use
/// the explicit `segments` wrapper.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum CitationLocator {
    /// A single labeled locator.
    Single(LocatorSegment),
    /// Multiple ordered locator segments.
    Compound {
        /// Ordered locator segments.
        segments: Vec<LocatorSegment>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum CitationLocatorRepr {
    Single(LocatorSegment),
    Compound { segments: Vec<LocatorSegment> },
}

impl<'de> Deserialize<'de> for CitationLocator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        match CitationLocatorRepr::deserialize(deserializer)? {
            CitationLocatorRepr::Single(segment) => Ok(Self::Single(segment)),
            CitationLocatorRepr::Compound { segments } => {
                Self::compound(segments).map_err(D::Error::custom)
            }
        }
    }
}

impl CitationLocator {
    /// Create a single-segment locator.
    pub fn single(label: LocatorType, value: impl Into<LocatorValue>) -> Self {
        Self::Single(LocatorSegment::new(label, value))
    }

    /// Create a compound locator with two or more segments.
    ///
    /// # Errors
    ///
    /// Returns an error when fewer than two locator segments are supplied.
    pub fn compound(segments: Vec<LocatorSegment>) -> Result<Self, &'static str> {
        if segments.len() < 2 {
            return Err("compound locators must contain at least two segments");
        }
        Ok(Self::Compound { segments })
    }

    /// Returns the ordered locator segments as a slice.
    pub fn segments(&self) -> &[LocatorSegment] {
        match self {
            Self::Single(segment) => std::slice::from_ref(segment),
            Self::Compound { segments } => segments.as_slice(),
        }
    }

    /// Returns true if this locator contains multiple segments.
    pub fn is_compound(&self) -> bool {
        matches!(self, Self::Compound { .. })
    }

    /// Returns a stable string form used for locator comparison.
    pub fn canonical_string(&self) -> String {
        self.segments()
            .iter()
            .map(|segment| format!("{}:{}", segment.label.as_key(), segment.value.value_str()))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(feature = "schema")]
impl JsonSchema for CitationLocator {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CitationLocator".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let single_schema = generator.subschema_for::<LocatorSegment>();
        let compound_schema = schemars::json_schema!({
            "type": "object",
            "properties": {
                "segments": generator.subschema_for::<Vec<LocatorSegment>>()
            },
            "required": ["segments"]
        });
        schemars::json_schema!({
            "oneOf": [single_schema, compound_schema]
        })
    }
}

fn normalize_kind_key(value: &str) -> Option<String> {
    let mut normalized = String::new();
    let mut pending_dash = false;

    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !normalized.is_empty() {
                normalized.push('-');
            }
            normalized.push(ch.to_ascii_lowercase());
            pending_dash = false;
        } else if !normalized.is_empty() {
            pending_dash = true;
        }
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// A single citation item referencing a bibliography entry.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct CitationItem {
    /// The reference ID (citekey).
    pub id: String,
    /// Canonical locator value for pinpoint citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<CitationLocator>,
    /// Prefix text before this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Suffix text after this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Explicit integral name-memory state override for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integral_name_state: Option<IntegralNameState>,
}

impl CitationItem {
    /// Returns the canonical locator segments when present.
    pub fn locator_segments(&self) -> Option<&[LocatorSegment]> {
        self.locator.as_ref().map(CitationLocator::segments)
    }
}

/// Normalize a textual locator string into the canonical locator model.
///
/// # Panics
///
/// This function does not panic under normal use; the internal `unwrap` is
/// guarded by the preceding segment-count match.
pub fn normalize_locator_text(
    locator: &str,
    aliases: &[(String, LocatorType)],
) -> Option<CitationLocator> {
    let locator = locator.trim();
    if locator.is_empty() {
        return None;
    }

    let raw_segments = split_locator_segments(locator, aliases);
    let segments: Vec<LocatorSegment> = raw_segments
        .into_iter()
        .filter_map(|segment| parse_locator_segment(segment, aliases))
        .collect();

    match segments.len() {
        0 => None,
        1 => Some(CitationLocator::Single(
            segments.into_iter().next().unwrap(),
        )),
        _ => CitationLocator::compound(segments).ok(),
    }
}

fn split_locator_segments<'a>(locator: &'a str, aliases: &[(String, LocatorType)]) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = 0;

    for (idx, ch) in locator.char_indices() {
        if ch != ',' {
            continue;
        }

        let candidate = locator[idx + ch.len_utf8()..].trim_start();
        if begins_with_locator_label(candidate, aliases) {
            parts.push(locator[start..idx].trim());
            start = idx + ch.len_utf8();
        }
    }

    parts.push(locator[start..].trim());
    parts
}

fn parse_locator_segment(
    segment: &str,
    aliases: &[(String, LocatorType)],
) -> Option<LocatorSegment> {
    let segment = segment.trim();
    if segment.is_empty() {
        return None;
    }

    if let Some((label, rest)) = strip_locator_label(segment, aliases) {
        let value = rest.trim_start_matches(':').trim();
        if value.is_empty() {
            return None;
        }
        return Some(LocatorSegment::new(label, value));
    }

    Some(LocatorSegment::new(LocatorType::Page, segment))
}

fn begins_with_locator_label(segment: &str, aliases: &[(String, LocatorType)]) -> bool {
    strip_locator_label(segment, aliases).is_some()
}

fn strip_locator_label<'a>(
    segment: &'a str,
    aliases: &[(String, LocatorType)],
) -> Option<(LocatorType, &'a str)> {
    let lower = segment.to_lowercase();
    let mut best: Option<(LocatorType, usize)> = None;

    for (alias, label) in aliases {
        if let Some(remainder) = lower.strip_prefix(alias)
            && alias_boundary(remainder)
        {
            let alias_len = alias.len();
            if best
                .as_ref()
                .is_none_or(|(_, best_len)| alias_len > *best_len)
            {
                best = Some((label.clone(), alias_len));
            }
        }
    }

    best.map(|(label, alias_len)| (label, segment[alias_len..].trim_start()))
}

fn alias_boundary(remainder: &str) -> bool {
    remainder.is_empty()
        || remainder.starts_with(':')
        || remainder.starts_with('.')
        || remainder.starts_with(char::is_whitespace)
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
    fn test_citation_simple_constructor_defaults() {
        let citation = Citation::simple("kuhn1962");

        assert_eq!(citation.items.len(), 1);
        assert_eq!(citation.items[0].id, "kuhn1962");
        assert_eq!(citation.mode, CitationMode::NonIntegral);
        assert_eq!(citation.position, None);
        assert!(!citation.suppress_author);
        assert_eq!(citation.note_number, None);
        assert_eq!(citation.prefix, None);
        assert_eq!(citation.suffix, None);
    }

    #[test]
    fn test_citation_default_fields_are_omitted_in_serialization() {
        let citation = Citation::simple("kuhn1962");
        let json = serde_json::to_value(&citation).unwrap();
        let object = json.as_object().unwrap();

        assert!(!object.contains_key("mode"));
        assert!(!object.contains_key("suppress-author"));

        let explicit = Citation {
            mode: CitationMode::Integral,
            suppress_author: true,
            ..citation
        };
        let explicit_json = serde_json::to_value(&explicit).unwrap();
        let explicit_object = explicit_json.as_object().unwrap();

        assert_eq!(explicit_object.get("mode").unwrap(), "integral");
        assert_eq!(explicit_object.get("suppress-author").unwrap(), true);
    }

    #[test]
    fn test_citation_item_with_locator() {
        let json = r#"
        {
            "id": "kuhn1962",
            "locator": {
                "label": "page",
                "value": "42-45"
            }
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "kuhn1962");
        assert_eq!(
            item.locator,
            Some(CitationLocator::single(LocatorType::Page, "42-45"))
        );
    }

    #[test]
    fn test_compound_locator_serde_roundtrip() {
        let json = r#"
        {
            "id": "smith2020",
            "locator": {
                "segments": [
                    { "label": "chapter", "value": "3" },
                    { "label": "section", "value": "42" }
                ]
            }
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        let segs = item.locator.as_ref().unwrap().segments();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].label, LocatorType::Chapter);
        assert_eq!(segs[0].value.value_str(), "3");
        assert_eq!(segs[1].label, LocatorType::Section);
        assert_eq!(segs[1].value.value_str(), "42");

        // Round-trip
        let serialized = serde_json::to_string(&item).unwrap();
        let deserialized: CitationItem = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.locator, item.locator);
    }

    #[test]
    fn test_compound_locator_rejects_single_segment() {
        let err = CitationLocator::compound(vec![LocatorSegment::new(LocatorType::Page, "42")])
            .expect_err("single-segment compound locator must be rejected");
        assert!(err.contains("at least two"));
    }

    #[test]
    fn test_citation_locator_canonical_string_is_stable() {
        let locator = CitationLocator::compound(vec![
            LocatorSegment::new(LocatorType::Page, "23"),
            LocatorSegment::new(LocatorType::Line, "13"),
        ])
        .unwrap();

        assert_eq!(locator.canonical_string(), "page:23,line:13");
    }

    #[test]
    fn test_custom_locator_type_round_trips_as_plain_string() {
        let json = r#"
        {
            "id": "score2024",
            "locator": {
                "label": "Movement",
                "value": "II"
            }
        }
        "#;

        let item: CitationItem = serde_json::from_str(json).expect("custom locator should parse");
        let locator = item.locator.expect("custom locator should exist");
        let segment = &locator.segments()[0];

        assert_eq!(segment.label, LocatorType::Custom("movement".to_string()));
        let serialized = serde_json::to_value(&CitationItem {
            id: "score2024".to_string(),
            locator: Some(locator),
            ..Default::default()
        })
        .expect("custom locator should serialize");

        assert_eq!(serialized["locator"]["label"], "movement");
    }

    #[test]
    fn test_custom_locator_type_normalizes_manual_construction() {
        let locator = LocatorType::Custom("Reel Label".to_string());

        assert_eq!(locator.as_key(), "reel-label");
        assert_eq!(
            locator,
            LocatorType::from_key("reel-label").expect("known custom key should parse")
        );
        assert_eq!(
            serde_json::to_string(&locator).expect("custom locator should serialize"),
            "\"reel-label\""
        );
    }

    #[test]
    fn test_locator_segments_single() {
        let item = CitationItem {
            id: "test".to_string(),
            locator: Some(CitationLocator::single(LocatorType::Page, "42")),
            ..Default::default()
        };
        let segments = item.locator_segments().unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].label, LocatorType::Page);
    }

    #[test]
    fn test_locator_segments_none() {
        let item = CitationItem {
            id: "test".to_string(),
            ..Default::default()
        };
        assert!(item.locator_segments().is_none());
    }

    #[test]
    fn test_single_locator_serializes_without_segments_wrapper() {
        let item = CitationItem {
            id: "test".to_string(),
            locator: Some(CitationLocator::single(LocatorType::Page, "42")),
            ..Default::default()
        };
        let json = serde_json::to_value(&item).unwrap();
        let locator = json
            .as_object()
            .unwrap()
            .get("locator")
            .and_then(serde_json::Value::as_object)
            .unwrap();
        assert!(locator.contains_key("label"));
        assert!(!locator.contains_key("segments"));
    }

    #[test]
    fn test_compound_locator_deserialization() {
        let json = r#"
        {
            "id": "smith2020",
            "locator": {
                "segments": [
                    { "label": "page", "value": "23" },
                    { "label": "line", "value": "13" }
                ]
            }
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        let segs = item.locator.as_ref().unwrap().segments();
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
            "locator": {
                "label": "figure",
                "value": {
                    "value": "A-3",
                    "plural": false
                }
            }
        }
        "#;
        let item: CitationItem = serde_json::from_str(json).unwrap();
        let segs = item.locator.as_ref().unwrap().segments();
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

    #[test]
    fn test_normalize_locator_text_with_explicit_aliases() {
        let aliases = vec![
            ("page".to_string(), LocatorType::Page),
            ("p.".to_string(), LocatorType::Page),
            ("chapter".to_string(), LocatorType::Chapter),
            ("ch.".to_string(), LocatorType::Chapter),
            ("section".to_string(), LocatorType::Section),
            ("§".to_string(), LocatorType::Section),
        ];

        // Bare number defaults to Page
        assert_eq!(
            normalize_locator_text("45", &aliases),
            Some(CitationLocator::single(LocatorType::Page, "45"))
        );

        // Explicit label
        assert_eq!(
            normalize_locator_text("chapter 2", &aliases),
            Some(CitationLocator::single(LocatorType::Chapter, "2"))
        );

        // Abbreviated label
        assert_eq!(
            normalize_locator_text("ch. 3", &aliases),
            Some(CitationLocator::single(LocatorType::Chapter, "3"))
        );

        // Symbol label
        assert_eq!(
            normalize_locator_text("§ 4", &aliases),
            Some(CitationLocator::single(LocatorType::Section, "4"))
        );

        // Compound locator
        let compound = normalize_locator_text("chapter 2, page 10", &aliases).unwrap();
        assert!(compound.is_compound());
        let segs = compound.segments();
        assert_eq!(segs[0].label, LocatorType::Chapter);
        assert_eq!(segs[1].label, LocatorType::Page);

        // Empty or invalid input
        assert_eq!(normalize_locator_text("", &aliases), None);
        assert_eq!(normalize_locator_text("   ", &aliases), None);
        assert_eq!(normalize_locator_text("chapter:", &aliases), None);
    }

    #[test]
    fn test_normalize_locator_text_with_abbreviated_aliases() {
        let aliases = vec![
            ("page".to_string(), LocatorType::Page),
            ("pp.".to_string(), LocatorType::Page),
            ("vol.".to_string(), LocatorType::Volume),
        ];

        assert_eq!(
            normalize_locator_text("page 45", &aliases),
            Some(CitationLocator::single(LocatorType::Page, "45"))
        );
        assert_eq!(
            normalize_locator_text("pp. 10-12", &aliases),
            Some(CitationLocator::single(LocatorType::Page, "10-12"))
        );
        assert_eq!(
            normalize_locator_text("vol. 1", &aliases),
            Some(CitationLocator::single(LocatorType::Volume, "1"))
        );
    }
}
