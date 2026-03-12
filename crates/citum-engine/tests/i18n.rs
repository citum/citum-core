/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::Processor;
use citum_engine::values::{
    effective_field_language, effective_item_language, resolve_multilingual_string,
};
use citum_schema::{
    BibliographySpec, CitationSpec, LocalizedTemplateSpec, Style, StyleInfo,
    options::{Config, MultilingualConfig, MultilingualMode, Processing, TitleRendering},
    reference::contributor::{Contributor, MultilingualName, StructuredName},
    reference::types::{
        Collection, CollectionComponent, MultilingualComplex, MultilingualString, Title,
    },
    reference::{EdtfString, InputReference, Monograph, MonographType, Parent},
};
use std::collections::HashMap;

// --- Helper Functions ---

fn build_ml_style(name_mode: MultilingualMode, preferred_script: Option<String>) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Multilingual Test".to_string()),
            id: Some("ml-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            multilingual: Some(MultilingualConfig {
                name_mode: Some(name_mode),
                preferred_script,
                ..Default::default()
            }),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Short),
                citum_schema::tc_date!(Issued, Year),
            ]),
            delimiter: Some(", ".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// --- Multilingual Resolution Tests ---

#[test]
fn test_resolve_simple_string() {
    let simple = MultilingualString::Simple("Hello".to_string());
    let result = resolve_multilingual_string(&simple, None, None, None, "en");
    assert_eq!(result, "Hello");
}

#[test]
fn test_resolve_primary_mode() {
    let complex = MultilingualComplex {
        original: "战争与和平".to_string(),
        lang: Some("zh".to_string()),
        transliterations: {
            let mut map = HashMap::new();
            map.insert(
                "zh-Latn-pinyin".to_string(),
                "Zhànzhēng yǔ Hépíng".to_string(),
            );
            map
        },
        translations: {
            let mut map = HashMap::new();
            map.insert("en".to_string(), "War and Peace".to_string());
            map
        },
    };

    let ml_string = MultilingualString::Complex(complex);
    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Primary),
        None,
        None,
        "en",
    );

    assert_eq!(result, "战争与和平");
}

#[test]
fn test_resolve_transliterated_exact_match() {
    let complex = MultilingualComplex {
        original: "東京".to_string(),
        lang: Some("ja".to_string()),
        transliterations: {
            let mut map = HashMap::new();
            map.insert("ja-Latn-hepburn".to_string(), "Tōkyō".to_string());
            map.insert("ja-Latn-kunrei".to_string(), "Tôkyô".to_string());
            map
        },
        translations: {
            let mut map = HashMap::new();
            map.insert("en".to_string(), "Tokyo".to_string());
            map
        },
    };

    let ml_string = MultilingualString::Complex(complex);

    // Exact match for hepburn
    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Transliterated),
        Some(&["ja-Latn-hepburn".to_string()]),
        None,
        "en",
    );
    assert_eq!(result, "Tōkyō");
}

#[test]
fn test_resolve_transliterated_prefix_match() {
    let complex = MultilingualComplex {
        original: "東京".to_string(),
        lang: Some("ja".to_string()),
        transliterations: {
            let mut map = HashMap::new();
            map.insert("ja-Latn-hepburn".to_string(), "Tōkyō".to_string());
            map
        },
        translations: HashMap::new(),
    };

    let ml_string = MultilingualString::Complex(complex);

    // Prefix match: "ja-Latn" should match "ja-Latn-hepburn"
    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Transliterated),
        Some(&["ja-Latn".to_string()]),
        None,
        "en",
    );
    assert_eq!(result, "Tōkyō");
}

#[test]
fn test_resolve_transliterated_fallback_to_original() {
    let complex = MultilingualComplex {
        original: "东京".to_string(),
        lang: Some("zh".to_string()),
        transliterations: HashMap::new(), // No transliterations available
        translations: HashMap::new(),
    };

    let ml_string = MultilingualString::Complex(complex);

    // Should fallback to original
    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Transliterated),
        None,
        Some(&"Latn".to_string()),
        "en",
    );
    assert_eq!(result, "东京");
}

#[test]
fn test_resolve_translated_mode() {
    let complex = MultilingualComplex {
        original: "战争与和平".to_string(),
        lang: Some("zh".to_string()),
        transliterations: HashMap::new(),
        translations: {
            let mut map = HashMap::new();
            map.insert("en".to_string(), "War and Peace".to_string());
            map.insert("fr".to_string(), "Guerre et Paix".to_string());
            map
        },
    };

    let ml_string = MultilingualString::Complex(complex);

    // English translation
    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Translated),
        None,
        None,
        "en",
    );
    assert_eq!(result, "War and Peace");

    // French translation
    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Translated),
        None,
        None,
        "fr",
    );
    assert_eq!(result, "Guerre et Paix");
}

#[test]
fn test_resolve_combined_mode() {
    let complex = MultilingualComplex {
        original: "战争与和平".to_string(),
        lang: Some("zh".to_string()),
        transliterations: {
            let mut map = HashMap::new();
            map.insert(
                "zh-Latn-pinyin".to_string(),
                "Zhànzhēng yǔ Hépíng".to_string(),
            );
            map
        },
        translations: {
            let mut map = HashMap::new();
            map.insert("en".to_string(), "War and Peace".to_string());
            map
        },
    };

    let ml_string = MultilingualString::Complex(complex);

    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Combined),
        Some(&["zh-Latn-pinyin".to_string()]),
        None,
        "en",
    );

    assert_eq!(result, "Zhànzhēng yǔ Hépíng [War and Peace]");
}

#[test]
fn test_resolve_combined_fallback() {
    let complex = MultilingualComplex {
        original: "东京".to_string(),
        lang: Some("zh".to_string()),
        transliterations: HashMap::new(),
        translations: {
            let mut map = HashMap::new();
            map.insert("en".to_string(), "Tokyo".to_string());
            map
        },
    };

    let ml_string = MultilingualString::Complex(complex);

    // No transliteration, should use original + translation
    let result = resolve_multilingual_string(
        &ml_string,
        Some(&MultilingualMode::Combined),
        None,
        Some(&"Latn".to_string()),
        "en",
    );

    assert_eq!(result, "东京 [Tokyo]");
}

#[test]
fn test_resolve_multilingual_name_simple() {
    let name = Contributor::StructuredName(StructuredName {
        given: MultilingualString::Simple("John".to_string()),
        family: MultilingualString::Simple("Smith".to_string()),
        suffix: None,
        dropping_particle: None,
        non_dropping_particle: None,
    });

    let result = citum_engine::values::resolve_multilingual_name(&name, None, None, None, "en");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].given, Some("John".to_string()));
    assert_eq!(result[0].family, Some("Smith".to_string()));
}

#[test]
fn test_resolve_multilingual_name_transliterated() {
    let name = Contributor::Multilingual(MultilingualName {
        original: StructuredName {
            given: MultilingualString::Simple("Лев".to_string()),
            family: MultilingualString::Simple("Толстой".to_string()),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: None,
        },
        lang: Some("ru".to_string()),
        transliterations: {
            let mut map = HashMap::new();
            map.insert(
                "Latn".to_string(),
                StructuredName {
                    given: MultilingualString::Simple("Leo".to_string()),
                    family: MultilingualString::Simple("Tolstoy".to_string()),
                    suffix: None,
                    dropping_particle: None,
                    non_dropping_particle: None,
                },
            );
            map
        },
        translations: HashMap::new(),
    });

    let result = citum_engine::values::resolve_multilingual_name(
        &name,
        Some(&MultilingualMode::Transliterated),
        Some(&["Latn".to_string()]),
        None,
        "en",
    );

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].given, Some("Leo".to_string()));
    assert_eq!(result[0].family, Some("Tolstoy".to_string()));
}

#[test]
fn test_resolve_multilingual_name_prefix_match() {
    let name = Contributor::Multilingual(MultilingualName {
        original: StructuredName {
            given: MultilingualString::Simple("Лев".to_string()),
            family: MultilingualString::Simple("Толстой".to_string()),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: None,
        },
        lang: Some("ru".to_string()),
        transliterations: {
            let mut map = HashMap::new();
            map.insert(
                "ru-Latn-alalc97".to_string(),
                StructuredName {
                    given: MultilingualString::Simple("Lev".to_string()),
                    family: MultilingualString::Simple("Tolstoi".to_string()),
                    suffix: None,
                    dropping_particle: None,
                    non_dropping_particle: None,
                },
            );
            map
        },
        translations: HashMap::new(),
    });

    // Prefix "Latn" should match "ru-Latn-alalc97"
    let result = citum_engine::values::resolve_multilingual_name(
        &name,
        Some(&MultilingualMode::Transliterated),
        Some(&["Latn".to_string()]),
        None,
        "en",
    );

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].given, Some("Lev".to_string()));
    assert_eq!(result[0].family, Some("Tolstoi".to_string()));
}

#[test]
fn test_resolve_multilingual_name_fallback_to_original() {
    let name = Contributor::Multilingual(MultilingualName {
        original: StructuredName {
            given: MultilingualString::Simple("Лев".to_string()),
            family: MultilingualString::Simple("Толстой".to_string()),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: None,
        },
        lang: Some("ru".to_string()),
        transliterations: HashMap::new(),
        translations: HashMap::new(),
    });

    // No transliterations available, should use original
    let result = citum_engine::values::resolve_multilingual_name(
        &name,
        Some(&MultilingualMode::Transliterated),
        Some(&["Latn".to_string()]),
        None,
        "en",
    );

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].given, Some("Лев".to_string()));
    assert_eq!(result[0].family, Some("Толстой".to_string()));
}

#[test]
fn test_multilingual_config_deserialization() {
    let yaml = r#"
multilingual:
  title-mode: "transliterated"
  name-mode: "combined"
  preferred-script: "Latn"
  scripts:
    cjk:
      use-native-ordering: true
      delimiter: ""
"#;

    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let mlt = config.multilingual.unwrap();

    assert_eq!(mlt.title_mode, Some(MultilingualMode::Transliterated));
    assert_eq!(mlt.name_mode, Some(MultilingualMode::Combined));
    assert_eq!(mlt.preferred_script, Some("Latn".to_string()));

    let cjk_config = mlt.scripts.get("cjk").unwrap();
    assert!(cjk_config.use_native_ordering);
    assert_eq!(cjk_config.delimiter, Some("".to_string()));
}

// --- Multilingual Rendering Tests ---

#[test]
fn test_multilingual_rendering_original() {
    let style = build_ml_style(MultilingualMode::Primary, None);

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "item1".to_string(),
        make_multilingual_book(
            "item1", "東京", "太郎", "ja", "ja-Latn", "Tokyo", "Taro", 2020, "Title",
        ),
    );

    let processor = Processor::new(style, bib);
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!("item1"))
            .unwrap(),
        "東京, 2020"
    );
}

#[test]
fn test_multilingual_rendering_transliterated() {
    let style = build_ml_style(MultilingualMode::Transliterated, Some("Latn".to_string()));

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "item1".to_string(),
        make_multilingual_book(
            "item1", "東京", "太郎", "ja", "ja-Latn", "Tokyo", "Taro", 2020, "Title",
        ),
    );

    let processor = Processor::new(style, bib);
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!("item1"))
            .unwrap(),
        "Tokyo, 2020"
    );
}

#[test]
fn test_multilingual_rendering_combined() {
    let style = build_ml_style(MultilingualMode::Combined, Some("Latn".to_string()));

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "item1".to_string(),
        make_multilingual_book(
            "item1", "東京", "太郎", "ja", "ja-Latn", "Tokyo", "Taro", 2020, "Title",
        ),
    );

    let processor = Processor::new(style, bib);
    // Note: Combined mode for names is currently transliterated only in resolve_multilingual_name
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!("item1"))
            .unwrap(),
        "Tokyo, 2020"
    );
}

#[test]
fn test_multilingual_rendering_numeric_integral_translated() {
    let mut style = build_ml_style(MultilingualMode::Translated, None);
    style.options.as_mut().unwrap().processing = Some(Processing::Numeric);
    style.citation.as_mut().unwrap().template =
        Some(vec![citum_schema::tc_contributor!(Author, Short)]);

    let mut bib = indexmap::IndexMap::new();
    let mut translations = HashMap::new();
    translations.insert(
        "en-US".to_string(),
        StructuredName {
            family: MultilingualString::Simple("Tolstoy".to_string()),
            given: MultilingualString::Simple("Leo".to_string()),
            ..Default::default()
        },
    );

    bib.insert(
        "item1".to_string(),
        citum_schema::reference::InputReference::Monograph(Box::new(
            citum_schema::reference::Monograph {
                id: Some("item1".to_string()),
                r#type: citum_schema::reference::MonographType::Book,
                title: Some(citum_schema::reference::Title::Single(
                    "War and Peace".to_string(),
                )),
                container_title: None,
                author: Some(Contributor::Multilingual(MultilingualName {
                    original: StructuredName {
                        family: MultilingualString::Simple("Толстой".to_string()),
                        given: MultilingualString::Simple("Лев".to_string()),
                        ..Default::default()
                    },
                    lang: Some("ru".to_string()),
                    transliterations: HashMap::new(),
                    translations,
                })),
                editor: None,
                translator: None,
                recipient: None,
                interviewer: None,
                issued: citum_schema::reference::EdtfString("1869".to_string()),
                publisher: None,
                url: None,
                accessed: None,
                language: None,
                field_languages: Default::default(),
                note: None,
                isbn: None,
                doi: None,
                edition: None,
                report_number: None,
                collection_number: None,
                genre: None,
                medium: None,
                archive: None,
                archive_location: None,
                keywords: None,
                original_date: None,
                original_title: None,
                ads_bibcode: None,
            },
        )),
    );

    let processor = Processor::new(style, bib);
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!(
                "item1",
                mode = citum_schema::citation::CitationMode::Integral
            ))
            .unwrap(),
        "Tolstoy [1]"
    );
}

#[test]
fn test_effective_field_language_prefers_field_languages() {
    let reference = InputReference::Monograph(Box::new(Monograph {
        id: Some("item1".to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Multilingual(MultilingualComplex {
            original: "Titel".to_string(),
            lang: Some("de".to_string()),
            transliterations: HashMap::new(),
            translations: HashMap::new(),
        })),
        container_title: None,
        author: None,
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
        issued: EdtfString("2024".to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: Some("fr".to_string()),
        field_languages: HashMap::from([("title".to_string(), "en".to_string())]),
        note: None,
        isbn: None,
        doi: None,
        edition: None,
        report_number: None,
        collection_number: None,
        genre: None,
        medium: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original_date: None,
        original_title: None,
        ads_bibcode: None,
    }));

    assert_eq!(
        effective_field_language(&reference, "title", reference.title().as_ref()),
        Some("en".to_string())
    );
}

#[test]
fn test_effective_item_language_falls_back_to_multilingual_title_lang() {
    let reference = InputReference::Monograph(Box::new(Monograph {
        id: Some("item1".to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Multilingual(MultilingualComplex {
            original: "東京".to_string(),
            lang: Some("ja".to_string()),
            transliterations: HashMap::new(),
            translations: HashMap::new(),
        })),
        container_title: None,
        author: None,
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
        issued: EdtfString("2024".to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: None,
        field_languages: HashMap::new(),
        note: None,
        isbn: None,
        doi: None,
        edition: None,
        report_number: None,
        collection_number: None,
        genre: None,
        medium: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original_date: None,
        original_title: None,
        ads_bibcode: None,
    }));

    assert_eq!(effective_item_language(&reference), Some("ja".to_string()));
}

#[test]
fn test_citation_localized_template_selection_uses_item_language() {
    let style = Style {
        info: StyleInfo {
            title: Some("Localized Citation".to_string()),
            ..Default::default()
        },
        citation: Some(CitationSpec {
            template: Some(vec![citum_schema::tc_variable!(Note)]),
            locales: Some(vec![
                LocalizedTemplateSpec {
                    locale: Some(vec!["de".to_string()]),
                    default: None,
                    template: vec![citum_schema::tc_variable!(Publisher)],
                },
                LocalizedTemplateSpec {
                    locale: None,
                    default: Some(true),
                    template: vec![citum_schema::tc_variable!(Note)],
                },
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert(
        "de-item".to_string(),
        InputReference::Monograph(Box::new(Monograph {
            id: Some("de-item".to_string()),
            r#type: MonographType::Book,
            title: Some(Title::Single("Titel".to_string())),
            container_title: None,
            author: None,
            editor: None,
            translator: None,
            recipient: None,
            interviewer: None,
            issued: EdtfString("2024".to_string()),
            publisher: Some(Contributor::SimpleName(
                citum_schema::reference::SimpleName {
                    name: MultilingualString::Simple("Verlag".to_string()),
                    location: None,
                },
            )),
            url: None,
            accessed: None,
            language: Some("de-AT".to_string()),
            field_languages: HashMap::new(),
            note: Some("fallback".to_string()),
            isbn: None,
            doi: None,
            edition: None,
            report_number: None,
            collection_number: None,
            genre: None,
            medium: None,
            archive: None,
            archive_location: None,
            keywords: None,
            original_date: None,
            original_title: None,
            ads_bibcode: None,
        })),
    );
    bibliography.insert(
        "fr-item".to_string(),
        InputReference::Monograph(Box::new(Monograph {
            id: Some("fr-item".to_string()),
            r#type: MonographType::Book,
            title: Some(Title::Single("Titre".to_string())),
            container_title: None,
            author: None,
            editor: None,
            translator: None,
            recipient: None,
            interviewer: None,
            issued: EdtfString("2024".to_string()),
            publisher: Some(Contributor::SimpleName(
                citum_schema::reference::SimpleName {
                    name: MultilingualString::Simple("Editeur".to_string()),
                    location: None,
                },
            )),
            url: None,
            accessed: None,
            language: Some("fr".to_string()),
            field_languages: HashMap::new(),
            note: Some("fallback".to_string()),
            isbn: None,
            doi: None,
            edition: None,
            report_number: None,
            collection_number: None,
            genre: None,
            medium: None,
            archive: None,
            archive_location: None,
            keywords: None,
            original_date: None,
            original_title: None,
            ads_bibcode: None,
        })),
    );

    let processor = Processor::new(style, bibliography);
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!("de-item"))
            .unwrap(),
        "Verlag"
    );
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!("fr-item"))
            .unwrap(),
        "fallback"
    );
}

#[test]
fn test_bibliography_localized_template_selection_uses_multilingual_title_lang() {
    let style = Style {
        info: StyleInfo {
            title: Some("Localized Bibliography".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![citum_schema::tc_variable!(Note)]),
            locales: Some(vec![
                LocalizedTemplateSpec {
                    locale: Some(vec!["ja".to_string()]),
                    default: None,
                    template: vec![citum_schema::tc_title!(Primary)],
                },
                LocalizedTemplateSpec {
                    locale: None,
                    default: Some(true),
                    template: vec![citum_schema::tc_variable!(Note)],
                },
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert(
        "item1".to_string(),
        InputReference::Monograph(Box::new(Monograph {
            id: Some("item1".to_string()),
            r#type: MonographType::Book,
            title: Some(Title::Multilingual(MultilingualComplex {
                original: "東京".to_string(),
                lang: Some("ja".to_string()),
                transliterations: HashMap::new(),
                translations: HashMap::new(),
            })),
            container_title: None,
            author: None,
            editor: None,
            translator: None,
            recipient: None,
            interviewer: None,
            issued: EdtfString("2024".to_string()),
            publisher: None,
            url: None,
            accessed: None,
            language: None,
            field_languages: HashMap::new(),
            note: Some("fallback".to_string()),
            isbn: None,
            doi: None,
            edition: None,
            report_number: None,
            collection_number: None,
            genre: None,
            medium: None,
            archive: None,
            archive_location: None,
            keywords: None,
            original_date: None,
            original_title: None,
            ads_bibcode: None,
        })),
    );

    let processor = Processor::new(style, bibliography);
    assert_eq!(processor.render_bibliography(), "東京");
}

#[test]
fn test_mixed_language_title_formatting_uses_field_languages() {
    let style = Style {
        info: StyleInfo {
            title: Some("Mixed Language Titles".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            titles: Some(citum_schema::options::TitlesConfig {
                component: Some(TitleRendering {
                    quote: Some(true),
                    locale_overrides: Some(HashMap::from([(
                        "de".to_string(),
                        TitleRendering {
                            quote: Some(false),
                            emph: Some(true),
                            ..Default::default()
                        },
                    )])),
                    ..Default::default()
                }),
                container_monograph: Some(TitleRendering {
                    emph: Some(true),
                    locale_overrides: Some(HashMap::from([(
                        "en".to_string(),
                        TitleRendering {
                            emph: Some(false),
                            quote: Some(true),
                            ..Default::default()
                        },
                    )])),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_title!(Primary),
                citum_schema::tc_title!(ParentMonograph),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let reference = InputReference::CollectionComponent(Box::new(CollectionComponent {
        id: Some("chapter-1".to_string()),
        r#type: citum_schema::reference::MonographComponentType::Chapter,
        title: Some(Title::Single("English Article".to_string())),
        author: None,
        translator: None,
        issued: EdtfString("2024".to_string()),
        parent: Parent::Embedded(Collection {
            id: None,
            r#type: citum_schema::reference::CollectionType::EditedBook,
            title: Some(Title::Single("Deutscher Sammelband".to_string())),
            short_title: None,
            editor: None,
            translator: None,
            issued: EdtfString("2024".to_string()),
            publisher: None,
            collection_number: None,
            url: None,
            accessed: None,
            language: Some("de".to_string()),
            field_languages: HashMap::new(),
            note: None,
            isbn: None,
            keywords: None,
        }),
        pages: None,
        url: None,
        accessed: None,
        language: Some("de".to_string()),
        field_languages: HashMap::from([
            ("title".to_string(), "en".to_string()),
            ("parent-monograph.title".to_string(), "de".to_string()),
        ]),
        note: None,
        doi: None,
        genre: None,
        medium: None,
        keywords: None,
    }));

    let bibliography = indexmap::IndexMap::from([("chapter-1".to_string(), reference)]);
    let processor = Processor::new(style, bibliography);

    assert_eq!(
        processor.render_bibliography(),
        "“English Article”. _Deutscher Sammelband_"
    );
}
