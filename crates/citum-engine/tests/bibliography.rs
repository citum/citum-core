#![allow(missing_docs, reason = "test")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::{Processor, render::html::Html, render::plain::PlainText};
use citum_schema::{
    BibliographySpec, CitationSpec, Style, StyleInfo,
    options::{
        AndOptions, ArticleJournalBibliographyConfig, ArticleJournalNoPageFallback,
        BibliographyOptions, Config, ContributorConfig, DelimiterPrecedesLast,
        DemoteNonDroppingParticle, DisplayAsSort, LinkAnchor, LinkTarget, LinksConfig,
        MultilingualConfig, MultilingualMode, Processing, ProcessingCustom, Sort, SortKey,
        SortSpec,
    },
    reference::{
        Contributor, EdtfString, InputReference, Monograph, MonographType, Numbering,
        NumberingType, Serial, SerialComponent, SerialComponentType, SerialType, StructuredName,
        Title, WorkRelation,
        types::{ArchiveInfo, EprintInfo, MultilingualComplex, MultilingualString},
    },
    template::{
        DateForm, DateVariable, DelimiterPunctuation, NumberVariable, Rendering, SimpleVariable,
        TemplateComponent, TemplateDate, TemplateGroup, TemplateNumber, TemplateTitle,
        TemplateVariable, TitleForm, TitleType,
    },
};
use std::collections::HashMap;
use std::fs;
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
            wrap: Some(citum_schema::template::WrapPunctuation::Brackets.into()),
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

fn build_group_with_suppressed_child_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Grouped Suppression Test".to_string()),
            id: Some("grouped-suppression-test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![TemplateComponent::Group(TemplateGroup {
                group: vec![
                    TemplateComponent::Variable(TemplateVariable {
                        variable: SimpleVariable::Url,
                        rendering: Rendering {
                            suppress: Some(true),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    TemplateComponent::Title(TemplateTitle {
                        title: TitleType::Primary,
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

fn build_status_bibliography_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Status Test".to_string()),
            id: Some("status-test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    ..Default::default()
                }),
                TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::Status,
                    rendering: Rendering {
                        prefix: Some(". ".to_string()),
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

fn build_anonymous_entry_policy_style() -> Style {
    let style_yaml = r#"
info:
  title: Anonymous Entry Policy Test
  id: anonymous-entry-policy-test
bibliography:
  type-variants:
    entry-dictionary:
      - contributor: author
        form: long
      - variable: version
      - title: primary
      - title: parent-monograph
      - date: issued
        form: year
      - variable: doi
        prefix: "https://doi.org/"
      - variable: url
    entry-encyclopedia:
      - contributor: author
        form: long
      - title: primary
      - title: parent-serial
      - date: issued
        form: year
      - variable: doi
        prefix: "https://doi.org/"
      - variable: url
"#;

    serde_yaml::from_str(style_yaml).expect("style should parse")
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
            wrap: Some(citum_schema::template::WrapPunctuation::Brackets.into()),
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
                                            citum_schema::template::WrapPunctuation::Parentheses
                                                .into(),
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

fn build_archive_eprint_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Archive and Eprint Test".to_string()),
            id: Some("archive-eprint-test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_title!(Primary, suffix = ". "),
                variable_component(SimpleVariable::ArchiveName, None, None),
                variable_component(SimpleVariable::ArchiveCollection, Some(", "), None),
                variable_component(SimpleVariable::ArchiveCollectionId, Some(", "), None),
                variable_component(SimpleVariable::ArchiveSeries, Some(", Series "), None),
                variable_component(SimpleVariable::ArchiveBox, Some(", Box "), None),
                variable_component(SimpleVariable::ArchiveFolder, Some(", Folder "), None),
                variable_component(SimpleVariable::ArchiveItem, Some(", Item "), None),
                variable_component(SimpleVariable::ArchiveLocation, Some(", "), None),
                variable_component(SimpleVariable::ArchivePlace, Some(", "), None),
                variable_component(SimpleVariable::ArchiveUrl, Some(", "), None),
                variable_component(SimpleVariable::EprintServer, Some(", "), None),
                variable_component(SimpleVariable::EprintId, Some(":"), None),
                variable_component(SimpleVariable::EprintClass, Some(" ["), Some("]")),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_archive_location_fallback_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Archive Location Fallback Test".to_string()),
            id: Some("archive-location-fallback-test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                citum_schema::tc_title!(Primary, suffix = ". "),
                variable_component(SimpleVariable::ArchiveName, None, None),
                variable_component(SimpleVariable::ArchiveCollection, Some(", "), None),
                variable_component(SimpleVariable::ArchiveLocation, Some(", "), None),
                variable_component(SimpleVariable::ArchivePlace, Some(", "), None),
                variable_component(SimpleVariable::ArchiveUrl, Some(", "), None),
                variable_component(SimpleVariable::EprintServer, Some(", "), None),
                variable_component(SimpleVariable::EprintId, Some(":"), None),
                variable_component(SimpleVariable::EprintClass, Some(" ["), Some("]")),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn variable_component(
    variable: SimpleVariable,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> TemplateComponent {
    TemplateComponent::Variable(TemplateVariable {
        variable,
        rendering: Rendering {
            prefix: prefix.map(str::to_string),
            suffix: suffix.map(str::to_string),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn build_multilingual_archive_name_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Multilingual Archive Name Test".to_string()),
            id: Some("multilingual-archive-name-test".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            multilingual: Some(MultilingualConfig {
                name_mode: Some(MultilingualMode::Transliterated),
                preferred_script: Some("Latn".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::ArchiveName,
                ..Default::default()
            })]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_archive_eprint_reference() -> InputReference {
    InputReference::Monograph(Box::new(Monograph {
        short_title: None,
        id: Some("archive-eprint-ref".to_string()),
        r#type: MonographType::Preprint,
        title: Some(Title::Single("Archive-Aware Preprint".to_string())),
        container: None,
        author: None,
        editor: None,
        translator: None,
        issued: EdtfString("2026-02".to_string()),
        publisher: None,
        url: Some(Url::parse("https://arxiv.org/abs/2602.01234").expect("url should parse")),
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        numbering: Default::default(),
        genre: None,
        medium: None,
        archive: None,
        archive_location: None,
        archive_info: Some(ArchiveInfo {
            name: Some(MultilingualString::Simple("Houghton Library".to_string())),
            place: Some("Cambridge, MA".to_string()),
            collection: Some("Ada Lovelace Papers".to_string()),
            collection_id: Some("MS Am 1280".to_string()),
            series: Some("Correspondence".to_string()),
            r#box: Some("12".to_string()),
            folder: Some("4".to_string()),
            item: Some("7".to_string()),
            url: Some(Url::parse("https://example.com/archive").expect("url should parse")),
            ..Default::default()
        }),
        eprint: Some(EprintInfo {
            id: "2602.01234".to_string(),
            server: "arxiv".to_string(),
            class: Some("cs.DL".to_string()),
        }),
        keywords: None,
        original: None,
        ads_bibcode: None,
        ..Default::default()
    }))
}

fn make_multilingual_archive_name_reference() -> InputReference {
    InputReference::Monograph(Box::new(Monograph {
        short_title: None,
        id: Some("archive-name-ref".to_string()),
        r#type: MonographType::Document,
        title: Some(Title::Single("Repository Record".to_string())),
        container: None,
        author: None,
        editor: None,
        translator: None,
        issued: EdtfString("2024".to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        numbering: Default::default(),
        genre: None,
        medium: None,
        archive: None,
        archive_location: None,
        archive_info: Some(ArchiveInfo {
            name: Some(MultilingualString::Complex(MultilingualComplex {
                original: "東京国立博物館".to_string(),
                lang: Some("ja".to_string()),
                transliterations: HashMap::from([(
                    "ja-Latn-hepburn".to_string(),
                    "Tokyo National Museum".to_string(),
                )]),
                translations: HashMap::new(),
            })),
            ..Default::default()
        }),
        eprint: None,
        keywords: None,
        original: None,
        ads_bibcode: None,
        ..Default::default()
    }))
}

fn make_historical_archive_reference() -> InputReference {
    InputReference::Monograph(Box::new(Monograph {
        short_title: None,
        id: Some("dead-sea-scrolls-demo".to_string()),
        r#type: MonographType::Manuscript,
        title: Some(Title::Single("The Community Rule (1QS)".to_string())),
        container: None,
        author: None,
        editor: None,
        translator: None,
        issued: EdtfString("-0099".to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        numbering: Default::default(),
        genre: Some("manuscript-scroll".to_string()),
        medium: None,
        archive: None,
        archive_location: None,
        archive_info: Some(ArchiveInfo {
            name: Some(MultilingualString::Simple(
                "Israel Antiquities Authority".to_string(),
            )),
            location: Some("Shrine of the Book".to_string()),
            place: Some("Jerusalem".to_string()),
            ..Default::default()
        }),
        eprint: None,
        keywords: None,
        original: None,
        ads_bibcode: None,
        ..Default::default()
    }))
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

#[test]
fn checked_in_archive_demo_style_renders_historical_manuscript_with_bc_suffix() {
    announce_behavior(
        "The checked-in archival demo style renders historical manuscript years with a BC suffix.",
    );

    let style_path = project_root()
        .join("examples")
        .join("archive-eprint-demo-style.yaml");
    let style_yaml = fs::read_to_string(&style_path).expect("demo style should load");
    let style: Style = serde_yaml::from_str(&style_yaml).expect("demo style should parse");

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "dead-sea-scrolls-demo".to_string(),
        make_historical_archive_reference(),
    );

    let rendered = Processor::new(style, bib).render_bibliography();
    assert!(
        rendered.contains(
            "The Community Rule (1QS). Manuscript scroll, 100 BC, Israel Antiquities Authority, Shrine of the Book, Jerusalem"
        ),
        "historical manuscript output should match the checked-in docs example: {rendered}"
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
    let mut numbering = vec![Numbering {
        r#type: NumberingType::Volume,
        value: "12".to_string(),
    }];
    if let Some(i) = issue {
        numbering.push(Numbering {
            r#type: NumberingType::Issue,
            value: i.to_string(),
        });
    }

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
        container: Some(WorkRelation::Embedded(Box::new(InputReference::Serial(
            Box::new(Serial {
                r#type: SerialType::AcademicJournal,
                title: Some(Title::Single("Journal of Fallbacks".to_string())),
                ..Default::default()
            }),
        )))),
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        doi: doi.map(str::to_string),
        ads_bibcode: None,
        pages: pages.map(str::to_string),
        numbering,
        genre: None,
        medium: None,
        archive_info: None,
        eprint: None,
        keywords: None,
        reviewed: None,
        original: None,
        ..Default::default()
    }))
}

struct EntryReferenceParams<'a> {
    id: &'a str,
    entry_type: &'a str,
    title: &'a str,
    container_title: &'a str,
    year: i32,
    doi: Option<&'a str>,
    url: Option<&'a str>,
    author: Option<(&'a str, &'a str)>,
}

fn make_entry_reference(params: EntryReferenceParams<'_>) -> InputReference {
    let mut fixture = serde_json::json!({
        "id": params.id,
        "type": params.entry_type,
        "title": params.title,
        "container-title": params.container_title,
        "issued": {"date-parts": [[params.year]]},
    });

    if let Some(doi) = params.doi {
        fixture["DOI"] = serde_json::json!(doi);
    }
    if let Some(url) = params.url {
        fixture["URL"] = serde_json::json!(url);
    }
    if let Some((family, given)) = params.author {
        fixture["author"] = serde_json::json!([{ "family": family, "given": given }]);
    }

    let legacy: csl_legacy::csl_json::Reference =
        serde_json::from_value(fixture).expect("entry fixture should parse");
    legacy.into()
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
        short_title: None,
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(format!("Title {id}"))),
        container: None,
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
        numbering: Default::default(),
        genre: None,
        medium: None,
        archive_info: None,
        eprint: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original: None,
        ads_bibcode: None,
        ..Default::default()
    }))
}

fn make_editor_only_book(
    id: &str,
    title: &str,
    year: &str,
    family: &str,
    given: &str,
) -> InputReference {
    InputReference::Monograph(Box::new(Monograph {
        short_title: None,
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title.to_string())),
        container: None,
        author: None,
        editor: Some(Contributor::StructuredName(StructuredName {
            given: given.into(),
            family: family.into(),
            ..Default::default()
        })),
        translator: None,
        issued: EdtfString(year.to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        numbering: Default::default(),
        genre: None,
        medium: None,
        archive_info: None,
        eprint: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original: None,
        ads_bibcode: None,
        ..Default::default()
    }))
}

fn make_multi_editor_only_book(
    id: &str,
    title: &str,
    year: &str,
    editors: Vec<(&str, &str)>,
) -> InputReference {
    let editors = editors
        .into_iter()
        .map(|(family, given)| {
            Contributor::StructuredName(StructuredName {
                given: given.into(),
                family: family.into(),
                ..Default::default()
            })
        })
        .collect();

    InputReference::Monograph(Box::new(Monograph {
        short_title: None,
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title.to_string())),
        container: None,
        author: None,
        editor: Some(Contributor::ContributorList(
            citum_schema::reference::ContributorList(editors),
        )),
        translator: None,
        issued: EdtfString(year.to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        numbering: Default::default(),
        genre: None,
        medium: None,
        archive_info: None,
        eprint: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original: None,
        ads_bibcode: None,
        ..Default::default()
    }))
}

fn make_editor_substitute_bibliography() -> indexmap::IndexMap<String, InputReference> {
    citum_schema::bib_map![
        "ancient-tale" => make_editor_only_book(
            "ancient-tale",
            "The Ancient Tale",
            "1850",
            "Grimm",
            "Jacob",
        ),
        "ipcc2023" => make_multi_editor_only_book(
            "ipcc2023",
            "Climate Change 2023: Synthesis Report",
            "2023",
            vec![("Lee", "Hoesung"), ("Romero", "Jose")],
        ),
    ]
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

#[test]
fn legal_case_parent_serial_uses_reporter_as_container_title() {
    let style = Style {
        info: StyleInfo {
            title: Some("Legal Reporter Parent Serial Test".to_string()),
            id: Some("legal-reporter-parent-serial-test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![TemplateComponent::Group(TemplateGroup {
                group: vec![
                    TemplateComponent::Title(TemplateTitle {
                        title: TitleType::ParentSerial,
                        ..Default::default()
                    }),
                    TemplateComponent::Number(TemplateNumber {
                        number: NumberVariable::Volume,
                        ..Default::default()
                    }),
                    TemplateComponent::Number(TemplateNumber {
                        number: NumberVariable::Pages,
                        ..Default::default()
                    }),
                ],
                delimiter: Some(DelimiterPunctuation::Slash),
                ..Default::default()
            })]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "ITEM-LEGAL-1",
        "type": "legal_case",
        "title": "Brown v. Board of Education",
        "authority": "U.S. Supreme Court",
        "volume": "347",
        "container-title": "U.S. Reports",
        "page": "483",
        "issued": { "date-parts": [[1954, 5, 17]] }
    }))
    .expect("legal case fixture should parse");

    let mut bib = indexmap::IndexMap::new();
    bib.insert("ITEM-LEGAL-1".to_string(), legacy.into());

    let processor = Processor::new(style, bib);
    assert_eq!(processor.render_bibliography(), "U.S. Reports/347/483");
}

fn suppressed_group_children_do_not_leave_stray_delimiters() {
    let style = build_group_with_suppressed_child_style();
    let mut reference = make_book("grouped", "Smith", "Jane", 2024, "Grouped Title");
    if let InputReference::Monograph(book) = &mut reference {
        book.url = Some(Url::parse("https://example.com/grouped").expect("url should parse"));
    }

    let mut bib = indexmap::IndexMap::new();
    bib.insert("grouped".to_string(), reference);

    let processor = Processor::new(style, bib);
    assert_eq!(processor.render_bibliography(), "Grouped Title");
}

fn status_variables_render_in_bibliography_templates() {
    let style = build_status_bibliography_style();
    let mut reference = make_book(
        "status",
        "Lexicographer",
        "A.",
        2025,
        "Oxford English Dictionary",
    );
    if let InputReference::Monograph(book) = &mut reference {
        book.status = Some("last modified".to_string());
    }

    let mut bib = indexmap::IndexMap::new();
    bib.insert("status".to_string(), reference);

    let processor = Processor::new(style, bib);
    assert_eq!(
        processor.render_bibliography(),
        "Oxford English Dictionary. last modified"
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

fn type_variant_article_journal_fallback_preserves_variant_precedence() {
    let style_yaml = r#"
info:
  title: Type Variant Regression
  id: type-variant-regression
bibliography:
  options:
    article-journal:
      no-page-fallback: doi
  template:
    - title: primary
      prefix: "DEFAULT "
  type-variants:
    article-journal:
      - contributor: author
        form: long
      - number: volume
        prefix: ", "
      - number: pages
        prefix: ": "
      - variable: doi
        prefix: " DOI: "
"#;
    let style: Style = serde_yaml::from_str(style_yaml).expect("style should parse");

    let without_pages: csl_legacy::csl_json::Reference =
        serde_json::from_value(serde_json::json!({
            "id": "without-pages",
            "type": "article-journal",
            "title": "Fallback Title",
            "author": [{"family": "Smith", "given": "Jane"}],
            "issued": {"date-parts": [[2020]]},
            "volume": "12",
            "DOI": "10.1000/no-pages"
        }))
        .expect("no-pages fixture should parse");
    let with_pages: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "with-pages",
        "type": "article-journal",
        "title": "Detailed Title",
        "author": [{"family": "Jones", "given": "Alex"}],
        "issued": {"date-parts": [[2021]]},
        "volume": "18",
        "page": "33-40",
        "DOI": "10.1000/with-pages"
    }))
    .expect("with-pages fixture should parse");

    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert("without-pages".to_string(), without_pages.into());
    bibliography.insert("with-pages".to_string(), with_pages.into());

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_selected_bibliography_with_format::<PlainText, _>([
        "without-pages".to_string(),
        "with-pages".to_string(),
    ]);

    assert!(
        rendered.contains("Smith DOI: 10.1000/no-pages"),
        "DOI fallback should keep the type-variant output: {rendered}"
    );
    assert!(
        rendered.contains("Alex Jones, 18: 33–40"),
        "page-bearing articles should retain the detail block from the type variant: {rendered}"
    );
    assert!(
        !rendered.contains("DEFAULT Fallback Title")
            && !rendered.contains("DEFAULT Detailed Title"),
        "matching type variants must take precedence over the default bibliography template: {rendered}"
    );
    assert!(
        !rendered.contains("10.1000/with-pages"),
        "standard-detail mode should still suppress DOI when pages are present: {rendered}"
    );
}

fn anonymous_entry_type_variants_reorder_online_entries_and_drop_print_fallback_rows() {
    let style = build_anonymous_entry_policy_style();

    let online_encyclopedia = make_entry_reference(EntryReferenceParams {
        id: "online-encyclopedia",
        entry_type: "entry-encyclopedia",
        title: "Stevie Nicks",
        container_title: "Wikipedia",
        year: 2025,
        doi: None,
        url: Some("https://en.wikipedia.org/w/index.php?title=Stevie_Nicks&oldid=1279222290"),
        author: None,
    });
    let print_dictionary = make_entry_reference(EntryReferenceParams {
        id: "print-dictionary",
        entry_type: "entry-dictionary",
        title: "hootenanny, n.",
        container_title: "Oxford English Dictionary",
        year: 1976,
        doi: None,
        url: None,
        author: None,
    });
    let print_encyclopedia = make_entry_reference(EntryReferenceParams {
        id: "print-encyclopedia",
        entry_type: "entry-encyclopedia",
        title: "Feathers",
        container_title: "Johnson's Universal Cyclopaedia",
        year: 1886,
        doi: None,
        url: None,
        author: None,
    });
    let authorful_encyclopedia = make_entry_reference(EntryReferenceParams {
        id: "authorful-encyclopedia",
        entry_type: "entry-encyclopedia",
        title: "Ellington, Duke",
        container_title: "Grove Music Online",
        year: 2013,
        doi: Some("10.1093/gmo/9781561592630.article.A2249397"),
        url: None,
        author: Some(("Piras", "Marcello")),
    });

    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert("print-dictionary".to_string(), print_dictionary);
    bibliography.insert("print-encyclopedia".to_string(), print_encyclopedia);
    bibliography.insert("online-encyclopedia".to_string(), online_encyclopedia);
    bibliography.insert("authorful-encyclopedia".to_string(), authorful_encyclopedia);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_selected_bibliography_with_format::<PlainText, _>([
        "print-dictionary".to_string(),
        "print-encyclopedia".to_string(),
        "online-encyclopedia".to_string(),
        "authorful-encyclopedia".to_string(),
    ]);

    let rendered_lines: Vec<&str> = rendered
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    assert_eq!(
        rendered_lines.len(),
        2,
        "anonymous print-like dictionary and encyclopedia rows should be suppressed: {rendered}"
    );
    assert!(
        rendered.contains(
            "Wikipedia. 2025. Stevie Nicks. https://en.wikipedia.org/w/index.php?title=Stevie_Nicks&oldid=1279222290"
        ),
        "online anonymous encyclopedia entries should be container-led and URL-backed: {rendered}"
    );
    assert!(
        !rendered.contains("1976") && !rendered.contains("Johnson's Universal Cyclopaedia"),
        "print-like anonymous dictionary and encyclopedia rows should be dropped: {rendered}"
    );
    assert!(
        rendered.contains("Marcello Piras"),
        "authorful encyclopedia entries should still render instead of being suppressed: {rendered}"
    );
}

#[test]
fn apa_dataset_without_title_falls_back_to_bracketed_label_version_and_doi() {
    let style = load_style("styles/apa-7th.yaml");
    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "apa-titleless-dataset",
        "type": "dataset",
        "author": [{ "family": "Author", "given": "First A." }],
        "issued": { "date-parts": [[2013]] },
        "DOI": "10.1234/5678",
        "genre": "Untitled dataset",
        "version": "2.1",
        "language": "en"
    }))
    .expect("dataset fixture should parse");

    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert("apa-titleless-dataset".to_string(), legacy.into());

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_selected_bibliography_with_format::<PlainText, _>([
        "apa-titleless-dataset".to_string(),
    ]);

    assert_eq!(
        rendered.trim(),
        "Author, F. A. (2013). [Untitled dataset] (Version 2.1). https://doi.org/10.1234/5678"
    );
}

#[test]
fn apa_web_native_entries_render_without_retrieved_fallbacks() {
    let style = load_style("styles/apa-7th.yaml");
    let legacy_items = [
        serde_json::json!({
            "id": "6188419/IC98IKSD",
            "type": "webpage",
            "container-title": "Website title",
            "genre": "page type",
            "language": "en",
            "note": "part-number: 1\npart-title: Part title\neditor: Editor || A. A.",
            "title": "58 Web page",
            "URL": "https://example.com",
            "author": [{ "family": "Author", "given": "A. A." }],
            "translator": [{ "family": "Translator", "given": "A. A." }],
            "accessed": { "date-parts": [[2018, 7, 15]] },
            "issued": { "date-parts": [[2018]] }
        }),
        serde_json::json!({
            "id": "6188419/XA2MLUAS",
            "type": "post-weblog",
            "container-title": "Website title",
            "genre": "Type",
            "language": "en",
            "title": "59 Blog post",
            "URL": "https://example.com",
            "author": [{ "family": "Author", "given": "A. A." }],
            "accessed": { "date-parts": [[2018, 7, 15]] },
            "issued": { "date-parts": [[2018]] }
        }),
        serde_json::json!({
            "id": "6188419/HCFRWJZR",
            "type": "post",
            "container-title": "Website title",
            "genre": "Type",
            "language": "la",
            "title": "60 Forum post",
            "URL": "https://example.com",
            "author": [{ "family": "Author", "given": "A. A." }],
            "accessed": { "date-parts": [[2018, 7, 15]] },
            "issued": { "date-parts": [[2018]] }
        }),
    ];

    let bibliography = legacy_items
        .into_iter()
        .map(|value| {
            let legacy: csl_legacy::csl_json::Reference =
                serde_json::from_value(value).expect("fixture should parse");
            (legacy.id.clone(), legacy.into())
        })
        .collect();

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_selected_bibliography_with_format::<PlainText, _>([
        "6188419/IC98IKSD".to_string(),
        "6188419/XA2MLUAS".to_string(),
        "6188419/HCFRWJZR".to_string(),
    ]);

    let lines = rendered
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();

    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0],
        "Author, A. A. (2018a). 58 Web page: Pt. 1. Part title (A. A. Editor, ed.; A. A. Translator, Trans.) [Page type]. Website Title. https://example.com/"
    );
    assert_eq!(
        lines[1],
        "Author, A. A. (2018b). 59 Blog post [Type]. Website Title. https://example.com/"
    );
    assert_eq!(
        lines[2],
        "Author, A. A. (2018c). 60 Forum post [Type]. Website title. https://example.com/"
    );
    assert!(!rendered.contains("Retrieved "));
}

#[test]
fn apa_magazine_and_newspaper_entries_keep_special_format_translators_and_direct_urls() {
    let style = load_style("styles/apa-7th.yaml");
    let legacy_items = [
        serde_json::json!({
            "id": "6188419/BXMWCMVJ",
            "type": "article-magazine",
            "container-title": "Journal Title",
            "ISSN": "0000-0000",
            "issue": "5",
            "language": "en",
            "note": "medium: special format\ngenre: type\nsection: department",
            "page": "1-100",
            "title": "15 Magazine article",
            "URL": "http://example.com",
            "volume": "32",
            "author": [{ "family": "Author", "given": "First A." }],
            "translator": [{ "family": "Translator", "given": "Third A." }],
            "issued": { "date-parts": [[2018, 7, 14]] }
        }),
        serde_json::json!({
            "id": "6188419/389M98AT",
            "type": "article-newspaper",
            "container-title": "Newspaper Title",
            "edition": "evening",
            "ISSN": "0000-0000",
            "language": "en",
            "note": "medium: Special format\ngenre: Type",
            "page": "1-100",
            "title": "17 Newspaper article",
            "URL": "http://example.com",
            "author": [{ "family": "Author", "given": "First A." }],
            "translator": [{ "family": "Translator", "given": "Third A." }],
            "issued": { "date-parts": [[2018, 7, 14]] }
        }),
    ];

    let bibliography = legacy_items
        .into_iter()
        .map(|value| {
            let legacy: csl_legacy::csl_json::Reference =
                serde_json::from_value(value).expect("fixture should parse");
            (legacy.id.clone(), legacy.into())
        })
        .collect();

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_selected_bibliography_with_format::<PlainText, _>([
        "6188419/BXMWCMVJ".to_string(),
        "6188419/389M98AT".to_string(),
    ]);

    let lines = rendered
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Author, F. A. (2018a, "));
    assert!(lines[0].contains("15 Magazine article (T. A. Translator, Trans.)"));
    assert!(lines[0].contains("[Type; Special format]"));
    assert!(lines[0].contains("1–100. http://example.com/"));
    assert!(lines[1].contains("Author, F. A. (2018b, "));
    assert!(lines[1].contains("17 Newspaper article (T. A. Translator, Trans.)"));
    assert!(lines[1].contains("[Type; Special format]"));
    assert!(lines[1].contains("Newspaper Title"));
    assert!(lines[1].contains("1–100"));
    assert!(lines[1].contains("http://example.com/"));
    assert!(!rendered.contains("Retrieved "));
}

#[test]
fn apa_structural_entries_use_component_packaging_instead_of_generic_fallbacks() {
    let style = load_style("styles/apa-7th.yaml");
    let legacy_items = [
        serde_json::json!({
            "id": "6188419/RYT8J733",
            "type": "report",
            "genre": "Technical report",
            "note": "container-title: Report title",
            "page": "126-145",
            "publisher": "Publisher",
            "publisher-place": "City, ST",
            "title": "24 Chapter in a report",
            "URL": "https://example.com",
            "author": [{ "family": "Chapter", "given": "Author M., Jr." }],
            "editor": [
                { "family": "Editor", "given": "First A." },
                { "family": "Editor", "given": "Second" }
            ],
            "number": "12345",
            "issued": { "date-parts": [[2016]] }
        }),
        serde_json::json!({
            "id": "6188419/Q2MWRA2D",
            "type": "entry-encyclopedia",
            "container-title": "Title of book: a subtitle",
            "DOI": "10.1234/5678",
            "edition": "2",
            "note": "original-date: 1901\noriginal-title: Original title\ncontainer-title-short: Title of book",
            "page": "123-128",
            "publisher": "Publisher",
            "publisher-place": "Place, ST",
            "title": "45 Encyclopedia entry",
            "URL": "http://example.com",
            "volume": "2",
            "translator": [{ "family": "Editor", "given": "S. S." }],
            "editor": [{ "family": "Editor", "given": "S. S." }],
            "author": [{ "family": "Author", "given": "First A." }],
            "issued": { "date-parts": [[2013]] }
        }),
        serde_json::json!({
            "id": "6188419/2G36L2LR",
            "type": "paper-conference",
            "container-title": "Proceedings",
            "DOI": "10.1234/5678",
            "note": "event-date: 2010",
            "page": "123-128",
            "publisher": "Publisher",
            "publisher-place": "Place, ST",
            "title": "56 Conference paper",
            "URL": "http://example.com",
            "volume": "2",
            "translator": [{ "family": "Editor", "given": "S. S." }],
            "editor": [{ "family": "Editor", "given": "S. S." }],
            "author": [{ "family": "Author", "given": "First A." }],
            "issued": { "date-parts": [[2013]] }
        }),
    ];

    let bibliography = legacy_items
        .into_iter()
        .map(|value| {
            let legacy: csl_legacy::csl_json::Reference =
                serde_json::from_value(value).expect("fixture should parse");
            (legacy.id.clone(), legacy.into())
        })
        .collect();

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_selected_bibliography_with_format::<PlainText, _>([
        "6188419/RYT8J733".to_string(),
        "6188419/Q2MWRA2D".to_string(),
        "6188419/2G36L2LR".to_string(),
    ]);

    let lines = rendered
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();

    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0],
        "Author, F. A. (2013a) (2). 45 Encyclopedia entry (S. S. Editor, Trans.). In S. S. Editor, ed., _Title of book: a subtitle_ (2 ed., pp. 123–128). Publisher. https://doi.org/10.1234/5678 http://example.com/"
    );
    assert_eq!(
        lines[1],
        "Author, F. A. (2013b). 56 Conference paper (S. S. Editor, Trans.). In S. S. Editor, ed., _Proceedings_ (pp. 123–128). Publisher. https://doi.org/10.1234/5678 http://example.com/"
    );
    assert_eq!(
        lines[2],
        "Chapter, A. M. J. (2016). 24 Chapter in a report. In F. A. Editor, & S. Editor, eds., _Report title_ (pp. 126–145). Publisher. https://example.com/"
    );
    assert!(!rendered.contains("Retrieved "));
    assert!(!rendered.contains("[Technical report]"));
}

fn bibliography_local_entry_links_apply_on_the_default_render_path() {
    let style = build_bibliography_entry_link_style();
    let reference = InputReference::Monograph(Box::new(Monograph {
        short_title: None,
        id: Some("linked-book".to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single("Linked Book".to_string())),
        container: None,
        author: None,
        editor: None,
        translator: None,
        issued: EdtfString("2024".to_string()),
        publisher: None,
        url: Some(Url::parse("https://example.com/linked-book").expect("valid url")),
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        numbering: Default::default(),
        genre: None,
        medium: None,
        archive_info: None,
        eprint: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original: None,
        ads_bibcode: None,
        ..Default::default()
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

#[test]
fn editor_author_substitute_omits_verb_role_label_in_bibliography() {
    let mut style = load_style("styles/apa-7th.yaml");
    let config = style.options.get_or_insert_with(Default::default);
    let contributors = config.contributors.get_or_insert_with(Default::default);
    contributors.role = Some(citum_schema::options::contributors::RoleOptions {
        preset: Some(citum_schema::options::contributors::RoleLabelPreset::VerbPrefix),
        ..Default::default()
    });

    let bib = make_editor_substitute_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor
        .render_selected_bibliography_with_format::<citum_engine::render::plain::PlainText, _>(
            vec!["ancient-tale".to_string(), "ipcc2023".to_string()],
        );

    assert!(
        result.contains("Grimm, J. (1850). _The Ancient Tale_"),
        "editor substitute should render as the effective author without a verb label: {result}"
    );
    assert!(
        result.contains("Lee, H., & Romero, J. (2023). _Climate Change 2023: Synthesis Report_"),
        "multi-editor substitute should render as names only when occupying the author slot: {result}"
    );
    assert!(
        !result.contains("edited by Jacob Grimm") && !result.contains("edited by Hoesung Lee"),
        "verb-prefix labels should not survive when editors substitute into the author slot: {result}"
    );
}

#[test]
fn given_archive_info_and_eprint_when_rendering_bibliography_then_new_variables_resolve() {
    let style = build_archive_eprint_style();
    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "archive-eprint-ref".to_string(),
        make_archive_eprint_reference(),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert_eq!(
        result,
        "Archive-Aware Preprint. Houghton Library, Ada Lovelace Papers, MS Am 1280, Series Correspondence, Box 12, Folder 4, Item 7, Cambridge, MA, https://example.com/archive, arxiv:2602.01234 [cs.DL]"
    );
}

#[test]
fn given_archive_location_override_when_rendering_bibliography_then_legacy_fallback_still_works() {
    let style = build_archive_location_fallback_style();
    let mut bib = indexmap::IndexMap::new();
    let mut reference = make_archive_eprint_reference();

    let InputReference::Monograph(monograph) = &mut reference else {
        panic!("archive test fixture should be a monograph");
    };
    monograph.id = Some("archive-eprint-location-ref".to_string());
    monograph.archive_info = Some(ArchiveInfo {
        name: Some(MultilingualString::Simple("Houghton Library".to_string())),
        place: Some("Cambridge, MA".to_string()),
        collection: Some("Ada Lovelace Papers".to_string()),
        location: Some("MS Am 1280, Box 12, Folder 4".to_string()),
        url: Some(Url::parse("https://example.com/archive").expect("url should parse")),
        ..Default::default()
    });
    bib.insert("archive-eprint-location-ref".to_string(), reference);

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert_eq!(
        result,
        "Archive-Aware Preprint. Houghton Library, Ada Lovelace Papers, MS Am 1280, Box 12, Folder 4, Cambridge, MA, https://example.com/archive, arxiv:2602.01234 [cs.DL]"
    );
}

#[test]
fn given_multilingual_archive_name_when_rendering_then_name_mode_controls_archive_name() {
    let style = build_multilingual_archive_name_style();
    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "archive-name-ref".to_string(),
        make_multilingual_archive_name_reference(),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    assert_eq!(result, "Tokyo National Museum");
}

#[test]
fn given_legacy_archive_fields_when_converting_then_archive_info_is_hydrated() {
    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "ITEM-ARCHIVE-1",
        "type": "manuscript",
        "title": "Commonplace Book",
        "issued": { "date-parts": [[1720]] },
        "archive": "Bodleian Library",
        "archive_location": "MS Bodl. Or. 579, fol. 23r"
    }))
    .expect("legacy reference should parse");

    let reference: InputReference = legacy.into();
    if let InputReference::Monograph(m) = reference {
        assert_eq!(m.archive, Some("Bodleian Library".to_string()));
        assert_eq!(
            m.archive_location,
            Some("MS Bodl. Or. 579, fol. 23r".to_string())
        );

        let archive_info = m.archive_info.expect("archive info should be hydrated");
        assert_eq!(
            archive_info
                .name
                .expect("archive name should exist")
                .to_string(),
            "Bodleian Library"
        );
        assert_eq!(
            archive_info.location,
            Some("MS Bodl. Or. 579, fol. 23r".to_string())
        );
    } else {
        panic!("Expected Monograph");
    }
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

    #[test]
    fn suppressed_group_children_do_not_leave_stray_delimiters() {
        announce_behavior(
            "Suppressed children inside grouped bibliography components should not leave stray delimiters.",
        );
        super::suppressed_group_children_do_not_leave_stray_delimiters();
    }

    #[test]
    fn status_variables_render_in_bibliography_templates() {
        announce_behavior(
            "Bibliography templates should render status variables when the reference carries publication-status metadata.",
        );
        super::status_variables_render_in_bibliography_templates();
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
    fn matching_type_variants_keep_precedence_when_article_journal_fallbacks_apply() {
        announce_behavior(
            "A matching article-journal type variant should keep precedence over the default bibliography template even when the no-page DOI fallback toggles between detail and DOI output.",
        );
        super::type_variant_article_journal_fallback_preserves_variant_precedence();
    }

    #[test]
    fn royal_society_of_chemistry_restores_the_legacy_page_less_doi_behavior() {
        announce_behavior(
            "The Royal Society of Chemistry bibliography should restore the legacy page-less journal behavior by rendering DOI instead of the standard detail block.",
        );
        super::royal_society_of_chemistry_restores_legacy_page_less_doi_behavior();
    }
}

mod anonymous_entry_type_variants {
    use super::announce_behavior;

    #[test]
    fn chicago_like_anonymous_entries_prefer_online_container_rows_and_drop_print_fallbacks() {
        announce_behavior(
            "Chicago-like anonymous dictionary and encyclopedia bibliography variants should keep online entries in container-led order while omitting the print-style fallback rows that the oracle does not emit.",
        );
        super::anonymous_entry_type_variants_reorder_online_entries_and_drop_print_fallback_rows();
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
