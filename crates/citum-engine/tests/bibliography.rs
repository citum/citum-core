/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::Processor;
use citum_schema::{
    BibliographySpec, CitationSpec, Style, StyleInfo,
    options::{
        AndOptions, BibliographyConfig, Config, ContributorConfig, DelimiterPrecedesLast,
        DemoteNonDroppingParticle, DisplayAsSort, Processing, ProcessingCustom, Sort, SortKey,
        SortSpec,
    },
    reference::{Contributor, InputReference, Monograph, MonographType, StructuredName, Title},
    template::{
        DelimiterPunctuation, SimpleVariable, TemplateComponent, TemplateList, TemplateTitle,
        TemplateVariable, TitleForm, TitleType,
    },
};

// --- Helper Functions ---

fn build_numeric_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Numeric Test".to_string()),
            id: Some("numeric-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Numeric),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![citum_schema::tc_number!(CitationNumber)]),
            wrap: Some(citum_schema::template::WrapPunctuation::Brackets),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_number!(CitationNumber, suffix = ". "),
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_date!(Issued, Year, prefix = " (", suffix = ")"),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_sorted_style(sort: Vec<SortSpec>) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Sorted Test".to_string()),
            id: Some("sort-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                sort: Some(citum_schema::options::SortEntry::Explicit(Sort {
                    template: sort,
                    shorten_names: false,
                    render_substitutions: false,
                })),
                ..Default::default()
            })),
            contributors: Some(ContributorConfig {
                display_as_sort: Some(DisplayAsSort::All),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_date!(Issued, Year, prefix = " "),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_title_year_sorted_style(sort: Vec<SortSpec>) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Title Year Sorted Test".to_string()),
            id: Some("title-year-sort-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                sort: Some(citum_schema::options::SortEntry::Explicit(Sort {
                    template: sort,
                    shorten_names: false,
                    render_substitutions: false,
                })),
                ..Default::default()
            })),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_title!(Primary, prefix = ". "),
                citum_schema::tc_date!(Issued, Year, prefix = " "),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_container_title_short_style(title_type: TitleType) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Container Title Short Test".to_string()),
            id: Some("container-title-short-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Numeric),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![TemplateComponent::List(TemplateList {
                items: vec![
                    TemplateComponent::Variable(TemplateVariable {
                        variable: SimpleVariable::ContainerTitleShort,
                        ..Default::default()
                    }),
                    TemplateComponent::Title(TemplateTitle {
                        title: title_type.clone(),
                        form: Some(TitleForm::Short),
                        ..Default::default()
                    }),
                    TemplateComponent::Title(TemplateTitle {
                        title: title_type,
                        form: Some(TitleForm::Long),
                        ..Default::default()
                    }),
                ],
                delimiter: Some(DelimiterPunctuation::Slash),
                ..Default::default()
            })]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_processing_style(processing: Processing) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Processing Default Sort Test".to_string()),
            id: Some("processing-default-sort-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(processing),
            contributors: Some(ContributorConfig {
                display_as_sort: Some(DisplayAsSort::All),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_date!(Issued, Year, prefix = " "),
                citum_schema::tc_title!(Primary, prefix = ". "),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_style_with_substitute(substitute: Option<String>) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Subsequent Author Substitute Test".to_string()),
            id: Some("sub-test".to_string()),
            ..Default::default()
        },
        templates: None,
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            bibliography: Some(BibliographyConfig {
                subsequent_author_substitute: substitute,
                entry_suffix: Some(".".to_string()),
                ..Default::default()
            }),
            contributors: Some(ContributorConfig {
                display_as_sort: Some(DisplayAsSort::First),
                ..Default::default()
            }),
            ..Default::default()
        }),
        citation: None,
        bibliography: Some(BibliographySpec {
            options: None,
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_date!(Issued, Year),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_particle_book(
    id: &str,
    family: &str,
    given: &str,
    particle: Option<&str>,
) -> InputReference {
    InputReference::Monograph(Box::new(Monograph {
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Title::Single(format!("Title {id}")),
        author: Some(Contributor::StructuredName(StructuredName {
            family: family.into(),
            given: given.into(),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: particle.map(Into::into),
        })),
        editor: None,
        translator: None,
        issued: citum_schema::reference::EdtfString("2000".to_string()),
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
        keywords: None,
        original_date: None,
        original_title: None,
    }))
}

fn make_name_particle_style(display_as_sort: DisplayAsSort) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Hyphenated Particle Test".to_string()),
            id: Some("hyphenated-particle-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                sort: Some(citum_schema::options::SortEntry::Explicit(Sort {
                    template: vec![SortSpec {
                        key: SortKey::Author,
                        ascending: true,
                    }],
                    shorten_names: false,
                    render_substitutions: false,
                })),
                ..Default::default()
            })),
            contributors: Some(ContributorConfig {
                display_as_sort: Some(display_as_sort),
                demote_non_dropping_particle: Some(DemoteNonDroppingParticle::DisplayAndSort),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![citum_schema::tc_contributor!(Author, Long)]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// --- Sorting Tests ---

#[test]
fn test_sorting_by_author() {
    let style = build_sorted_style(vec![SortSpec {
        key: SortKey::Author,
        ascending: true,
    }]);

    let mut bib = indexmap::IndexMap::new();
    bib.insert("z".to_string(), make_book("z", "Zoe", "Z", 2020, "Title Z"));
    bib.insert(
        "a".to_string(),
        make_book("a", "Adam", "A", 2020, "Title A"),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // Adam should come before Zoe
    assert!(result.find("Adam").unwrap() < result.find("Zoe").unwrap());
}

#[test]
fn test_sorting_by_year() {
    let style = build_sorted_style(vec![SortSpec {
        key: SortKey::Year,
        ascending: true,
    }]);

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "item1".to_string(),
        make_book("item1", "Smith", "J", 2022, "Title B"),
    );
    bib.insert(
        "item2".to_string(),
        make_book("item2", "Smith", "J", 2020, "Title A"),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // 2020 should come before 2022
    assert!(result.find("2020").unwrap() < result.find("2022").unwrap());
}

#[test]
fn test_sorting_empty_dates_bibliography() {
    // Upstream provenance: CSL fixture `date_SortEmptyDatesBibliography`.
    let style = build_title_year_sorted_style(vec![
        SortSpec {
            key: SortKey::Year,
            ascending: true,
        },
        SortSpec {
            key: SortKey::Title,
            ascending: true,
        },
    ]);

    fn make_undated_book(id: &str, title: &str) -> InputReference {
        let mut reference = make_book(id, "Smith", "Jane", 2000, title);
        if let InputReference::Monograph(monograph) = &mut reference {
            monograph.issued = citum_schema::reference::EdtfString(String::new());
        }
        reference
    }

    let mut bib = indexmap::IndexMap::new();
    bib.insert("item1".to_string(), make_undated_book("item1", "BookA"));
    bib.insert(
        "item2".to_string(),
        make_book("item2", "Smith", "Jane", 2000, "BookB"),
    );
    bib.insert("item3".to_string(), make_undated_book("item3", "BookC"));
    bib.insert(
        "item4".to_string(),
        make_book("item4", "Smith", "Jane", 1999, "BookD"),
    );
    bib.insert("item5".to_string(), make_undated_book("item5", "BookE"));

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert!(result.find("BookD 1999").unwrap() < result.find("BookB 2000").unwrap());
    assert!(result.find("BookB 2000").unwrap() < result.find("BookA").unwrap());
    assert!(result.find("BookA").unwrap() < result.find("BookC").unwrap());
    assert!(result.find("BookC").unwrap() < result.find("BookE").unwrap());
}

#[test]
fn test_container_title_short_from_journal_abbreviation() {
    // Upstream provenance: CSL fixtures `bugreports_ContainerTitleShort` and
    // `variables_ContainerTitleShort`.
    let style = build_container_title_short_style(TitleType::ParentSerial);

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "ITEM-1",
        "type": "article-journal",
        "title": "Ignored",
        "container-title": "Anonymous Journal",
        "journalAbbreviation": "Anon J"
    }))
    .expect("legacy fixture should parse");

    let mut bib = indexmap::IndexMap::new();
    bib.insert("ITEM-1".to_string(), legacy.into());

    let processor = Processor::new(style, bib);
    assert_eq!(
        processor.render_bibliography(),
        "Anon J/Anon J/Anonymous Journal"
    );
}

#[test]
fn test_container_title_short_from_container_title_short_field() {
    // Upstream provenance: CSL fixtures `bugreports_ContainerTitleShort` and
    // `variables_ContainerTitleShort`.
    let style = build_container_title_short_style(TitleType::ParentMonograph);

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "ITEM-2",
        "type": "chapter",
        "title": "Ignored",
        "container-title": "Anonymous Journal One",
        "container-title-short": "Journal-1"
    }))
    .expect("legacy fixture should parse");

    let mut bib = indexmap::IndexMap::new();
    bib.insert("ITEM-2".to_string(), legacy.into());

    let processor = Processor::new(style, bib);
    assert_eq!(
        processor.render_bibliography(),
        "Journal-1/Journal-1/Anonymous Journal One"
    );
}

#[test]
fn test_sorting_multiple_keys() {
    let style = build_sorted_style(vec![
        SortSpec {
            key: SortKey::Author,
            ascending: true,
        },
        SortSpec {
            key: SortKey::Year,
            ascending: false,
        },
    ]);

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "item1".to_string(),
        make_book("item1", "Smith", "J", 2020, "Title A"),
    );
    bib.insert(
        "item2".to_string(),
        make_book("item2", "Smith", "J", 2022, "Title B"),
    );
    bib.insert(
        "item3".to_string(),
        make_book("item3", "Adams", "A", 2021, "Title C"),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // Adams (2021) should be first
    // Then Smith (2022) - because descending year
    // Then Smith (2020)
    assert!(result.find("Adams").unwrap() < result.find("Smith, J 2022").unwrap());
    assert!(result.find("Smith, J 2022").unwrap() < result.find("Smith, J 2020").unwrap());
}

#[test]
fn test_author_date_processing_defaults_bibliography_to_author_date_title() {
    let style = build_processing_style(Processing::AuthorDate);

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "zeta".to_string(),
        make_book("zeta", "Smith", "Jane", 2020, "Zeta Work"),
    );
    bib.insert(
        "alpha".to_string(),
        make_book("alpha", "Smith", "Jane", 2020, "Alpha Work"),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert!(result.find("Alpha Work").unwrap() < result.find("Zeta Work").unwrap());
}

#[test]
fn test_note_processing_defaults_bibliography_to_author_title_date() {
    let style = build_processing_style(Processing::Note);

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "zeta".to_string(),
        make_book("zeta", "Smith", "Jane", 2020, "Zeta Work"),
    );
    bib.insert(
        "alpha".to_string(),
        make_book("alpha", "Smith", "Jane", 2022, "Alpha Work"),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert!(result.find("Alpha Work").unwrap() < result.find("Zeta Work").unwrap());
}

// --- Substitution Tests ---

#[test]
fn test_subsequent_author_substitute() {
    let style = make_style_with_substitute(Some("———".to_string()));

    let bib = citum_schema::bib_map![
        "ref1" => make_book("ref1", "Smith", "John", 2020, "Book A"),
        "ref2" => make_book("ref2", "Smith", "John", 2021, "Book B"),
    ];
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // ref1 comes first (2020), then ref2 (2021). ref2 should have substituted author.
    // Note: Implicit separator ". " + Implicit suffix "."
    let expected = "Smith, John. 2020.\n\n———. 2021.";
    assert_eq!(result, expected);
}

#[test]
fn test_magic_subsequentauthorsubstitute() {
    // Upstream provenance: CSL fixture `magic_SubsequentAuthorSubstitute`.
    let style = Style {
        info: StyleInfo {
            title: Some("Magic Subsequent Author Substitute Test".to_string()),
            id: Some("magic-subsequent-author-substitute-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            bibliography: Some(BibliographyConfig {
                subsequent_author_substitute: Some("———".to_string()),
                ..Default::default()
            }),
            contributors: Some(ContributorConfig {
                and: Some(AndOptions::Text),
                delimiter_precedes_last: Some(DelimiterPrecedesLast::Never),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_title!(Primary, prefix = ", "),
                citum_schema::tc_date!(Issued, Year, prefix = " (", suffix = ")"),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let bib = citum_schema::bib_map![
        "item-1" => make_book_multi_author("item-1", vec![("Smith", "John"), ("Roe", "Jane")], 2000, "Book A"),
        "item-2" => make_book_multi_author("item-2", vec![("Smith", "John"), ("Roe", "Jane")], 2001, "Book B"),
        "item-3" => make_book("item-3", "Smith", "John", 2002, "Book C"),
    ];

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();
    assert_eq!(
        result,
        "John Smith and Jane Roe, Book A (2000)\n\n———, Book B (2001)\n\nJohn Smith, Book C (2002)"
    );
}

#[test]
fn test_no_substitute_if_different() {
    let style = make_style_with_substitute(Some("———".to_string()));

    let bib = citum_schema::bib_map![
        "ref1" => make_book("ref1", "Smith", "John", 2020, "Book A"),
        "ref2" => make_book("ref2", "Doe", "Jane", 2021, "Book B"),
    ];

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // Doe comes before Smith alphabetically
    let expected = "Doe, Jane. 2021.\n\nSmith, John. 2020.";
    assert_eq!(result, expected);
}

#[test]
fn test_name_hyphenatednondroppingparticle1() {
    // Upstream provenance: CSL fixture `name_HyphenatedNonDroppingParticle1`.
    let style = make_name_particle_style(DisplayAsSort::All);

    let bib = citum_schema::bib_map![
        "ITEM-1" => make_particle_book("ITEM-1", "One", "Alan", Some("al-")),
        "ITEM-2" => make_particle_book("ITEM-2", "Marple", "Mary", None),
        "ITEM-3" => make_particle_book("ITEM-3", "Participle", "Paul", None),
    ];
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert_eq!(result, "Marple, Mary\n\nOne, Alan al-\n\nParticiple, Paul");
}

#[test]
fn test_name_hyphenatednondroppingparticle2() {
    // Upstream provenance: CSL fixture `name_HyphenatedNonDroppingParticle2`.
    let style = make_name_particle_style(DisplayAsSort::None);

    let bib = citum_schema::bib_map![
        "ITEM-1" => make_particle_book("ITEM-1", "One", "Alan", Some("al-")),
        "ITEM-2" => make_particle_book("ITEM-2", "Marple", "Mary", None),
        "ITEM-3" => make_particle_book("ITEM-3", "Participle", "Paul", None),
    ];
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert_eq!(result, "Mary Marple\n\nAlan al-One\n\nPaul Participle");
}

// --- Numeric Bibliography Tests ---

#[test]
fn test_numeric_bibliography() {
    let style = build_numeric_style();

    let bib =
        citum_schema::bib_map!["item1" => make_book("item1", "Smith", "John", 2020, "Title A")];
    let processor = Processor::new(style, bib);
    // Must process citation to assign number
    processor
        .process_citation(&citum_schema::cite!("item1"))
        .unwrap();

    let result = processor.render_bibliography();
    assert_eq!(result, "1. John Smith (2020)");
}

#[test]
fn test_anonymous_works_sort_by_title_without_article() {
    let style = build_title_year_sorted_style(vec![
        SortSpec {
            key: SortKey::Author,
            ascending: true,
        },
        SortSpec {
            key: SortKey::Year,
            ascending: true,
        },
    ]);

    let mut bib = indexmap::IndexMap::new();
    // Anonymous work with "The" article should sort as "Chicago Manual"
    bib.insert(
        "anon1".to_string(),
        make_book("anon1", "", "", 2018, "The Chicago Manual of Style"),
    );
    // Another anonymous work starting with title after article
    bib.insert(
        "anon2".to_string(),
        make_book("anon2", "", "", 2015, "A Guide to Citation"),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // "The Chicago..." (C) should come BEFORE "A Guide..." (G) when articles are stripped
    assert!(result.find("The Chicago").unwrap() < result.find("A Guide").unwrap());
}

#[test]
fn test_anonymous_same_year_tiebreak() {
    let style = build_sorted_style(vec![
        SortSpec {
            key: SortKey::Author,
            ascending: true,
        },
        SortSpec {
            key: SortKey::Year,
            ascending: true,
        },
    ]);

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "anon1".to_string(),
        make_book("anon1", "", "", 2020, "The Chicago Manual"),
    );
    bib.insert(
        "anon2".to_string(),
        make_book("anon2", "", "", 2020, "An Earlier Publication"),
    );
    bib.insert(
        "anon3".to_string(),
        make_book("anon3", "", "", 2019, "The Chicago Manual"),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // 2019 entry should come before 2020 entries
    assert!(result.find("2019").unwrap() < result.find("2020").unwrap());
}
