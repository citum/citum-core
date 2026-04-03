//! Specialized work types: classic works, patents, datasets, standards, software, and events.

use super::common::{
    FieldLanguageMap, HasNumbering, LangID, NormalizeNumbering, Numbering, RefID, Title,
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

/// Event metadata for conferences, performances, broadcasts, and recordings.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct Event {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Event name (e.g., conference title, performance name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Recurring event series or container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<WorkRelation>,
    /// Event location (city, venue).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Event date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<EdtfString>,
    /// Event genre (e.g., "conference", "performance", "broadcast", "talk").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    /// Broadcaster, network, or streaming platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Performer(s) or presenter(s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performer: Option<Contributor>,
    /// Organizer or sponsor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizer: Option<Contributor>,
    /// URL for the event.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A classic work (Aristotle, Bible, etc.) with standard citation forms.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(from = "ClassicDeser", rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Classic {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Work title (e.g., "Nicomachean Ethics")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// The primary container for this work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<WorkRelation>,
    /// Author (e.g., "Aristotle")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Contributor>,
    /// Editor of this edition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<Contributor>,
    /// Translator of this edition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translator: Option<Contributor>,
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
    /// Numbering identifiers (e.g., volume, section, number, chapter in standard reference system).
    /// Flat shorthand fields are accepted on input for authoring ergonomics and normalized
    /// into canonical `numbering` entries during deserialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
    /// Publication date of this edition (not original)
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// Publisher of this edition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Contributor>,
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
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
struct ClassicDeser {
    id: Option<RefID>,
    title: Option<Title>,
    container: Option<WorkRelation>,
    author: Option<Contributor>,
    editor: Option<Contributor>,
    translator: Option<Contributor>,
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
    keywords: Option<Vec<String>>,
}

impl From<ClassicDeser> for Classic {
    fn from(raw: ClassicDeser) -> Self {
        let mut classic = Self {
            id: raw.id,
            title: raw.title,
            container: raw.container,
            author: raw.author,
            editor: raw.editor,
            translator: raw.translator,
            volume: raw.volume,
            issue: raw.issue,
            edition: raw.edition,
            number: raw.number,
            numbering: raw.numbering,
            issued: raw.issued,
            publisher: raw.publisher,
            url: raw.url,
            accessed: raw.accessed,
            language: raw.language,
            field_languages: raw.field_languages,
            note: raw.note,
            keywords: raw.keywords,
        };
        classic.normalize_numbering();
        classic
    }
}

impl HasNumbering for Classic {
    fn numbering(&self) -> &[Numbering] {
        &self.numbering
    }
}

impl NormalizeNumbering for Classic {
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

/// A patent.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Patent {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Patent title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Inventor(s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Contributor>,
    /// Assignee (patent holder)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<Contributor>,
    /// Patent number (e.g., "U.S. Patent No. 7,347,809")
    pub patent_number: String,
    /// Application number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_number: Option<String>,
    /// Filing date
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filing_date: Option<EdtfString>,
    /// Issue/grant date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// Jurisdiction (e.g., "US", "EP", "JP")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    /// Patent office (e.g., "U.S. Patent and Trademark Office")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// URL for the patent.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

/// A research dataset.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Dataset {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Dataset title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Dataset author(s)/creator(s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Contributor>,
    /// Publication/release date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// Publisher or repository (e.g., "Zenodo", "Dryad")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Contributor>,
    /// Version number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// File format. Prefer IANA media types (e.g., `"text/csv"`) or common
    /// abbreviations (e.g., `"NetCDF"`, `"HDF5"`) where no IANA type exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Dataset size (e.g., "2.4 GB", "150,000 records")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Repository or archive name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI", skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    /// URL for the dataset.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

/// A technical standard or specification.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Standard {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Standard title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Standards organization (e.g., "ISO", "ANSI", "IEEE")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Standard number (e.g., "ISO 8601", "IEEE 754-2008")
    pub standard_number: String,
    /// Publication date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// Publication status. Canonical controlled-vocabulary values: `"published"`, `"draft"`, `"withdrawn"`.
    /// See `docs/policies/ENUM_VOCABULARY_POLICY.md` for matching rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Publisher (usually same as authority)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Contributor>,
    /// URL for the standard.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

/// Software or source code.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Software {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Software title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Author(s)/developer(s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Contributor>,
    /// Release date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// Publisher or repository (e.g., "GitHub", "Zenodo")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Contributor>,
    /// Version number (e.g., "4.1.0", "v2.3.1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// SPDX license identifier preferred (e.g., `"MIT"`, `"GPL-3.0-only"`, `"Apache-2.0"`).
    /// See <https://spdx.org/licenses/> for the authoritative identifier list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Platform (e.g., "Windows", "macOS", "Linux", "cross-platform")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI", skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    /// URL for the software.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the software documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}
