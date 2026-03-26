#![allow(missing_docs, reason = "test")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::{Processor, render::html::Html};
use citum_schema::{
    BibliographySpec, CitationSpec, Style, StyleInfo,
    options::{
        AndOptions, ArticleJournalBibliographyConfig, ArticleJournalNoPageFallback,
        BibliographyOptions, Config, ContributorConfig, DelimiterPrecedesLast,
        DemoteNonDroppingParticle, DisplayAsSort, LinkAnchor, LinkTarget, LinksConfig, Processing,
        ProcessingCustom, Sort, SortKey, SortSpec,
    },
    reference::{
        Contributor, EdtfString, InputReference, Monograph, MonographType, NumOrStr, Parent,
        Serial, SerialComponent, SerialComponentType, SerialType, StructuredName, Title,
    },
    template::{
        DateForm, DateVariable, DelimiterPunctuation, NumberVariable, Rendering, SimpleVariable,
        TemplateComponent, TemplateDate, TemplateGroup, TemplateNumber, TemplateTitle,
        TemplateVariable, TitleForm, TitleType,
    },
};
use url::Url;

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
            template: Some(vec![TemplateComponent::Group(TemplateGroup {
                group: vec![
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

fn build_article_journal_no_page_fallback_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Article Journal Fallback Test".to_string()),
            id: Some("article-journal-fallback-test".to_string()),
            ..Default::default()
        },
        options: Some(Config::default()),
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                article_journal: Some(ArticleJournalBibliographyConfig {
                    no_page_fallback: Some(ArticleJournalNoPageFallback::Doi),
                }),
                separator: Some(", ".to_string()),
                ..Default::default()
            }),
            template: Some(vec![
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::ParentSerial,
                    ..Default::default()
                }),
                TemplateComponent::Group(TemplateGroup {
                    group: vec![
                        TemplateComponent::Date(TemplateDate {
                            date: DateVariable::Issued,
                            form: DateForm::Year,
                            ..Default::default()
                        }),
                        TemplateComponent::Number(TemplateNumber {
                            number: NumberVariable::Volume,
                            ..Default::default()
                        }),
                        TemplateComponent::Number(TemplateNumber {
                            number: NumberVariable::Issue,
                            rendering: Rendering {
                                prefix: Some("(".to_string()),
                                suffix: Some(")".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        TemplateComponent::Number(TemplateNumber {
                            number: NumberVariable::Pages,
                            rendering: Rendering {
                                prefix: Some("pp. ".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    delimiter: Some(DelimiterPunctuation::Comma),
                    ..Default::default()
                }),
                TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::Doi,
                    rendering: Rendering {
                        prefix: Some("DOI:".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_bibliography_entry_link_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Bibliography Entry Link Test".to_string()),
            id: Some("bibliography-entry-link-test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                links: Some(LinksConfig {
                    url: Some(true),
                    target: Some(LinkTarget::Url),
                    anchor: Some(LinkAnchor::Entry),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            template: Some(vec![citum_schema::tc_title!(Primary)]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_bibliography_local_note_sort_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Bibliography Local Note Sort Test".to_string()),
            id: Some("bibliography-local-note-sort-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Numeric),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                processing: Some(Processing::Note),
                ..Default::default()
            }),
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

fn build_bibliography_local_numeric_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Bibliography Local Numeric Test".to_string()),
            id: Some("bibliography-local-numeric-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                processing: Some(Processing::Numeric),
                ..Default::default()
            }),
            template: Some(vec![
                citum_schema::tc_number!(CitationNumber, suffix = ". "),
                citum_schema::tc_contributor!(Author, Long),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_numeric_citation_style_with_bibliography_local_note_sort() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Numeric Citation Local Note Sort Test".to_string()),
            id: Some("numeric-citation-local-note-sort-test".to_string()),
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
            options: Some(BibliographyOptions {
                processing: Some(Processing::Note),
                ..Default::default()
            }),
            template: Some(vec![
                citum_schema::tc_number!(CitationNumber, suffix = ". "),
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_title!(Primary, prefix = ". "),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_inline_article_journal_detail_group_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Inline Article Journal Detail Group Test".to_string()),
            id: Some("inline-article-journal-detail-group-test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                TemplateComponent::Contributor(citum_schema::template::TemplateContributor {
                    contributor: citum_schema::template::ContributorRole::Author,
                    form: citum_schema::template::ContributorForm::Long,
                    rendering: Rendering {
                        suffix: Some(". ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::ParentSerial,
                    rendering: Rendering {
                        emph: Some(true),
                        suffix: Some(". ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                TemplateComponent::Group(TemplateGroup {
                    group: vec![
                        TemplateComponent::Number(TemplateNumber {
                            number: NumberVariable::Volume,
                            ..Default::default()
                        }),
                        TemplateComponent::Group(TemplateGroup {
                            group: vec![
                                TemplateComponent::Number(TemplateNumber {
                                    number: NumberVariable::Issue,
                                    ..Default::default()
                                }),
                                TemplateComponent::Date(TemplateDate {
                                    date: DateVariable::Issued,
                                    form: DateForm::YearMonth,
                                    rendering: Rendering {
                                        wrap: Some(
                                            citum_schema::template::WrapPunctuation::Parentheses,
                                        ),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            delimiter: Some(DelimiterPunctuation::Space),
                            ..Default::default()
                        }),
                        TemplateComponent::Number(TemplateNumber {
                            number: NumberVariable::Pages,
                            rendering: Rendering {
                                prefix: Some("pp. ".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    delimiter: Some(DelimiterPunctuation::Comma),
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn bibliography_html_injects_sparse_indices_from_type_template() {
    let style_yaml = r#"
info:
  title: Indexed Bibliography Preview
  id: indexed-bibliography-preview
bibliography:
  type-variants:
    article-journal:
      - contributor: author
        form: long
      - variable: doi
        prefix: " "
      - title: primary
        prefix: ". "
"#;
    let style: Style = serde_yaml::from_str(style_yaml).expect("style should parse");

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "ITEM-1",
        "type": "article-journal",
        "title": "Preview Article",
        "author": [{"family": "Smith", "given": "Jane"}]
    }))
    .expect("legacy fixture should parse");

    let mut bib = indexmap::IndexMap::new();
    bib.insert("ITEM-1".to_string(), legacy.into());

    let processor = Processor::new(style, bib).with_inject_ast_indices(true);
    let rendered = processor.render_bibliography_with_format::<Html>();

    assert!(
        rendered.contains(r#"class="csln-author" data-index="0""#),
        "author wrapper should carry the first type-template index: {rendered}"
    );
    assert!(
        rendered.contains(r#"class="csln-title" data-index="2""#),
        "title wrapper should carry the sparse third type-template index: {rendered}"
    );
    assert!(
        !rendered.contains(r#"data-index="1""#),
        "missing DOI output should preserve sparse template indices: {rendered}"
    );
}

fn build_list_index_preview_style(use_type_template: bool) -> Style {
    let bibliography_yaml = if use_type_template {
        r#"
bibliography:
  type-variants:
    article-journal:
      - items:
          - contributor: author
            form: long
          - title: primary
            prefix: ". "
        delimiter: ", "
"#
    } else {
        r#"
bibliography:
  template:
    - items:
        - contributor: author
          form: long
        - title: primary
          prefix: ". "
      delimiter: ", "
"#
    };

    let yaml = format!(
        r#"
info:
  title: List Index Preview
  id: {}
{}
"#,
        if use_type_template {
            "list-index-preview-type"
        } else {
            "list-index-preview-default"
        },
        bibliography_yaml
    );

    serde_yaml::from_str(&yaml).expect("style should parse")
}

fn assert_list_preview_inherits_parent_index(use_type_template: bool) {
    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "ITEM-1",
        "type": "article-journal",
        "title": "Preview Article",
        "author": [{"family": "Smith", "given": "Jane"}]
    }))
    .expect("legacy fixture should parse");

    let mut bib = indexmap::IndexMap::new();
    bib.insert("ITEM-1".to_string(), legacy.into());

    let processor = Processor::new(build_list_index_preview_style(use_type_template), bib)
        .with_inject_ast_indices(true);
    let rendered = processor.render_bibliography_with_format::<Html>();

    assert!(
        rendered.contains(r#"class="csln-author" data-index="0""#),
        "list-rendered author should inherit the parent top-level index: {rendered}"
    );
    assert!(
        rendered.contains(r#"class="csln-title" data-index="0""#),
        "list-rendered title should inherit the parent top-level index: {rendered}"
    );
}

fn make_article_journal_with_detail(
    id: &str,
    issued: &str,
    issue: Option<&str>,
    pages: Option<&str>,
    doi: Option<&str>,
) -> InputReference {
    InputReference::SerialComponent(Box::new(SerialComponent {
        id: Some(id.to_string()),
        r#type: SerialComponentType::Article,
        title: Some(Title::Single("Fallback Article".to_string())),
        author: Some(Contributor::StructuredName(StructuredName {
            family: "Doe".into(),
            given: "Jane".into(),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: None,
        })),
        translator: None,
        issued: EdtfString(issued.to_string()),
        parent: Parent::Embedded(Serial {
            r#type: SerialType::AcademicJournal,
            title: Some(Title::Single("Journal of Fallbacks".to_string())),
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
        doi: doi.map(str::to_string),
        ads_bibcode: None,
        pages: pages.map(str::to_string),
        volume: Some(NumOrStr::Str("12".to_string())),
        issue: issue.map(|value| NumOrStr::Str(value.to_string())),
        genre: None,
        medium: None,
        keywords: None,
    }))
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
            contributors: Some(ContributorConfig {
                display_as_sort: Some(DisplayAsSort::First),
                ..Default::default()
            }),
            ..Default::default()
        }),
        citation: None,
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                subsequent_author_substitute: substitute,
                entry_suffix: Some(".".to_string()),
                ..Default::default()
            }),
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
        title: Some(Title::Single(format!("Title {id}"))),
        container_title: None,
        author: Some(Contributor::StructuredName(StructuredName {
            family: family.into(),
            given: given.into(),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: particle.map(Into::into),
        })),
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
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
        archive: None,
        archive_location: None,
        keywords: None,
        original_date: None,
        original_title: None,
        ads_bibcode: None,
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

fn sorting_by_author_orders_entries_alphabetically() {
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

fn sorting_by_year_places_earlier_years_first() {
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

fn sorting_empty_dates_pushes_undated_items_after_dated_ones() {
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

fn container_title_short_uses_journal_abbreviation_when_present() {
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

fn container_title_short_prefers_explicit_short_field() {
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

fn sorting_multiple_keys_applies_secondary_ordering_within_author_groups() {
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

fn author_date_processing_defaults_bibliography_to_author_date_title_order() {
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

fn note_processing_defaults_bibliography_to_author_title_date_order() {
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

fn subsequent_author_substitute_replaces_repeated_author_lines() {
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

fn magic_subsequent_author_substitute_reuses_the_full_author_group() {
    // Upstream provenance: CSL fixture `magic_SubsequentAuthorSubstitute`.
    let style = Style {
        info: StyleInfo {
            title: Some("Magic Subsequent Author Substitute Test".to_string()),
            id: Some("magic-subsequent-author-substitute-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            contributors: Some(ContributorConfig {
                and: Some(AndOptions::Text),
                delimiter_precedes_last: Some(DelimiterPrecedesLast::Never),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                subsequent_author_substitute: Some("———".to_string()),
                ..Default::default()
            }),
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

fn subsequent_author_substitute_does_not_apply_to_different_authors() {
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

fn hyphenated_non_dropping_particles_sort_correctly_in_sort_order() {
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

fn hyphenated_non_dropping_particles_render_correctly_in_display_order() {
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

fn numeric_bibliography_uses_assigned_citation_numbers() {
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

fn anonymous_works_sort_by_title_ignoring_leading_articles() {
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

fn anonymous_works_with_the_same_year_still_sort_by_year_first() {
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

fn article_journal_with_pages_keeps_standard_detail_block() {
    let style = build_article_journal_no_page_fallback_style();
    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "article-with-pages".to_string(),
        make_article_journal_with_detail(
            "article-with-pages",
            "2024",
            Some("3"),
            Some("101-109"),
            Some("10.1234/fallback"),
        ),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert!(result.contains("Journal of Fallbacks"));
    assert!(result.contains("2024"));
    assert!(result.contains("12"));
    assert!(result.contains("101"));
    assert!(!result.contains("DOI:10.1234/fallback"));
}

fn page_less_article_journal_swaps_detail_block_for_doi() {
    let style = build_article_journal_no_page_fallback_style();
    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "article-without-pages".to_string(),
        make_article_journal_with_detail(
            "article-without-pages",
            "2024",
            Some("3"),
            None,
            Some("10.1234/fallback"),
        ),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert!(result.contains("Journal of Fallbacks"));
    assert!(result.contains("DOI:10.1234/fallback"));
    assert!(!result.contains("2024"));
    assert!(!result.contains("pp."));
}

fn bibliography_local_entry_links_apply_on_the_default_render_path() {
    let style = build_bibliography_entry_link_style();
    let reference = InputReference::Monograph(Box::new(Monograph {
        id: Some("linked-book".to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single("Linked Book".to_string())),
        author: None,
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
        issued: EdtfString("2024".to_string()),
        publisher: None,
        container_title: None,
        url: Some(Url::parse("https://example.com/linked-book").expect("valid url")),
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
    }));
    let bib = citum_schema::bib_map!["linked-book" => reference];

    let processor = Processor::new(style, bib);
    let rendered = processor.render_bibliography_with_format::<Html>();

    assert!(
        rendered.contains(r#"href="https://example.com/linked-book""#),
        "bibliography-local entry link should be applied on the default path: {rendered}"
    );
}

fn bibliography_local_processing_changes_default_bibliography_sort() {
    let style = build_bibliography_local_note_sort_style();
    let bib = citum_schema::bib_map![
        "book-b" => make_book("book-b", "Zimmer", "Zed", 2021, "Later Book"),
        "book-a" => make_book("book-a", "Adams", "Amy", 2020, "Earlier Book")
    ];

    let processor = Processor::new(style, bib);
    let rendered = processor.render_bibliography();

    assert!(
        rendered.find("Amy Adams").unwrap() < rendered.find("Zed Zimmer").unwrap(),
        "bibliography-local processing should drive default bibliography sort: {rendered}"
    );
}

fn bibliography_local_numeric_processing_assigns_bibliography_numbers() {
    let style = build_bibliography_local_numeric_style();
    let bib = citum_schema::bib_map![
        "book-a" => make_book("book-a", "Smith", "John", 2020, "Title A"),
        "book-b" => make_book("book-b", "Brown", "Beth", 2021, "Title B")
    ];

    let processor = Processor::new(style, bib);
    let rendered = processor.render_bibliography();

    assert!(
        rendered.contains("1. John Smith"),
        "bibliography-local numeric processing should assign bibliography numbers: {rendered}"
    );
    assert!(
        rendered.contains("2. Beth Brown"),
        "bibliography-local numeric processing should assign bibliography numbers: {rendered}"
    );
}

fn numeric_citations_follow_bibliography_local_sort_when_assigning_numbers() {
    let style = build_numeric_citation_style_with_bibliography_local_note_sort();
    let bib = citum_schema::bib_map![
        "book-b" => make_book("book-b", "Zimmer", "Zed", 2021, "Later Book"),
        "book-a" => make_book("book-a", "Adams", "Amy", 2020, "Earlier Book")
    ];

    let processor = Processor::new(style, bib);
    let citation = processor
        .process_citation(&citum_schema::cite!("book-b"))
        .expect("citation should render");
    let bibliography = processor.render_bibliography();

    assert_eq!(
        citation, "[2]",
        "citation numbering should follow bibliography-local sort order"
    );
    assert!(
        bibliography.find("1. Amy Adams").unwrap() < bibliography.find("2. Zed Zimmer").unwrap(),
        "bibliography numbering should stay aligned with citation numbering: {bibliography}"
    );
}

#[test]
fn nested_inline_article_journal_detail_group_renders_issue_and_parenthesized_year_month() {
    let style = build_inline_article_journal_detail_group_style();

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "article-inline-detail".to_string(),
        make_article_journal_with_detail(
            "article-inline-detail",
            "2024-01",
            Some("3"),
            Some("1-12"),
            None,
        ),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert!(
        result.contains("12, 3 (January 2024), pp. 1–12"),
        "expected nested inline detail group rendering, got {result}"
    );
}

#[test]
fn nested_inline_article_journal_detail_group_suppresses_missing_issue_without_extra_spacing() {
    let style = build_inline_article_journal_detail_group_style();

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "article-inline-detail-no-issue".to_string(),
        make_article_journal_with_detail(
            "article-inline-detail-no-issue",
            "2024-01",
            None,
            Some("1-12"),
            None,
        ),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert!(
        result.contains("12, (January 2024), pp. 1–12"),
        "expected clean suppression around missing issue, got {result}"
    );
}

fn royal_society_of_chemistry_restores_legacy_page_less_doi_behavior() {
    let style = load_style("styles/royal-society-of-chemistry.yaml");
    let bib = citum_engine::io::load_bibliography(
        &project_root().join("tests/fixtures/references-expanded.json"),
    )
    .expect("expanded bibliography should load");
    let processor = Processor::new(style, bib);
    let result = processor
        .render_selected_bibliography_with_format::<citum_engine::render::plain::PlainText, _>(
            vec!["ITEM-1".to_string()],
        );

    assert!(result.contains("DOI:10.1234/example"));
    assert!(!result.contains("pp."));
}

mod sorting {
    use super::announce_behavior;

    #[test]
    fn author_sorting_orders_entries_alphabetically() {
        announce_behavior(
            "A bibliography sorted by author should place entries in alphabetical family-name order.",
        );
        super::sorting_by_author_orders_entries_alphabetically();
    }

    #[test]
    fn year_sorting_places_earlier_years_first() {
        announce_behavior(
            "A bibliography sorted by year should place earlier years before later years.",
        );
        super::sorting_by_year_places_earlier_years_first();
    }

    #[test]
    fn empty_dates_move_undated_entries_after_dated_ones() {
        announce_behavior(
            "When sorting by year, undated bibliography entries should fall after the dated entries.",
        );
        super::sorting_empty_dates_pushes_undated_items_after_dated_ones();
    }

    #[test]
    fn multiple_sort_keys_apply_secondary_ordering_inside_author_groups() {
        announce_behavior(
            "Multiple bibliography sort keys should apply the secondary key within an author group.",
        );
        super::sorting_multiple_keys_applies_secondary_ordering_within_author_groups();
    }

    #[test]
    fn author_date_processing_uses_author_date_title_as_the_default_bibliography_sort() {
        announce_behavior(
            "Author-date processing should default bibliography ordering to author, then date, then title.",
        );
        super::author_date_processing_defaults_bibliography_to_author_date_title_order();
    }

    #[test]
    fn note_processing_uses_author_title_date_as_the_default_bibliography_sort() {
        announce_behavior(
            "Note-style processing should default bibliography ordering to author, then title, then date.",
        );
        super::note_processing_defaults_bibliography_to_author_title_date_order();
    }

    #[test]
    fn anonymous_titles_ignore_leading_articles_during_sorting() {
        announce_behavior(
            "Anonymous bibliography entries should ignore leading articles like The or A when sorting by title.",
        );
        super::anonymous_works_sort_by_title_ignoring_leading_articles();
    }

    #[test]
    fn anonymous_same_year_entries_keep_years_in_order_before_tiebreaks() {
        announce_behavior(
            "Anonymous entries should still respect year ordering before applying same-year tiebreakers.",
        );
        super::anonymous_works_with_the_same_year_still_sort_by_year_first();
    }
}

mod title_short_resolution {
    use super::announce_behavior;

    #[test]
    fn journal_abbreviations_populate_container_title_short() {
        announce_behavior(
            "A journal abbreviation should populate container-title-short in bibliography rendering.",
        );
        super::container_title_short_uses_journal_abbreviation_when_present();
    }

    #[test]
    fn explicit_container_title_short_fields_take_precedence() {
        announce_behavior(
            "An explicit container-title-short field should take precedence over the long container title.",
        );
        super::container_title_short_prefers_explicit_short_field();
    }
}

mod substitution {
    use super::announce_behavior;

    #[test]
    fn repeated_authors_can_be_replaced_with_the_substitute_marker() {
        announce_behavior(
            "Repeated bibliography authors should be replaced by the configured subsequent-author substitute marker.",
        );
        super::subsequent_author_substitute_replaces_repeated_author_lines();
    }

    #[test]
    fn repeated_multi_author_groups_can_reuse_the_substitute_marker() {
        announce_behavior(
            "Repeated multi-author bibliography entries should reuse the substitute marker for the full repeated author group.",
        );
        super::magic_subsequent_author_substitute_reuses_the_full_author_group();
    }

    #[test]
    fn different_authors_do_not_trigger_the_substitute_marker() {
        announce_behavior(
            "Different bibliography authors should never trigger the subsequent-author substitute marker.",
        );
        super::subsequent_author_substitute_does_not_apply_to_different_authors();
    }
}

mod contributor_particles {
    use super::announce_behavior;

    #[test]
    fn hyphenated_particles_sort_correctly_in_sort_order() {
        announce_behavior(
            "Hyphenated non-dropping particles should sort correctly when contributor names are rendered in sort order.",
        );
        super::hyphenated_non_dropping_particles_sort_correctly_in_sort_order();
    }

    #[test]
    fn hyphenated_particles_render_correctly_in_display_order() {
        announce_behavior(
            "Hyphenated non-dropping particles should stay attached correctly when contributor names are rendered in display order.",
        );
        super::hyphenated_non_dropping_particles_render_correctly_in_display_order();
    }
}

mod article_journal_no_page_fallback {
    use super::announce_behavior;

    #[test]
    fn journal_articles_with_pages_keep_the_standard_detail_block() {
        announce_behavior(
            "A bibliography article-journal entry with pages should keep the standard detail block and suppress the DOI fallback path.",
        );
        super::article_journal_with_pages_keeps_standard_detail_block();
    }

    #[test]
    fn page_less_journal_articles_can_swap_the_detail_block_for_a_doi() {
        announce_behavior(
            "A bibliography article-journal entry without pages should swap its normal year-volume-pages detail block for DOI output when the style opts in.",
        );
        super::page_less_article_journal_swaps_detail_block_for_doi();
    }

    #[test]
    fn royal_society_of_chemistry_restores_the_legacy_page_less_doi_behavior() {
        announce_behavior(
            "The Royal Society of Chemistry bibliography should restore the legacy page-less journal behavior by rendering DOI instead of the standard detail block.",
        );
        super::royal_society_of_chemistry_restores_legacy_page_less_doi_behavior();
    }
}

mod numeric_styles {
    use super::announce_behavior;

    #[test]
    fn numeric_bibliographies_use_the_assigned_citation_number() {
        announce_behavior(
            "A numeric bibliography should reuse the citation number assigned during citation rendering.",
        );
        super::numeric_bibliography_uses_assigned_citation_numbers();
    }

    #[test]
    fn bibliography_local_numeric_processing_assigns_numbers_on_the_default_path() {
        announce_behavior(
            "Bibliography-local numeric processing should assign citation numbers on the default bibliography render path even when the top-level style is not numeric.",
        );
        super::bibliography_local_numeric_processing_assigns_bibliography_numbers();
    }

    #[test]
    fn numeric_citations_follow_the_bibliography_local_sort_order() {
        announce_behavior(
            "Numeric citation numbering should stay aligned with bibliography-local sort rules when the bibliography overrides the processing family.",
        );
        super::numeric_citations_follow_bibliography_local_sort_when_assigning_numbers();
    }
}

mod local_overrides {
    use super::announce_behavior;

    #[test]
    fn bibliography_local_entry_links_apply_on_the_default_render_path() {
        announce_behavior(
            "Bibliography-local shared overrides should apply on the default bibliography render path.",
        );
        super::bibliography_local_entry_links_apply_on_the_default_render_path();
    }

    #[test]
    fn bibliography_local_processing_can_change_the_default_sort_order() {
        announce_behavior(
            "Bibliography-local processing should control the default bibliography sort order when no explicit bibliography sort is declared.",
        );
        super::bibliography_local_processing_changes_default_bibliography_sort();
    }
}

mod annotated_html_preview {
    use super::announce_behavior;
    use rstest::rstest;

    #[test]
    fn bibliography_type_templates_preserve_sparse_component_indices() {
        announce_behavior(
            "Annotated bibliography HTML should preserve original type-template indices when intermediate components do not render.",
        );
        super::bibliography_html_injects_sparse_indices_from_type_template();
    }

    #[rstest]
    #[case::default_template(false, "default bibliography template")]
    #[case::type_template(true, "matching bibliography type-template")]
    fn given_a_top_level_list_when_rendering_annotated_html_then_list_children_inherit_the_parent_index(
        #[case] use_type_template: bool,
        #[case] source: &str,
    ) {
        announce_behavior(&format!(
            "Annotated bibliography HTML should map list-rendered child wrappers back to the parent top-level index when rendering via {source}."
        ));
        super::assert_list_preview_inherits_parent_index(use_type_template);
    }
}
