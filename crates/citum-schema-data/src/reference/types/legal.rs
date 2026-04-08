//! Legal document types: cases, statutes, treaties, hearings, regulations, and briefs.

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

/// A legal case (court decision).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct LegalCase {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Case name (e.g., "Brown v. Board of Education")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Court or authority (e.g., "U.S. Supreme Court")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Reporter volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Reporter abbreviation (e.g., "U.S.", "F.2d")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporter: Option<String>,
    /// First page of case in reporter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    /// Creation or origination date of the legal work.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Decision date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// URL for the case.
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
    /// DOI identifier.
    #[serde(alias = "DOI", skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    /// Keywords or subject tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

/// A statute or legislative act.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Statute {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Statute name (e.g., "Civil Rights Act of 1964")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Legislative body (e.g., "U.S. Congress")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Code volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Code abbreviation (e.g., "U.S.C.", "Pub. L.")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Statute or public-law number when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Page or entry locator for session laws and registers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    /// Creation or origination date of the statute.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Section or page number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Optional chapter or session identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_number: Option<String>,
    /// Enactment or publication date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// URL for the statute.
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

/// An international treaty or agreement.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Treaty {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Treaty name (e.g., "Treaty of Versailles")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Parties to the treaty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Contributor>,
    /// Treaty series volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Treaty series abbreviation (e.g., "U.N.T.S.")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporter: Option<String>,
    /// Page or treaty number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    /// Creation or origination date of the treaty text.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Signing or ratification date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// URL for the treaty.
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

/// A legislative or administrative hearing.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Hearing {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Hearing title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Legislative body conducting the hearing (e.g., "U.S. Senate Committee on Finance")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Session or congress number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_number: Option<String>,
    /// Creation or origination date of the hearing record.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Hearing date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// URL for the hearing record.
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

/// An administrative regulation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Regulation {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Regulation title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Regulatory authority (e.g., "EPA", "Federal Register")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Code volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Creation or origination date of the regulation text.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Code abbreviation (e.g., "C.F.R.", "Fed. Reg.")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Section or page number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Publication or effective date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// URL for the regulation.
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

/// A legal brief or filing.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Brief {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Brief title or case name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Court (e.g., "U.S. Supreme Court")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Author/filer of the brief
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Contributor>,
    /// Docket number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docket_number: Option<String>,
    /// Creation or origination date of the brief.
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub created: EdtfString,
    /// Filing date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    #[serde(default, skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// URL for the brief.
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
