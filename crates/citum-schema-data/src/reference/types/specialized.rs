//! Specialized work types: classic works, patents, datasets, standards, and software.

use super::common::{FieldLanguageMap, LangID, RefID, Title};
use crate::reference::contributor::Contributor;
use crate::reference::date::EdtfString;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
use std::collections::HashMap;
use url::Url;

/// A classic work (Aristotle, Bible, etc.) with standard citation forms.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Classic {
    /// Unique identifier for this reference.
    pub id: Option<RefID>,
    /// Work title (e.g., "Nicomachean Ethics")
    pub title: Option<Title>,
    /// Author (e.g., "Aristotle")
    pub author: Option<Contributor>,
    /// Editor of this edition
    pub editor: Option<Contributor>,
    /// Translator of this edition
    pub translator: Option<Contributor>,
    /// Volume in standard reference system
    pub volume: Option<String>,
    /// Section, book, or chapter in standard reference system
    pub section: Option<String>,
    /// Publication date of this edition (not original)
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// Publisher of this edition
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
    /// Keywords or subject tags.
    pub keywords: Option<Vec<String>>,
}

/// A patent.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Patent {
    /// Unique identifier for this reference.
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
    /// URL for the patent.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the document.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// Keywords or subject tags.
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
    /// File format. Prefer IANA media types (e.g., `"text/csv"`) or common
    /// abbreviations (e.g., `"NetCDF"`, `"HDF5"`) where no IANA type exists.
    pub format: Option<String>,
    /// Dataset size (e.g., "2.4 GB", "150,000 records")
    pub size: Option<String>,
    /// Repository or archive name
    pub repository: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// URL for the dataset.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the dataset.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// Keywords or subject tags.
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
    /// Publication status. Canonical controlled-vocabulary values: `"published"`, `"draft"`, `"withdrawn"`.
    /// See `docs/policies/ENUM_VOCABULARY_POLICY.md` for matching rules.
    pub status: Option<String>,
    /// Publisher (usually same as authority)
    pub publisher: Option<Contributor>,
    /// URL for the standard.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the document.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// Keywords or subject tags.
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
    /// SPDX license identifier preferred (e.g., `"MIT"`, `"GPL-3.0-only"`, `"Apache-2.0"`).
    /// See <https://spdx.org/licenses/> for the authoritative identifier list.
    pub license: Option<String>,
    /// Platform (e.g., "Windows", "macOS", "Linux", "cross-platform")
    pub platform: Option<String>,
    /// DOI identifier.
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// URL for the software.
    #[serde(alias = "URL")]
    pub url: Option<Url>,
    /// Date the URL was accessed.
    #[cfg_attr(feature = "bindings", specta(type = Option<String>))]
    pub accessed: Option<EdtfString>,
    /// BCP 47 language of the software documentation.
    pub language: Option<LangID>,
    /// Per-field language overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_languages: FieldLanguageMap,
    /// Freeform note.
    pub note: Option<String>,
    /// Keywords or subject tags.
    pub keywords: Option<Vec<String>>,
}
