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
    /// URL for the case.
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
    /// DOI identifier.
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    /// Keywords or subject tags.
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
    pub id: Option<RefID>,
    /// Statute name (e.g., "Civil Rights Act of 1964")
    pub title: Option<Title>,
    /// Legislative body (e.g., "U.S. Congress")
    pub authority: Option<String>,
    /// Code volume
    pub volume: Option<String>,
    /// Code abbreviation (e.g., "U.S.C.", "Pub. L.")
    pub code: Option<String>,
    /// Statute or public-law number when present.
    pub number: Option<String>,
    /// Page or entry locator for session laws and registers.
    pub page: Option<String>,
    /// Section or page number
    pub section: Option<String>,
    /// Optional chapter or session identifier.
    pub chapter_number: Option<String>,
    /// Enactment or publication date
    #[cfg_attr(feature = "bindings", specta(type = String))]
    pub issued: EdtfString,
    /// URL for the statute.
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

/// An international treaty or agreement.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Treaty {
    /// Unique identifier for this reference.
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
    /// URL for the treaty.
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

/// A legislative or administrative hearing.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Hearing {
    /// Unique identifier for this reference.
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
    /// URL for the hearing record.
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

/// An administrative regulation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Regulation {
    /// Unique identifier for this reference.
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
    /// URL for the regulation.
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

/// A legal brief or filing.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
// deny_unknown_fields removed: incompatible with #[serde(tag)] on InputReference (serde limitation - tag field is replayed into inner struct)
pub struct Brief {
    /// Unique identifier for this reference.
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
    /// URL for the brief.
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
