//! Hierarchical work types: monographs, collections, serials, and their components.

use super::common::{
    ArchiveInfo, EprintInfo, FieldLanguageMap, HasNumbering, LangID, NormalizeNumbering, NumOrStr,
    Numbering, Publisher, RefID, Title,
};
use crate::reference::WorkRelation;
use crate::reference::contributor::{
    Contributor, ContributorEntry, ContributorList, ContributorRole,
};
use crate::reference::date::EdtfString;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
use std::collections::HashMap;
use url::Url;

/// Fold named contributor shorthands into the contributors vec.
///
/// Explicit `contributors` entries come first (source order). Named
/// shorthands are appended only when no existing entry already carries
/// the same role and contributor value.
fn fold_contributors(
    mut contributors: Vec<ContributorEntry>,
    shorthands: &[(ContributorRole, Option<&Contributor>)],
) -> Vec<ContributorEntry> {
    for (role, maybe_c) in shorthands {
        if let Some(c) = maybe_c
            && !contributors
                .iter()
                .any(|e| &e.role == role && &e.contributor == *c)
        {
            contributors.push(ContributorEntry {
                role: role.clone(),
                contributor: (*c).clone(),
            });
        }
    }
    contributors
}

/// Collect contributors with a given role from a slice of entries.
fn collect_contributors_by_role(
    entries: &[ContributorEntry],
    role: &ContributorRole,
) -> Option<Contributor> {
    let matching: Vec<&Contributor> = entries
        .iter()
        .filter(|entry| &entry.role == role)
        .map(|entry| &entry.contributor)
        .collect();

    match matching.len() {
        0 => None,
        1 => Some(matching[0].clone()),
        _ => Some(Contributor::ContributorList(ContributorList(
            matching.into_iter().cloned().collect(),
        ))),
    }
}

/// A monograph, such as a book or a report, is a monolithic work published or produced as a complete entity.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(from = "MonographDeser", rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Monograph {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Subtype for style-directed formatting.
    pub r#type: MonographType,
    /// Title of the monographic work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Optional short form of the title for style-directed rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_title: Option<String>,
    /// The primary container for this work (e.g., a multivolume set or series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<WorkRelation>,
    /// Author(s) of the work.
    #[serde(skip_serializing)]
    pub author: Option<Contributor>,
    /// Editor(s) of the work.
    #[serde(skip_serializing)]
    pub editor: Option<Contributor>,
    /// Translator(s) of the work.
    #[serde(skip_serializing)]
    pub translator: Option<Contributor>,
    /// Unified contributor list with explicit role tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<ContributorEntry>,
    /// Creation or origination date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// Publisher of the work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Publisher>,
    /// URL for the work.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// ISBN identifier.
    #[serde(alias = "ISBN", skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI", skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    /// ADS bibcode identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    /// Generic document number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., volume, issue, edition).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
    /// Free-text genre descriptor using kebab-case canonical forms (e.g., `"phd-thesis"`, `"short-film"`).
    /// See `docs/reference/GENRE_AND_MEDIUM_VALUES.md` for canonical values and `docs/policies/ENUM_VOCABULARY_POLICY.md`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    /// Free-text medium descriptor using kebab-case canonical forms (e.g., `"film"`, `"television"`).
    /// See `docs/reference/GENRE_AND_MEDIUM_VALUES.md` for canonical values and `docs/policies/ENUM_VOCABULARY_POLICY.md`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    /// Archive or repository name for unpublished material.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive: Option<String>,
    /// Archive location, shelfmark, or call number for unpublished material.
    #[serde(alias = "archive_location", skip_serializing_if = "Option::is_none")]
    pub archive_location: Option<String>,
    /// Structured archival location metadata. When present, preferred over legacy `archive` and `archive_location`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_info: Option<ArchiveInfo>,
    /// Preprint server identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eprint: Option<EprintInfo>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    /// Original publication relation (for reprints or translations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<WorkRelation>,
    /// Publication status (e.g., `"forthcoming"`, `"in press"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Date the work became or will become publicly available.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_date: Option<EdtfString>,
    /// Physical dimensions or format (e.g., `"24 x 30 cm"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Duration or running time in ISO 8601 or freeform (e.g., `"PT2H30M"`, `"90 min"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    /// Reference list count or citation string (e.g., `"42 refs"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<String>,
    /// Cartographic scale for maps and globes (e.g., `"1:250,000"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    contributors: Vec<ContributorEntry>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
    created: EdtfString,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
    issued: EdtfString,
    publisher: Option<Publisher>,
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
    status: Option<String>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    available_date: Option<EdtfString>,
    size: Option<String>,
    duration: Option<String>,
    references: Option<String>,
    scale: Option<String>,
}

impl From<MonographDeser> for Monograph {
    fn from(raw: MonographDeser) -> Self {
        let contributors = fold_contributors(
            raw.contributors,
            &[
                (ContributorRole::Author, raw.author.as_ref()),
                (ContributorRole::Editor, raw.editor.as_ref()),
                (ContributorRole::Translator, raw.translator.as_ref()),
            ],
        );
        let author = collect_contributors_by_role(&contributors, &ContributorRole::Author);
        let editor = collect_contributors_by_role(&contributors, &ContributorRole::Editor);
        let translator = collect_contributors_by_role(&contributors, &ContributorRole::Translator);
        let mut monograph = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            short_title: raw.short_title,
            container: raw.container,
            author,
            editor,
            translator,
            contributors,
            created: raw.created,
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
            status: raw.status,
            available_date: raw.available_date,
            size: raw.size,
            duration: raw.duration,
            references: raw.references,
            scale: raw.scale,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Collection subtype for style-directed formatting.
    pub r#type: CollectionType,
    /// Title of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Optional short form of the title for style-directed rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_title: Option<String>,
    /// The primary container for this collection (e.g., a series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<WorkRelation>,
    /// Editor(s) of the collection.
    #[serde(skip_serializing)]
    pub editor: Option<Contributor>,
    /// Translator(s) of the collection.
    #[serde(skip_serializing)]
    pub translator: Option<Contributor>,
    /// Unified contributor list with explicit role tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<ContributorEntry>,
    /// Creation or origination date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// Publisher of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Publisher>,
    /// Volume number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Issue number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    /// Edition (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Generic document number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., volume, collection number).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
    /// URL for the collection.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// ISBN identifier.
    #[serde(alias = "ISBN", skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,
    /// Originating event (e.g., conference) for proceedings-type containers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<WorkRelation>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    contributors: Vec<ContributorEntry>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
    created: EdtfString,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
    issued: EdtfString,
    publisher: Option<Publisher>,
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
    event: Option<WorkRelation>,
    keywords: Option<Vec<String>>,
}

impl From<CollectionDeser> for Collection {
    fn from(raw: CollectionDeser) -> Self {
        let contributors = fold_contributors(
            raw.contributors,
            &[
                (ContributorRole::Editor, raw.editor.as_ref()),
                (ContributorRole::Translator, raw.translator.as_ref()),
            ],
        );
        let editor = collect_contributors_by_role(&contributors, &ContributorRole::Editor);
        let translator = collect_contributors_by_role(&contributors, &ContributorRole::Translator);
        let mut collection = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            short_title: raw.short_title,
            container: raw.container,
            editor,
            translator,
            contributors,
            created: raw.created,
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
            event: raw.event,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Component subtype for style-directed formatting.
    pub r#type: MonographComponentType,
    /// Title of the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Author(s) of the component.
    #[serde(skip_serializing)]
    pub author: Option<Contributor>,
    /// Translator(s) of the component.
    #[serde(skip_serializing)]
    pub translator: Option<Contributor>,
    /// Unified contributor list with explicit role tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<ContributorEntry>,
    /// Creation or origination date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// The parent collection or monograph.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    /// Generic document number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., chapter number, part number).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
    /// Page range within the parent container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<NumOrStr>,
    /// URL for the component.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI", skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    /// Free-text genre descriptor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    /// Free-text medium descriptor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    /// Publication status (e.g., `"forthcoming"`, `"last modified"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Structured archival location metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_info: Option<ArchiveInfo>,
    /// Preprint server identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eprint: Option<EprintInfo>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    /// Original publication relation.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    contributors: Vec<ContributorEntry>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
    created: EdtfString,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
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
    status: Option<String>,
    archive_info: Option<ArchiveInfo>,
    eprint: Option<EprintInfo>,
    keywords: Option<Vec<String>>,
    original: Option<WorkRelation>,
}

impl From<CollectionComponentDeser> for CollectionComponent {
    fn from(raw: CollectionComponentDeser) -> Self {
        let contributors = fold_contributors(
            raw.contributors,
            &[
                (ContributorRole::Author, raw.author.as_ref()),
                (ContributorRole::Translator, raw.translator.as_ref()),
            ],
        );
        let author = collect_contributors_by_role(&contributors, &ContributorRole::Author);
        let translator = collect_contributors_by_role(&contributors, &ContributorRole::Translator);
        let mut component = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            author,
            translator,
            contributors,
            created: raw.created,
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
            status: raw.status,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Component subtype for style-directed formatting.
    pub r#type: SerialComponentType,
    /// Title of the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Author(s) of the component.
    #[serde(skip_serializing)]
    pub author: Option<Contributor>,
    /// Translator(s) of the component.
    #[serde(skip_serializing)]
    pub translator: Option<Contributor>,
    /// Unified contributor list with explicit role tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<ContributorEntry>,
    /// Creation or origination date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Publication date.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// The parent work, such as a magazine, journal, or book set.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    /// Generic document number (shorthand for numbering).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Numbering identifiers (e.g., volume, issue, number).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
    /// URL for the component.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI", skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    /// ADS bibcode identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ads_bibcode: Option<String>,
    /// Page range within the parent serial issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,
    /// Free-text genre descriptor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    /// Free-text medium descriptor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    /// Structured archival location metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_info: Option<ArchiveInfo>,
    /// Preprint server identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eprint: Option<EprintInfo>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    /// Section within the serial issue (e.g., `"A"`, `"Sports"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Publication status (e.g., `"forthcoming"`, `"ahead of print"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Date the component became publicly available (e.g., ahead-of-print date).
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_date: Option<EdtfString>,
    /// Work relation for reviews.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed: Option<WorkRelation>,
    /// Original publication relation.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    contributors: Vec<ContributorEntry>,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
    created: EdtfString,
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default)]
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
    section: Option<String>,
    status: Option<String>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    available_date: Option<EdtfString>,
    reviewed: Option<WorkRelation>,
    original: Option<WorkRelation>,
}

impl From<SerialComponentDeser> for SerialComponent {
    fn from(raw: SerialComponentDeser) -> Self {
        let contributors = fold_contributors(
            raw.contributors,
            &[
                (ContributorRole::Author, raw.author.as_ref()),
                (ContributorRole::Translator, raw.translator.as_ref()),
            ],
        );
        let author = collect_contributors_by_role(&contributors, &ContributorRole::Author);
        let translator = collect_contributors_by_role(&contributors, &ContributorRole::Translator);
        let mut component = Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            author,
            translator,
            contributors,
            created: raw.created,
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
            section: raw.section,
            status: raw.status,
            available_date: raw.available_date,
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
#[serde(from = "SerialDeser", rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Serial {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Serial subtype for style-directed formatting.
    pub r#type: SerialType,
    /// Title of the serial.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Optional short form of the title for style-directed rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_title: Option<String>,
    /// The parent container for this serial (e.g., a larger series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<WorkRelation>,
    /// Editor(s) of the serial.
    #[serde(skip_serializing)]
    pub editor: Option<Contributor>,
    /// Unified contributor list with explicit role tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<ContributorEntry>,
    /// Publisher of the serial.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Publisher>,
    /// URL for the serial.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the serial.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// ISSN identifier.
    #[serde(alias = "ISSN", skip_serializing_if = "Option::is_none")]
    pub issn: Option<String>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
struct SerialDeser {
    id: Option<RefID>,
    r#type: SerialType,
    title: Option<Title>,
    short_title: Option<String>,
    container: Option<WorkRelation>,
    editor: Option<Contributor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    contributors: Vec<ContributorEntry>,
    publisher: Option<Publisher>,
    #[serde(alias = "URL")]
    url: Option<Url>,
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    accessed: Option<EdtfString>,
    language: Option<LangID>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    field_languages: FieldLanguageMap,
    note: Option<String>,
    #[serde(alias = "ISSN")]
    issn: Option<String>,
}

impl From<SerialDeser> for Serial {
    fn from(raw: SerialDeser) -> Self {
        let contributors = fold_contributors(
            raw.contributors,
            &[(ContributorRole::Editor, raw.editor.as_ref())],
        );
        let editor = collect_contributors_by_role(&contributors, &ContributorRole::Editor);

        Self {
            id: raw.id,
            r#type: raw.r#type,
            title: raw.title,
            short_title: raw.short_title,
            container: raw.container,
            editor,
            contributors,
            publisher: raw.publisher,
            url: raw.url,
            accessed: raw.accessed,
            language: raw.language,
            field_languages: raw.field_languages,
            note: raw.note,
            issn: raw.issn,
        }
    }
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
