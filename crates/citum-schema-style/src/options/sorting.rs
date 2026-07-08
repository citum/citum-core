/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Bibliography sorting policy options.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Style-level or bibliography-local bibliography sorting policy.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct SortingConfig {
    /// Collator locale used for text comparisons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<SortingLocale>,
    /// Multilingual sort-key mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multilingual: Option<SortingMultilingualMode>,
    /// Forward-compatibility storage for unknown sorting policy keys.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub unknown_fields: std::collections::BTreeMap<String, serde_yaml::Value>,
}

impl SortingConfig {
    /// Merge another sorting config into this one, with `other` taking precedence.
    pub fn merge(&mut self, other: &SortingConfig) {
        if let Some(locale) = &other.locale {
            self.locale = Some(locale.clone());
        }
        if let Some(multilingual) = other.multilingual {
            self.multilingual = Some(multilingual);
        }
        for (key, value) in &other.unknown_fields {
            self.unknown_fields.insert(key.clone(), value.clone());
        }
    }

    /// Resolve the configured locale policy, defaulting to `auto`.
    #[must_use]
    pub fn effective_locale(&self) -> SortingLocale {
        self.locale.clone().unwrap_or_default()
    }

    /// Resolve the configured multilingual mode, defaulting to `uniform`.
    #[must_use]
    pub fn effective_multilingual(&self) -> SortingMultilingualMode {
        self.multilingual.unwrap_or_default()
    }
}

/// Collator locale policy for bibliography sorting.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum SortingLocale {
    /// Use the effective bibliography locale for collation.
    #[default]
    Auto,
    /// Use an explicit BCP 47 locale tag for collation.
    Bcp47(String),
}

impl SortingLocale {
    /// Borrow the explicit locale tag, or return `None` for `auto`.
    #[must_use]
    pub fn as_explicit_tag(&self) -> Option<&str> {
        match self {
            Self::Auto => None,
            Self::Bcp47(tag) => Some(tag.as_str()),
        }
    }
}

impl Serialize for SortingLocale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Bcp47(tag) => serializer.serialize_str(tag),
        }
    }
}

impl<'de> Deserialize<'de> for SortingLocale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SortingLocaleVisitor;

        impl de::Visitor<'_> for SortingLocaleVisitor {
            type Value = SortingLocale;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("\"auto\" or a BCP 47 locale tag")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(E::custom("sorting.locale must not be empty"));
                }
                if trimmed == "auto" {
                    Ok(SortingLocale::Auto)
                } else {
                    Ok(SortingLocale::Bcp47(trimmed.to_string()))
                }
            }
        }

        deserializer.deserialize_str(SortingLocaleVisitor)
    }
}

#[cfg(feature = "schema")]
impl JsonSchema for SortingLocale {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SortingLocale".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "description": "\"auto\" or an explicit BCP 47 locale tag used for collation."
        })
    }
}

/// Multilingual bibliography sort-key mode.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum SortingMultilingualMode {
    /// Use the existing single-collator sort key behavior.
    #[default]
    Uniform,
    /// Prefer hidden romanized sort keys when supplied by reference data.
    Romanized,
    /// Expand to script partitioning when no explicit partitioning is configured.
    PerScript,
}
