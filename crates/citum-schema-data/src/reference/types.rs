//! Reference data types and structures for citation items.
//!
//! This module defines the core data model for bibliographic references, including
//! different work types (monographs, serials, legal documents, etc.) and multilingual
//! support. References can be structured as flat items or with parent-child relationships.

use crate::reference::contributor::Contributor;
use crate::reference::date::EdtfString;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use url::Url;

pub type RefID = String;
pub type LangID = String;
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

/// A monograph, such as a book or a report, is a monolithic work published or produced as a complete entity.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Monograph {
    pub id: Option<RefID>,
    pub r#type: MonographType,
    pub title: Option<Title>,
    /// Parent or container title for monographic interviews and similar sources.
    pub container_title: Option<Title>,
    pub author: Option<Contributor>,
    pub editor: Option<Contributor>,
    pub translator: Option<Contributor>,
    /// Recipient for personal communications such as letters or emails.
    pub recipient: Option<Contributor>,
    /// Interviewer for interview-style references.
    pub interviewer: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    pub publisher: Option<Contributor>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    #[serde(alias = "ISBN")]
    pub isbn: Option<String>,
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// ADS bibcode identifier.
    pub ads_bibcode: Option<String>,
    pub edition: Option<String>,
    pub report_number: Option<String>,
    pub collection_number: Option<String>,
    pub genre: Option<String>,
    pub medium: Option<String>,
    /// Archive or repository name for unpublished material.
    pub archive: Option<String>,
    /// Archive location, shelfmark, or call number for unpublished material.
    #[serde(alias = "archive_location")]
    pub archive_location: Option<String>,
    pub keywords: Option<Vec<String>>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub original_date: Option<EdtfString>,
    pub original_title: Option<Title>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MonographType {
    #[default]
    Book,
    Manual,
    Report,
    Thesis,
    Webpage,
    Post,
    /// An interview treated as a standalone monographic source.
    Interview,
    /// An unpublished manuscript or archival document.
    Manuscript,
    PersonalCommunication,
    Document,
}

/// A collection of works, such as an anthology or proceedings.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Collection {
    pub id: Option<RefID>,
    pub r#type: CollectionType,
    pub title: Option<Title>,
    /// Optional short form of the parent title for style-directed rendering.
    pub short_title: Option<String>,
    pub editor: Option<Contributor>,
    pub translator: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    pub publisher: Option<Contributor>,
    pub collection_number: Option<String>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    #[serde(alias = "ISBN")]
    pub isbn: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// Types of collections.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum CollectionType {
    Anthology,
    Proceedings,
    EditedBook,
    EditedVolume,
}

/// A component of a larger monograph, such as a chapter in a book.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct CollectionComponent {
    pub id: Option<RefID>,
    pub r#type: MonographComponentType,
    pub title: Option<Title>,
    pub author: Option<Contributor>,
    pub translator: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    pub parent: Parent<Collection>,
    pub pages: Option<NumOrStr>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    pub genre: Option<String>,
    pub medium: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// Types of monograph components.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MonographComponentType {
    Chapter,
    Document,
}

/// A component of a larger serial publication; for example a journal or newspaper article.
/// The parent serial is referenced by its ID.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct SerialComponent {
    pub id: Option<RefID>,
    pub r#type: SerialComponentType,
    pub title: Option<Title>,
    pub author: Option<Contributor>,
    pub translator: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// The parent work, such as a magazine or journal.
    pub parent: Parent<Serial>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// ADS bibcode identifier.
    pub ads_bibcode: Option<String>,
    pub pages: Option<String>,
    pub volume: Option<NumOrStr>,
    pub issue: Option<NumOrStr>,
    pub genre: Option<String>,
    pub medium: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// Types of serial components.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub enum SerialComponentType {
    Article,
    Post,
    Review,
}

/// A serial publication (journal, magazine, etc.).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct Serial {
    pub r#type: SerialType,
    pub title: Option<Title>,
    /// Optional short form of the parent title for style-directed rendering.
    pub short_title: Option<String>,
    pub editor: Option<Contributor>,
    pub publisher: Option<Contributor>,
    #[serde(alias = "ISSN")]
    pub issn: Option<String>,
}

/// Types of serial publications.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SerialType {
    AcademicJournal,
    Blog,
    Magazine,
    Newspaper,
    Newsletter,
    Proceedings,
    Podcast,
    BroadcastProgram,
}

/// A parent reference (either embedded or by ID).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum Parent<T> {
    Embedded(T),
    Id(RefID),
}

/// A parent reference (either Monograph or Serial).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum ParentReference {
    Monograph(Box<Monograph>),
    Serial(Box<Serial>),
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
    pub full: Option<String>,
    pub main: String,
    pub sub: Subtitle,
}

/// The subtitle can either be a string, as is the common case, or a vector of strings.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum Subtitle {
    String(String),
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
    Edtf(citum_edtf::Edtf),
    Literal(String),
}

/// A legal case (court decision).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct LegalCase {
    pub id: Option<RefID>,
    /// Case name (e.g., "Brown v. Board of Education")
    pub title: Option<Title>,
    /// Court or authority (e.g., "U.S. Supreme Court")
    pub authority: String,
    /// Reporter volume
    pub volume: Option<String>,
    /// Reporter abbreviation (e.g., "U.S.", "F.2d")
    pub reporter: Option<String>,
    /// First page of case in reporter
    pub page: Option<String>,
    /// Decision date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// A statute or legislative act.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Statute {
    pub id: Option<RefID>,
    /// Statute name (e.g., "Civil Rights Act of 1964")
    pub title: Option<Title>,
    /// Legislative body (e.g., "U.S. Congress")
    pub authority: Option<String>,
    /// Code volume
    pub volume: Option<String>,
    /// Code abbreviation (e.g., "U.S.C.", "Pub. L.")
    pub code: Option<String>,
    /// Section or page number
    pub section: Option<String>,
    /// Enactment or publication date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// An international treaty or agreement.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Treaty {
    pub id: Option<RefID>,
    /// Treaty name (e.g., "Treaty of Versailles")
    pub title: Option<Title>,
    /// Parties to the treaty
    pub author: Option<Contributor>,
    /// Treaty series volume
    pub volume: Option<String>,
    /// Treaty series abbreviation (e.g., "U.N.T.S.")
    pub reporter: Option<String>,
    /// Page or treaty number
    pub page: Option<String>,
    /// Signing or ratification date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// A legislative or administrative hearing.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Hearing {
    pub id: Option<RefID>,
    /// Hearing title
    pub title: Option<Title>,
    /// Legislative body conducting the hearing (e.g., "U.S. Senate Committee on Finance")
    pub authority: Option<String>,
    /// Session or congress number
    pub session_number: Option<String>,
    /// Hearing date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// An administrative regulation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Regulation {
    pub id: Option<RefID>,
    /// Regulation title
    pub title: Option<Title>,
    /// Regulatory authority (e.g., "EPA", "Federal Register")
    pub authority: Option<String>,
    /// Code volume
    pub volume: Option<String>,
    /// Code abbreviation (e.g., "C.F.R.", "Fed. Reg.")
    pub code: Option<String>,
    /// Section or page number
    pub section: Option<String>,
    /// Publication or effective date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// A legal brief or filing.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Brief {
    pub id: Option<RefID>,
    /// Brief title or case name
    pub title: Option<Title>,
    /// Court (e.g., "U.S. Supreme Court")
    pub authority: Option<String>,
    /// Author/filer of the brief
    pub author: Option<Contributor>,
    /// Docket number
    pub docket_number: Option<String>,
    /// Filing date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// A classic work (Aristotle, Bible, etc.) with standard citation forms.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Classic {
    pub id: Option<RefID>,
    /// Work title (e.g., "Nicomachean Ethics")
    pub title: Option<Title>,
    /// Author (e.g., "Aristotle")
    pub author: Option<Contributor>,
    /// Editor or translator
    pub editor: Option<Contributor>,
    pub translator: Option<Contributor>,
    /// Volume in standard reference system
    pub volume: Option<String>,
    /// Section, book, or chapter in standard reference system
    pub section: Option<String>,
    /// Publication date of this edition (not original)
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    pub publisher: Option<Contributor>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// A patent.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Patent {
    pub id: Option<RefID>,
    /// Patent title
    pub title: Option<Title>,
    /// Inventor(s)
    pub author: Option<Contributor>,
    /// Assignee (patent holder)
    pub assignee: Option<Contributor>,
    /// Patent number (e.g., "U.S. Patent No. 7,347,809")
    pub patent_number: String,
    /// Application number
    pub application_number: Option<String>,
    /// Filing date
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub filing_date: Option<EdtfString>,
    /// Issue/grant date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Jurisdiction (e.g., "US", "EP", "JP")
    pub jurisdiction: Option<String>,
    /// Patent office (e.g., "U.S. Patent and Trademark Office")
    pub authority: Option<String>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// A research dataset.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Dataset {
    pub id: Option<RefID>,
    /// Dataset title
    pub title: Option<Title>,
    /// Dataset author(s)/creator(s)
    pub author: Option<Contributor>,
    /// Publication/release date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Publisher or repository (e.g., "Zenodo", "Dryad")
    pub publisher: Option<Contributor>,
    /// Version number
    pub version: Option<String>,
    /// File format (e.g., "CSV", "NetCDF", "HDF5")
    pub format: Option<String>,
    /// Dataset size (e.g., "2.4 GB", "150,000 records")
    pub size: Option<String>,
    /// Repository or archive name
    pub repository: Option<String>,
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// A technical standard or specification.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Standard {
    pub id: Option<RefID>,
    /// Standard title
    pub title: Option<Title>,
    /// Standards organization (e.g., "ISO", "ANSI", "IEEE")
    pub authority: Option<String>,
    /// Standard number (e.g., "ISO 8601", "IEEE 754-2008")
    pub standard_number: String,
    /// Publication date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Status (e.g., "published", "draft", "withdrawn")
    pub status: Option<String>,
    /// Publisher (usually same as authority)
    pub publisher: Option<Contributor>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}

/// Software or source code.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Software {
    pub id: Option<RefID>,
    /// Software title
    pub title: Option<Title>,
    /// Author(s)/developer(s)
    pub author: Option<Contributor>,
    /// Release date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Publisher or repository (e.g., "GitHub", "Zenodo")
    pub publisher: Option<Contributor>,
    /// Version number (e.g., "4.1.0", "v2.3.1")
    pub version: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// License (e.g., "MIT", "GPL-3.0", "Apache-2.0")
    pub license: Option<String>,
    /// Platform (e.g., "Windows", "macOS", "Linux", "cross-platform")
    pub platform: Option<String>,
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    pub language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    pub note: Option<String>,
    pub keywords: Option<Vec<String>>,
}
