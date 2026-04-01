//! Hierarchical work types: monographs, collections, serials, and their components.

use super::common::{
    ArchiveInfo, EprintInfo, FieldLanguageMap, HasNumbering, LangID, NormalizeNumbering, NumOrStr,
    Numbering, RefID, Title,
};
use crate::reference::WorkRelation;
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
#[serde(from = "MonographDeser", rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Monograph {
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Subtype for style-directed formatting.
    pub r#type: MonographType,
    /// Title of the monographic work.
    pub title: Option<Title>,
    /// Optional short form of the title for style-directed rendering.
    pub short_title: Option<String>,
    /// The primary container for this work (e.g., a multivolume set or series).
    pub container: Option<WorkRelation>,
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
    /// Volume number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Issue number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    /// Edition (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Part or report number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., volume, issue, edition).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
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
    /// Original publication relation (for reprints or translations).
    pub original: Option<WorkRelation>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
struct MonographDeser {
    id: Option<RefID>,
    r#type: MonographType,
    title: Option<Title>,
    short_title: Option<String>,
    container: Option<WorkRelation>,
    author: Option<Contributor>,
    editor: Option<Contributor>,
    translator: Option<Contributor>,
    recipient: Option<Contributor>,
    interviewer: Option<Contributor>,
    guest: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    issued: EdtfString,
    publisher: Option<Contributor>,
    #[serde(alias = "URL")]
    url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    accessed: Option<EdtfString>,
    language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    field_languages: FieldLanguageMap,
    note: Option<String>,
    #[serde(alias = "ISBN")]
    isbn: Option<String>,
    #[serde(alias = "DOI")]
    doi: Option<String>,
    ads_bibcode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    volume: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    issue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    number: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    numbering: Vec<Numbering>,
    genre: Option<String>,
    medium: Option<String>,
    archive: Option<String>,
    #[serde(alias = "archive_location")]
    archive_location: Option<String>,
    archive_info: Option<ArchiveInfo>,
    eprint: Option<EprintInfo>,
    keywords: Option<Vec<String>>,
    original: Option<WorkRelation>,
}

impl From<MonographDeser> for Monograph {
    fn from(raw: MonographDeser) -> Self {
        let mut monograph = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            short_title: raw.short_title,
            container: raw.container,
            author: raw.author,
            editor: raw.editor,
            translator: raw.translator,
            recipient: raw.recipient,
            interviewer: raw.interviewer,
            guest: raw.guest,
            issued: raw.issued,
            publisher: raw.publisher,
            url: raw.url,
            accessed: raw.accessed,
            language: raw.language,
            field_languages: raw.field_languages,
            note: raw.note,
            isbn: raw.isbn,
            doi: raw.doi,
            ads_bibcode: raw.ads_bibcode,
            volume: raw.volume,
            issue: raw.issue,
            edition: raw.edition,
            number: raw.number,
            numbering: raw.numbering,
            genre: raw.genre,
            medium: raw.medium,
            archive: raw.archive,
            archive_location: raw.archive_location,
            archive_info: raw.archive_info,
            eprint: raw.eprint,
            keywords: raw.keywords,
            original: raw.original,
        };
        monograph.normalize_numbering();
        monograph
    }
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
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(from = "CollectionDeser", rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Collection {
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Collection subtype for style-directed formatting.
    pub r#type: CollectionType,
    /// Title of the collection.
    pub title: Option<Title>,
    /// Optional short form of the title for style-directed rendering.
    pub short_title: Option<String>,
    /// The primary container for this collection (e.g., a series).
    pub container: Option<WorkRelation>,
    /// Editor(s) of the collection.
    pub editor: Option<Contributor>,
    /// Translator(s) of the collection.
    pub translator: Option<Contributor>,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Publisher of the collection.
    pub publisher: Option<Contributor>,
    /// Volume number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Issue number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    /// Edition (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Part or report number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., volume, collection number).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
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

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
struct CollectionDeser {
    id: Option<RefID>,
    r#type: CollectionType,
    title: Option<Title>,
    short_title: Option<String>,
    container: Option<WorkRelation>,
    editor: Option<Contributor>,
    translator: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    issued: EdtfString,
    publisher: Option<Contributor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    volume: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    issue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    number: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    numbering: Vec<Numbering>,
    #[serde(alias = "URL")]
    url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    accessed: Option<EdtfString>,
    language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    field_languages: FieldLanguageMap,
    note: Option<String>,
    #[serde(alias = "ISBN")]
    isbn: Option<String>,
    keywords: Option<Vec<String>>,
}

impl From<CollectionDeser> for Collection {
    fn from(raw: CollectionDeser) -> Self {
        let mut collection = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            short_title: raw.short_title,
            container: raw.container,
            editor: raw.editor,
            translator: raw.translator,
            issued: raw.issued,
            publisher: raw.publisher,
            volume: raw.volume,
            issue: raw.issue,
            edition: raw.edition,
            number: raw.number,
            numbering: raw.numbering,
            url: raw.url,
            accessed: raw.accessed,
            language: raw.language,
            field_languages: raw.field_languages,
            note: raw.note,
            isbn: raw.isbn,
            keywords: raw.keywords,
        };
        collection.normalize_numbering();
        collection
    }
}

/// Discriminates collection subtypes for style-directed formatting.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum CollectionType {
    /// A curated anthology of independent works (e.g., short stories, essays).
    #[default]
    Anthology,
    /// Published proceedings of a conference or symposium.
    Proceedings,
    /// A book assembled from contributions by multiple authors under an editor.
    EditedBook,
    /// An edited volume that may span multiple books or a series.
    EditedVolume,
}

/// A component of a larger monograph, such as a chapter in a book.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(from = "CollectionComponentDeser", rename_all = "kebab-case")]
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
    /// The parent collection or monograph.
    pub container: Option<WorkRelation>,
    /// Volume number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Issue number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    /// Edition (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Part or report number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., chapter number, part number).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
    /// Page range within the parent container.
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
    /// Original publication relation.
    pub original: Option<WorkRelation>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
struct CollectionComponentDeser {
    id: Option<RefID>,
    r#type: MonographComponentType,
    title: Option<Title>,
    author: Option<Contributor>,
    translator: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    issued: EdtfString,
    container: Option<WorkRelation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    volume: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    issue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    number: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    numbering: Vec<Numbering>,
    pages: Option<NumOrStr>,
    #[serde(alias = "URL")]
    url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    accessed: Option<EdtfString>,
    language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    field_languages: FieldLanguageMap,
    note: Option<String>,
    #[serde(alias = "DOI")]
    doi: Option<String>,
    genre: Option<String>,
    medium: Option<String>,
    archive_info: Option<ArchiveInfo>,
    eprint: Option<EprintInfo>,
    keywords: Option<Vec<String>>,
    original: Option<WorkRelation>,
}

impl From<CollectionComponentDeser> for CollectionComponent {
    fn from(raw: CollectionComponentDeser) -> Self {
        let mut component = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            author: raw.author,
            translator: raw.translator,
            issued: raw.issued,
            container: raw.container,
            volume: raw.volume,
            issue: raw.issue,
            edition: raw.edition,
            number: raw.number,
            numbering: raw.numbering,
            pages: raw.pages,
            url: raw.url,
            accessed: raw.accessed,
            language: raw.language,
            field_languages: raw.field_languages,
            note: raw.note,
            doi: raw.doi,
            genre: raw.genre,
            medium: raw.medium,
            archive_info: raw.archive_info,
            eprint: raw.eprint,
            keywords: raw.keywords,
            original: raw.original,
        };
        component.normalize_numbering();
        component
    }
}

/// Discriminates monograph-component subtypes for style-directed formatting.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MonographComponentType {
    /// A chapter within a book or edited volume.
    #[default]
    Chapter,
    /// A document component that does not fit a more specific subtype.
    Document,
}

/// A component of a larger serial publication; for example a journal or newspaper article.
/// The parent serial is referenced by its ID.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(from = "SerialComponentDeser", rename_all = "kebab-case")]
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
    /// The parent work, such as a magazine, journal, or book set.
    pub container: Option<WorkRelation>,
    /// Volume number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Issue number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    /// Edition (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Part or report number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., volume, issue, part).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
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
    /// Work relation for reviews.
    pub reviewed: Option<WorkRelation>,
    /// Original publication relation.
    pub original: Option<WorkRelation>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
struct SerialComponentDeser {
    id: Option<RefID>,
    r#type: SerialComponentType,
    title: Option<Title>,
    author: Option<Contributor>,
    translator: Option<Contributor>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    issued: EdtfString,
    container: Option<WorkRelation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    volume: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    issue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    number: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    numbering: Vec<Numbering>,
    #[serde(alias = "URL")]
    url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    accessed: Option<EdtfString>,
    language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    field_languages: FieldLanguageMap,
    note: Option<String>,
    #[serde(alias = "DOI")]
    doi: Option<String>,
    ads_bibcode: Option<String>,
    pages: Option<String>,
    genre: Option<String>,
    medium: Option<String>,
    archive_info: Option<ArchiveInfo>,
    eprint: Option<EprintInfo>,
    keywords: Option<Vec<String>>,
    reviewed: Option<WorkRelation>,
    original: Option<WorkRelation>,
}

impl From<SerialComponentDeser> for SerialComponent {
    fn from(raw: SerialComponentDeser) -> Self {
        let mut component = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            author: raw.author,
            translator: raw.translator,
            issued: raw.issued,
            container: raw.container,
            volume: raw.volume,
            issue: raw.issue,
            edition: raw.edition,
            number: raw.number,
            numbering: raw.numbering,
            url: raw.url,
            accessed: raw.accessed,
            language: raw.language,
            field_languages: raw.field_languages,
            note: raw.note,
            doi: raw.doi,
            ads_bibcode: raw.ads_bibcode,
            pages: raw.pages,
            genre: raw.genre,
            medium: raw.medium,
            archive_info: raw.archive_info,
            eprint: raw.eprint,
            keywords: raw.keywords,
            reviewed: raw.reviewed,
            original: raw.original,
        };
        component.normalize_numbering();
        component
    }
}

/// Discriminates serial-component subtypes for style-directed formatting.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub enum SerialComponentType {
    /// A peer-reviewed or editorial article in a journal, magazine, or newspaper.
    #[default]
    Article,
    /// A post within an online serial (blog, news site, social feed).
    Post,
    /// A review published in a serial (book review, film review, etc.).
    Review,
}

/// A serial publication (journal, magazine, etc.).
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct Serial {
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Serial subtype for style-directed formatting.
    pub r#type: SerialType,
    /// Title of the serial.
    pub title: Option<Title>,
    /// Optional short form of the title for style-directed rendering.
    pub short_title: Option<String>,
    /// The parent container for this serial (e.g., a larger series).
    pub container: Option<WorkRelation>,
    /// Editor(s) of the serial.
    pub editor: Option<Contributor>,
    /// Publisher of the serial.
    pub publisher: Option<Contributor>,
    /// URL for the serial.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the serial.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// ISSN identifier.
    #[serde(alias = "ISSN")]
    pub issn: Option<String>,
}

impl HasNumbering for Monograph {
    fn numbering(&self) -> &[Numbering] {
        &self.numbering
    }
}

impl NormalizeNumbering for Monograph {
    fn numbering_mut(&mut self) -> &mut Vec<Numbering> {
        &mut self.numbering
    }

    fn volume_mut(&mut self) -> &mut Option<String> {
        &mut self.volume
    }

    fn issue_mut(&mut self) -> &mut Option<String> {
        &mut self.issue
    }

    fn edition_mut(&mut self) -> &mut Option<String> {
        &mut self.edition
    }

    fn number_mut(&mut self) -> &mut Option<String> {
        &mut self.number
    }
}

impl HasNumbering for Collection {
    fn numbering(&self) -> &[Numbering] {
        &self.numbering
    }
}

impl NormalizeNumbering for Collection {
    fn numbering_mut(&mut self) -> &mut Vec<Numbering> {
        &mut self.numbering
    }

    fn volume_mut(&mut self) -> &mut Option<String> {
        &mut self.volume
    }

    fn issue_mut(&mut self) -> &mut Option<String> {
        &mut self.issue
    }

    fn edition_mut(&mut self) -> &mut Option<String> {
        &mut self.edition
    }

    fn number_mut(&mut self) -> &mut Option<String> {
        &mut self.number
    }
}

impl HasNumbering for CollectionComponent {
    fn numbering(&self) -> &[Numbering] {
        &self.numbering
    }
}

impl NormalizeNumbering for CollectionComponent {
    fn numbering_mut(&mut self) -> &mut Vec<Numbering> {
        &mut self.numbering
    }

    fn volume_mut(&mut self) -> &mut Option<String> {
        &mut self.volume
    }

    fn issue_mut(&mut self) -> &mut Option<String> {
        &mut self.issue
    }

    fn edition_mut(&mut self) -> &mut Option<String> {
        &mut self.edition
    }

    fn number_mut(&mut self) -> &mut Option<String> {
        &mut self.number
    }
}

impl HasNumbering for SerialComponent {
    fn numbering(&self) -> &[Numbering] {
        &self.numbering
    }
}

impl NormalizeNumbering for SerialComponent {
    fn numbering_mut(&mut self) -> &mut Vec<Numbering> {
        &mut self.numbering
    }

    fn volume_mut(&mut self) -> &mut Option<String> {
        &mut self.volume
    }

    fn issue_mut(&mut self) -> &mut Option<String> {
        &mut self.issue
    }

    fn edition_mut(&mut self) -> &mut Option<String> {
        &mut self.edition
    }

    fn number_mut(&mut self) -> &mut Option<String> {
        &mut self.number
    }
}

/// Types of serial publications.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SerialType {
    /// An academic peer-reviewed journal.
    #[default]
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
