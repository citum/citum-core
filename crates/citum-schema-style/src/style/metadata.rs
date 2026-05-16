/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Style metadata and classification types.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(unused_imports, reason = "Referenced by intra-doc links.")]
use crate::ResolutionError;

/// Discipline/field classification for a citation style.
///
/// Values correspond to the CSL 1.0 `<category field="..."/>` attribute,
/// `generic-base` is silently ignored during migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum CitationField {
    /// Anthropology styles.
    Anthropology,
    /// Biology styles.
    Biology,
    /// Botany styles.
    Botany,
    /// Chemistry styles.
    Chemistry,
    /// Communications studies styles.
    Communications,
    /// Engineering styles.
    Engineering,
    /// Geography styles.
    Geography,
    /// Geology styles.
    Geology,
    /// History styles.
    History,
    /// Humanities styles.
    Humanities,
    /// Law styles.
    Law,
    /// Linguistics styles.
    Linguistics,
    /// Literature styles.
    Literature,
    /// Mathematics styles.
    Math,
    /// Medicine styles.
    Medicine,
    /// Philosophy styles.
    Philosophy,
    /// Physics styles.
    Physics,
    #[serde(rename = "political-science")]
    /// Political science styles.
    PoliticalScience,
    /// Psychology styles.
    Psychology,
    /// General science styles.
    Science,
    #[serde(rename = "social-science")]
    /// Social science styles.
    SocialScience,
    /// Sociology styles.
    Sociology,
    /// Theology styles.
    Theology,
    /// Zoology styles.
    Zoology,
}

/// A hyperlink associated with a style (documentation, self-link, etc.).
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StyleLink {
    /// Link target for related style metadata.
    pub href: String,
    /// Relationship type for the link, such as `self` or `documentation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
}

/// A person credit (author or contributor) for a style.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StylePerson {
    /// Display name for the credited person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Contact email for the credited person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// URI identifying the credited person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Provenance block for styles adapted from a CSL 1.0 source.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StyleSource {
    /// The original CSL style ID (URI).
    pub csl_id: String,
    /// Who performed the adaptation (e.g., "citum-migrate" or a person's name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapted_by: Option<String>,
    /// License URI (e.g., CC BY-SA).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Original CSL style authors.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub original_authors: Vec<StylePerson>,
    /// Links from the original CSL style (documentation, self, etc.).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<StyleLink>,
}

/// Style metadata.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StyleInfo {
    /// Human-readable title of the style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Stable identifier for the style, usually a URI or slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Short summary of the style's intended use or provenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Default locale for the style (e.g., "en-US", "de-DE").
    /// Used for locale-aware term resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_locale: Option<String>,
    /// Discipline classifications for this style.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<CitationField>,
    /// Provenance: set when this style was adapted from a CSL 1.0 source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<StyleSource>,
    /// Concise display name for the style family, used by UIs to label
    /// search results and match banners (e.g. `"APA"`, `"Chicago Notes"`,
    /// `"MLA"`). Omit for journal-specific styles whose full title is their
    /// identity. Combine with `edition` to produce labels like `"APA 7th"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Edition or version qualifier used alongside `short_name` to
    /// disambiguate multiple editions of the same style family
    /// (e.g. `"7th"`, `"18th edition"`). Omit when only one edition exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Minimum Citum engine version required to load and render this style.
    ///
    /// Accepts a [`semver::VersionReq`]-compatible string (e.g. `">=0.38.0"`,
    /// `"^0.40"`). When the running engine does not satisfy the requirement,
    /// the resolver returns
    /// [`ResolutionError::VersionMismatch`]
    /// instead of attempting to deserialize fields the engine may not
    /// understand. Omit for styles that use only stable, long-lived features.
    #[serde(rename = "citum-version", skip_serializing_if = "Option::is_none")]
    pub citum_version: Option<String>,
}

impl StyleInfo {
    /// Returns `true` when all fields are absent (no content to merge).
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.id.is_none()
            && self.description.is_none()
            && self.default_locale.is_none()
            && self.fields.is_empty()
            && self.source.is_none()
            && self.short_name.is_none()
            && self.edition.is_none()
            && self.citum_version.is_none()
    }
}
