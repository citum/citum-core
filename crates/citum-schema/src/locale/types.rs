//! Locale-specific term types and definitions.
//!
//! This module defines the data structures for representing locale information including
//! general terms (prepositions, conjunctions), contributor role terms, locator terms for
//! pages and chapters, date-related terms, and month names. These are used by citation
//! processors to render localized output.

/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Form for term lookup.
///
/// Specifies which form variant of a term should be used in citation output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TermForm {
    /// Long form of a term (e.g., "page" vs "p.").
    Long,
    /// Short form of a term (e.g., "p." vs "page").
    Short,
    /// Verb form of a term (e.g., "edited by").
    Verb,
    /// Short verb form of a term (e.g., "ed." vs "edited by").
    VerbShort,
    /// Symbol form of a term (e.g., "§" for section).
    Symbol,
}

/// A list of general terms for citation formatting.
///
/// These are the standard terms that appear in bibliographies and citations,
/// including prepositions (in, at, from, by), punctuation terms (and, et al),
/// date-related terms (accessed, no-date, circa), locator terms (page, chapter, volume),
/// and special phrases (ibid, forthcoming, available-at).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum GeneralTerm {
    #[default]
    /// The preposition "in" (e.g., "in Smith, 2020").
    In,
    /// The term used for access dates (e.g., "accessed May 1").
    Accessed,
    /// The term used for retrieval statements (e.g., "retrieved from URL").
    Retrieved,
    /// The preposition "at" (e.g., "at the conference").
    At,
    /// The preposition "from" (e.g., "from the publisher").
    From,
    /// The preposition "by" (e.g., "by John Smith").
    By,
    /// The term used when no date is available (e.g., "n.d.").
    NoDate,
    /// The term used for anonymous authorship (e.g., "anonymous").
    Anonymous,
    /// The term used for approximate dates (e.g., "circa").
    Circa,
    /// The phrase used for availability statements (e.g., "available at URL").
    AvailableAt,
    /// The term used for immediately repeated citations (e.g., "ibid.").
    Ibid,
    /// The conjunction "and" (e.g., "Smith and Jones").
    And,
    /// The abbreviation for omitted additional names (e.g., "et al.").
    EtAl,
    /// The phrase "and others" (generic use).
    AndOthers,
    /// The term used for forthcoming works (e.g., "forthcoming").
    Forthcoming,
    /// The term used for online resources (e.g., "online").
    Online,
    /// The phrase used to introduce reviewed works (e.g., "review of").
    ReviewOf,
    /// The phrase used for original publication references (e.g., "originally published").
    OriginalWorkPublished,
    /// The term used for patents (e.g., "patent").
    Patent,
    /// The general term for volume locators (e.g., "volume", "vol.").
    Volume,
    /// The general term for issue locators (e.g., "issue", "no.").
    Issue,
    /// The general term for page locators (e.g., "page", "p.", "pp.").
    Page,
    /// The general term for chapter locators (e.g., "chapter", "ch.").
    Chapter,
    /// The general term for editions (e.g., "edition", "ed.").
    Edition,
    /// The general term for section locators (e.g., "section", "§").
    Section,
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
    /// "no date" term for missing dates.
    pub no_date: Option<String>,
    /// "retrieved" term for access dates.
    pub retrieved: Option<String>,
    /// All other general terms flattened into a map.
    #[serde(flatten, default)]
    pub general: std::collections::HashMap<GeneralTerm, SimpleTerm>,
}

impl Terms {
    /// Create English (US) terms.
    pub fn en_us() -> Self {
        Self {
            and: Some("and".into()),
            and_symbol: Some("&".into()),
            and_others: Some("and others".into()),
            anonymous: SimpleTerm {
                long: "anonymous".into(),
                short: "anon.".into(),
            },
            at: Some("at".into()),
            accessed: Some("accessed".into()),
            available_at: Some("available at".into()),
            by: Some("by".into()),
            circa: SimpleTerm {
                long: "circa".into(),
                short: "c.".into(),
            },
            et_al: Some("et al.".into()),
            from: Some("from".into()),
            ibid: Some("ibid.".into()),
            in_: Some("in".into()),
            no_date: Some("n.d.".into()),
            retrieved: Some("retrieved".into()),
            general: std::collections::HashMap::new(),
        }
    }
}

/// A simple term with long and short forms.
///
/// Used for terms that have a primary long form and a shorter variant.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SimpleTerm {
    /// The long form of the term (e.g., "anonymous").
    pub long: String,
    /// The short form of the term (e.g., "anon.").
    pub short: String,
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
}

/// A term with singular and plural forms.
///
/// Used to represent terms that change depending on count, such as "page" vs "pages".
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SingularPlural {
    /// Singular form (e.g., "page").
    pub singular: String,
    /// Plural form (e.g., "pages").
    pub plural: String,
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
}

impl DateTerms {
    /// Create English (US) date terms.
    pub fn en_us() -> Self {
        Self {
            months: MonthNames::en_us(),
            seasons: vec![
                "Spring".into(),
                "Summer".into(),
                "Autumn".into(),
                "Winter".into(),
            ],
            uncertainty_term: Some("uncertain".into()),
            open_ended_term: Some("present".into()),
            am: Some("AM".into()),
            pm: Some("PM".into()),
            timezone_utc: Some("UTC".into()),
        }
    }
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

impl MonthNames {
    /// Create English month names.
    pub fn en_us() -> Self {
        Self {
            long: vec![
                "January".into(),
                "February".into(),
                "March".into(),
                "April".into(),
                "May".into(),
                "June".into(),
                "July".into(),
                "August".into(),
                "September".into(),
                "October".into(),
                "November".into(),
                "December".into(),
            ],
            short: vec![
                "Jan.".into(),
                "Feb.".into(),
                "Mar.".into(),
                "Apr.".into(),
                "May".into(),
                "June".into(),
                "July".into(),
                "Aug.".into(),
                "Sept.".into(),
                "Oct.".into(),
                "Nov.".into(),
                "Dec.".into(),
            ],
        }
    }
}

#[cfg(test)]
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
            (r#""review-of""#, GeneralTerm::ReviewOf),
            (
                r#""original-work-published""#,
                GeneralTerm::OriginalWorkPublished,
            ),
            (r#""patent""#, GeneralTerm::Patent),
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
            long: "anonymous".to_string(),
            short: "anon.".to_string(),
        };

        assert_eq!(term.long, "anonymous");
        assert_eq!(term.short, "anon.");
    }

    /// Test that SingularPlural provides both singular and plural forms.
    #[test]
    fn test_singular_plural_construction() {
        let term = SingularPlural {
            singular: "page".to_string(),
            plural: "pages".to_string(),
        };

        assert_eq!(term.singular, "page");
        assert_eq!(term.plural, "pages");
    }

    /// Test that Terms::en_us() returns expected English terms.
    #[test]
    fn test_terms_en_us_defaults() {
        let terms = Terms::en_us();

        assert_eq!(terms.and, Some("and".to_string()));
        assert_eq!(terms.and_symbol, Some("&".to_string()));
        assert_eq!(terms.and_others, Some("and others".to_string()));
        assert_eq!(terms.by, Some("by".to_string()));
        assert_eq!(terms.from, Some("from".to_string()));
        assert_eq!(terms.et_al, Some("et al.".to_string()));
        assert_eq!(terms.in_, Some("in".to_string()));
        assert_eq!(terms.no_date, Some("n.d.".to_string()));
        assert_eq!(terms.ibid, Some("ibid.".to_string()));

        assert_eq!(terms.anonymous.long, "anonymous");
        assert_eq!(terms.anonymous.short, "anon.");
        assert_eq!(terms.circa.long, "circa");
        assert_eq!(terms.circa.short, "c.");
    }

    /// Test that DateTerms::en_us() provides all month names and seasons.
    #[test]
    fn test_date_terms_en_us_months() {
        let date_terms = DateTerms::en_us();

        assert_eq!(date_terms.months.long.len(), 12);
        assert_eq!(date_terms.months.short.len(), 12);
        assert_eq!(date_terms.months.long[0], "January");
        assert_eq!(date_terms.months.short[0], "Jan.");
        assert_eq!(date_terms.months.long[11], "December");
        assert_eq!(date_terms.months.short[11], "Dec.");
    }

    /// Test that DateTerms::en_us() provides all four season names.
    #[test]
    fn test_date_terms_en_us_seasons() {
        let date_terms = DateTerms::en_us();

        assert_eq!(date_terms.seasons.len(), 4);
        assert_eq!(date_terms.seasons[0], "Spring");
        assert_eq!(date_terms.seasons[1], "Summer");
        assert_eq!(date_terms.seasons[2], "Autumn");
        assert_eq!(date_terms.seasons[3], "Winter");
    }

    /// Test that MonthNames::en_us() provides standard English month names.
    #[test]
    fn test_month_names_en_us() {
        let months = MonthNames::en_us();

        assert_eq!(months.long.len(), 12);
        assert_eq!(months.short.len(), 12);
        assert_eq!(months.long[5], "June");
        assert_eq!(months.short[5], "June");
        assert_eq!(months.long[8], "September");
        assert_eq!(months.short[8], "Sept.");
    }
}
