#![allow(missing_docs)]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
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
use rstest::rstest;
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

fn given_a_simple_string_when_resolved_then_the_original_text_is_returned() {
    let simple = MultilingualString::Simple("Hello".to_string());
    let result = resolve_multilingual_string(&simple, None, None, None, "en");
    assert_eq!(result, "Hello");
}

fn given_primary_mode_when_resolving_a_multilingual_title_then_the_original_script_is_returned() {
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

fn given_an_exact_transliteration_match_when_resolving_then_that_transliteration_is_used() {
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

fn given_a_transliteration_prefix_match_when_resolving_then_the_matching_transliteration_is_used() {
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

fn given_no_transliteration_when_transliterated_mode_is_requested_then_the_original_text_is_used() {
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

fn given_translated_mode_when_resolving_then_the_requested_locale_translation_is_used() {
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

fn given_combined_mode_when_transliteration_and_translation_exist_then_both_are_combined() {
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

fn given_combined_mode_without_transliteration_when_resolving_then_original_and_translation_are_combined()
 {
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

fn given_a_simple_structured_name_when_resolved_then_the_name_parts_are_preserved() {
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

fn given_a_multilingual_name_with_requested_script_when_resolved_then_the_transliterated_name_is_used()
 {
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

fn given_a_multilingual_name_with_a_script_prefix_match_when_resolved_then_the_matching_transliteration_is_used()
 {
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

fn given_a_multilingual_name_without_transliterations_when_resolved_then_the_original_name_is_used()
{
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

fn given_multilingual_yaml_options_when_deserialized_then_the_config_keeps_script_preferences() {
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
    assert_eq!(cjk_config.delimiter, Some(String::new()));
}

// --- Multilingual Rendering Tests ---

#[rstest]
#[case::primary(MultilingualMode::Primary, None, "東京, 2020")]
#[case::transliterated(
    MultilingualMode::Transliterated,
    Some("Latn".to_string()),
    "Tokyo, 2020"
)]
#[case::combined(
    MultilingualMode::Combined,
    Some("Latn".to_string()),
    "Tokyo, 2020"
)]
#[rstest]
fn given_a_multilingual_author_when_rendering_a_citation_then_the_selected_name_mode_controls_the_output(
    #[case] mode: MultilingualMode,
    #[case] preferred_script: Option<String>,
    #[case] expected: &str,
) {
    announce_behavior(&format!(
        "A multilingual citation should render author names according to {:?} mode{}.",
        mode,
        preferred_script
            .as_deref()
            .map(|script| format!(" with preferred script {script}"))
            .unwrap_or_default()
    ));

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "item1".to_string(),
        make_multilingual_book(common::MultilingualBookParams {
            id: "item1",
            original_family: "東京",
            original_given: "太郎",
            lang: "ja",
            translit_script: "ja-Latn",
            translit_family: "Tokyo",
            translit_given: "Taro",
            year: 2020,
            title: "Title",
        }),
    );

    // Given a multilingual citation style and a single Japanese reference.
    let style = build_ml_style(mode, preferred_script);
    let processor = Processor::new(style, bib);

    // When the citation is rendered.
    let rendered = processor
        .process_citation(&citum_schema::cite!("item1"))
        .unwrap();

    // Then the selected name mode controls the visible author text.
    assert_eq!(rendered, expected);
}

fn given_translated_numeric_integral_citations_when_rendered_then_the_translated_name_is_used_as_the_anchor()
 {
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

fn given_field_language_overrides_when_resolving_the_effective_field_language_then_the_field_override_wins()
 {
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

fn given_no_item_language_when_resolving_the_effective_item_language_then_the_multilingual_title_language_is_used()
 {
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

#[allow(clippy::too_many_lines)] // test functions naturally exceed 100 lines
fn given_localized_citation_templates_when_the_item_language_matches_then_the_locale_specific_template_is_selected()
 {
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

fn given_localized_bibliography_templates_when_only_the_multilingual_title_has_a_language_then_that_language_still_selects_the_template()
 {
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

fn given_mixed_language_titles_when_rendering_the_bibliography_then_field_languages_drive_the_title_formatting_overrides()
 {
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

mod string_resolution {
    use super::announce_behavior;

    #[test]
    fn simple_strings_return_the_original_text() {
        announce_behavior(
            "A simple multilingual string should resolve to its original text unchanged.",
        );
        super::given_a_simple_string_when_resolved_then_the_original_text_is_returned();
    }

    #[test]
    fn primary_mode_uses_the_original_script() {
        announce_behavior(
            "Primary multilingual mode should keep the original script instead of transliterating or translating.",
        );
        super::given_primary_mode_when_resolving_a_multilingual_title_then_the_original_script_is_returned();
    }

    #[test]
    fn exact_transliteration_matches_are_used() {
        announce_behavior(
            "An exact transliteration script match should use that transliterated value.",
        );
        super::given_an_exact_transliteration_match_when_resolving_then_that_transliteration_is_used();
    }

    #[test]
    fn transliteration_prefix_matches_are_used() {
        announce_behavior(
            "A transliteration script prefix should match the more specific transliteration variant.",
        );
        super::given_a_transliteration_prefix_match_when_resolving_then_the_matching_transliteration_is_used();
    }

    #[test]
    fn transliterated_mode_falls_back_to_the_original_text() {
        announce_behavior(
            "If no transliteration is available, transliterated mode should fall back to the original text.",
        );
        super::given_no_transliteration_when_transliterated_mode_is_requested_then_the_original_text_is_used();
    }

    #[test]
    fn translated_mode_uses_the_requested_locale_translation() {
        announce_behavior(
            "Translated mode should select the translation that matches the requested locale.",
        );
        super::given_translated_mode_when_resolving_then_the_requested_locale_translation_is_used();
    }

    #[test]
    fn combined_mode_uses_transliteration_and_translation_when_both_exist() {
        announce_behavior(
            "Combined mode should join the transliteration with the locale translation when both exist.",
        );
        super::given_combined_mode_when_transliteration_and_translation_exist_then_both_are_combined();
    }

    #[test]
    fn combined_mode_falls_back_to_original_plus_translation() {
        announce_behavior(
            "Combined mode should fall back to original text plus translation when no transliteration exists.",
        );
        super::given_combined_mode_without_transliteration_when_resolving_then_original_and_translation_are_combined();
    }
}

mod name_resolution {
    use super::announce_behavior;

    #[test]
    fn simple_structured_names_keep_their_name_parts() {
        announce_behavior(
            "A plain structured name should resolve without changing its family or given parts.",
        );
        super::given_a_simple_structured_name_when_resolved_then_the_name_parts_are_preserved();
    }

    #[test]
    fn requested_scripts_use_the_matching_transliterated_name() {
        announce_behavior(
            "Requested scripts should select the matching transliterated contributor name.",
        );
        super::given_a_multilingual_name_with_requested_script_when_resolved_then_the_transliterated_name_is_used();
    }

    #[test]
    fn script_prefixes_match_transliterated_names() {
        announce_behavior(
            "Script-prefix matching should work for multilingual contributor transliterations as well.",
        );
        super::given_a_multilingual_name_with_a_script_prefix_match_when_resolved_then_the_matching_transliteration_is_used();
    }

    #[test]
    fn missing_transliterations_fall_back_to_the_original_name() {
        announce_behavior(
            "Contributor names without transliterations should fall back to the original-script name.",
        );
        super::given_a_multilingual_name_without_transliterations_when_resolved_then_the_original_name_is_used();
    }
}

mod multilingual_rendering {
    use super::announce_behavior;

    #[test]
    fn translated_numeric_integral_citations_use_the_translated_anchor_name() {
        announce_behavior(
            "Translated numeric integral citations should use the translated author name as the narrative anchor.",
        );
        super::given_translated_numeric_integral_citations_when_rendered_then_the_translated_name_is_used_as_the_anchor();
    }

    #[test]
    fn field_language_overrides_win_for_effective_field_language() {
        announce_behavior(
            "A field-specific language override should win when computing the effective field language.",
        );
        super::given_field_language_overrides_when_resolving_the_effective_field_language_then_the_field_override_wins();
    }

    #[test]
    fn multilingual_title_languages_become_the_effective_item_language() {
        announce_behavior(
            "If an item has no top-level language, the multilingual title language should become the effective item language.",
        );
        super::given_no_item_language_when_resolving_the_effective_item_language_then_the_multilingual_title_language_is_used();
    }

    #[test]
    fn localized_citation_templates_follow_the_item_language() {
        announce_behavior(
            "Localized citation templates should follow the resolved item language when selecting a template.",
        );
        super::given_localized_citation_templates_when_the_item_language_matches_then_the_locale_specific_template_is_selected();
    }

    #[test]
    fn localized_bibliography_templates_can_follow_the_multilingual_title_language() {
        announce_behavior(
            "A multilingual title language should be enough to select a localized bibliography template.",
        );
        super::given_localized_bibliography_templates_when_only_the_multilingual_title_has_a_language_then_that_language_still_selects_the_template();
    }

    #[test]
    fn field_languages_drive_mixed_language_title_formatting() {
        announce_behavior(
            "Mixed-language titles should format each field according to its field-language override.",
        );
        super::given_mixed_language_titles_when_rendering_the_bibliography_then_field_languages_drive_the_title_formatting_overrides();
    }
}

mod config {
    use super::announce_behavior;

    #[test]
    fn multilingual_yaml_options_keep_script_preferences() {
        announce_behavior(
            "Deserializing multilingual YAML options should preserve script preferences and mode settings.",
        );
        super::given_multilingual_yaml_options_when_deserialized_then_the_config_keeps_script_preferences();
    }
}
