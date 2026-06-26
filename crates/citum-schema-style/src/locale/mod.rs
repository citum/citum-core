/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Locale definitions for Citum.
//!
//! Locales provide language-specific terms, date formats, and punctuation rules
//! for citation formatting.

mod date_patterns;
mod embedded;
/// Locator text normalization.
pub mod locator;
/// Message evaluation for parameterized locale strings.
pub mod message;
mod message_ids;
/// Raw locale types used during locale file parsing.
pub mod raw;
mod raw_conversion;
mod sort;
mod terms;
/// Structured locale types used by the processor.
pub mod types;
mod vocab;

use crate::citation::LocatorType;
use crate::template::ContributorRole;
pub use message::{MessageArgs, MessageEvaluator, Mf2MessageEvaluator};
pub use raw::{RawLocale, RawTermValue};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
pub use terms::ArchiveHierarchyField;
pub use types::*;

/// A list of month names (12 elements for Jan-Dec).
pub type MonthList = Vec<String>;

/// A locale definition containing language-specific terms and formatting rules.
///
/// The `evaluator` field holds the message evaluation engine, selected based on
/// `evaluation.message_syntax`. This allows for trait-based swapping to ICU4X
/// implementations in the future without changing call sites.
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Locale {
    /// The locale identifier (e.g., "en-US", "de-DE").
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub locale: String,
    /// Date-related terms (months, seasons).
    #[serde(default)]
    pub dates: DateTerms,
    /// Contributor role terms (editor, translator, etc.).
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub roles: HashMap<ContributorRole, ContributorTerm>,
    /// Locator terms (page, chapter, etc.).
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub locators: HashMap<LocatorType, LocatorTerm>,
    /// General terms (and, et al., etc.).
    #[serde(default)]
    pub terms: Terms,
    /// Whether to place periods/commas inside quotation marks.
    /// true = American style ("text."), false = British style ("text".)
    #[serde(default)]
    pub punctuation_in_quote: bool,
    /// Articles to strip from titles when sorting (e.g., "the", "a", "an" for English).
    /// These should be lowercase and will be matched case-insensitively.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sort_articles: Vec<String>,
    /// Schema version from the source locale file (None = legacy v1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale_schema_version: Option<String>,
    /// Runtime evaluation configuration.
    #[serde(default)]
    pub evaluation: EvaluationConfig,
    /// ICU MF1 messages keyed by message ID (populated for v2 locales).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub messages: HashMap<String, String>,
    /// Named date format presets: symbolic name → CLDR pattern.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub date_formats: HashMap<String, String>,
    /// Number formatting options.
    #[serde(default)]
    pub number_formats: NumberFormats,
    /// Grammar options.
    #[serde(default)]
    pub grammar_options: GrammarOptions,
    /// Backwards-compatibility aliases: old term key → new message ID.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub legacy_term_aliases: HashMap<String, String>,
    /// Vocabulary maps for genre and medium display text.
    #[serde(default, skip_serializing_if = "VocabMap::is_empty")]
    pub vocab: VocabMap,
    /// Message evaluator implementation (not serialized; set during load).
    #[serde(skip, default = "default_evaluator")]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub evaluator: Arc<dyn MessageEvaluator>,
}

/// Default message evaluator (MF2).
fn default_evaluator() -> Arc<dyn MessageEvaluator> {
    Arc::new(Mf2MessageEvaluator)
}

impl Default for Locale {
    fn default() -> Self {
        Self {
            locale: String::default(),
            dates: DateTerms::default(),
            roles: HashMap::default(),
            locators: HashMap::default(),
            terms: Terms::default(),
            punctuation_in_quote: false,
            sort_articles: Vec::default(),
            locale_schema_version: None,
            evaluation: EvaluationConfig::default(),
            messages: HashMap::default(),
            date_formats: HashMap::default(),
            number_formats: NumberFormats::default(),
            grammar_options: GrammarOptions::default(),
            legacy_term_aliases: HashMap::default(),
            vocab: VocabMap::default(),
            evaluator: default_evaluator(),
        }
    }
}

impl fmt::Debug for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Locale")
            .field("locale", &self.locale)
            .field("dates", &self.dates)
            .field("roles", &self.roles)
            .field("locators", &self.locators)
            .field("terms", &self.terms)
            .field("punctuation_in_quote", &self.punctuation_in_quote)
            .field("sort_articles", &self.sort_articles)
            .field("locale_schema_version", &self.locale_schema_version)
            .field("evaluation", &self.evaluation)
            .field("messages", &self.messages)
            .field("date_formats", &self.date_formats)
            .field("number_formats", &self.number_formats)
            .field("grammar_options", &self.grammar_options)
            .field("legacy_term_aliases", &self.legacy_term_aliases)
            .field("vocab", &self.vocab)
            .field("evaluator", &"<MessageEvaluator>")
            .finish()
    }
}

impl Locale {
    /// Create a new English (US) locale with default terms.
    pub fn en_us() -> Self {
        Self {
            locale: "en-US".into(),
            dates: DateTerms::en_us(),
            roles: embedded::en_us_role_terms(),
            locators: embedded::en_us_locator_terms(),
            terms: Terms::en_us(),
            punctuation_in_quote: true,
            sort_articles: vec!["the".into(), "a".into(), "an".into()],
            locale_schema_version: None,
            evaluation: EvaluationConfig {
                message_syntax: MessageSyntax::Mf2,
            },
            messages: embedded::en_us_archive_messages(),
            date_formats: HashMap::new(),
            number_formats: NumberFormats {
                decimal_separator: ".".into(),
                thousands_separator: ",".into(),
                minimum_digits: 1,
            },
            grammar_options: GrammarOptions {
                punctuation_in_quote: true,
                nbsp_before_colon: false,
                open_quote: "\u{201C}".into(),
                close_quote: "\u{201D}".into(),
                open_inner_quote: "\u{2018}".into(),
                close_inner_quote: "\u{2019}".into(),
                serial_comma: true,
                page_range_delimiter: "\u{2013}".into(),
            },
            legacy_term_aliases: HashMap::new(),
            vocab: embedded::embedded_en_us_vocab().clone(),
            evaluator: Arc::new(Mf2MessageEvaluator),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;

    #[test]
    fn test_en_us_locale_model_defaults() {
        let locale = Locale::en_us();
        assert_eq!(locale.locale, "en-US");
        assert!(locale.punctuation_in_quote);
        assert_eq!(locale.sort_articles, ["the", "a", "an"]);
        assert!(locale.roles.contains_key(&ContributorRole::Editor));
        assert!(locale.locators.contains_key(&LocatorType::Page));
    }

    #[test]
    fn test_locale_deserialization() {
        let json = r#"{
            "locale": "en-US",
            "dates": {
                "months": {
                    "long": ["January", "February", "March", "April", "May", "June",
                             "July", "August", "September", "October", "November", "December"],
                    "short": ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                              "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
                },
                "seasons": ["Spring", "Summer", "Autumn", "Winter"]
            },
            "roles": {},
            "terms": {
                "and": "and",
                "et-al": "et al."
            }
        }"#;

        let locale: Locale = serde_json::from_str(json).unwrap();
        assert_eq!(locale.locale, "en-US");
        assert_eq!(locale.dates.months.long[0], "January");
        assert_eq!(locale.terms.and.as_ref().unwrap(), "and");
    }

    #[test]
    fn test_yaml_locale_loading() {
        let yaml = r#"
locale: de-DE
dates:
  months:
    long:
      - Januar
      - Februar
      - März
      - April
      - Mai
      - Juni
      - Juli
      - August
      - September
      - Oktober
      - November
      - Dezember
    short:
      - Jan.
      - Feb.
      - März
      - Apr.
      - Mai
      - Juni
      - Juli
      - Aug.
      - Sep.
      - Okt.
      - Nov.
      - Dez.
  seasons:
    - Frühling
    - Sommer
    - Herbst
    - Winter
terms:
  and:
    long: und
    symbol: "&"
  et_al:
    long: "u. a."
"#;

        let locale = Locale::from_yaml_str(yaml).unwrap();
        assert_eq!(locale.locale, "de-DE");
        assert_eq!(locale.terms.and.as_deref(), Some("und"));
        assert_eq!(locale.terms.et_al.as_deref(), Some("u. a."));
        assert_eq!(locale.dates.months.long[0], "Januar");
        assert_eq!(locale.dates.months.long[2], "März");
    }

    /// v2 locale with grammar-options overrides punctuation_in_quote correctly.
    #[test]
    fn test_v2_grammar_options_sync_punctuation_in_quote() {
        let yaml = r#"
locale-schema-version: "2"
locale: en-GB
grammar-options:
  punctuation-in-quote: false
"#;
        let locale = Locale::from_yaml_str(yaml).unwrap();
        // grammar_options is the authoritative source for v2 locales
        assert!(!locale.grammar_options.punctuation_in_quote);
        // legacy field is synced from grammar_options
        assert!(!locale.punctuation_in_quote);
    }

    /// v1 locale (no grammar-options) derives punctuation_in_quote from locale ID.
    #[test]
    fn test_v1_locale_derives_punctuation_from_locale_id() {
        let yaml = r#"
locale: en-US
"#;
        let locale = Locale::from_yaml_str(yaml).unwrap();
        // en-US uses American style (inside)
        assert!(locale.punctuation_in_quote);
        assert!(locale.grammar_options.punctuation_in_quote);
    }

    /// apply_override merges messages key-by-key into the base locale.
    #[test]
    fn test_apply_override_merges_messages() {
        let mut locale = Locale::en_us();
        locale
            .messages
            .insert("term.page-label".into(), "p.".into());
        let ov = LocaleOverride {
            messages: [("term.page-label".into(), "pg.".into())].into(),
            ..Default::default()
        };
        locale.apply_override(&ov);
        assert_eq!(
            locale.messages.get("term.page-label").map(|s| s.as_str()),
            Some("pg.")
        );
    }

    /// The hardcoded en-US locale includes phrase messages used by style
    /// `message:` components, not only legacy term compatibility messages.
    #[test]
    fn test_en_us_locale_resolves_phrase_messages() {
        let locale = Locale::en_us();
        let args = MessageArgs {
            named: [("container".to_string(), "Book Title".to_string())].into(),
            ..Default::default()
        };

        assert_eq!(
            locale.resolve_message("pattern.in-container", &args),
            Some("in Book Title".to_string())
        );
    }

    /// apply_override with grammar_options replaces block and syncs punctuation_in_quote.
    #[test]
    fn test_apply_override_grammar_options_syncs_punctuation() {
        let mut locale = Locale::en_us();
        locale.punctuation_in_quote = false;
        let ov = LocaleOverride {
            grammar_options: Some(GrammarOptions {
                punctuation_in_quote: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        locale.apply_override(&ov);
        assert!(locale.punctuation_in_quote);
        assert!(locale.grammar_options.punctuation_in_quote);
    }

    #[test]
    fn embedded_locale_ids_include_all_bundled_locale_files() {
        for id in [
            "en-US", "ar-AR", "de-DE", "es-ES", "eu-ES", "fr-FR", "tr-TR",
        ] {
            assert!(
                crate::embedded::EMBEDDED_LOCALE_IDS.contains(&id),
                "{id} should be listed as an embedded locale"
            );
        }
    }

    #[test]
    fn bundled_ar_ar_and_eu_es_locales_are_embedded_and_parseable() {
        for id in ["ar-AR", "eu-ES"] {
            let bytes = crate::embedded::get_locale_bytes(id).expect("locale should be embedded");
            let yaml = std::str::from_utf8(bytes).expect("embedded locale should be utf-8");
            let locale = Locale::from_yaml_str(yaml).expect("embedded locale should parse");

            assert_eq!(locale.locale, id);
        }
    }
}
