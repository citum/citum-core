use super::*;
use crate::Processor;
use crate::processor::rendering::grouped::group_citation_items_by_author;
use citum_schema::citation::{Citation, CitationItem, CitationMode, IntegralNameState};
use citum_schema::options::{
    Config, IntegralNameConfig, IntegralNameContexts, IntegralNameForm, IntegralNameRule,
    IntegralNameScope, Processing,
};
use citum_schema::template::*;
use citum_schema::{CitationSpec, Style, StyleInfo};
use csl_legacy::csl_json::{
    DateVariable as LegacyDateVariable, Name, Reference as LegacyReference,
};

fn make_reference(
    id: &str,
    ref_type: &str,
    author: Option<(&str, &str)>,
    year: i32,
    title: &str,
) -> Reference {
    Reference::from(LegacyReference {
        id: id.to_string(),
        ref_type: ref_type.to_string(),
        author: author.map(|(family, given)| vec![Name::new(family, given)]),
        title: Some(title.to_string()),
        issued: Some(LegacyDateVariable::year(year)),
        ..Default::default()
    })
}

fn grouped_author_date_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Grouped Author Date".to_string()),
            id: Some("grouped-author-date".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: citum_schema::template::DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering {
                        prefix: Some(", ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]),
            wrap: Some(WrapPunctuation::Parentheses),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn integral_name_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Integral Name Memory".to_string()),
            id: Some("integral-name-memory".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            integral_names: Some(IntegralNameConfig {
                rule: Some(IntegralNameRule::FullThenShort),
                scope: Some(IntegralNameScope::Document),
                contexts: Some(IntegralNameContexts::BodyAndNotes),
                subsequent_form: Some(IntegralNameForm::Short),
            }),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            integral: Some(Box::new(CitationSpec {
                template: Some(vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    rendering: Rendering::default(),
                    ..Default::default()
                })]),
                ..Default::default()
            })),
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: citum_schema::template::DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering {
                        wrap: Some(WrapPunctuation::Parentheses),
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

fn legal_case_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Legal Case Grouping".to_string()),
            id: Some("legal-case-grouping".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    form: None,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: citum_schema::template::DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering {
                        prefix: Some(", ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]),
            wrap: Some(WrapPunctuation::Parentheses),
            multi_cite_delimiter: Some("; ".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn test_variable_key_includes_context() {
    // Date with no prefix/suffix
    let date1 = TemplateComponent::Date(TemplateDate {
        date: citum_schema::template::DateVariable::Issued,
        form: DateForm::Year,
        rendering: Rendering::default(),
        fallback: None,
        links: None,
        custom: None,
    });

    // Same date with prefix
    let date2 = TemplateComponent::Date(TemplateDate {
        date: citum_schema::template::DateVariable::Issued,
        form: DateForm::Year,
        rendering: Rendering {
            prefix: Some(", ".to_string()),
            ..Default::default()
        },
        fallback: None,
        links: None,
        custom: None,
    });

    // Same date with suffix
    let date3 = TemplateComponent::Date(TemplateDate {
        date: citum_schema::template::DateVariable::Issued,
        form: DateForm::Year,
        rendering: Rendering {
            suffix: Some(".".to_string()),
            ..Default::default()
        },
        fallback: None,
        links: None,
        custom: None,
    });

    let key1 = get_variable_key(&date1);
    let key2 = get_variable_key(&date2);
    let key3 = get_variable_key(&date3);

    // All three should have different keys due to different contexts
    assert_ne!(key1, key2);
    assert_ne!(key1, key3);
    assert_ne!(key2, key3);

    // Verify the keys include context markers
    assert_eq!(key1, Some("date:Issued".to_string()));
    assert_eq!(key2, Some("date:Issued:, ".to_string()));
    assert_eq!(key3, Some("date:Issued:.".to_string()));
}

#[test]
fn test_substituted_contributor_keys_block_contextual_duplicate_components() {
    let mut tracker = TemplateComponentTracker::default();
    let translated_component = TemplateComponent::Contributor(TemplateContributor {
        contributor: ContributorRole::Translator,
        form: ContributorForm::Long,
        rendering: Rendering {
            suffix: Some(", translator".to_string()),
            ..Default::default()
        },
        ..Default::default()
    });
    let translator_key =
        get_variable_key(&translated_component).expect("translator component should have a key");

    tracker.mark_rendered(None, Some("contributor:Translator"));

    assert!(tracker.should_skip(Some(&translator_key)));
}

#[test]
fn test_strip_author_component_nested_list() {
    let nested = TemplateComponent::Group(TemplateGroup {
        group: vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                form: ContributorForm::Short,
                and: None,
                shorten: None,
                label: None,
                name_order: None,
                name_form: None,
                delimiter: None,
                sort_separator: None,
                links: None,
                rendering: Rendering::default(),
                custom: None,
            }),
            TemplateComponent::Date(TemplateDate {
                date: citum_schema::template::DateVariable::Issued,
                form: DateForm::Year,
                rendering: Rendering::default(),
                fallback: None,
                links: None,
                custom: None,
            }),
        ],
        delimiter: Some(DelimiterPunctuation::Space),
        rendering: Rendering::default(),
        custom: None,
    });

    let filtered = strip_author_component(&nested).expect("list should remain");
    let TemplateComponent::Group(filtered_list) = filtered else {
        panic!("expected list");
    };

    assert_eq!(filtered_list.group.len(), 1);
    assert!(matches!(filtered_list.group[0], TemplateComponent::Date(_)));
}

#[test]
fn affix_content_normalizes_prefix_and_suffix_spacing() {
    let style = Style::default();
    let bibliography = Bibliography::new();
    let locale = Locale::default();
    let config = Config::default();
    let hints = HashMap::new();
    let citation_numbers = RefCell::new(HashMap::new());
    let compound_set_by_ref = HashMap::new();
    let compound_member_index = HashMap::new();
    let compound_sets = IndexMap::new();
    let renderer = Renderer::new(
        RendererResources {
            style: &style,
            bibliography: &bibliography,
            locale: &locale,
            config: &config,
        },
        &hints,
        &citation_numbers,
        CompoundRenderData {
            set_by_ref: &compound_set_by_ref,
            member_index: &compound_member_index,
            sets: &compound_sets,
        },
        true,
        false,
    );
    let fmt = crate::render::plain::PlainText;

    assert_eq!(
        renderer.affix_content(&fmt, "body".to_string(), Some("see"), Some("n. 2")),
        "see body n. 2"
    );
    assert_eq!(
        renderer.affix_content(&fmt, "body".to_string(), Some("see "), Some(", n. 2")),
        "see body, n. 2"
    );
}

#[test]
fn grouped_author_date_strips_leading_affix_from_tail_components() {
    let mut bibliography = Bibliography::new();
    bibliography.insert(
        "item1".to_string(),
        make_reference("item1", "book", Some(("Kuhn", "Thomas")), 1962, "Book A"),
    );
    bibliography.insert(
        "item2".to_string(),
        make_reference("item2", "book", Some(("Kuhn", "Thomas")), 1963, "Book B"),
    );
    let processor = Processor::new(grouped_author_date_style(), bibliography);

    let citation = Citation {
        items: vec![
            CitationItem {
                id: "item1".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "item2".to_string(),
                ..Default::default()
            },
        ],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };

    assert_eq!(
        processor
            .process_citation(&citation)
            .expect("grouped citation should render"),
        "(Kuhn, 1962, 1963)"
    );
}

#[test]
fn grouping_helper_matches_citation_wide_preserve_behavior() {
    let style = grouped_author_date_style();
    let config = style.options.clone().unwrap_or_default();
    let locale = Locale::default();
    let mut bibliography = Bibliography::new();
    bibliography.insert(
        "item1".to_string(),
        make_reference("item1", "book", Some(("Kuhn", "Thomas")), 1962, "Book A"),
    );
    bibliography.insert(
        "item2".to_string(),
        make_reference("item2", "book", Some(("Kuhn", "Thomas")), 1963, "Book B"),
    );
    bibliography.insert(
        "item3".to_string(),
        make_reference("item3", "book", Some(("Smith", "John")), 2020, "Book C"),
    );

    let mut hints = HashMap::new();
    hints.insert(
        "item3".to_string(),
        ProcHints {
            min_names_to_show: Some(2),
            ..Default::default()
        },
    );
    let citation_numbers = RefCell::new(HashMap::new());
    let compound_set_by_ref = HashMap::new();
    let compound_member_index = HashMap::new();
    let compound_sets = IndexMap::new();
    let renderer = Renderer::new(
        RendererResources {
            style: &style,
            bibliography: &bibliography,
            locale: &locale,
            config: &config,
        },
        &hints,
        &citation_numbers,
        CompoundRenderData {
            set_by_ref: &compound_set_by_ref,
            member_index: &compound_member_index,
            sets: &compound_sets,
        },
        true,
        false,
    );
    let items = vec![
        CitationItem {
            id: "item1".to_string(),
            ..Default::default()
        },
        CitationItem {
            id: "item2".to_string(),
            ..Default::default()
        },
        CitationItem {
            id: "item3".to_string(),
            ..Default::default()
        },
    ];

    let groups = group_citation_items_by_author(&renderer, &items);

    assert_eq!(groups.len(), 3);
    assert_eq!(
        groups[0]
            .1
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["item1"]
    );
    assert_eq!(
        groups[1]
            .1
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["item2"]
    );
    assert_eq!(
        groups[2]
            .1
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["item3"]
    );
}

#[test]
fn explicit_integral_template_honors_integral_name_state() {
    let mut bibliography = Bibliography::new();
    bibliography.insert(
        "item1".to_string(),
        make_reference("item1", "book", Some(("Smith", "John")), 2020, "Book A"),
    );
    let processor = Processor::new(integral_name_style(), bibliography);

    let first = Citation {
        mode: CitationMode::Integral,
        items: vec![CitationItem {
            id: "item1".to_string(),
            integral_name_state: Some(IntegralNameState::First),
            ..Default::default()
        }],
        ..Default::default()
    };
    let subsequent = Citation {
        mode: CitationMode::Integral,
        items: vec![CitationItem {
            id: "item1".to_string(),
            integral_name_state: Some(IntegralNameState::Subsequent),
            ..Default::default()
        }],
        ..Default::default()
    };

    assert_eq!(
        processor
            .process_citation(&first)
            .expect("first integral citation should render"),
        "John Smith"
    );
    assert_eq!(
        processor
            .process_citation(&subsequent)
            .expect("subsequent integral citation should render"),
        "Smith"
    );
}

#[test]
fn legal_cases_render_per_item_instead_of_grouped_year_compression() {
    let mut bibliography = Bibliography::new();
    bibliography.insert(
        "case1".to_string(),
        make_reference("case1", "legal-case", None, 1954, "Brown v. Board"),
    );
    bibliography.insert(
        "case2".to_string(),
        make_reference("case2", "legal-case", None, 1955, "Brown v. Board"),
    );
    let processor = Processor::new(legal_case_style(), bibliography);

    let citation = Citation {
        items: vec![
            CitationItem {
                id: "case1".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "case2".to_string(),
                ..Default::default()
            },
        ],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };

    let rendered = processor
        .process_citation(&citation)
        .expect("legal-case citation should render");
    assert!(
        rendered.contains("Brown v. Board"),
        "full legal-case title should be preserved in each item"
    );
    assert!(
        rendered.contains(';'),
        "legal-case items should remain separate within the citation"
    );
    assert!(
        !rendered.contains("1954, 1955"),
        "legal-case items should not collapse into grouped year compression"
    );
}

use std::str::FromStr;

#[allow(clippy::too_many_lines, reason = "integration test fixture setup")]
#[test]
fn test_type_specific_rendering() {
    let mut bibliography = Bibliography::new();
    bibliography.insert(
        "article1".to_string(),
        make_reference(
            "article1",
            "article-journal",
            Some(("Smith", "John")),
            2020,
            "Title A",
        ),
    );
    bibliography.insert(
        "book1".to_string(),
        make_reference("book1", "book", Some(("Doe", "Jane")), 2021, "Title B"),
    );

    let mut type_variants = IndexMap::new();
    // Article variant: Author (Short), Year
    type_variants.insert(
        TypeSelector::from_str("article-journal").unwrap(),
        vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                form: ContributorForm::Short,
                ..Default::default()
            }),
            TemplateComponent::Date(TemplateDate {
                date: citum_schema::template::DateVariable::Issued,
                form: DateForm::Year,
                rendering: Rendering {
                    prefix: Some(", ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
    );
    // Book variant: Author (Short), Title (Primary), Year
    type_variants.insert(
        TypeSelector::from_str("book").unwrap(),
        vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                form: ContributorForm::Short,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                rendering: Rendering {
                    emph: Some(true),
                    prefix: Some(", ".to_string()),
                    ..Default::default()
                },
                links: None,
                custom: None,
                ..Default::default()
            }),
            TemplateComponent::Date(TemplateDate {
                date: citum_schema::template::DateVariable::Issued,
                form: DateForm::Year,
                rendering: Rendering {
                    prefix: Some(", ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
    );

    let style = Style {
        info: StyleInfo {
            title: Some("Type Specific".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            type_variants: Some(type_variants),
            template: Some(vec![TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Locator,
                rendering: Rendering::default(),
                links: None,
                custom: None,
            })]),
            wrap: Some(WrapPunctuation::Parentheses),
            ..Default::default()
        }),
        ..Default::default()
    };

    let processor = Processor::new(style, bibliography);

    let cite_article = Citation {
        items: vec![CitationItem {
            id: "article1".to_string(),
            ..Default::default()
        }],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };
    let cite_book = Citation {
        items: vec![CitationItem {
            id: "book1".to_string(),
            ..Default::default()
        }],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };

    assert_eq!(
        processor.process_citation(&cite_article).unwrap(),
        "(Smith, 2020)"
    );
    assert_eq!(
        processor.process_citation(&cite_book).unwrap(),
        "(Doe, _Title B_, 2021)"
    );
}

#[allow(clippy::too_many_lines, reason = "integration test fixture setup")]
#[test]
fn test_bibliography_type_specific_rendering() {
    use crate::processor::rendering::RendererResources;
    use citum_schema::BibliographySpec;

    let mut bibliography = Bibliography::new();
    let interview_ref = Reference::from(LegacyReference {
        id: "ref1".to_string(),
        ref_type: "interview".to_string(),
        author: Some(vec![Name::new("Arendt", "Hannah")]),
        title: Some("Thinking in Public".to_string()),
        issued: Some(LegacyDateVariable::year(1975)),
        interviewer: Some(vec![Name::new("Young-Bruehl", "Elisabeth")]),
        publisher: Some("Schocken Books".to_string()),
        ..Default::default()
    });
    bibliography.insert("ref1".to_string(), interview_ref);

    let mut type_variants = IndexMap::new();
    type_variants.insert(
        TypeSelector::from_str("interview").unwrap(),
        vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                form: ContributorForm::Long,
                name_order: Some(NameOrder::FamilyFirst),
                ..Default::default()
            }),
            TemplateComponent::Date(TemplateDate {
                date: citum_schema::template::DateVariable::Issued,
                form: DateForm::Year,
                rendering: Rendering {
                    wrap: Some(WrapPunctuation::Parentheses),
                    prefix: Some(" ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                rendering: Rendering {
                    emph: Some(true),
                    prefix: Some(" ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Interviewer,
                form: ContributorForm::Long,
                name_order: Some(NameOrder::FamilyFirst),
                rendering: Rendering {
                    wrap: Some(WrapPunctuation::Parentheses),
                    prefix: Some(" ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                rendering: Rendering {
                    prefix: Some(". ".to_string()),
                    suffix: Some(".".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
    );

    let style = Style {
        info: StyleInfo {
            title: Some("Bib Type Specific".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            type_variants: Some(type_variants),
            template: Some(vec![TemplateComponent::Title(TemplateTitle::default())]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let locale = Locale::default(); // Note: default locale might not have "Interviewer" term set up in a way that auto-labels work if we don't specify them.
    // But we are specifying the interviewer component explicitly.
    let config = Config::default();
    let disambig = HashMap::new();
    let series_map = RefCell::new(HashMap::new());
    let set_by_ref = HashMap::new();
    let member_index = HashMap::new();
    let sets = IndexMap::new();

    let renderer = crate::processor::rendering::Renderer::new(
        RendererResources {
            style: &style,
            bibliography: &bibliography,
            locale: &locale,
            config: &config,
        },
        &disambig,
        &series_map,
        crate::processor::rendering::CompoundRenderData {
            set_by_ref: &set_by_ref,
            member_index: &member_index,
            sets: &sets,
        },
        false,
        false,
    );

    let reference = bibliography.get("ref1").unwrap();
    let proc_template = renderer
        .process_bibliography_entry_with_format::<crate::render::plain::PlainText>(reference, 1)
        .unwrap();
    let result = crate::render::bibliography::render_entry_body_with_format::<
        crate::render::plain::PlainText,
    >(&crate::render::component::ProcEntry {
        id: "ref1".to_string(),
        template: proc_template,
        metadata: crate::render::format::ProcEntryMetadata::default(),
    });

    assert!(
        result.contains("Arendt, Hannah"),
        "Author missing: {}",
        result
    );
    assert!(result.contains("(1975)"), "Date missing: {}", result);
    assert!(
        result.contains("_Thinking in Public_"),
        "Title missing: {}",
        result
    );
    assert!(
        result.contains("Young-Bruehl, Elisabeth"),
        "Interviewer missing: {}",
        result
    );
    assert!(
        result.contains("Schocken Books"),
        "Publisher missing: {}",
        result
    );
}
