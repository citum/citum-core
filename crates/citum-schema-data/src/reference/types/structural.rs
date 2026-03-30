//! Hierarchical work types: monographs, collections, serials, and their components.

use super::common::{ArchiveInfo, EprintInfo, FieldLanguageMap, LangID, NumOrStr, RefID, Title};
use crate::reference::contributor::Contributor;
use crate::reference::date::EdtfString;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
use std::collections::HashMap;
use url::Url;

/// A monograph, such as a book or a report, is a monolithic work published or produced as a complete entity.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Monograph {
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Monograph subtype for style-directed formatting.
    pub r#type: MonographType,
    /// Title of the monographic work.
    pub title: Option<Title>,
    /// Parent or container title for monographic interviews and similar sources.
    pub container_title: Option<Title>,
    /// Author(s) of the work.
    pub author: Option<Contributor>,
    /// Editor(s) of the work.
    pub editor: Option<Contributor>,
    /// Translator(s) of the work.
    pub translator: Option<Contributor>,
    /// Recipient for personal communications such as letters or emails.
    pub recipient: Option<Contributor>,
    /// Interviewer for interview-style references.
    pub interviewer: Option<Contributor>,
    /// Guest for interview or podcast-style references.
    pub guest: Option<Contributor>,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Publisher of the work.
    pub publisher: Option<Contributor>,
    /// URL for the work.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the work.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// ISBN identifier.
    #[serde(alias = "ISBN")]
    pub isbn: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// ADS bibcode identifier.
    pub ads_bibcode: Option<String>,
    /// Edition descriptor.
    pub edition: Option<String>,
    /// Report number for technical reports.
    pub report_number: Option<String>,
    /// Collection or series number.
    pub collection_number: Option<String>,
    /// Free-text genre descriptor using kebab-case canonical forms (e.g., `"phd-thesis"`, `"short-film"`).
    /// See `docs/reference/GENRE_AND_MEDIUM_VALUES.md` for canonical values and `docs/policies/ENUM_VOCABULARY_POLICY.md`.
    pub genre: Option<String>,
    /// Free-text medium descriptor using kebab-case canonical forms (e.g., `"film"`, `"television"`).
    /// See `docs/reference/GENRE_AND_MEDIUM_VALUES.md` for canonical values and `docs/policies/ENUM_VOCABULARY_POLICY.md`.
    pub medium: Option<String>,
    /// Archive or repository name for unpublished material.
    pub archive: Option<String>,
    /// Archive location, shelfmark, or call number for unpublished material.
    #[serde(alias = "archive_location")]
    pub archive_location: Option<String>,
    /// Structured archival location metadata. When present, preferred over legacy `archive` and `archive_location`.
    pub archive_info: Option<ArchiveInfo>,
    /// Preprint server identifier.
    pub eprint: Option<EprintInfo>,
    /// Keywords or subject tags.
    pub keywords: Option<Vec<String>>,
    /// Original publication date (for reprints or translations).
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub original_date: Option<EdtfString>,
    /// Original title (for translations).
    pub original_title: Option<Title>,
}

/// Discriminates monograph subtypes for style-directed formatting.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MonographType {
    /// A book or monograph (default).
    #[default]
    Book,
    /// A technical manual or user guide.
    Manual,
    /// A technical or institutional report.
    Report,
    /// An academic thesis or dissertation.
    Thesis,
    /// A webpage or standalone web document.
    Webpage,
    /// A standalone post (e.g., social media, forum).
    Post,
    /// An interview treated as a standalone monographic source.
    Interview,
    /// An unpublished manuscript or archival document.
    Manuscript,
    /// A preprint hosted on a preprint server (arXiv, bioRxiv, SSRN, etc.).
    ///
    /// The preprint server has a custodial relationship with the work (hosting and
    /// preservation), not an editorial one. This parallels an archived manuscript
    /// held by a repository.
    Preprint,
    /// A letter, email, or other personal communication.
    PersonalCommunication,
    /// A generic standalone document that does not fit a more specific subtype.
    Document,
}

/// A collection of works, such as an anthology or proceedings.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Collection {
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Collection subtype for style-directed formatting.
    pub r#type: CollectionType,
    /// Title of the collection.
    pub title: Option<Title>,
    /// Optional short form of the parent title for style-directed rendering.
    pub short_title: Option<String>,
    /// Editor(s) of the collection.
    pub editor: Option<Contributor>,
    /// Translator(s) of the collection.
    pub translator: Option<Contributor>,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Publisher of the collection.
    pub publisher: Option<Contributor>,
    /// Collection or series number.
    pub collection_number: Option<String>,
    /// URL for the collection.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the collection.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// ISBN identifier.
    #[serde(alias = "ISBN")]
    pub isbn: Option<String>,
    /// Keywords or subject tags.
    pub keywords: Option<Vec<String>>,
}

/// Discriminates collection subtypes for style-directed formatting.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum CollectionType {
    /// A curated anthology of independent works (e.g., short stories, essays).
    Anthology,
    /// Published proceedings of a conference or symposium.
    Proceedings,
    /// A book assembled from contributions by multiple authors under an editor.
    EditedBook,
    /// An edited volume that may span multiple books or a series.
    EditedVolume,
}

/// A component of a larger monograph, such as a chapter in a book.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct CollectionComponent {
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Component subtype for style-directed formatting.
    pub r#type: MonographComponentType,
    /// Title of the component.
    pub title: Option<Title>,
    /// Author(s) of the component.
    pub author: Option<Contributor>,
    /// Translator(s) of the component.
    pub translator: Option<Contributor>,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// The parent collection.
    pub parent: Parent<Collection>,
    /// Page range within the parent collection.
    pub pages: Option<NumOrStr>,
    /// URL for the component.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the component.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// Free-text genre descriptor.
    pub genre: Option<String>,
    /// Free-text medium descriptor.
    pub medium: Option<String>,
    /// Structured archival location metadata.
    pub archive_info: Option<ArchiveInfo>,
    /// Preprint server identifier.
    pub eprint: Option<EprintInfo>,
    /// Keywords or subject tags.
    pub keywords: Option<Vec<String>>,
}

/// Discriminates monograph-component subtypes for style-directed formatting.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MonographComponentType {
    /// A chapter within a book or edited volume.
    Chapter,
    /// A document component that does not fit a more specific subtype.
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
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Component subtype for style-directed formatting.
    pub r#type: SerialComponentType,
    /// Title of the component.
    pub title: Option<Title>,
    /// Author(s) of the component.
    pub author: Option<Contributor>,
    /// Translator(s) of the component.
    pub translator: Option<Contributor>,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// The parent work, such as a magazine or journal.
    pub parent: Parent<Serial>,
    /// URL for the component.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the component.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// ADS bibcode identifier.
    pub ads_bibcode: Option<String>,
    /// Page range within the parent serial issue.
    pub pages: Option<String>,
    /// Volume number of the parent serial.
    pub volume: Option<NumOrStr>,
    /// Issue number of the parent serial.
    pub issue: Option<NumOrStr>,
    /// Free-text genre descriptor.
    pub genre: Option<String>,
    /// Free-text medium descriptor.
    pub medium: Option<String>,
    /// Structured archival location metadata.
    pub archive_info: Option<ArchiveInfo>,
    /// Preprint server identifier.
    pub eprint: Option<EprintInfo>,
    /// Keywords or subject tags.
    pub keywords: Option<Vec<String>>,
}

/// Discriminates serial-component subtypes for style-directed formatting.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub enum SerialComponentType {
    /// A peer-reviewed or editorial article in a journal, magazine, or newspaper.
    Article,
    /// A post within an online serial (blog, news site, social feed).
    Post,
    /// A review published in a serial (book review, film review, etc.).
    Review,
}

/// A serial publication (journal, magazine, etc.).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct Serial {
    /// Serial subtype for style-directed formatting.
    pub r#type: SerialType,
    /// Title of the serial.
    pub title: Option<Title>,
    /// Optional short form of the parent title for style-directed rendering.
    pub short_title: Option<String>,
    /// Editor(s) of the serial.
    pub editor: Option<Contributor>,
    /// Publisher of the serial.
    pub publisher: Option<Contributor>,
    /// ISSN identifier.
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
    /// An academic peer-reviewed journal.
    AcademicJournal,
    /// A blog or personal website updated periodically.
    Blog,
    /// A general-interest magazine.
    Magazine,
    /// A newspaper.
    Newspaper,
    /// A newsletter.
    Newsletter,
    /// A conference proceedings serial.
    Proceedings,
    /// A podcast.
    Podcast,
    /// A broadcast program (radio, television).
    BroadcastProgram,
}

/// A parent reference (either embedded or by ID).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum Parent<T> {
    /// The parent is embedded inline.
    Embedded(T),
    /// The parent is referenced by its ID.
    Id(RefID),
}

/// A parent reference (either Monograph or Serial).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum ParentReference {
    /// A monograph parent.
    Monograph(Box<Monograph>),
    /// A serial parent.
    Serial(Box<Serial>),
}
