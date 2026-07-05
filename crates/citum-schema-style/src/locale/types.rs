/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Locale-specific term types and definitions.
//!
//! This module defines the data structures for representing locale information including
//! general terms (prepositions, conjunctions), contributor role terms, locator terms for
//! pages and chapters, date-related terms, and month names. These are used by citation
//! processors to render localized output.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

crate::str_enum! {
    /// Form for term lookup.
    ///
    /// Specifies which form variant of a term should be used in citation output.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum TermForm {
        /// Long form of a term (e.g., "page" vs "p.").
        Long = "long",
        /// Short form of a term (e.g., "p." vs "page").
        Short = "short",
        /// Verb form of a term (e.g., "edited by").
        Verb = "verb",
        /// Short verb form of a term (e.g., "ed." vs "edited by").
        VerbShort = "verb-short",
        /// Symbol form of a term (e.g., "§" for section).
        Symbol = "symbol"
    }
}

crate::str_enum! {
    /// Grammatical gender used for locale agreement and term selection.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum GrammaticalGender {
        /// Masculine grammatical gender.
        Masculine = "masculine",
        /// Feminine grammatical gender.
        Feminine = "feminine",
        /// Neuter grammatical gender.
        Neuter = "neuter",
        /// Common or shared grammatical gender.
        Common = "common"
    }
}

/// A value that may vary by grammatical gender.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum MaybeGendered<T> {
    /// The value is the same for all genders.
    Plain(T),
    /// The value varies by grammatical gender.
    Gendered {
        /// Masculine variant.
        #[serde(skip_serializing_if = "Option::is_none")]
        masculine: Option<T>,
        /// Feminine variant.
        #[serde(skip_serializing_if = "Option::is_none")]
        feminine: Option<T>,
        /// Neuter variant.
        #[serde(skip_serializing_if = "Option::is_none")]
        neuter: Option<T>,
        /// Common or gender-unspecified variant.
        #[serde(skip_serializing_if = "Option::is_none")]
        common: Option<T>,
    },
}

impl<T: Default> Default for MaybeGendered<T> {
    fn default() -> Self {
        Self::Plain(T::default())
    }
}

impl<T> From<T> for MaybeGendered<T> {
    fn from(value: T) -> Self {
        Self::Plain(value)
    }
}

impl From<&str> for MaybeGendered<String> {
    fn from(value: &str) -> Self {
        Self::Plain(value.to_string())
    }
}

impl<T> MaybeGendered<T> {
    fn by_gender(&self, requested: GrammaticalGender) -> Option<&T> {
        match self {
            Self::Plain(value) => Some(value),
            Self::Gendered {
                masculine,
                feminine,
                neuter,
                common,
            } => match requested {
                GrammaticalGender::Masculine => masculine.as_ref(),
                GrammaticalGender::Feminine => feminine.as_ref(),
                GrammaticalGender::Neuter => neuter.as_ref(),
                GrammaticalGender::Common => common.as_ref(),
                _ => None,
            },
        }
    }

    /// Resolve only the explicitly requested slot.
    pub fn resolve_strict(&self, requested: Option<GrammaticalGender>) -> Option<&T> {
        match self {
            Self::Plain(value) => Some(value),
            Self::Gendered { .. } => requested.and_then(|gender| self.by_gender(gender)),
        }
    }

    /// Resolve a value using the documented production fallback order.
    pub fn resolve_with_fallback(&self, requested: Option<GrammaticalGender>) -> Option<&T> {
        match self {
            Self::Plain(value) => Some(value),
            Self::Gendered {
                masculine,
                feminine,
                neuter,
                common,
            } => requested
                .and_then(|gender| self.by_gender(gender))
                .or(common.as_ref())
                .or(masculine.as_ref())
                .or(feminine.as_ref())
                .or(neuter.as_ref()),
        }
    }

    /// Resolve to a neutral/default form without selecting gendered slots.
    pub fn resolve_neutral(&self) -> Option<&T> {
        match self {
            Self::Plain(value) => Some(value),
            Self::Gendered { common, .. } => common.as_ref(),
        }
    }
}

impl MaybeGendered<String> {
    /// Resolve to the default production string.
    pub fn as_default_str(&self) -> &str {
        self.resolve_with_fallback(None)
            .map(String::as_str)
            .unwrap_or("")
    }

    /// Whether the default resolved value is empty.
    pub fn is_empty(&self) -> bool {
        self.as_default_str().is_empty()
    }

    /// Resolve to a borrowed string using the default production path.
    pub fn as_str(&self) -> &str {
        self.as_default_str()
    }

    /// Lowercase the default resolved value.
    pub fn to_lowercase(&self) -> String {
        self.as_default_str().to_lowercase()
    }
}

crate::str_enum! {
    /// A list of general terms for citation formatting.
    ///
    /// These are the standard terms that appear in bibliographies and citations,
    /// including prepositions (in, at, from, by), punctuation terms (and, et al),
    /// date-related terms (accessed, no-date, circa), locator terms (page, chapter, volume),
    /// and special phrases (ibid, forthcoming, available-at).
    #[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
    pub enum GeneralTerm {
        #[default]
        /// The preposition "in" (e.g., "in Smith, 2020").
        In = "in",
        /// The term used for access dates (e.g., "accessed May 1").
        Accessed = "accessed",
        /// The term used to introduce citation access dates (e.g., "cited May 1").
        Cited = "cited",
        /// The term used for retrieval statements (e.g., "retrieved from URL").
        Retrieved = "retrieved",
        /// The preposition "at" (e.g., "at the conference").
        At = "at",
        /// The preposition "from" (e.g., "from the publisher").
        From = "from",
        /// The preposition "of" (e.g., "special issue of").
        Of = "of",
        /// The preposition "to" (e.g., "from x to y").
        To = "to",
        /// The preposition "by" (e.g., "by John Smith").
        By = "by",
        /// The term used when no date is available (e.g., "n.d.").
        NoDate = "no-date",
        /// The term used for anonymous authorship (e.g., "anonymous").
        Anonymous = "anonymous",
        /// The term used for approximate dates (e.g., "circa").
        Circa = "circa",
        /// The phrase used for availability statements (e.g., "available at URL").
        AvailableAt = "available-at",
        /// The term used for immediately repeated citations (e.g., "ibid.").
        Ibid = "ibid",
        /// The conjunction "and" (e.g., "Smith and Jones").
        And = "and",
        /// The abbreviation for omitted additional names (e.g., "et al.").
        EtAl = "et-al",
        /// The phrase "and others" (generic use).
        AndOthers = "and-others",
        /// The term used for forthcoming works (e.g., "forthcoming").
        Forthcoming = "forthcoming",
        /// The term used for online resources (e.g., "online").
        Online = "online",
        /// The adverb "here".
        Here = "here",
        /// The term used for deposited materials.
        Deposited = "deposited",
        /// The phrase used to introduce reviewed works (e.g., "review of").
        ReviewOf = "review-of",
        /// The phrase used for original publication references (e.g., "originally published").
        OriginalWorkPublished = "original-work-published",
        /// The term used for patents (e.g., "patent").
        Patent = "patent",
        /// The term used for "issued" in patent entries (e.g., ", issued June 9, 2010").
        Issued = "issued",
        /// The general term for volume locators (e.g., "volume", "vol.").
        Volume = "volume",
        /// The general term for issue locators (e.g., "issue", "no.").
        Issue = "issue",
        /// The general term for page locators (e.g., "page", "p.", "pp.").
        Page = "page",
        /// The general term for chapter locators (e.g., "chapter", "ch.").
        Chapter = "chapter",
        /// The general term for editions (e.g., "edition", "ed.").
        Edition = "edition",
        /// The general term for section locators (e.g., "section", "§").
        Section = "section",
        /// The label for personal communications (e.g., "personal communication").
        PersonalCommunication = "personal-communication",
        /// The general term for a version/release label (e.g., "version" in "Version 2.1").
        Version = "version"
    }
}

/// General terms used in citations and bibliographies.
///
/// Contains prepositions, conjunctions, and common phrases that appear in citation output.
/// Includes both simple string terms and `SimpleTerm` fields with long/short variants.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Terms {
    /// The word "and" (e.g., "Smith and Jones").
    pub and: Option<String>,
    /// Symbol form of "and" (e.g., "&").
    pub and_symbol: Option<String>,
    /// "and others" phrase for generic use.
    pub and_others: Option<String>,
    /// Anonymous author term (has long and short forms).
    #[serde(default)]
    pub anonymous: SimpleTerm,
    /// "at" preposition.
    pub at: Option<String>,
    /// "accessed" term for URLs.
    pub accessed: Option<String>,
    /// "available at" phrase for URLs.
    pub available_at: Option<String>,
    /// "by" preposition.
    pub by: Option<String>,
    /// "circa" term for approximate dates (has long and short forms).
    #[serde(default)]
    pub circa: SimpleTerm,
    /// "et al." abbreviation.
    pub et_al: Option<String>,
    /// "from" preposition.
    pub from: Option<String>,
    /// "ibid." term for repeated citations.
    pub ibid: Option<String>,
    /// "in" preposition.
    pub in_: Option<String>,
    /// Legacy short-form fallback for the "no date" term when no structured term is loaded.
    ///
    /// This remains deserializable for backward compatibility, but it is not serialized
    /// to avoid colliding with the structured `no-date` entry from `general`.
    #[serde(skip_serializing)]
    pub no_date: Option<String>,
    /// "retrieved" term for access dates.
    pub retrieved: Option<String>,
    /// All other general terms flattened into a map.
    #[serde(flatten, default)]
    pub general: std::collections::HashMap<GeneralTerm, SimpleTerm>,
}

/// A simple term with long and short forms.
///
/// Used for terms that have a primary long form and a shorter variant.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SimpleTerm {
    /// The long form of the term (e.g., "anonymous").
    pub long: MaybeGendered<String>,
    /// The short form of the term (e.g., "anon.").
    pub short: MaybeGendered<String>,
}

/// Terms for contributor roles.
///
/// Defines forms for roles like editor, translator, etc. in singular, plural, and verb forms.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ContributorTerm {
    /// Singular form (e.g., "editor", "translator").
    pub singular: SimpleTerm,
    /// Plural form (e.g., "editors", "translators").
    pub plural: SimpleTerm,
    /// Verb form (e.g., "edited by", "translated by").
    pub verb: SimpleTerm,
}

/// Terms for locators (page, chapter, etc.).
///
/// Defines forms for locator terms that can appear in long, short, and symbol variants,
/// each with singular and plural options.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct LocatorTerm {
    /// Long form (e.g., "page"/"pages").
    #[serde(default)]
    pub long: Option<SingularPlural>,
    /// Short form (e.g., "p."/"pp.").
    #[serde(default)]
    pub short: Option<SingularPlural>,
    /// Symbol form (e.g., "§"/"§§").
    #[serde(default)]
    pub symbol: Option<SingularPlural>,
    /// Lexical gender for noun agreement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<GrammaticalGender>,
}

/// A term with singular and plural forms.
///
/// Used to represent terms that change depending on count, such as "page" vs "pages".
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SingularPlural {
    /// Singular form (e.g., "page").
    pub singular: MaybeGendered<String>,
    /// Plural form (e.g., "pages").
    pub plural: MaybeGendered<String>,
}

/// Date-related terms.
///
/// Contains month names, season names, and terms for date modifiers like uncertainty,
/// open-ended ranges, and time period notation.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DateTerms {
    /// Month names (full and abbreviated forms).
    #[serde(default)]
    pub months: MonthNames,
    /// Season names (e.g., "Spring", "Summer", "Autumn", "Winter").
    #[serde(default)]
    pub seasons: Vec<String>,
    /// Term for uncertain dates (e.g., "uncertain").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncertainty_term: Option<String>,
    /// Term for open-ended date ranges (e.g., "present").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_ended_term: Option<String>,
    /// AM period term (e.g., "AM").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub am: Option<String>,
    /// PM period term (e.g., "PM").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pm: Option<String>,
    /// UTC timezone term (e.g., "UTC").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone_utc: Option<String>,
    /// Era suffix for year zero and negative years (e.g., "BC").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_era: Option<String>,
    /// Era suffix for positive years in BC/AD profile (e.g., "AD").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad: Option<String>,
    /// Era suffix for negative years in BC/AD profile (e.g., "BC").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bc: Option<String>,
    /// Era suffix for negative years in BCE/CE profile (e.g., "BCE").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bce: Option<String>,
    /// Era suffix for positive years in BCE/CE profile (e.g., "CE").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ce: Option<String>,
}

/// Month name lists.
///
/// Contains both full and abbreviated month names for a given locale.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct MonthNames {
    /// Full month names (e.g., "January", "February", ..., "December").
    pub long: Vec<String>,
    /// Abbreviated month names (e.g., "Jan.", "Feb.", ..., "Dec.").
    pub short: Vec<String>,
}

/// Number formatting options for a locale.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct NumberFormats {
    /// Decimal separator (e.g., "." for en-US, "," for de-DE).
    #[serde(default = "default_decimal_separator")]
    pub decimal_separator: String,
    /// Thousands separator (e.g., "," for en-US, "." for de-DE).
    #[serde(default = "default_thousands_separator")]
    pub thousands_separator: String,
    /// Minimum digits to display.
    #[serde(default = "default_minimum_digits")]
    pub minimum_digits: u8,
}

fn default_decimal_separator() -> String {
    ".".into()
}

fn default_thousands_separator() -> String {
    ",".into()
}

fn default_minimum_digits() -> u8 {
    1
}

/// Grammar options that vary by language or regional convention.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct GrammarOptions {
    /// Whether to place periods/commas inside closing quotation marks (American style).
    #[serde(default)]
    pub punctuation_in_quote: bool,
    /// Whether to use a non-breaking space before colon/question mark (French style).
    #[serde(default)]
    pub nbsp_before_colon: bool,
    /// Opening outer quotation mark character.
    #[serde(default = "default_open_quote")]
    pub open_quote: String,
    /// Closing outer quotation mark character.
    #[serde(default = "default_close_quote")]
    pub close_quote: String,
    /// Opening inner (nested) quotation mark character.
    #[serde(default = "default_open_inner_quote")]
    pub open_inner_quote: String,
    /// Closing inner (nested) quotation mark character.
    #[serde(default = "default_close_inner_quote")]
    pub close_inner_quote: String,
    /// Whether to use a serial (Oxford) comma before the final list item.
    #[serde(default)]
    pub serial_comma: bool,
    /// Delimiter between page range endpoints.
    #[serde(default = "default_page_range_delimiter")]
    pub page_range_delimiter: String,
}

impl Default for GrammarOptions {
    fn default() -> Self {
        Self {
            punctuation_in_quote: false,
            nbsp_before_colon: false,
            open_quote: default_open_quote(),
            close_quote: default_close_quote(),
            open_inner_quote: default_open_inner_quote(),
            close_inner_quote: default_close_inner_quote(),
            serial_comma: false,
            page_range_delimiter: default_page_range_delimiter(),
        }
    }
}

fn default_open_quote() -> String {
    "\u{201C}".into()
}

fn default_close_quote() -> String {
    "\u{201D}".into()
}

fn default_open_inner_quote() -> String {
    "\u{2018}".into()
}

fn default_close_inner_quote() -> String {
    "\u{2019}".into()
}

fn default_page_range_delimiter() -> String {
    "\u{2013}".into()
}

/// Message syntax variant active in a locale file.
///
/// Controls which `MessageEvaluator` implementation the engine selects at
/// locale-load time.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum MessageSyntax {
    /// Plain text only; parameterized messages are not evaluated.
    #[default]
    Static,
    /// ICU Message Format 2 evaluation (requires `Mf2MessageEvaluator`).
    Mf2,
}

/// Runtime evaluation options for a locale.
///
/// Declares which message syntax the `messages` map uses and controls
/// evaluator selection. May grow with additional fields (custom function
/// declarations, evaluator hints) without breaking existing locale files.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct EvaluationConfig {
    /// Message syntax used in this locale's `messages` map.
    #[serde(default)]
    pub message_syntax: MessageSyntax,
}

/// Vocabulary maps for genre and medium display text.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct VocabMap {
    /// Genre canonical key → display string.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub genre: HashMap<String, String>,
    /// Medium canonical key → display string.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub medium: HashMap<String, String>,
}

impl VocabMap {
    /// Returns true if both maps are empty.
    pub fn is_empty(&self) -> bool {
        self.genre.is_empty() && self.medium.is_empty()
    }
}

/// Partial patch applied on top of a base [`crate::locale::Locale`] for style-specific overrides.
///
/// A `LocaleOverride` allows a style to customize specific messages, grammar options,
/// and legacy term aliases without duplicating the entire base locale. All fields are
/// merged key-by-key into the target locale.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", default)]
pub struct LocaleOverride {
    /// Message IDs to replace in the base locale (key-by-key insertion/replacement).
    pub messages: std::collections::HashMap<String, String>,
    /// If present, replaces the entire grammar-options block.
    pub grammar_options: Option<GrammarOptions>,
    /// Additional or replacement legacy term aliases (key-by-key insertion/replacement).
    pub legacy_term_aliases: std::collections::HashMap<String, String>,
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

    /// Test that GeneralTerm variants deserialize from expected string values.
    #[test]
    fn test_general_term_deserialization() {
        let json_tests = vec![
            (r#""in""#, GeneralTerm::In),
            (r#""accessed""#, GeneralTerm::Accessed),
            (r#""retrieved""#, GeneralTerm::Retrieved),
            (r#""at""#, GeneralTerm::At),
            (r#""from""#, GeneralTerm::From),
            (r#""of""#, GeneralTerm::Of),
            (r#""to""#, GeneralTerm::To),
            (r#""by""#, GeneralTerm::By),
            (r#""no-date""#, GeneralTerm::NoDate),
            (r#""anonymous""#, GeneralTerm::Anonymous),
            (r#""circa""#, GeneralTerm::Circa),
            (r#""available-at""#, GeneralTerm::AvailableAt),
            (r#""ibid""#, GeneralTerm::Ibid),
            (r#""and""#, GeneralTerm::And),
            (r#""et-al""#, GeneralTerm::EtAl),
            (r#""and-others""#, GeneralTerm::AndOthers),
            (r#""forthcoming""#, GeneralTerm::Forthcoming),
            (r#""online""#, GeneralTerm::Online),
            (r#""here""#, GeneralTerm::Here),
            (r#""deposited""#, GeneralTerm::Deposited),
            (r#""review-of""#, GeneralTerm::ReviewOf),
            (
                r#""original-work-published""#,
                GeneralTerm::OriginalWorkPublished,
            ),
            (r#""patent""#, GeneralTerm::Patent),
            (r#""issued""#, GeneralTerm::Issued),
            (r#""volume""#, GeneralTerm::Volume),
            (r#""issue""#, GeneralTerm::Issue),
            (r#""page""#, GeneralTerm::Page),
            (r#""chapter""#, GeneralTerm::Chapter),
            (r#""edition""#, GeneralTerm::Edition),
            (r#""section""#, GeneralTerm::Section),
        ];

        for (json_str, expected) in json_tests {
            let result: GeneralTerm = serde_json::from_str(json_str)
                .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", json_str, e));
            assert_eq!(result, expected, "Mismatch for {}", json_str);
        }
    }

    /// Test that TermForm variants deserialize from expected string values.
    #[test]
    fn test_term_form_deserialization() {
        let form_long: TermForm = serde_json::from_str(r#""long""#).unwrap();
        assert_eq!(form_long, TermForm::Long);

        let form_short: TermForm = serde_json::from_str(r#""short""#).unwrap();
        assert_eq!(form_short, TermForm::Short);

        let form_verb: TermForm = serde_json::from_str(r#""verb""#).unwrap();
        assert_eq!(form_verb, TermForm::Verb);

        let form_verb_short: TermForm = serde_json::from_str(r#""verb-short""#).unwrap();
        assert_eq!(form_verb_short, TermForm::VerbShort);

        let form_symbol: TermForm = serde_json::from_str(r#""symbol""#).unwrap();
        assert_eq!(form_symbol, TermForm::Symbol);
    }

    /// Test that SimpleTerm can be constructed and provides both forms.
    #[test]
    fn test_simple_term_construction() {
        let term = SimpleTerm {
            long: "anonymous".into(),
            short: "anon.".into(),
        };

        assert_eq!(term.long, MaybeGendered::Plain("anonymous".to_string()));
        assert_eq!(term.short, MaybeGendered::Plain("anon.".to_string()));
    }

    /// Test that SingularPlural provides both singular and plural forms.
    #[test]
    fn test_singular_plural_construction() {
        let term = SingularPlural {
            singular: "page".into(),
            plural: "pages".into(),
        };

        assert_eq!(term.singular, MaybeGendered::Plain("page".to_string()));
        assert_eq!(term.plural, MaybeGendered::Plain("pages".to_string()));
    }

    /// The legacy no-date fallback (`no_date`) must not serialize alongside
    /// the structured `no-date` entry in `general`, even though both are
    /// populated (as they are for a fully-loaded en-US locale).
    #[test]
    fn test_terms_serializes_single_no_date_entry() {
        let terms = Terms {
            no_date: Some("n.d.".to_string()),
            general: std::collections::HashMap::from([(
                GeneralTerm::NoDate,
                SimpleTerm {
                    long: "no date".into(),
                    short: "n.d.".into(),
                },
            )]),
            ..Default::default()
        };
        let value = serde_json::to_value(&terms).unwrap();
        let object = value.as_object().unwrap();

        assert_eq!(
            object.get("no-date"),
            Some(&serde_json::json!({
                "long": "no date",
                "short": "n.d."
            }))
        );
        assert_eq!(object.get("no_date"), None);
    }

    /// The YAML-derived `Locale::en_us()` provides the same general terms
    /// that the deleted hardcoded `Terms::en_us()` constructor used to.
    #[test]
    fn test_locale_en_us_terms_defaults() {
        let locale = super::super::Locale::en_us();
        let terms = &locale.terms;

        assert_eq!(terms.and, Some("and".to_string()));
        assert_eq!(terms.and_symbol, Some("&".to_string()));
        assert_eq!(terms.and_others, Some("and others".to_string()));
        assert_eq!(terms.et_al, Some("et al.".to_string()));
        assert_eq!(terms.ibid, Some("ibid.".to_string()));

        // "circa" is parsed into the flattened `general` map (keyed by
        // GeneralTerm), not the legacy dedicated `circa` field, so resolve
        // it via the same public API callers use.
        assert_eq!(
            locale.general_term(&GeneralTerm::Circa, &TermForm::Long, None),
            Some("circa")
        );
        assert_eq!(
            locale.general_term(&GeneralTerm::Circa, &TermForm::Short, None),
            Some("c.")
        );
    }

    /// The YAML-derived `Locale::en_us()` provides month names and seasons
    /// that the deleted hardcoded `DateTerms::en_us()` constructor used to.
    #[test]
    fn test_locale_en_us_dates_months_and_seasons() {
        let dates = &super::super::Locale::en_us().dates;

        assert_eq!(dates.months.long.len(), 12);
        assert_eq!(dates.months.short.len(), 12);
        assert_eq!(dates.months.long[0], "January");
        assert_eq!(dates.months.long[11], "December");

        assert_eq!(dates.seasons.len(), 4);
        assert_eq!(dates.seasons[0], "Spring");
        assert_eq!(dates.seasons[1], "Summer");
        assert_eq!(dates.seasons[2], "Autumn");
        assert_eq!(dates.seasons[3], "Winter");
    }

    /// The YAML-derived `Locale::en_us()` provides era suffixes that the
    /// deleted hardcoded `DateTerms::en_us()` constructor used to.
    #[test]
    fn test_locale_en_us_dates_before_era() {
        let dates = &super::super::Locale::en_us().dates;

        assert_eq!(dates.before_era.as_deref(), Some("BC"));
        assert_eq!(dates.ad.as_deref(), Some("AD"));
        assert_eq!(dates.bc.as_deref(), Some("BC"));
        assert_eq!(dates.bce.as_deref(), Some("BCE"));
        assert_eq!(dates.ce.as_deref(), Some("CE"));
    }
}
