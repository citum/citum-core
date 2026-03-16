/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Locale definitions for Citum.
//!
//! Locales provide language-specific terms, date formats, and punctuation rules
//! for citation formatting.

/// Raw locale types used during locale file parsing.
pub mod raw;
/// Structured locale types used by the processor.
pub mod types;

use crate::citation::LocatorType;
use crate::template::ContributorRole;
pub use raw::{RawLocale, RawTermValue};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub use types::*;

/// A list of month names (12 elements for Jan-Dec).
pub type MonthList = Vec<String>;

/// A locale definition containing language-specific terms and formatting rules.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Locale {
    /// The locale identifier (e.g., "en-US", "de-DE").
    pub locale: String,
    /// Date-related terms (months, seasons).
    #[serde(default)]
    pub dates: DateTerms,
    /// Contributor role terms (editor, translator, etc.).
    #[serde(default)]
    pub roles: HashMap<ContributorRole, ContributorTerm>,
    /// Locator terms (page, chapter, etc.).
    #[serde(default)]
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
}

/// Extract English (US) role terms.
fn en_us_role_terms() -> HashMap<ContributorRole, ContributorTerm> {
    let mut roles = HashMap::new();

    roles.insert(
        ContributorRole::Editor,
        ContributorTerm {
            singular: SimpleTerm {
                long: "editor".into(),
                short: "ed.".into(),
            },
            plural: SimpleTerm {
                long: "editors".into(),
                short: "eds.".into(),
            },
            verb: SimpleTerm {
                long: "edited by".into(),
                short: "ed.".into(),
            },
        },
    );

    roles.insert(
        ContributorRole::Translator,
        ContributorTerm {
            singular: SimpleTerm {
                long: "translator".into(),
                short: "Trans.".into(),
            },
            plural: SimpleTerm {
                long: "translators".into(),
                short: "Trans.".into(),
            },
            verb: SimpleTerm {
                long: "translated by".into(),
                short: "Trans.".into(),
            },
        },
    );

    roles.insert(
        ContributorRole::Director,
        ContributorTerm {
            singular: SimpleTerm {
                long: "director".into(),
                short: "Dir.".into(),
            },
            plural: SimpleTerm {
                long: "directors".into(),
                short: "dirs.".into(),
            },
            verb: SimpleTerm {
                long: "directed by".into(),
                short: "dir.".into(),
            },
        },
    );

    roles
}

/// Extract English (US) locator terms.
fn en_us_locator_terms() -> HashMap<LocatorType, LocatorTerm> {
    let mut locators = HashMap::new();
    locators.insert(
        LocatorType::Page,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: "page".into(),
                plural: "pages".into(),
            }),
            short: Some(SingularPlural {
                singular: "p.".into(),
                plural: "pp.".into(),
            }),
            symbol: None,
        },
    );

    locators.insert(
        LocatorType::Chapter,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: "chapter".into(),
                plural: "chapters".into(),
            }),
            short: Some(SingularPlural {
                singular: "ch.".into(),
                plural: "chs.".into(),
            }),
            symbol: None,
        },
    );

    locators.insert(
        LocatorType::Volume,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: "volume".into(),
                plural: "volumes".into(),
            }),
            short: Some(SingularPlural {
                singular: "vol.".into(),
                plural: "vols.".into(),
            }),
            symbol: None,
        },
    );

    locators.insert(
        LocatorType::Section,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: "section".into(),
                plural: "sections".into(),
            }),
            short: Some(SingularPlural {
                singular: "sec.".into(),
                plural: "secs.".into(),
            }),
            symbol: Some(SingularPlural {
                singular: "§".into(),
                plural: "§§".into(),
            }),
        },
    );

    locators.insert(
        LocatorType::Part,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: "part".into(),
                plural: "parts".into(),
            }),
            short: Some(SingularPlural {
                singular: "pt.".into(),
                plural: "pts.".into(),
            }),
            symbol: None,
        },
    );

    locators.insert(
        LocatorType::Supplement,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: "supplement".into(),
                plural: "supplements".into(),
            }),
            short: Some(SingularPlural {
                singular: "suppl.".into(),
                plural: "suppls.".into(),
            }),
            symbol: None,
        },
    );

    locators
}

impl Locale {
    /// Create a new English (US) locale with default terms.
    pub fn en_us() -> Self {
        Self {
            locale: "en-US".into(),
            dates: DateTerms::en_us(),
            roles: en_us_role_terms(),
            locators: en_us_locator_terms(),
            terms: Terms::en_us(),
            punctuation_in_quote: true,
            sort_articles: vec!["the".into(), "a".into(), "an".into()],
        }
    }

    /// Strip leading articles from a string for sorting.
    ///
    /// Uses locale-specific articles (e.g., "the", "a", "an" for English;
    /// "der", "die", "das" for German). Falls back to English articles
    /// if no locale-specific articles are defined.
    pub fn strip_sort_articles<'a>(&self, s: &'a str) -> &'a str {
        let s = s.trim();

        // Default English articles
        const DEFAULT_ARTICLES: &[&str] = &["the", "a", "an"];

        if self.sort_articles.is_empty() {
            // Use default English articles
            for article in DEFAULT_ARTICLES {
                let prefix = format!("{} ", article);
                if s.to_lowercase().starts_with(&prefix) {
                    return &s[prefix.len()..];
                }
            }
        } else {
            // Use locale-specific articles
            for article in &self.sort_articles {
                let prefix = format!("{} ", article);
                if s.to_lowercase().starts_with(&prefix) {
                    return &s[prefix.len()..];
                }
            }
        }
        s
    }

    /// Get default articles for a locale based on language code.
    fn default_articles_for_locale(locale_id: &str) -> Vec<String> {
        // Extract language code (first 2 chars)
        let lang = &locale_id[..2.min(locale_id.len())];
        match lang {
            "en" => vec!["the".into(), "a".into(), "an".into()],
            "de" => vec![
                "der".into(),
                "die".into(),
                "das".into(),
                "ein".into(),
                "eine".into(),
            ],
            "fr" => vec![
                "le".into(),
                "la".into(),
                "les".into(),
                "l'".into(),
                "un".into(),
                "une".into(),
            ],
            "es" => vec![
                "el".into(),
                "la".into(),
                "los".into(),
                "las".into(),
                "un".into(),
                "una".into(),
            ],
            "it" => vec![
                "il".into(),
                "lo".into(),
                "la".into(),
                "i".into(),
                "gli".into(),
                "le".into(),
                "un".into(),
                "una".into(),
            ],
            "pt" => vec![
                "o".into(),
                "a".into(),
                "os".into(),
                "as".into(),
                "um".into(),
                "uma".into(),
            ],
            "nl" => vec!["de".into(), "het".into(), "een".into()],
            _ => vec![], // Fall back to English defaults in strip_sort_articles
        }
    }

    /// Get a contributor role term.
    pub fn role_term(&self, role: &ContributorRole, plural: bool, form: TermForm) -> Option<&str> {
        let term = self.roles.get(role)?;
        let simple = if plural { &term.plural } else { &term.singular };
        Some(match form {
            TermForm::Long => &simple.long,
            TermForm::Short => &simple.short,
            TermForm::Verb => &term.verb.long,
            TermForm::VerbShort => &term.verb.short,
            _ => &simple.long, // Fallback
        })
    }

    /// Get a locator term.
    pub fn locator_term(
        &self,
        locator: &LocatorType,
        plural: bool,
        form: TermForm,
    ) -> Option<&str> {
        let term = self.locators.get(locator)?;
        let form_term = match form {
            TermForm::Long => &term.long,
            TermForm::Short => &term.short,
            TermForm::Symbol => &term.symbol,
            _ => &term.short, // Fallback
        };

        if let Some(ft) = form_term {
            Some(if plural { &ft.plural } else { &ft.singular })
        } else {
            None
        }
    }

    /// Get a general term by type and form.
    pub fn general_term(&self, term: &GeneralTerm, form: TermForm) -> Option<&str> {
        // First try the flattened map
        if let Some(simple) = self.terms.general.get(term) {
            return Some(match form {
                TermForm::Long => &simple.long,
                TermForm::Short => &simple.short,
                _ => &simple.long,
            });
        }

        // Fallback to specific fields for common terms
        match term {
            GeneralTerm::And => self.terms.and.as_deref(),
            GeneralTerm::EtAl => self.terms.et_al.as_deref(),
            GeneralTerm::AndOthers => self.terms.and_others.as_deref(),
            GeneralTerm::Accessed => self.terms.accessed.as_deref(),
            GeneralTerm::Ibid => self.terms.ibid.as_deref(),
            GeneralTerm::In => self.terms.in_.as_deref(),
            GeneralTerm::NoDate => self
                .terms
                .general
                .get(term)
                .map(|value| match form {
                    TermForm::Long => value.long.as_str(),
                    TermForm::Short => value.short.as_str(),
                    _ => value.long.as_str(),
                })
                .or(self.terms.no_date.as_deref()),
            GeneralTerm::Retrieved => self.terms.retrieved.as_deref(),
            GeneralTerm::At => self.terms.at.as_deref(),
            GeneralTerm::By => self.terms.by.as_deref(),
            GeneralTerm::From => self.terms.from.as_deref(),
            GeneralTerm::Of => self
                .terms
                .general
                .get(term)
                .map(|value| value.long.as_str()),
            GeneralTerm::To => self
                .terms
                .general
                .get(term)
                .map(|value| value.long.as_str()),
            GeneralTerm::Anonymous => Some(&self.terms.anonymous.long),
            GeneralTerm::Circa => Some(&self.terms.circa.long),
            // Fallback to locators for shared terms
            GeneralTerm::Volume => self.locator_term(&LocatorType::Volume, false, form),
            GeneralTerm::Issue => self.locator_term(&LocatorType::Issue, false, form),
            GeneralTerm::Page => self.locator_term(&LocatorType::Page, false, form),
            GeneralTerm::Chapter => self.locator_term(&LocatorType::Chapter, false, form),
            GeneralTerm::Section => self.locator_term(&LocatorType::Section, false, form),
            GeneralTerm::Here => self
                .terms
                .general
                .get(term)
                .map(|value| value.long.as_str()),
            GeneralTerm::Deposited => self
                .terms
                .general
                .get(term)
                .map(|value| value.long.as_str()),
            _ => None,
        }
    }

    /// Get the "and" term based on style preference.
    pub fn and_term(&self, use_symbol: bool) -> &str {
        if use_symbol {
            self.terms.and_symbol.as_deref().unwrap_or("&")
        } else {
            self.terms.and.as_deref().unwrap_or("and")
        }
    }

    /// Get the "et al." term.
    pub fn et_al(&self) -> &str {
        self.terms.et_al.as_deref().unwrap_or("et al.")
    }

    /// Get a month name.
    pub fn month_name(&self, month: u8, short: bool) -> &str {
        let idx = (month.saturating_sub(1)) as usize;
        if short {
            self.dates
                .months
                .short
                .get(idx)
                .map(|s| s.as_str())
                .unwrap_or("")
        } else {
            self.dates
                .months
                .long
                .get(idx)
                .map(|s| s.as_str())
                .unwrap_or("")
        }
    }
}

impl Locale {
    /// Load a locale from a YAML string.
    ///
    /// # Errors
    ///
    /// Returns an error when the YAML cannot be parsed into a locale.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, String> {
        let raw: raw::RawLocale = serde_yaml::from_str(yaml)
            .map_err(|e| format!("Failed to parse locale YAML: {}", e))?;

        Ok(Self::from_raw(raw))
    }

    /// Load a locale by ID (e.g., "en-US", "de-DE") from a locales directory.
    /// Falls back to en-US if the locale file is not found.
    pub fn load(locale_id: &str, locales_dir: &std::path::Path) -> Self {
        let extensions = ["yaml", "yml", "json", "cbor"];

        for ext in &extensions {
            let file_name = format!("{}.{}", locale_id, ext);
            let file_path = locales_dir.join(&file_name);

            if file_path.exists() {
                match Self::from_file(&file_path) {
                    Ok(locale) => return locale,
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to load locale {}.{}: {}",
                            locale_id, ext, e
                        );
                    }
                }
            }
        }

        // Try fallback to base locale (e.g., "de" from "de-DE")
        if locale_id.contains('-') {
            let base = locale_id.split('-').next().unwrap_or("en");
            // Try all files that start with base
            if let Ok(entries) = std::fs::read_dir(locales_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if (name_str.starts_with(base)
                        && extensions.iter().any(|ext| name_str.ends_with(ext)))
                        && let Ok(locale) = Self::from_file(&entry.path())
                    {
                        return locale;
                    }
                }
            }
        }

        // Default to hardcoded en-US
        Self::en_us()
    }

    /// Load locale from a file path directly (detects format).
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or its contents cannot be
    /// parsed as a supported locale format.
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("Failed to read locale file: {}", e))?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

        match ext {
            "cbor" => ciborium::de::from_reader::<raw::RawLocale, _>(std::io::Cursor::new(&bytes))
                .map(Self::from_raw)
                .map_err(|e| format!("Failed to parse CBOR locale: {}", e)),
            "json" => serde_json::from_slice::<raw::RawLocale>(&bytes)
                .map(Self::from_raw)
                .map_err(|e| format!("Failed to parse JSON locale: {}", e)),
            _ => {
                let content = String::from_utf8_lossy(&bytes);
                Self::from_yaml_str(&content)
            }
        }
    }

    /// Convert a RawLocale to a Locale.
    fn from_raw(raw: raw::RawLocale) -> Self {
        // Determine punctuation-in-quote from locale ID
        // en-US uses American style (inside), en-GB and others use outside
        let punctuation_in_quote = raw.locale.starts_with("en-US")
            || (raw.locale.starts_with("en") && !raw.locale.starts_with("en-GB"));

        // Start from en-US defaults so partially specified locale files still
        // have complete term/locator coverage (e.g., page/section labels).
        let mut locale = Locale::en_us();
        locale.locale = raw.locale.clone();
        locale.dates = DateTerms {
            months: MonthNames {
                long: raw.dates.months.long,
                short: raw.dates.months.short,
            },
            seasons: raw.dates.seasons,
            uncertainty_term: raw.dates.uncertainty_term,
            open_ended_term: raw.dates.open_ended_term,
            am: raw.dates.am,
            pm: raw.dates.pm,
            timezone_utc: raw.dates.timezone_utc,
        };
        locale.punctuation_in_quote = punctuation_in_quote;
        // Set locale-specific articles based on language
        locale.sort_articles = Self::default_articles_for_locale(&raw.locale);

        // Map raw terms to structured terms and locators
        for (key, value) in &raw.terms {
            // First try to parse as a locator
            if let Some(locator_type) = Self::parse_locator_type(key) {
                if let Some(forms) = Self::get_forms(value) {
                    let locator_term = LocatorTerm {
                        long: Self::extract_singular_plural(forms.get("long").as_ref()),
                        short: Self::extract_singular_plural(forms.get("short").as_ref()),
                        symbol: Self::extract_singular_plural(forms.get("symbol").as_ref()),
                    };
                    locale.locators.insert(locator_type, locator_term);
                }
                continue;
            }

            match key.as_str() {
                "and" => {
                    if let Some(forms) = Self::get_forms(value) {
                        if let Some(v) = forms.get("long").and_then(|v| v.as_string()) {
                            locale.terms.and = Some(v.to_string());
                        }
                        if let Some(v) = forms.get("symbol").and_then(|v| v.as_string()) {
                            locale.terms.and_symbol = Some(v.to_string());
                        }
                    }
                }
                "et_al" => {
                    if let Some(forms) = Self::get_forms(value)
                        && let Some(v) = forms.get("long").and_then(|v| v.as_string())
                    {
                        locale.terms.et_al = Some(v.to_string());
                    }
                }
                "and others" | "and_others" => {
                    if let Some(forms) = Self::get_forms(value)
                        && let Some(v) = forms.get("long").and_then(|v| v.as_string())
                    {
                        locale.terms.and_others = Some(v.to_string());
                    }
                }
                "accessed" => {
                    if let Some(forms) = Self::get_forms(value)
                        && let Some(v) = forms.get("long").and_then(|v| v.as_string())
                    {
                        locale.terms.accessed = Some(v.to_string());
                    }
                }
                "ibid" => {
                    if let Some(forms) = Self::get_forms(value)
                        && let Some(v) = forms.get("long").and_then(|v| v.as_string())
                    {
                        locale.terms.ibid = Some(v.to_string());
                    }
                }
                "no_date" | "no date" => {
                    let simple = Self::extract_simple_term_from_raw(value);
                    locale.terms.no_date = Some(simple.short.clone());
                    locale.terms.general.insert(GeneralTerm::NoDate, simple);
                }
                _ => {
                    // Try to parse as GeneralTerm
                    if let Some(general_term) = Self::parse_general_term(key) {
                        let simple = Self::extract_simple_term_from_raw(value);
                        locale.terms.general.insert(general_term, simple);
                    }
                }
            }
        }

        // Map raw roles to structured roles (simplified for now)
        for (key, role_term) in &raw.roles {
            if let Some(role) = Self::parse_role_name(key) {
                let contributor_term = ContributorTerm {
                    singular: Self::extract_simple_term(&role_term.long, &role_term.short, false),
                    plural: Self::extract_simple_term(&role_term.long, &role_term.short, true),
                    verb: Self::extract_verb_term(&role_term.verb, &role_term.verb_short),
                };
                locale.roles.insert(role, contributor_term);
            }
        }

        locale
    }

    fn get_forms(value: &raw::RawTermValue) -> Option<&HashMap<String, raw::RawTermValue>> {
        match value {
            raw::RawTermValue::Forms(forms) => Some(forms),
            _ => None,
        }
    }

    fn parse_locator_type(name: &str) -> Option<LocatorType> {
        match name {
            "algorithm" => Some(LocatorType::Algorithm),
            "book" => Some(LocatorType::Book),
            "chapter" => Some(LocatorType::Chapter),
            "clause" => Some(LocatorType::Clause),
            "column" => Some(LocatorType::Column),
            "corollary" => Some(LocatorType::Corollary),
            "definition" => Some(LocatorType::Definition),
            "division" => Some(LocatorType::Division),
            "figure" => Some(LocatorType::Figure),
            "folio" => Some(LocatorType::Folio),
            "issue" => Some(LocatorType::Issue),
            "lemma" => Some(LocatorType::Lemma),
            "line" => Some(LocatorType::Line),
            "note" => Some(LocatorType::Note),
            "number" => Some(LocatorType::Number),
            "opus" => Some(LocatorType::Opus),
            "page" => Some(LocatorType::Page),
            "paragraph" => Some(LocatorType::Paragraph),
            "part" => Some(LocatorType::Part),
            "problem" => Some(LocatorType::Problem),
            "proposition" => Some(LocatorType::Proposition),
            "recital" => Some(LocatorType::Recital),
            "schedule" => Some(LocatorType::Schedule),
            "section" => Some(LocatorType::Section),
            "subclause" | "sub-clause" | "sub_clause" => Some(LocatorType::Subclause),
            "subdivision" | "sub-division" | "sub_division" => Some(LocatorType::Subdivision),
            "subparagraph" | "sub-paragraph" | "sub_paragraph" => Some(LocatorType::Subparagraph),
            "subsection" | "sub-section" | "sub_section" => Some(LocatorType::Subsection),
            "sub_verbo" | "sub-verbo" => Some(LocatorType::SubVerbo),
            "supplement" => Some(LocatorType::Supplement),
            "surah" => Some(LocatorType::Surah),
            "theorem" => Some(LocatorType::Theorem),
            "verse" => Some(LocatorType::Verse),
            "volume" => Some(LocatorType::Volume),
            "volume-book" | "volume_book" => Some(LocatorType::VolumeBook),
            "volume-periodical" | "volume_periodical" => Some(LocatorType::VolumePeriodical),
            _ => None,
        }
    }

    fn parse_role_name(name: &str) -> Option<ContributorRole> {
        match name {
            "author" => Some(ContributorRole::Author),
            "editor" => Some(ContributorRole::Editor),
            "translator" => Some(ContributorRole::Translator),
            "director" => Some(ContributorRole::Director),
            "compiler" => Some(ContributorRole::Composer), // Close mapping
            "illustrator" => Some(ContributorRole::Illustrator),
            "collection-editor" => Some(ContributorRole::CollectionEditor),
            "container-author" => Some(ContributorRole::ContainerAuthor),
            "editorial-director" => Some(ContributorRole::EditorialDirector),
            "textual-editor" | "textual_editor" => Some(ContributorRole::TextualEditor),
            "interviewer" => Some(ContributorRole::Interviewer),
            "original-author" => Some(ContributorRole::OriginalAuthor),
            "recipient" => Some(ContributorRole::Recipient),
            "reviewed-author" => Some(ContributorRole::ReviewedAuthor),
            "composer" => Some(ContributorRole::Composer),
            _ => None,
        }
    }

    fn extract_singular_plural(value: Option<&&raw::RawTermValue>) -> Option<SingularPlural> {
        match value {
            Some(raw::RawTermValue::SingularPlural { singular, plural }) => Some(SingularPlural {
                singular: singular.clone(),
                plural: plural.clone(),
            }),
            Some(raw::RawTermValue::Simple(s)) => Some(SingularPlural {
                singular: s.clone(),
                plural: s.clone(), // Fallback if only one form provided
            }),
            Some(raw::RawTermValue::Forms(forms)) => {
                let singular = forms
                    .get("singular")
                    .and_then(|v| Self::extract_term_string(v, false));
                let plural = forms
                    .get("plural")
                    .and_then(|v| Self::extract_term_string(v, false));

                singular.map(|s| SingularPlural {
                    plural: plural.unwrap_or_else(|| s.clone()),
                    singular: s,
                })
            }
            _ => None,
        }
    }

    fn extract_simple_term(
        long: &Option<raw::RawTermValue>,
        short: &Option<raw::RawTermValue>,
        plural: bool,
    ) -> SimpleTerm {
        let long_str = long
            .as_ref()
            .and_then(|v| Self::extract_term_string(v, plural))
            .unwrap_or_default();

        let short_str = short
            .as_ref()
            .and_then(|v| Self::extract_term_string(v, plural))
            .unwrap_or_default();

        SimpleTerm {
            long: long_str,
            short: short_str,
        }
    }

    fn extract_term_string(value: &raw::RawTermValue, plural: bool) -> Option<String> {
        match value {
            raw::RawTermValue::Simple(s) => Some(s.clone()),
            raw::RawTermValue::SingularPlural {
                singular,
                plural: p,
            } => Some(if plural { p.clone() } else { singular.clone() }),
            raw::RawTermValue::Forms(forms) => {
                let key = if plural { "plural" } else { "singular" };
                forms
                    .get(key)
                    .and_then(|v| Self::extract_term_string(v, false))
            }
        }
    }

    fn extract_verb_term(
        verb: &Option<raw::RawTermValue>,
        verb_short: &Option<raw::RawTermValue>,
    ) -> SimpleTerm {
        let long_str = verb
            .as_ref()
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();

        let short_str = verb_short
            .as_ref()
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();

        SimpleTerm {
            long: long_str,
            short: short_str,
        }
    }

    /// Parse a locale term key into a structured general-term identifier.
    pub fn parse_general_term(name: &str) -> Option<GeneralTerm> {
        match name {
            "in" => Some(GeneralTerm::In),
            "accessed" => Some(GeneralTerm::Accessed),
            "retrieved" => Some(GeneralTerm::Retrieved),
            "at" => Some(GeneralTerm::At),
            "from" => Some(GeneralTerm::From),
            "of" => Some(GeneralTerm::Of),
            "to" => Some(GeneralTerm::To),
            "by" => Some(GeneralTerm::By),
            "no-date" | "no_date" | "no date" => Some(GeneralTerm::NoDate),
            "anonymous" => Some(GeneralTerm::Anonymous),
            "circa" => Some(GeneralTerm::Circa),
            "available-at" | "available_at" | "available at" => Some(GeneralTerm::AvailableAt),
            "ibid" => Some(GeneralTerm::Ibid),
            "and" => Some(GeneralTerm::And),
            "et-al" | "et_al" | "et al" => Some(GeneralTerm::EtAl),
            "and-others" | "and_others" | "and others" => Some(GeneralTerm::AndOthers),
            "forthcoming" => Some(GeneralTerm::Forthcoming),
            "online" => Some(GeneralTerm::Online),
            "here" => Some(GeneralTerm::Here),
            "deposited" => Some(GeneralTerm::Deposited),
            "review-of" | "review_of" | "review of" => Some(GeneralTerm::ReviewOf),
            "original-work-published" => Some(GeneralTerm::OriginalWorkPublished),
            "patent" => Some(GeneralTerm::Patent),
            "volume" => Some(GeneralTerm::Volume),
            "issue" => Some(GeneralTerm::Issue),
            "page" => Some(GeneralTerm::Page),
            "chapter" => Some(GeneralTerm::Chapter),
            "edition" => Some(GeneralTerm::Edition),
            "section" => Some(GeneralTerm::Section),
            _ => None,
        }
    }

    fn extract_simple_term_from_raw(value: &raw::RawTermValue) -> SimpleTerm {
        match value {
            raw::RawTermValue::Simple(s) => SimpleTerm {
                long: s.clone(),
                short: s.clone(),
            },
            raw::RawTermValue::Forms(forms) => {
                let long = forms
                    .get("long")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string();
                let short = forms
                    .get("short")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&long)
                    .to_string();
                SimpleTerm { long, short }
            }
            raw::RawTermValue::SingularPlural { singular, .. } => SimpleTerm {
                long: singular.clone(),
                short: singular.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_en_us_locale() {
        let locale = Locale::en_us();
        assert_eq!(locale.locale, "en-US");
        assert_eq!(locale.and_term(false), "and");
        assert_eq!(locale.and_term(true), "&");
        assert_eq!(locale.et_al(), "et al.");
    }

    #[test]
    fn test_month_names() {
        let locale = Locale::en_us();
        assert_eq!(locale.month_name(1, false), "January");
        assert_eq!(locale.month_name(1, true), "Jan.");
        assert_eq!(locale.month_name(12, false), "December");
    }

    #[test]
    fn test_role_terms() {
        let locale = Locale::en_us();

        assert_eq!(
            locale.role_term(&ContributorRole::Editor, false, TermForm::Short),
            Some("ed.")
        );
        assert_eq!(
            locale.role_term(&ContributorRole::Editor, true, TermForm::Short),
            Some("eds.")
        );
        assert_eq!(
            locale.role_term(&ContributorRole::Translator, false, TermForm::Verb),
            Some("translated by")
        );
    }

    #[test]
    fn test_no_date_term_resolves_long_and_short_forms() {
        let locale = Locale::en_us();

        assert_eq!(
            locale.general_term(&GeneralTerm::NoDate, TermForm::Long),
            Some("no date")
        );
        assert_eq!(
            locale.general_term(&GeneralTerm::NoDate, TermForm::Short),
            Some("n.d.")
        );
    }

    #[test]
    fn test_no_date_term_falls_back_to_legacy_short_form() {
        let mut locale = Locale::default();
        locale.terms.no_date = Some("n.d.".to_string());

        assert_eq!(
            locale.general_term(&GeneralTerm::NoDate, TermForm::Short),
            Some("n.d.")
        );
        assert_eq!(
            locale.general_term(&GeneralTerm::NoDate, TermForm::Long),
            Some("n.d.")
        );
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
        assert_eq!(locale.and_term(false), "und");
        assert_eq!(locale.et_al(), "u. a.");
        assert_eq!(locale.month_name(1, false), "Januar");
        assert_eq!(locale.month_name(3, false), "März");
    }

    #[test]
    fn test_yaml_no_date_term_preserves_long_and_short_forms() {
        let yaml = r#"
locale: en-US
dates:
  months:
    long: [January, February, March, April, May, June, July, August, September, October, November, December]
    short: [Jan., Feb., Mar., Apr., May, June, July, Aug., Sept., Oct., Nov., Dec.]
  seasons: [Spring, Summer, Autumn, Winter]
roles: {}
terms:
  no date:
    long: no date
    short: n.d.
"#;

        let locale = Locale::from_yaml_str(yaml).unwrap();
        assert_eq!(
            locale.general_term(&GeneralTerm::NoDate, TermForm::Long),
            Some("no date")
        );
        assert_eq!(
            locale.general_term(&GeneralTerm::NoDate, TermForm::Short),
            Some("n.d.")
        );
        assert_eq!(locale.terms.no_date.as_deref(), Some("n.d."));
    }
}
