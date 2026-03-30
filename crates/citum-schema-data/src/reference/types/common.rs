//! Shared primitive types used across all reference categories.
//!
//! Includes multilingual string support, title representation, date types,
//! and reusable metadata structs (archive, eprint).

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use url::Url;

/// Unique identifier for a reference item.
pub type RefID = String;
/// BCP 47 language tag (e.g., `"en"`, `"de"`, `"ja"`).
pub type LangID = String;
/// Maps field names to their language tags for multilingual references.
pub type FieldLanguageMap = HashMap<String, LangID>;

/// A value that could be either a number or a string.
///
/// Used for fields that may contain numeric or string values, such as issue numbers,
/// volume numbers, or similar identifiers that can be formatted as either type.
/// The `Display` implementation shows the value in its appropriate form.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum NumOrStr {
    /// A numeric value.
    Number(i64),
    /// A string value.
    Str(String),
}

impl Display for NumOrStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Number(i) => write!(f, "{}", i),
            Self::Str(s) => write!(f, "{}", s),
        }
    }
}

/// A string that can be represented in multiple languages and scripts.
///
/// This is an enum that supports both simple strings and complex multilingual representations.
/// Use `Simple` for basic strings and `Complex` when you need to track original language,
/// transliterations, and translations.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum MultilingualString {
    /// A simple string in a single language.
    Simple(String),
    /// A complex multilingual string with original, transliterations, and translations.
    Complex(MultilingualComplex),
}

/// Complex multilingual representation with original, transliterations, and translations.
///
/// Allows capturing the original text in its native script, along with transliterations
/// (phonetic representations in different scripts) and translations (semantic equivalents
/// in different languages). This is essential for accurately rendering bibliographies in
/// multilingual and non-Latin-script contexts.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct MultilingualComplex {
    /// The text in its original script and language.
    pub original: String,
    /// ISO 639/BCP 47 language code for the original text.
    pub lang: Option<LangID>,
    /// Transliterations/Transcriptions of the original text into other scripts.
    ///
    /// Keys are typically script codes (e.g., "Latn" for Latin) or full BCP 47 tags.
    /// Values are the transliterated or transcribed text.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub transliterations: HashMap<String, String>,
    /// Translations of the text into other languages.
    ///
    /// Keys are ISO 639/BCP 47 language codes (e.g., "en" for English, "fr" for French).
    /// Values are the translated text.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub translations: HashMap<LangID, String>,
}

impl From<String> for MultilingualString {
    fn from(s: String) -> Self {
        Self::Simple(s)
    }
}

impl From<&str> for MultilingualString {
    fn from(s: &str) -> Self {
        Self::Simple(s.to_string())
    }
}

impl Display for MultilingualString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Simple(s) => write!(f, "{}", s),
            Self::Complex(c) => write!(f, "{}", c.original),
        }
    }
}

impl Default for MultilingualString {
    fn default() -> Self {
        Self::Simple(String::new())
    }
}

/// Structured archival location metadata for unpublished and archival material.
///
/// Models the hierarchical location of an item within an archive, following
/// common archival description standards (DACS, ISAD(G), EAD).
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct ArchiveInfo {
    /// Name of the archive or repository holding the material.
    ///
    /// Uses `MultilingualString` to support international institution names
    /// (e.g., 国立国会図書館 / National Diet Library).
    pub name: Option<MultilingualString>,
    /// Geographic location (city/country) of the archive.
    ///
    /// Parallel to `SimpleName.location`; both carry geographic-place semantics.
    pub place: Option<String>,
    /// Name of the collection within the archive.
    pub collection: Option<String>,
    /// Identifier for the collection (call number, accession number, etc.).
    pub collection_id: Option<String>,
    /// Name of the series within the collection.
    pub series: Option<String>,
    /// Box number within the collection.
    ///
    /// Uses raw identifier syntax `r#box`, which serde serializes as `"box"` transparently.
    pub r#box: Option<String>,
    /// Folder number within the box.
    pub folder: Option<String>,
    /// Item identifier within the folder.
    pub item: Option<String>,
    /// Display override for the archival location (shelfmark, call number, or complex location string).
    ///
    /// When present, overrides the structured fields for display. Acts as the legacy
    /// `archive_location` fallback for complex shelfmarks that don't fit the structured model.
    pub location: Option<String>,
    /// URL for the archival item (e.g. digitized finding aid or item page).
    pub url: Option<Url>,
}

/// Preprint server identifier following the biblatex eprint model.
///
/// Used on all three reference classes (Monograph, CollectionComponent, SerialComponent)
/// because journal articles routinely carry arXiv or similar identifiers.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct EprintInfo {
    /// The identifier on the preprint server (e.g. "2301.00001").
    pub id: String,
    /// The preprint server name in canonical lowercase form.
    ///
    /// Producers may supply mixed-case values such as "arXiv" or "SSRN";
    /// implementations should treat the field case-insensitively and compare
    /// or normalize on the lowercase form.
    pub server: String,
    /// Subject class or category used by the server (e.g. "cs.CL").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
}

/// A title can be a single string, a structured title, or a multilingual title.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum Title {
    /// A title in a single language.
    Single(String),
    /// A structured title.
    Structured(StructuredTitle),
    /// A complex multilingual title.
    Multilingual(MultilingualComplex),
    /// A title in multiple languages.
    Multi(Vec<(LangID, String)>),
    /// A structured title in multiple languages.
    MultiStructured(Vec<(LangID, StructuredTitle)>),
    /// An abbreviated title (shorthand, full).
    Shorthand(String, String),
}

/// Where title parts are meaningful, use this struct; Citum processors will not parse title strings.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct StructuredTitle {
    /// Full rendered title string (optional override).
    pub full: Option<String>,
    /// Main title component.
    pub main: String,
    /// Subtitle component.
    pub sub: Subtitle,
}

/// The subtitle can either be a string, as is the common case, or a vector of strings.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum Subtitle {
    /// A single subtitle string.
    String(String),
    /// Multiple subtitle strings.
    Vector(Vec<String>),
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Title::Single(s) => write!(f, "{}", s),
            Title::Multi(_m) => write!(f, "[multilingual title]"),
            Title::Multilingual(m) => write!(f, "{}", m.original),
            Title::Structured(s) => {
                let subtitle = match &s.sub {
                    Subtitle::String(s) => s.clone(),
                    Subtitle::Vector(v) => v.join(", "),
                };
                write!(f, "{}: {}", s.main, subtitle)
            }
            Title::MultiStructured(_m) => write!(f, "[multilingual structured title]"),
            Title::Shorthand(s, t) => write!(f, "{} ({})", s, t),
        }
    }
}

/// Date type.
#[derive(Debug, Clone, PartialEq)]
pub enum RefDate {
    /// A parsed EDTF date.
    Edtf(citum_edtf::Edtf),
    /// A literal date string that could not be parsed as EDTF.
    Literal(String),
}
