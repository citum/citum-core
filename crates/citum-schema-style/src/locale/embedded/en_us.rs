/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Hardcoded en-US locale data — role terms, locator terms, archive messages,
//! and the vocab map extracted from the embedded `en-US.yaml` asset.
//!
//! These functions seed [`Locale::en_us`] and provide the fallback baseline
//! every other locale inherits from before applying overrides.

use crate::citation::LocatorType;
use crate::locale::raw;
use crate::locale::types::{
    ContributorTerm, LocatorTerm, MaybeGendered, SimpleTerm, SingularPlural, VocabMap,
};
use crate::template::ContributorRole;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

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

/// Archive hierarchy label messages for the hardcoded en-US locale.
///
/// Only the archive terms are pre-seeded here; all other message lookups fall
/// through to the legacy typed term maps so that the hardcoded `en_us()`
/// constructor stays consistent with the pre-existing test baseline.
pub(crate) fn en_us_archive_messages() -> HashMap<String, String> {
    [
        ("term.archive-collection-label".into(), "collection".into()),
        ("term.archive-series-label".into(), "series".into()),
        (
            "term.archive-box-label".into(),
            ".match {$count :plural}\nwhen one {box}\nwhen * {boxes}".into(),
        ),
        (
            "term.archive-folder-label".into(),
            ".match {$count :plural}\nwhen one {folder}\nwhen * {folders}".into(),
        ),
        (
            "term.archive-item-label".into(),
            ".match {$count :plural}\nwhen one {item}\nwhen * {items}".into(),
        ),
    ]
    .into()
}

/// Curated en-US genre and medium labels from the embedded locale asset.
pub(crate) fn embedded_en_us_vocab() -> &'static VocabMap {
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

/// Extract English (US) role terms.
pub(crate) fn en_us_role_terms() -> HashMap<ContributorRole, ContributorTerm> {
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
pub(crate) fn en_us_locator_terms() -> HashMap<LocatorType, LocatorTerm> {
    let mut locators = HashMap::new();
    locators.insert(
        LocatorType::Page,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: MaybeGendered::Plain("page".into()),
                plural: MaybeGendered::Plain("pages".into()),
            }),
            short: Some(SingularPlural {
                singular: MaybeGendered::Plain("p.".into()),
                plural: MaybeGendered::Plain("pp.".into()),
            }),
            symbol: None,
            gender: None,
        },
    );

    locators.insert(
        LocatorType::Chapter,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: MaybeGendered::Plain("chapter".into()),
                plural: MaybeGendered::Plain("chapters".into()),
            }),
            short: Some(SingularPlural {
                singular: MaybeGendered::Plain("ch.".into()),
                plural: MaybeGendered::Plain("chs.".into()),
            }),
            symbol: None,
            gender: None,
        },
    );

    locators.insert(
        LocatorType::Volume,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: MaybeGendered::Plain("volume".into()),
                plural: MaybeGendered::Plain("volumes".into()),
            }),
            short: Some(SingularPlural {
                singular: MaybeGendered::Plain("vol.".into()),
                plural: MaybeGendered::Plain("vols.".into()),
            }),
            symbol: None,
            gender: None,
        },
    );

    locators.insert(
        LocatorType::Section,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: MaybeGendered::Plain("section".into()),
                plural: MaybeGendered::Plain("sections".into()),
            }),
            short: Some(SingularPlural {
                singular: MaybeGendered::Plain("sec.".into()),
                plural: MaybeGendered::Plain("secs.".into()),
            }),
            symbol: Some(SingularPlural {
                singular: MaybeGendered::Plain("§".into()),
                plural: MaybeGendered::Plain("§§".into()),
            }),
            gender: None,
        },
    );

    locators.insert(
        LocatorType::Part,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: MaybeGendered::Plain("part".into()),
                plural: MaybeGendered::Plain("parts".into()),
            }),
            short: Some(SingularPlural {
                singular: MaybeGendered::Plain("pt.".into()),
                plural: MaybeGendered::Plain("pts.".into()),
            }),
            symbol: None,
            gender: None,
        },
    );

    locators.insert(
        LocatorType::Supplement,
        LocatorTerm {
            long: Some(SingularPlural {
                singular: MaybeGendered::Plain("supplement".into()),
                plural: MaybeGendered::Plain("supplements".into()),
            }),
            short: Some(SingularPlural {
                singular: MaybeGendered::Plain("suppl.".into()),
                plural: MaybeGendered::Plain("suppls.".into()),
            }),
            symbol: None,
            gender: None,
        },
    );

    locators
}
