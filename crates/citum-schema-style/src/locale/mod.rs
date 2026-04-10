/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Locale definitions for Citum.
//!
//! Locales provide language-specific terms, date formats, and punctuation rules
//! for citation formatting.

/// Locator text normalization.
pub mod locator;
/// Message evaluation for parameterized locale strings.
pub mod message;
/// Raw locale types used during locale file parsing.
pub mod raw;
/// Structured locale types used by the processor.
pub mod types;

use crate::citation::LocatorType;
use crate::template::ContributorRole;
pub use message::{MessageArgs, MessageEvaluator, Mf2MessageEvaluator};
pub use raw::{RawLocale, RawTermValue};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, OnceLock};
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

#[derive(Deserialize)]
struct EmbeddedVocabDocument {
    #[serde(default)]
    vocab: Option<raw::RawVocab>,
}

/// Extract one top-level YAML section while preserving its nested indentation.
fn extract_top_level_yaml_section(yaml: &str, key: &str) -> Option<String> {
    let header = format!("{key}:");
    let mut collected = Vec::new();
    let mut in_section = false;

    for line in yaml.lines() {
        let trimmed = line.trim_end_matches('\r');
        let is_top_level =
            !trimmed.is_empty() && !trimmed.starts_with(' ') && !trimmed.starts_with('\t');

        if in_section {
            if is_top_level {
                break;
            }
            collected.push(trimmed);
            continue;
        }

        if trimmed == header {
            in_section = true;
            collected.push(trimmed);
        }
    }

    if collected.is_empty() {
        None
    } else {
        Some(collected.join("\n"))
    }
}

/// Curated en-US genre and medium labels from the embedded locale asset.
fn embedded_en_us_vocab() -> &'static VocabMap {
    static EN_US_VOCAB: OnceLock<VocabMap> = OnceLock::new();

    EN_US_VOCAB.get_or_init(|| {
        crate::embedded::get_locale_bytes("en-US")
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .and_then(|yaml| extract_top_level_yaml_section(yaml, "vocab"))
            .and_then(|vocab_yaml| serde_yaml::from_str::<EmbeddedVocabDocument>(&vocab_yaml).ok())
            .and_then(|document| document.vocab)
            .map(|document| VocabMap {
                genre: document.genre,
                medium: document.medium,
            })
            .unwrap_or_default()
    })
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

    roles.insert(
        ContributorRole::Interviewer,
        ContributorTerm {
            singular: SimpleTerm {
                long: "Interviewer".into(),
                short: "Interviewer".into(),
            },
            plural: SimpleTerm {
                long: "Interviewers".into(),
                short: "Interviewers".into(),
            },
            verb: SimpleTerm {
                long: "interviewed by".into(),
                short: "interviewed by".into(),
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

/// Convert a kebab-case key to a human-readable display string.
///
/// Splits on `-`, capitalizes the first character of the first word, and joins with spaces.
fn kebab_to_display(key: &str) -> String {
    let mut words = key.split('-');
    let mut result = String::new();
    if let Some(first) = words.next() {
        let mut chars = first.chars();
        if let Some(c) = chars.next() {
            result.extend(c.to_uppercase());
            result.push_str(chars.as_str());
        }
        for word in words {
            result.push(' ');
            result.push_str(word);
        }
    }
    result
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
            locale_schema_version: None,
            evaluation: EvaluationConfig::default(),
            messages: HashMap::new(),
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
            vocab: embedded_en_us_vocab().clone(),
            evaluator: Arc::new(Mf2MessageEvaluator),
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

    /// Look up display text for a genre canonical key.
    ///
    /// Falls back to a readable form of the key if no translation found.
    pub fn lookup_genre(&self, key: &str) -> String {
        self.vocab
            .genre
            .get(key)
            .cloned()
            .unwrap_or_else(|| kebab_to_display(key))
    }

    /// Look up display text for a medium canonical key.
    ///
    /// Falls back to a readable form of the key if no translation found.
    pub fn lookup_medium(&self, key: &str) -> String {
        self.vocab
            .medium
            .get(key)
            .cloned()
            .unwrap_or_else(|| kebab_to_display(key))
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
        let term_text = match form {
            TermForm::Long => &simple.long,
            TermForm::Short => {
                if simple.short.is_empty() {
                    &simple.long
                } else {
                    &simple.short
                }
            }
            TermForm::Verb => &term.verb.long,
            TermForm::VerbShort => {
                if term.verb.short.is_empty() {
                    &term.verb.long
                } else {
                    &term.verb.short
                }
            }
            _ => &simple.long,
        };

        if term_text.is_empty() {
            None
        } else {
            Some(term_text.as_str())
        }
    }

    /// Resolve a contributor role term, evaluating MF1 messages when configured.
    pub fn resolved_role_term(
        &self,
        role: &ContributorRole,
        plural: bool,
        form: TermForm,
    ) -> Option<String> {
        if let Some(message_id) = Self::role_message_id(role, form)
            && let Some(resolved) =
                self.resolve_message_text(message_id, Some(u64::from(plural) + 1), &[])
        {
            return Some(resolved);
        }

        self.role_term(role, plural, form).map(ToOwned::to_owned)
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

    /// Resolve a locator term, evaluating MF1 messages when configured.
    pub fn resolved_locator_term(
        &self,
        locator: &LocatorType,
        plural: bool,
        form: TermForm,
    ) -> Option<String> {
        if let Some(message_id) = Self::locator_message_id(locator, form)
            && let Some(resolved) =
                self.resolve_message_text(message_id, Some(u64::from(plural) + 1), &[])
        {
            return Some(resolved);
        }

        self.locator_term(locator, plural, form)
            .map(ToOwned::to_owned)
            .or_else(|| {
                if let LocatorType::Custom(key) = locator {
                    self.locator_term_any_form(locator, plural)
                        .map(ToOwned::to_owned)
                        .or_else(|| Some(key.clone()))
                } else {
                    None
                }
            })
    }

    fn locator_term_any_form(&self, locator: &LocatorType, plural: bool) -> Option<&str> {
        let term = self.locators.get(locator)?;
        [&term.long, &term.short, &term.symbol]
            .into_iter()
            .flatten()
            .next()
            .map(|forms| {
                if plural {
                    forms.plural.as_str()
                } else {
                    forms.singular.as_str()
                }
            })
    }

    /// Get a general term by type and form.
    pub fn general_term(&self, term: &GeneralTerm, form: TermForm) -> Option<&str> {
        // Legacy borrowed lookup path: prefer plain v2 messages first, then
        // alias-backed messages, and finally the v1 term tables.
        let candidate_id = format!("term.{}", Self::general_term_to_message_id(term));
        if let Some(msg) = self.messages.get(&candidate_id) {
            // Only use plain messages here (no ICU variable syntax)
            if !msg.contains('{') {
                return Some(msg.as_str());
            }
        }
        // Check legacy_term_aliases
        let legacy_key = Self::general_term_to_legacy_key(term);
        if let Some(msg_id) = self.legacy_term_aliases.get(legacy_key)
            && let Some(msg) = self.messages.get(msg_id)
            && !msg.contains('{')
        {
            return Some(msg.as_str());
        }

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

    /// Resolve a general term, evaluating MF1 messages when configured.
    pub fn resolved_general_term(&self, term: &GeneralTerm, form: TermForm) -> Option<String> {
        if let Some(message_id) = Self::general_message_id(term, form)
            && let Some(resolved) = self.resolve_message_text(message_id, None, &[])
        {
            return Some(resolved);
        }

        self.general_term(term, form).map(ToOwned::to_owned)
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

    /// Map a GeneralTerm to its canonical message ID suffix (e.g., GeneralTerm::EtAl → "et-al").
    fn general_term_to_message_id(term: &GeneralTerm) -> &'static str {
        match term {
            GeneralTerm::And => "and",
            GeneralTerm::EtAl => "et-al",
            GeneralTerm::AndOthers => "and-others",
            GeneralTerm::Accessed => "accessed",
            GeneralTerm::Retrieved => "retrieved",
            GeneralTerm::NoDate => "no-date",
            GeneralTerm::Ibid => "ibid",
            GeneralTerm::In => "in",
            GeneralTerm::At => "at",
            GeneralTerm::By => "by",
            GeneralTerm::From => "from",
            GeneralTerm::Of => "of",
            GeneralTerm::To => "to",
            GeneralTerm::Anonymous => "anonymous",
            GeneralTerm::Circa => "circa",
            GeneralTerm::Forthcoming => "forthcoming",
            GeneralTerm::Online => "online",
            GeneralTerm::AvailableAt => "available-at",
            GeneralTerm::ReviewOf => "review-of",
            GeneralTerm::Here => "here",
            GeneralTerm::Deposited => "deposited",
            GeneralTerm::Patent => "patent",
            GeneralTerm::Volume => "volume",
            GeneralTerm::Issue => "issue",
            GeneralTerm::Page => "page",
            GeneralTerm::Chapter => "chapter",
            GeneralTerm::Edition => "edition",
            GeneralTerm::Section => "section",
            GeneralTerm::OriginalWorkPublished => "original-work-published",
        }
    }

    /// Map a GeneralTerm to its legacy CSL key string for alias lookup.
    fn general_term_to_legacy_key(term: &GeneralTerm) -> &'static str {
        match term {
            GeneralTerm::EtAl => "et_al",
            GeneralTerm::NoDate => "no_date",
            _ => Self::general_term_to_message_id(term),
        }
    }

    fn role_message_id(role: &ContributorRole, form: TermForm) -> Option<&'static str> {
        let prefix = match role {
            ContributorRole::Editor => "role.editor",
            ContributorRole::Translator => "role.translator",
            ContributorRole::Guest => "role.guest",
            _ => return None,
        };

        match form {
            TermForm::Long => Some(match prefix {
                "role.editor" => "role.editor.label-long",
                "role.translator" => "role.translator.label-long",
                "role.guest" => "role.guest.label-long",
                _ => return None,
            }),
            TermForm::Short => Some(match prefix {
                "role.editor" => "role.editor.label",
                "role.translator" => "role.translator.label",
                "role.guest" => "role.guest.label",
                _ => return None,
            }),
            TermForm::Verb | TermForm::VerbShort => Some(match prefix {
                "role.editor" => "role.editor.verb",
                "role.translator" => "role.translator.verb",
                "role.guest" => "role.guest.verb",
                _ => return None,
            }),
            TermForm::Symbol => None,
        }
    }

    fn locator_message_id(locator: &LocatorType, form: TermForm) -> Option<&'static str> {
        let prefix = match locator {
            LocatorType::Page => "term.page-label",
            LocatorType::Chapter => "term.chapter-label",
            LocatorType::Volume => "term.volume-label",
            LocatorType::Section => "term.section-label",
            LocatorType::Figure => "term.figure-label",
            LocatorType::Note => "term.note-label",
            _ => return None,
        };

        match form {
            TermForm::Long => Some(match prefix {
                "term.page-label" => "term.page-label-long",
                "term.chapter-label" => "term.chapter-label-long",
                "term.volume-label" => "term.volume-label-long",
                "term.section-label" => "term.section-label-long",
                "term.figure-label" => "term.figure-label-long",
                "term.note-label" => "term.note-label-long",
                _ => return None,
            }),
            TermForm::Short => Some(prefix),
            TermForm::Symbol | TermForm::Verb | TermForm::VerbShort => None,
        }
    }

    fn general_message_id(term: &GeneralTerm, form: TermForm) -> Option<&'static str> {
        match (term, form) {
            (GeneralTerm::And, _) => Some("term.and"),
            (GeneralTerm::EtAl, _) => Some("term.et-al"),
            (GeneralTerm::AndOthers, _) => Some("term.and-others"),
            (GeneralTerm::Accessed, _) => Some("term.accessed"),
            (GeneralTerm::Retrieved, _) => Some("term.retrieved"),
            (GeneralTerm::NoDate, TermForm::Long) => Some("term.no-date-long"),
            (GeneralTerm::NoDate, _) => Some("term.no-date"),
            (GeneralTerm::Forthcoming, _) => Some("term.forthcoming"),
            (GeneralTerm::Circa, TermForm::Long) => Some("term.circa-long"),
            (GeneralTerm::Circa, _) => Some("term.circa"),
            _ => None,
        }
    }

    fn resolve_message_text(
        &self,
        message_id: &str,
        count: Option<u64>,
        _variables: &[(&str, &str)],
    ) -> Option<String> {
        let message = self.messages.get(message_id)?;

        // Build MessageArgs for the evaluator
        let args = MessageArgs {
            count,
            ..MessageArgs::default()
        };

        // Store variables as owned Strings and then reference them
        // (This is a limitation of the current design; could be improved with owned args)
        // For now, we'll build a simple approach: just return the message if it's static
        if !message.contains('{') {
            return Some(message.clone());
        }

        // If no parameterized syntax, return None (fallback to legacy terms)
        if self.evaluation.message_syntax == MessageSyntax::Static {
            return None;
        }

        // Create a temporary variables map for the evaluator
        // This is a simplification; the evaluator trait should ideally accept the args directly
        // For now, we'll convert and handle in the message evaluator
        self.evaluator.evaluate(message, &args)
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
    #[allow(
        clippy::too_many_lines,
        reason = "Complex parsing of raw locale data with multiple term types"
    )]
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
            before_era: raw.dates.before_era,
            ad: raw.dates.ad,
            bc: raw.dates.bc,
            bce: raw.dates.bce,
            ce: raw.dates.ce,
        };
        locale.punctuation_in_quote = punctuation_in_quote;
        // Set locale-specific articles based on language
        locale.sort_articles = Self::default_articles_for_locale(&raw.locale);

        // v2 schema fields — copied verbatim (no structural transformation needed at this layer)
        locale.locale_schema_version = raw.locale_schema_version;
        locale.evaluation = raw.evaluation.unwrap_or_default();
        locale.messages = raw.messages;
        locale.date_formats = raw.date_formats;
        locale.legacy_term_aliases = raw.legacy_term_aliases;

        // Merge vocab overrides into the embedded en-US defaults.
        if let Some(raw_vocab) = raw.vocab {
            locale.vocab.genre.extend(raw_vocab.genre);
            locale.vocab.medium.extend(raw_vocab.medium);
        }

        // Merge grammar_options: use raw value if present, otherwise derive from locale ID
        if let Some(go) = raw.grammar_options {
            locale.grammar_options = go;
        } else {
            // Derive punctuation_in_quote from locale ID (preserving existing behaviour)
            locale.grammar_options.punctuation_in_quote = locale.punctuation_in_quote;
        }
        // For v2 locales that explicitly declare grammar_options, the grammar_options
        // field is the authoritative source. Sync back to the legacy punctuation_in_quote
        // field so all existing call sites in citum-engine get the correct value.
        locale.punctuation_in_quote = locale.grammar_options.punctuation_in_quote;

        // Merge number_formats if provided
        if let Some(nf) = raw.number_formats {
            locale.number_formats = nf;
        }

        let explicit_locator_keys: std::collections::HashSet<LocatorType> = raw
            .locators
            .keys()
            .filter_map(|key| Self::parse_builtin_locator_type(key))
            .collect();

        for (key, value) in &raw.locators {
            if let Some(locator_type) = Self::parse_locator_type(key)
                && let Some(forms) = Self::get_forms(value)
            {
                let locator_term = LocatorTerm {
                    long: Self::extract_singular_plural(forms.get("long").as_ref()),
                    short: Self::extract_singular_plural(forms.get("short").as_ref()),
                    symbol: Self::extract_singular_plural(forms.get("symbol").as_ref()),
                };
                locale.locators.insert(locator_type, locator_term);
            }
        }

        // Map raw terms to structured general terms.
        for (key, value) in &raw.terms {
            if let Some(locator_type) = Self::parse_builtin_locator_type(key)
                && !explicit_locator_keys.contains(&locator_type)
                && let Some(forms) = Self::get_forms(value)
            {
                let locator_term = LocatorTerm {
                    long: Self::extract_singular_plural(forms.get("long").as_ref()),
                    short: Self::extract_singular_plural(forms.get("short").as_ref()),
                    symbol: Self::extract_singular_plural(forms.get("symbol").as_ref()),
                };
                locale.locators.insert(locator_type, locator_term);
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

        // Set the message evaluator based on evaluation.message_syntax
        locale.evaluator = match locale.evaluation.message_syntax {
            MessageSyntax::Mf2 => Arc::new(Mf2MessageEvaluator),
            MessageSyntax::Static => Arc::new(Mf2MessageEvaluator),
        };

        locale
    }

    fn get_forms(value: &raw::RawTermValue) -> Option<&HashMap<String, raw::RawTermValue>> {
        match value {
            raw::RawTermValue::Forms(forms) => Some(forms),
            _ => None,
        }
    }

    fn parse_locator_type(name: &str) -> Option<LocatorType> {
        LocatorType::from_key(name).ok()
    }

    fn parse_builtin_locator_type(name: &str) -> Option<LocatorType> {
        match Self::parse_locator_type(name)? {
            LocatorType::Custom(_) => None,
            locator => Some(locator),
        }
    }

    fn parse_role_name(name: &str) -> Option<ContributorRole> {
        match name {
            "author" => Some(ContributorRole::Author),
            "chair" => Some(ContributorRole::Chair),
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

    /// Apply a partial override, merging its fields into this locale.
    ///
    /// Performs key-by-key insertion or replacement for:
    /// - `messages`: new or updated message IDs
    /// - `grammar_options`: if `Some`, replaces the entire block and syncs
    ///   `punctuation_in_quote` field
    /// - `legacy_term_aliases`: new or updated term aliases
    pub fn apply_override(&mut self, ov: &LocaleOverride) {
        for (k, v) in &ov.messages {
            self.messages.insert(k.clone(), v.clone());
        }
        if let Some(go) = &ov.grammar_options {
            self.grammar_options = go.clone();
            self.punctuation_in_quote = go.punctuation_in_quote;
        }
        for (k, v) in &ov.legacy_term_aliases {
            self.legacy_term_aliases.insert(k.clone(), v.clone());
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
    fn test_resolved_locator_term_evaluates_plural_message() {
        let locale = Locale::en_us();

        assert_eq!(
            locale.resolved_locator_term(&LocatorType::Page, false, TermForm::Short),
            Some("p.".to_string())
        );
        assert_eq!(
            locale.resolved_locator_term(&LocatorType::Page, true, TermForm::Short),
            Some("pp.".to_string())
        );
    }

    #[test]
    fn test_resolved_locator_term_falls_back_to_custom_locale_form_then_raw_key() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
locators:
  reel:
    long:
      singular: "reel"
      plural: "reels"
"#,
        )
        .expect("custom locale should parse");

        assert_eq!(
            locale.resolved_locator_term(
                &LocatorType::Custom("reel".to_string()),
                false,
                TermForm::Short
            ),
            Some("reel".to_string())
        );
        assert_eq!(
            locale.resolved_locator_term(
                &LocatorType::Custom("movement".to_string()),
                false,
                TermForm::Short
            ),
            Some("movement".to_string())
        );
    }

    #[test]
    fn test_legacy_locator_terms_under_terms_still_populate_locators() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
terms:
  page:
    short:
      singular: "pg."
      plural: "pgs."
"#,
        )
        .expect("legacy locator terms should parse");

        assert_eq!(
            locale.resolved_locator_term(&LocatorType::Page, false, TermForm::Short),
            Some("pg.".to_string())
        );
    }

    #[test]
    fn test_explicit_locators_override_legacy_terms_for_builtin_keys() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
terms:
  page:
    short:
      singular: "pg."
      plural: "pgs."
locators:
  page:
    short:
      singular: "p."
      plural: "pp."
"#,
        )
        .expect("mixed locator forms should parse");

        assert_eq!(
            locale.resolved_locator_term(&LocatorType::Page, false, TermForm::Short),
            Some("p.".to_string())
        );
    }

    #[test]
    fn test_non_locator_terms_are_not_reclassified_as_custom_locators() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
terms:
  and:
    long: "und"
"#,
        )
        .expect("general terms should parse");

        assert_eq!(locale.terms.and.as_deref(), Some("und"));
        assert!(
            !locale
                .locators
                .contains_key(&LocatorType::Custom("and".to_string()))
        );
    }

    #[test]
    fn test_resolved_role_term_evaluates_plural_message() {
        let locale = Locale::en_us();

        assert_eq!(
            locale.resolved_role_term(&ContributorRole::Editor, false, TermForm::Long),
            Some("editor".to_string())
        );
        assert_eq!(
            locale.resolved_role_term(&ContributorRole::Editor, true, TermForm::Long),
            Some("editors".to_string())
        );
    }

    #[test]
    fn test_lookup_genre_known_key() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  genre:
    phd-thesis: "PhD thesis"
"#,
        )
        .unwrap();
        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
    }

    #[test]
    fn test_lookup_medium_known_key() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  medium:
    television: "Television"
"#,
        )
        .unwrap();
        assert_eq!(locale.lookup_medium("television"), "Television");
    }

    #[test]
    fn test_lookup_genre_fallback() {
        let locale = Locale::en_us();
        // Unknown key → title-case first word + spaces
        assert_eq!(locale.lookup_genre("unknown-key"), "Unknown key");
    }

    #[test]
    fn test_en_us_locale_uses_embedded_vocab() {
        let locale = Locale::en_us();

        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
        assert_eq!(locale.lookup_medium("audio-cd"), "Audio CD");
    }

    #[test]
    fn test_from_yaml_str_inherits_embedded_vocab_defaults() {
        let locale = Locale::from_yaml_str("locale: en-US\n").unwrap();

        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
    }

    #[test]
    fn test_partial_genre_vocab_override_preserves_medium_defaults() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  genre:
    phd-thesis: "Doctoral dissertation"
"#,
        )
        .unwrap();

        assert_eq!(locale.lookup_genre("phd-thesis"), "Doctoral dissertation");
        assert_eq!(locale.lookup_medium("audio-cd"), "Audio CD");
    }

    #[test]
    fn test_partial_medium_vocab_override_preserves_genre_defaults() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  medium:
    television: "Broadcast television"
"#,
        )
        .unwrap();

        assert_eq!(locale.lookup_medium("television"), "Broadcast television");
        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
    }

    #[test]
    fn test_kebab_to_display_single_word() {
        assert_eq!(kebab_to_display("video"), "Video");
    }

    #[test]
    fn test_kebab_to_display_multiple_words() {
        assert_eq!(kebab_to_display("phd-thesis"), "Phd thesis");
        assert_eq!(kebab_to_display("audio-cd"), "Audio cd");
    }
}
