/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw locale format for YAML parsing.
/// This is a simpler format that uses string keys for terms.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct RawLocale {
    /// The locale identifier (e.g., "en-US", "de-DE").
    pub locale: String,
    /// Date-related terms.
    #[serde(default)]
    pub dates: RawDateTerms,
    /// Role terms keyed by role name.
    #[serde(default)]
    pub roles: HashMap<String, RawRoleTerm>,
    /// General terms keyed by term name.
    #[serde(default)]
    pub terms: HashMap<String, RawTermValue>,
    /// Schema version. Absent or "1" uses the legacy term-map path.
    /// "2" activates the new messages/dateFormats/grammarOptions path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale_schema_version: Option<String>,
    /// Runtime evaluation options (message syntax, evaluator hints).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation: Option<crate::locale::types::EvaluationConfig>,
    /// ICU Message Format 1 messages keyed by message ID (v2 locales only).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub messages: HashMap<String, String>,
    /// Named date format presets: symbolic name → CLDR date pattern (v2 locales only).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub date_formats: HashMap<String, String>,
    /// Locale-level number formatting options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_formats: Option<crate::locale::types::NumberFormats>,
    /// Grammar toggles that vary by language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grammar_options: Option<crate::locale::types::GrammarOptions>,
    /// Backwards-compatibility aliases: old CSL term key → new message ID.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub legacy_term_aliases: HashMap<String, String>,
}

/// Raw date terms for YAML parsing.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct RawDateTerms {
    /// Localized month names.
    #[serde(default)]
    pub months: RawMonthNames,
    /// Localized season names in display order.
    #[serde(default)]
    pub seasons: Vec<String>,
    /// Localized term for uncertain dates.
    #[serde(default)]
    pub uncertainty_term: Option<String>,
    /// Localized term for open-ended date ranges.
    #[serde(default)]
    pub open_ended_term: Option<String>,
    /// Localized ante meridiem marker.
    #[serde(default)]
    pub am: Option<String>,
    /// Localized post meridiem marker.
    #[serde(default)]
    pub pm: Option<String>,
    /// Localized label for UTC.
    #[serde(default)]
    pub timezone_utc: Option<String>,
    /// Localized era suffix for year zero and negative years.
    #[serde(default)]
    pub before_era: Option<String>,
    /// Localized era suffix for positive years in BC/AD profile (e.g., "AD").
    #[serde(default)]
    pub ad: Option<String>,
    /// Localized era suffix for negative years in BC/AD profile (e.g., "BC").
    #[serde(default)]
    pub bc: Option<String>,
    /// Localized era suffix for negative years in BCE/CE profile (e.g., "BCE").
    #[serde(default)]
    pub bce: Option<String>,
    /// Localized era suffix for positive years in BCE/CE profile (e.g., "CE").
    #[serde(default)]
    pub ce: Option<String>,
}

/// Raw month names for YAML parsing.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RawMonthNames {
    /// Full month names.
    #[serde(default)]
    pub long: Vec<String>,
    /// Abbreviated month names.
    #[serde(default)]
    pub short: Vec<String>,
}

/// Raw role term with form-keyed values.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RawRoleTerm {
    /// Long-form role term.
    #[serde(default)]
    pub long: Option<RawTermValue>,
    /// Short-form role term.
    #[serde(default)]
    pub short: Option<RawTermValue>,
    /// Verb-form role term.
    #[serde(default)]
    pub verb: Option<RawTermValue>,
    /// Short verb-form role term.
    #[serde(default, rename = "verb-short")]
    pub verb_short: Option<RawTermValue>,
}

/// A term value that can be a simple string or have singular/plural forms.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum RawTermValue {
    /// Simple string value.
    Simple(String),
    /// Form-keyed value (for terms with long/short forms).
    Forms(HashMap<String, RawTermValue>),
    /// Singular/plural forms.
    SingularPlural {
        /// Singular form of the term.
        singular: String,
        /// Plural form of the term.
        plural: String,
    },
}

impl Default for RawTermValue {
    fn default() -> Self {
        RawTermValue::Simple(String::new())
    }
}

impl RawTermValue {
    /// Get the simple string value.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            RawTermValue::Simple(s) => Some(s),
            _ => None,
        }
    }
}

/// Raw locale override format for YAML parsing.
///
/// Mirrors [`super::types::LocaleOverride`] for deserialization from style-level
/// locale override files.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", default)]
pub struct RawLocaleOverride {
    /// Message IDs to replace in the base locale.
    pub messages: HashMap<String, String>,
    /// If present, replaces the entire grammar-options block.
    pub grammar_options: Option<crate::locale::types::GrammarOptions>,
    /// Additional or replacement legacy term aliases.
    pub legacy_term_aliases: HashMap<String, String>,
}

impl From<RawLocaleOverride> for super::types::LocaleOverride {
    fn from(raw: RawLocaleOverride) -> Self {
        super::types::LocaleOverride {
            messages: raw.messages,
            grammar_options: raw.grammar_options,
            legacy_term_aliases: raw.legacy_term_aliases,
        }
    }
}
