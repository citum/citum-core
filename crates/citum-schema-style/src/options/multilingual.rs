/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};
use std::collections::HashMap;

/// Multilingual rendering options.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct MultilingualConfig {
    /// Preferred rendering mode for titles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_mode: Option<MultilingualMode>,
    /// Preferred rendering mode for names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_mode: Option<MultilingualMode>,
    /// Preferred script for transliterations (e.g., "Latn").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_script: Option<String>,
    /// Ordered priority list of BCP 47 transliteration tags (e.g. `["ja-Latn-hepburn", "ja-Latn"]`).
    /// Takes precedence over `preferred_script` when resolving transliterations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_transliteration: Option<Vec<String>>,
    /// Script-specific behavior configuration.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub scripts: HashMap<String, ScriptConfig>,
}

/// Rendering modes for multilingual content.
#[derive(Debug, PartialEq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum MultilingualMode {
    /// Use original script.
    Primary,
    /// Use transliteration.
    Transliterated,
    /// Use translation matching style locale.
    Translated,
    /// Combine transliteration and translation: `romanized [translated]`.
    /// Equivalent to `Pattern([transliterated, {translated, brackets}])`.
    Combined,
    /// Ordered sequence of views joined by spaces.
    ///
    /// Use this when a style requires more than two views — e.g. Chicago's
    /// `romanized original-script [translated]` or MLA's `original-script [translated]`.
    /// Each segment specifies a view and an optional bracket wrap.
    /// Segments whose resolved text is empty or identical to the previous
    /// segment are silently skipped (dedup).
    ///
    /// YAML form:
    /// ```yaml
    /// title-mode:
    ///   pattern:
    ///     - view: transliterated
    ///     - view: original-script
    ///     - view: translated
    ///       wrap: brackets
    /// ```
    Pattern(Vec<MultilingualSegment>),
}

/// A single view segment in a `MultilingualMode::Pattern`.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct MultilingualSegment {
    /// Which textual view to render for this segment.
    pub view: MultilingualView,
    /// Optional wrapping applied around the resolved text.
    #[serde(default, skip_serializing_if = "SegmentWrap::is_none")]
    pub wrap: SegmentWrap,
}

/// Which textual representation of a multilingual field to use in a pattern segment.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum MultilingualView {
    /// The original-script text as stored in the data.
    OriginalScript,
    /// The transliterated (romanized) form.
    Transliterated,
    /// The translation matching the style locale.
    Translated,
}

/// Bracket wrapping applied to a pattern segment.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum SegmentWrap {
    /// No wrapping.
    #[default]
    None,
    /// Wrap in square brackets: `[text]`.
    Brackets,
    /// Wrap in parentheses: `(text)`.
    Parentheses,
}

impl SegmentWrap {
    /// Returns `true` when the variant is `None` (used for skip-serializing).
    pub fn is_none(&self) -> bool {
        matches!(self, SegmentWrap::None)
    }

    /// Apply the wrap to a string slice, returning the wrapped form.
    pub fn apply(&self, text: &str) -> String {
        match self {
            SegmentWrap::None => text.to_string(),
            SegmentWrap::Brackets => format!("[{text}]"),
            SegmentWrap::Parentheses => format!("({text})"),
        }
    }
}

/// Configuration for specific scripts.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct ScriptConfig {
    /// Whether to use native ordering for this script (e.g., FamilyGiven for CJK).
    #[serde(default)]
    pub use_native_ordering: bool,
    /// Custom delimiter between name parts for this script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
    /// Custom delimiter between family and given name when this script is inverted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_separator: Option<String>,
    /// Punctuation convention to render for items whose effective language
    /// resolves to this script (e.g. remap CJK full-width delimiters to Latin
    /// half-width for Latin-script items in a bilingual style).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub punctuation: Option<PunctuationStyle>,
}

/// Punctuation convention applied to a script's rendered output.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum PunctuationStyle {
    /// Half-width Latin delimiters (`: , ( )`), with a trailing space after `:` and `,`.
    Latin,
    /// Full-width CJK delimiters (`： ， （ ）`), the unmodified default.
    FullWidth,
}

/// Custom deserializer for [`MultilingualMode`].
///
/// Unit variants are accepted as plain strings (`"primary"`, `"transliterated"`, etc.).
/// The `Pattern` variant is accepted as a single-key map `{pattern: [...]}`.
///
/// A hand-written `deserialize_any` visitor is used instead of serde's derived
/// `deserialize_enum` because `serde_yaml` cannot pass enum-variant input through
/// an outer `#[serde(untagged)]` wrapper — the standard derive would fail with
/// *"untagged and internally tagged enums do not support enum input"* when a
/// serialized `Pattern` value is round-tripped through [`crate::presets::MultilingualConfigEntry`].
impl<'de> de::Deserialize<'de> for MultilingualMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ModeVisitor;

        impl<'de> de::Visitor<'de> for ModeVisitor {
            type Value = MultilingualMode;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "a multilingual mode string (\"primary\", \"transliterated\", \
                     \"translated\", \"combined\") or a pattern object {{pattern: [...]}}"
                )
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "primary" => Ok(MultilingualMode::Primary),
                    "transliterated" => Ok(MultilingualMode::Transliterated),
                    "translated" => Ok(MultilingualMode::Translated),
                    "combined" => Ok(MultilingualMode::Combined),
                    _ => Err(E::unknown_variant(
                        v,
                        &[
                            "primary",
                            "transliterated",
                            "translated",
                            "combined",
                            "pattern",
                        ],
                    )),
                }
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let key: String = map
                    .next_key()?
                    .ok_or_else(|| de::Error::custom("expected \"pattern\" key, got empty map"))?;
                if key != "pattern" {
                    return Err(de::Error::unknown_field(&key, &["pattern"]));
                }
                let segments: Vec<MultilingualSegment> = map.next_value()?;
                if map.next_key::<String>()?.is_some() {
                    return Err(de::Error::custom("unexpected extra key in pattern object"));
                }
                Ok(MultilingualMode::Pattern(segments))
            }

            /// serde_yaml serialises `{pattern: [...]}` as an externally-tagged enum
            /// (not a plain map), so we also need to handle enum access.
            fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
                use de::VariantAccess as _;
                let (variant, access): (String, _) = data.variant()?;
                if variant == "pattern" {
                    let segments: Vec<MultilingualSegment> = access.newtype_variant()?;
                    Ok(MultilingualMode::Pattern(segments))
                } else {
                    Err(de::Error::unknown_variant(
                        &variant,
                        &[
                            "primary",
                            "transliterated",
                            "translated",
                            "combined",
                            "pattern",
                        ],
                    ))
                }
            }
        }

        deserializer.deserialize_any(ModeVisitor)
    }
}
