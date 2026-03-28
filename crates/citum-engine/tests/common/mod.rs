#![allow(missing_docs, reason = "test")]
/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/
#![allow(dead_code, reason = "test helpers")]

use std::{fs, path::PathBuf};

use citum_engine::Processor;
use citum_engine::io::load_bibliography;
use citum_schema::{
    CitationSpec, Style, StyleInfo,
    citation::{Citation, CitationItem, CitationMode},
    reference::{
        Contributor, ContributorList, EdtfString, InputReference as Reference, Monograph,
        MonographType, MultilingualString, Parent, Serial, SerialComponent, SerialComponentType,
        SerialType, StructuredName, Title,
    },
};

// --- Helper Functions for Test Data Construction ---

/// Create a native Reference for a book with minimal fields.
pub fn make_book(id: &str, family: &str, given: &str, year: i32, title: &str) -> Reference {
    citum_schema::ref_book!(id, family, given, year, title)
}

/// Create a native Reference with multiple authors.
pub fn make_book_multi_author(
    id: &str,
    authors: Vec<(&str, &str)>,
    year: i32,
    title: &str,
) -> Reference {
    let author_list: Vec<Contributor> = authors
        .into_iter()
        .map(|(family, given)| {
            Contributor::StructuredName(StructuredName {
                family: MultilingualString::Simple(family.to_string()),
                given: MultilingualString::Simple(given.to_string()),
                suffix: None,
                dropping_particle: None,
                non_dropping_particle: None,
            })
        })
        .collect();

    Reference::Monograph(Box::new(Monograph {
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title.to_string())),
        container_title: None,
        author: Some(Contributor::ContributorList(ContributorList(author_list))),
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
        guest: None,
        issued: EdtfString(year.to_string()),
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
    }))
}

/// Create a native Reference for an article-journal.
pub fn make_article(id: &str, family: &str, given: &str, year: i32, title: &str) -> Reference {
    citum_schema::ref_article!(id, family, given, year, title)
}

/// Create a native Reference for an article-journal with multiple authors.
pub fn make_article_multi_author(
    id: &str,
    authors: Vec<(&str, &str)>,
    year: i32,
    title: &str,
) -> Reference {
    let author_list: Vec<Contributor> = authors
        .into_iter()
        .map(|(family, given)| {
            Contributor::StructuredName(StructuredName {
                family: MultilingualString::Simple(family.to_string()),
                given: MultilingualString::Simple(given.to_string()),
                suffix: None,
                dropping_particle: None,
                non_dropping_particle: None,
            })
        })
        .collect();

    Reference::SerialComponent(Box::new(SerialComponent {
        id: Some(id.to_string()),
        r#type: SerialComponentType::Article,
        title: Some(Title::Single(title.to_string())),
        author: Some(Contributor::ContributorList(ContributorList(author_list))),
        translator: None,
        issued: EdtfString(year.to_string()),
        parent: Parent::Embedded(Serial {
            r#type: SerialType::AcademicJournal,
            title: Some(Title::Single(String::new())),
            short_title: None,
            editor: None,
            publisher: None,
            issn: None,
        }),
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        doi: None,
        pages: None,
        volume: None,
        issue: None,
        genre: None,
        medium: None,
        keywords: None,
        ads_bibcode: None,
    }))
}

pub struct MultilingualBookParams<'a> {
    pub id: &'a str,
    pub original_family: &'a str,
    pub original_given: &'a str,
    pub lang: &'a str,
    pub translit_script: &'a str,
    pub translit_family: &'a str,
    pub translit_given: &'a str,
    pub year: i32,
    pub title: &'a str,
}

pub fn make_multilingual_book(params: MultilingualBookParams) -> Reference {
    use citum_schema::reference::contributor::MultilingualName;
    use std::collections::HashMap;

    let mut transliterations = HashMap::new();
    transliterations.insert(
        params.translit_script.to_string(),
        StructuredName {
            family: MultilingualString::Simple(params.translit_family.to_string()),
            given: MultilingualString::Simple(params.translit_given.to_string()),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: None,
        },
    );

    Reference::Monograph(Box::new(Monograph {
        id: Some(params.id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(params.title.to_string())),
        container_title: None,
        author: Some(Contributor::Multilingual(MultilingualName {
            original: StructuredName {
                family: MultilingualString::Simple(params.original_family.to_string()),
                given: MultilingualString::Simple(params.original_given.to_string()),
                suffix: None,
                dropping_particle: None,
                non_dropping_particle: None,
            },
            lang: Some(params.lang.to_string()),
            transliterations,
            translations: HashMap::new(),
        })),
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
        guest: None,
        issued: EdtfString(params.year.to_string()),
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
    }))
}

// --- Test Execution Helpers ---

/// Execute a test case with default disambiguation settings (`year_suffix` only).
pub fn run_test_case_native(
    input: &[Reference],
    citation_items: &[Vec<&str>],
    expected: &str,
    mode: &str,
) {
    run_test_case_native_with_options(TestCaseOptions {
        input,
        citation_items,
        expected,
        mode,
        disambiguate_year_suffix: true,
        disambiguate_names: false,
        disambiguate_givenname: false,
        et_al_min: None,
        et_al_use_first: None,
    });
}

pub struct TestCaseOptions<'a> {
    pub input: &'a [Reference],
    pub citation_items: &'a [Vec<&'a str>],
    pub expected: &'a str,
    pub mode: &'a str,
    pub disambiguate_year_suffix: bool,
    pub disambiguate_names: bool,
    pub disambiguate_givenname: bool,
    pub et_al_min: Option<u8>,
    pub et_al_use_first: Option<u8>,
}

/// Execute a test case with custom disambiguation settings.
pub fn run_test_case_native_with_options(options: TestCaseOptions) {
    // Create author-date style with customizable disambiguation options
    let style = build_author_date_style(
        options.disambiguate_year_suffix,
        options.disambiguate_names,
        options.disambiguate_givenname,
        options.et_al_min,
        options.et_al_use_first,
    );

    // Build bibliography from native references
    let mut bibliography = indexmap::IndexMap::new();
    for item in options.input {
        if let Some(id) = item.id() {
            bibliography.insert(id, item.clone());
        }
    }

    let processor = Processor::new(style, bibliography);

    if options.mode == "citation" {
        let mut results = Vec::new();

        for batch in options.citation_items {
            let items: Vec<CitationItem> = batch
                .iter()
                .map(|id| CitationItem {
                    id: id.to_string(),
                    ..Default::default()
                })
                .collect();

            let citation = Citation {
                items,
                mode: CitationMode::NonIntegral,
                ..Default::default()
            };

            let res = processor
                .process_citation(&citation)
                .expect("Failed to process citation");
            results.push(res);
        }

        let actual = results.join("\n");
        assert_eq!(
            actual.trim(),
            options.expected.trim(),
            "Citation output mismatch"
        );
    } else if options.mode == "bibliography" {
        if !options.citation_items.is_empty() {
            for batch in options.citation_items {
                let items: Vec<CitationItem> = batch
                    .iter()
                    .map(|id| CitationItem {
                        id: id.to_string(),
                        ..Default::default()
                    })
                    .collect();
                let citation = Citation {
                    items,
                    ..Default::default()
                };
                processor.process_citation(&citation).ok();
            }
        }

        let actual = processor.render_bibliography();
        assert_eq!(
            actual.trim(),
            options.expected.trim(),
            "Bibliography output mismatch"
        );
    }
}

/// Return the repository root for integration-test fixtures and example files.
pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Load a YAML style relative to the repository root.
pub fn load_style(path: &str) -> Style {
    let style_path = project_root().join(path);
    let yaml = fs::read_to_string(&style_path)
        .unwrap_or_else(|err| panic!("failed to read style {}: {err}", style_path.display()));
    Style::from_yaml_str(&yaml)
        .unwrap_or_else(|err| panic!("failed to parse style {}: {err}", style_path.display()))
}

/// Load the shared example bibliography used by document behavior tests.
pub fn load_example_bibliography() -> indexmap::IndexMap<String, Reference> {
    load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse")
}

/// Load an example document relative to the repository root.
pub fn load_example_document(path: &str) -> String {
    fs::read_to_string(project_root().join(path))
        .unwrap_or_else(|err| panic!("failed to read example document {path}: {err}"))
}

/// Build a processor from a repo-relative style path and the shared example bibliography.
pub fn example_document_processor(style_path: &str) -> Processor {
    Processor::new(load_style(style_path), load_example_bibliography())
}

/// Emit a short behavior summary for narrative integration tests.
pub fn announce_behavior(summary: &str) {
    println!("behavior: {summary}");
}

/// Build an author-date style with customizable disambiguation options.
pub fn build_author_date_style(
    disambiguate_year_suffix: bool,
    disambiguate_names: bool,
    disambiguate_givenname: bool,
    et_al_min: Option<u8>,
    et_al_use_first: Option<u8>,
) -> Style {
    use citum_schema::options::{
        Config, ContributorConfig, Disambiguation, Processing, ProcessingCustom, ShortenListOptions,
    };
    use citum_schema::template::WrapPunctuation;

    // Build disambiguation config
    let disambiguate = if disambiguate_year_suffix || disambiguate_names || disambiguate_givenname {
        Some(Disambiguation {
            year_suffix: disambiguate_year_suffix,
            names: disambiguate_names,
            add_givenname: disambiguate_givenname,
        })
    } else {
        None
    };

    // Build contributors config with et-al settings and initialize_with for initials
    let contributors = Some(ContributorConfig {
        shorten: if et_al_min.is_some() || et_al_use_first.is_some() {
            Some(ShortenListOptions {
                min: et_al_min.unwrap_or(3),
                use_first: et_al_use_first.unwrap_or(1),
                ..Default::default()
            })
        } else {
            None
        },
        initialize_with: Some(" ".to_string()),
        name_form: Some(citum_schema::options::NameForm::Initials),
        ..Default::default()
    });

    // Citation template: Author (Year)
    let citation_template = vec![
        citum_schema::tc_contributor!(Author, Short),
        citum_schema::tc_date!(Issued, Year, wrap = WrapPunctuation::Parentheses),
    ];

    Style {
        info: StyleInfo {
            title: Some("Author-Date Disambiguation Test".to_string()),
            id: Some("http://test.example/disambiguation".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                disambiguate,
                ..Default::default()
            })),
            contributors,
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            sort: Some(citum_schema::grouping::GroupSortEntry::Explicit(
                citum_schema::grouping::GroupSort {
                    template: vec![
                        citum_schema::grouping::GroupSortKey {
                            key: citum_schema::grouping::SortKey::Author,
                            ascending: true,
                            order: None,
                            sort_order: None,
                        },
                        citum_schema::grouping::GroupSortKey {
                            key: citum_schema::grouping::SortKey::Issued,
                            ascending: true,
                            order: None,
                            sort_order: None,
                        },
                    ],
                },
            )),
            template: Some(citation_template),
            multi_cite_delimiter: Some("; ".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}
