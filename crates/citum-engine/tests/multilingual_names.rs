#![allow(missing_docs)]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use citum_engine::values::resolve_multilingual_name;
use citum_schema::options::MultilingualMode;
use citum_schema::reference::MultilingualString;
use citum_schema::reference::contributor::{Contributor, MultilingualName, StructuredName};
use rstest::rstest;
use std::collections::HashMap;

#[test]
fn primary_mode_returns_original_family_name() {
    let original = StructuredName {
        family: MultilingualString::Simple("Kuhn".to_string()),
        given: MultilingualString::Simple("Thomas".to_string()),
        ..Default::default()
    };

    let m = MultilingualName {
        original: original.clone(),
        lang: None,
        transliterations: HashMap::new(),
        translations: HashMap::new(),
    };

    let contributor = Contributor::Multilingual(m);

    let resolved = resolve_multilingual_name(
        &contributor,
        Some(&MultilingualMode::Primary),
        None,
        None,
        "en-US",
    );

    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].family, Some("Kuhn".to_string()));
}

/// Preferred scripts to try and the expected resolved family name.
#[rstest]
#[case::exact_match(&["ru-Latn-pinyin", "ru-Latn"], "Pǔxīkīn")]
#[case::fallback_to_second(&["non-existent", "ru-Latn"], "Pushkin")]
fn given_a_priority_list_when_resolving_then_the_highest_matching_script_wins(
    #[case] preferred: &[&str],
    #[case] expected_family: &str,
) {
    let mut transliterations = HashMap::new();
    transliterations.insert(
        "ru-Latn".to_string(),
        StructuredName {
            family: MultilingualString::Simple("Pushkin".to_string()),
            given: MultilingualString::Simple("Aleksandr".to_string()),
            ..Default::default()
        },
    );
    transliterations.insert(
        "ru-Latn-pinyin".to_string(),
        StructuredName {
            family: MultilingualString::Simple("Pǔxīkīn".to_string()),
            given: MultilingualString::Simple("Ālièshāndé".to_string()),
            ..Default::default()
        },
    );

    let contributor = Contributor::Multilingual(MultilingualName {
        original: StructuredName {
            family: MultilingualString::Simple("Пушкин".to_string()),
            given: MultilingualString::Simple("Александр".to_string()),
            ..Default::default()
        },
        lang: None,
        transliterations,
        translations: HashMap::new(),
    });

    let preferred_owned: Vec<String> = preferred.iter().map(|s| s.to_string()).collect();
    let resolved = resolve_multilingual_name(
        &contributor,
        Some(&MultilingualMode::Transliterated),
        Some(&preferred_owned),
        None,
        "en-US",
    );

    assert_eq!(resolved[0].family, Some(expected_family.to_string()));
}

#[test]
fn substring_script_preference_matches_containing_transliteration() {
    let mut transliterations = HashMap::new();
    transliterations.insert(
        "ru-Latn-special".to_string(),
        StructuredName {
            family: MultilingualString::Simple("Pushkin-Special".to_string()),
            given: MultilingualString::Simple("Aleksandr".to_string()),
            ..Default::default()
        },
    );

    let contributor = Contributor::Multilingual(MultilingualName {
        original: StructuredName {
            family: MultilingualString::Simple("Пушкин".to_string()),
            given: MultilingualString::Simple("Александр".to_string()),
            ..Default::default()
        },
        lang: None,
        transliterations,
        translations: HashMap::new(),
    });

    // "special" is a substring of "ru-Latn-special"
    let resolved = resolve_multilingual_name(
        &contributor,
        Some(&MultilingualMode::Transliterated),
        Some(&["special".to_string()]),
        None,
        "en-US",
    );

    assert_eq!(resolved[0].family, Some("Pushkin-Special".to_string()));
}
