/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;

use citum_engine::values::resolve_multilingual_name;
use citum_schema::options::MultilingualMode;
use citum_schema::reference::MultilingualString;
use citum_schema::reference::contributor::{Contributor, MultilingualName, StructuredName};
use std::collections::HashMap;

#[test]
fn test_resolve_multilingual_name_primary() {
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

#[test]
fn test_resolve_multilingual_name_transliteration_priority() {
    let original = StructuredName {
        family: MultilingualString::Simple("Пушкин".to_string()),
        given: MultilingualString::Simple("Александр".to_string()),
        ..Default::default()
    };

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

    let m = MultilingualName {
        original,
        lang: None,
        transliterations,
        translations: HashMap::new(),
    };

    let contributor = Contributor::Multilingual(m);

    // Test exact match priority
    let resolved = resolve_multilingual_name(
        &contributor,
        Some(&MultilingualMode::Transliterated),
        Some(&["ru-Latn-pinyin".to_string(), "ru-Latn".to_string()]),
        None,
        "en-US",
    );

    assert_eq!(resolved[0].family, Some("Pǔxīkīn".to_string()));

    // Test fallback to second priority
    let resolved2 = resolve_multilingual_name(
        &contributor,
        Some(&MultilingualMode::Transliterated),
        Some(&["non-existent".to_string(), "ru-Latn".to_string()]),
        None,
        "en-US",
    );

    assert_eq!(resolved2[0].family, Some("Pushkin".to_string()));
}

#[test]
fn test_resolve_multilingual_name_substring_match() {
    let original = StructuredName {
        family: MultilingualString::Simple("Пушкин".to_string()),
        given: MultilingualString::Simple("Александр".to_string()),
        ..Default::default()
    };

    let mut transliterations = HashMap::new();
    transliterations.insert(
        "ru-Latn-special".to_string(),
        StructuredName {
            family: MultilingualString::Simple("Pushkin-Special".to_string()),
            given: MultilingualString::Simple("Aleksandr".to_string()),
            ..Default::default()
        },
    );

    let m = MultilingualName {
        original,
        lang: None,
        transliterations,
        translations: HashMap::new(),
    };

    let contributor = Contributor::Multilingual(m);

    // Should match "ru-Latn-special" because it contains "special"
    let resolved = resolve_multilingual_name(
        &contributor,
        Some(&MultilingualMode::Transliterated),
        Some(&["special".to_string()]),
        None,
        "en-US",
    );

    assert_eq!(resolved[0].family, Some("Pushkin-Special".to_string()));
}
