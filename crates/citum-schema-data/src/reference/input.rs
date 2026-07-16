/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Public `InputReference` container and unknown-class payload.

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

#[cfg(feature = "bindings")]
use specta::Type;

use super::classes::ClassExtension;
use super::types::common::FieldLanguageMap;

/// Empty field-language map returned by accessors on unknown-class references.
///
/// `FieldLanguageMap` is a `HashMap`, whose `::new()` is not `const`, so a
/// `LazyLock` is required. The map is constructed once for the process and
/// reused by every unknown-class reference.
pub(crate) static EMPTY_FIELD_LANGUAGES: LazyLock<FieldLanguageMap> =
    LazyLock::new(FieldLanguageMap::new);

const RESERVED_IDENTIFIER_NAMES: &[&str] = &[
    "ads-bibcode",
    "doi",
    "docket-number",
    "eprint-id",
    "isbn",
    "issn",
    "patent-number",
    "pmcid",
    "pmid",
    "report-number",
    "standard-number",
    "url",
];

/// A validated name for a supplementary standardized identifier.
///
/// Names use lowercase kebab-case. Identifiers with dedicated Citum fields,
/// such as `doi` and `isbn`, are reserved and cannot be duplicated here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(transparent)]
pub struct IdentifierName(String);

impl IdentifierName {
    /// Validate and construct a supplementary identifier name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not lowercase kebab-case, does not
    /// begin with a letter, or is reserved for a first-class reference field.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.split('-').all(|segment| {
                !segment.is_empty()
                    && segment
                        .chars()
                        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
            })
            && value
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_lowercase());
        if !valid {
            return Err(
                "identifier name must be lowercase kebab-case and begin with a letter".to_string(),
            );
        }
        if RESERVED_IDENTIFIER_NAMES.contains(&value.as_str()) {
            return Err(format!(
                "identifier name `{value}` is reserved for a first-class reference field"
            ));
        }
        Ok(Self(value))
    }

    /// Return the validated wire-format name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for IdentifierName {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Borrow<str> for IdentifierName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'de> Deserialize<'de> for IdentifierName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Extensible standardized identifiers without dedicated Citum fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(transparent)]
pub struct SupplementaryIdentifiers(BTreeMap<IdentifierName, String>);

impl SupplementaryIdentifiers {
    /// Construct an empty supplementary identifier map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the value associated with a validated identifier name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }

    /// Insert a supplementary identifier value.
    pub fn insert(&mut self, name: IdentifierName, value: impl Into<String>) -> Option<String> {
        self.0.insert(name, value.into())
    }

    /// Return whether the map has no identifiers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The Reference model: a class-specific overlay reachable through accessor methods.
///
/// All shared bibliographic data (id, title, contributors, dates, publisher, ...)
/// lives inside the class-specific payload in `extension`. The accessor methods
/// (`id()`, `title()`, etc.) dispatch through the extension and are the public
/// read path; the typed setters (`set_id`, ...) are the public mutation path.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct InputReference {
    pub(crate) extension: ClassExtension,
    pub(crate) identifiers: SupplementaryIdentifiers,
}

/// Unknown reference-class payload captured by the discriminator dispatcher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct UnknownClassData {
    /// Raw `class:` string from the input object.
    pub class: String,
    /// Non-shared fields captured verbatim for round-trip preservation.
    #[cfg_attr(feature = "bindings", specta(type = serde_json::Value))]
    pub fields: JsonMap<String, JsonValue>,
}
