use super::*;
use citum_schema::options::{
    AndOptions, ContributorConfig, DisplayAsSort, LabelConfig, LabelPreset, Processing,
    ShortenListOptions,
};
use citum_schema::template::{
    ContributorForm, ContributorRole, DateForm, DateVariable as TDateVar, NumberVariable,
    Rendering, TemplateComponent, TemplateContributor, TemplateDate, TemplateNumber, TemplateTitle,
    TitleType, WrapPunctuation,
};
use citum_schema::{BibliographySpec, CitationSpec, StyleInfo};
use csl_legacy::csl_json::{DateVariable, Name, Reference as LegacyReference};

fn make_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("APA".to_string()),
            id: Some("apa".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            substitute: Some(citum_schema::options::SubstituteConfig::default()),
            contributors: Some(ContributorConfig {
                shorten: Some(ShortenListOptions {
                    min: 3,
                    use_first: 1,
                    ..Default::default()
                }),
                and: Some(AndOptions::Symbol),
                display_as_sort: Some(DisplayAsSort::First),
                ..Default::default()
            }),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            options: None,
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    name_order: None,
                    delimiter: None,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: TDateVar::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
            ]),
            wrap: Some(WrapPunctuation::Parentheses),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            options: None,
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    name_order: None,
                    delimiter: None,
                    and: None,
                    rendering: Default::default(),
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: TDateVar::Issued,
                    form: DateForm::Year,
                    rendering: Rendering {
                        wrap: Some(WrapPunctuation::Parentheses),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    form: None,
                    rendering: Rendering {
                        prefix: Some(". ".to_string()),
                        emph: Some(true),
                        ..Default::default()
                    },
                    overrides: None,
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        }),
        templates: None,
        ..Default::default()
    }
}

fn make_note_style() -> Style {
    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Note),
        ..Default::default()
    });
    style
}

fn make_bibliography() -> Bibliography {
    let mut bib = Bibliography::new();
    bib.insert(
        "kuhn1962".to_string(),
        Reference::from(LegacyReference {
            id: "kuhn1962".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas S.")]),
            title: Some("The Structure of Scientific Revolutions".to_string()),
            issued: Some(DateVariable::year(1962)),
            ..Default::default()
        }),
    );

    bib
}

fn make_numeric_books(ids: &[(&str, &str, i32, &str)]) -> Bibliography {
    let mut bib = Bibliography::new();
    for (id, family, year, title) in ids {
        bib.insert(
            (*id).to_string(),
            Reference::from(LegacyReference {
                id: (*id).to_string(),
                ref_type: "book".to_string(),
                author: Some(vec![Name::new(family, "Test")]),
                title: Some((*title).to_string()),
                issued: Some(DateVariable::year(*year)),
                ..Default::default()
            }),
        );
    }
    bib
}

/// Tests basic citation processing with author-date format.
///
/// Verifies that a simple citation with one item produces correctly formatted
/// output with author name and year wrapped in parentheses.
#[test]
fn test_process_citation() {
    let style = make_style();
    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    let citation = Citation {
        id: Some("c1".to_string()),
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = processor.process_citation(&citation).unwrap();
    assert_eq!(result, "(Kuhn, 1962)");
}

/// Tests that note citations receive proper sequential numbering.
///
/// Verifies that citations with missing note numbers are auto-assigned,
/// and that the numbering sequence is correct when some numbers are already provided.
#[test]
fn test_normalize_note_context_assigns_missing_numbers() {
    let style = make_note_style();
    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    let citations = vec![
        Citation {
            id: Some("c1".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            id: Some("c2".to_string()),
            note_number: Some(7),
            items: vec![crate::reference::CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            id: Some("c3".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    let normalized = processor.normalize_note_context(&citations);
    assert_eq!(normalized[0].note_number, Some(1));
    assert_eq!(normalized[1].note_number, Some(7));
    assert_eq!(normalized[2].note_number, Some(8));
}

/// Tests batch processing of multiple citations with the public API.
///
/// Verifies that multiple citations can be processed together and each produces
/// the expected formatted output independently.
#[test]
fn test_process_citations_batch_api() {
    let style = make_style();
    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    let citations = vec![
        Citation {
            id: Some("c1".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            id: Some("c2".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    let rendered = processor.process_citations(&citations).unwrap();
    assert_eq!(rendered.len(), 2);
    assert_eq!(rendered[0], "(Kuhn, 1962)");
    assert_eq!(rendered[1], "(Kuhn, 1962)");
}

/// Tests that a delimiter of "none" (with surrounding spaces) renders as no delimiter.
///
/// Verifies that when a delimiter is set to " none " (trimmed to "none"),
/// components are concatenated without any separator.
#[test]
fn test_process_citation_treats_trimmed_none_delimiter_as_empty() {
    let mut style = make_style();
    style.citation = Some(CitationSpec {
        template: Some(vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                form: ContributorForm::Short,
                ..Default::default()
            }),
            TemplateComponent::Date(TemplateDate {
                date: TDateVar::Issued,
                form: DateForm::Year,
                ..Default::default()
            }),
        ]),
        wrap: Some(WrapPunctuation::Parentheses),
        delimiter: Some(" none ".to_string()),
        ..Default::default()
    });

    let bib = make_bibliography();
    let processor = Processor::new(style, bib);
    let citation = Citation {
        id: Some("c1".to_string()),
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = processor.process_citation(&citation).unwrap();
    assert_eq!(result, "(Kuhn1962)");
}

/// Tests that locator labels are properly rendered using localized terms.
///
/// Verifies that a page locator renders with the appropriate term "p." and
/// that the full citation includes author, year, and locator information.
#[test]
fn test_citation_locator_label_renders_term() {
    let mut style = make_style();
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![
            citum_schema::TemplateComponent::Contributor(
                citum_schema::template::TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    ..Default::default()
                },
            ),
            citum_schema::TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: TDateVar::Issued,
                form: DateForm::Year,
                ..Default::default()
            }),
            citum_schema::TemplateComponent::Variable(citum_schema::template::TemplateVariable {
                variable: citum_schema::template::SimpleVariable::Locator,
                ..Default::default()
            }),
        ]),
        wrap: Some(WrapPunctuation::Parentheses),
        delimiter: Some(", ".to_string()),
        ..Default::default()
    });

    let bib = make_bibliography();
    let processor = Processor::new(style, bib);
    let citation = Citation {
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            label: Some(citum_schema::citation::LocatorType::Page),
            locator: Some("23".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let rendered = processor.process_citation(&citation).unwrap();
    assert_eq!(rendered, "(Kuhn, 1962, p. 23)");
}

/// Tests locator label rendering with explicitly loaded locale data.
///
/// Verifies that locator terms are correctly rendered when a locale is explicitly
/// loaded from the locale directory, not using defaults.
#[test]
fn test_citation_locator_label_renders_term_with_loaded_locale() {
    use std::path::Path;

    let mut style = make_style();
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![
            citum_schema::TemplateComponent::Contributor(
                citum_schema::template::TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    ..Default::default()
                },
            ),
            citum_schema::TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: TDateVar::Issued,
                form: DateForm::Year,
                ..Default::default()
            }),
            citum_schema::TemplateComponent::Variable(citum_schema::template::TemplateVariable {
                variable: citum_schema::template::SimpleVariable::Locator,
                ..Default::default()
            }),
        ]),
        wrap: Some(WrapPunctuation::Parentheses),
        delimiter: Some(", ".to_string()),
        ..Default::default()
    });

    let bib = make_bibliography();
    let locale = citum_schema::locale::Locale::load("en-US", Path::new("locales"));
    let processor = Processor::with_locale(style, bib, locale);
    let citation = Citation {
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            label: Some(citum_schema::citation::LocatorType::Page),
            locator: Some("23".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let rendered = processor.process_citation(&citation).unwrap();
    assert_eq!(rendered, "(Kuhn, 1962, p. 23)");
}

/// Tests the behavior of test_citation_locator_can_suppress_label.
#[test]
fn test_citation_locator_can_suppress_label() {
    let mut style = make_style();
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![
            citum_schema::TemplateComponent::Contributor(
                citum_schema::template::TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    ..Default::default()
                },
            ),
            citum_schema::TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: TDateVar::Issued,
                form: DateForm::Year,
                ..Default::default()
            }),
            citum_schema::TemplateComponent::Variable(citum_schema::template::TemplateVariable {
                variable: citum_schema::template::SimpleVariable::Locator,
                show_label: Some(false),
                ..Default::default()
            }),
        ]),
        wrap: Some(WrapPunctuation::Parentheses),
        delimiter: Some(", ".to_string()),
        ..Default::default()
    });

    let bib = make_bibliography();
    let processor = Processor::new(style, bib);
    let citation = Citation {
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            label: Some(citum_schema::citation::LocatorType::Page),
            locator: Some("23".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let rendered = processor.process_citation(&citation).unwrap();
    assert_eq!(rendered, "(Kuhn, 1962, 23)");
}

/// Tests the behavior of test_citation_locator_can_strip_label_periods.
#[test]
fn test_citation_locator_can_strip_label_periods() {
    let mut style = make_style();
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![
            citum_schema::TemplateComponent::Contributor(
                citum_schema::template::TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    ..Default::default()
                },
            ),
            citum_schema::TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: TDateVar::Issued,
                form: DateForm::Year,
                ..Default::default()
            }),
            citum_schema::TemplateComponent::Variable(citum_schema::template::TemplateVariable {
                variable: citum_schema::template::SimpleVariable::Locator,
                strip_label_periods: Some(true),
                ..Default::default()
            }),
        ]),
        wrap: Some(WrapPunctuation::Parentheses),
        delimiter: Some(", ".to_string()),
        ..Default::default()
    });

    let bib = make_bibliography();
    let processor = Processor::new(style, bib);
    let citation = Citation {
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            label: Some(citum_schema::citation::LocatorType::Page),
            locator: Some("23".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let rendered = processor.process_citation(&citation).unwrap();
    assert_eq!(rendered, "(Kuhn, 1962, p23)");
}

/// Tests the behavior of test_springer_locator_label_survives_sorting.
#[test]
fn test_springer_locator_label_survives_sorting() {
    use std::{fs, path::Path};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let style_path = root.join("styles/springer-basic-author-date.yaml");
    let bib_path = root.join("tests/fixtures/references-expanded.json");
    let cite_path = root.join("tests/fixtures/citations-expanded.json");

    let style_yaml = fs::read_to_string(&style_path).expect("style should read");
    let style: Style = serde_yaml::from_str(&style_yaml).expect("style should parse");
    let bibliography = crate::io::load_bibliography(&bib_path).expect("bib should load");
    let citations = crate::io::load_citations(&cite_path).expect("citations should load");

    let processor = Processor::new(style.clone(), bibliography);
    let citation = citations
        .iter()
        .find(|c| c.id.as_deref() == Some("with-locator"))
        .cloned()
        .expect("with-locator citation should exist");

    assert_eq!(
        citation.items[0].label,
        Some(citum_schema::citation::LocatorType::Page)
    );

    let spec = style.citation.as_ref().expect("citation spec should exist");
    let sorted = processor.sort_citation_items(citation.items.clone(), spec);
    assert_eq!(
        sorted[0].label,
        Some(citum_schema::citation::LocatorType::Page)
    );

    let rendered_default_locale = processor.process_citation(&citation).unwrap();
    assert!(
        rendered_default_locale.contains("p. 23"),
        "default locale render should include page label: {rendered_default_locale}"
    );

    let locales_dir = root.join("locales");
    let loaded_locale = citum_schema::locale::Locale::load("en-US", &locales_dir);
    let with_loaded = Processor::with_locale(
        style,
        crate::io::load_bibliography(&bib_path).unwrap(),
        loaded_locale,
    );
    let rendered_loaded_locale = with_loaded.process_citation(&citation).unwrap();
    assert!(
        rendered_loaded_locale.contains("p. 23"),
        "loaded locale render should include page label: {rendered_loaded_locale}"
    );
}

/// Tests parsed-style grouped author-date citations using the default locale path.
#[test]
fn test_harvard_cite_them_right_grouped_citations_render_cleanly() {
    use std::{fs, path::Path};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let style_path = root.join("styles/harvard-cite-them-right.yaml");
    let bib_path = root.join("tests/fixtures/references-expanded.json");
    let cite_path = root.join("tests/fixtures/citations-expanded.json");

    let style_yaml = fs::read_to_string(&style_path).expect("style should read");
    let style: Style = serde_yaml::from_str(&style_yaml).expect("style should parse");
    let bibliography = crate::io::load_bibliography(&bib_path).expect("bib should load");
    let citations = crate::io::load_citations(&cite_path).expect("citations should load");

    let processor = Processor::new(style, bibliography);

    let single_item = citations
        .iter()
        .find(|c| c.id.as_deref() == Some("single-item"))
        .cloned()
        .expect("single-item citation should exist");
    assert_eq!(
        processor.process_citation(&single_item).unwrap(),
        "(Kuhn, 1962)"
    );

    let with_locator = citations
        .iter()
        .find(|c| c.id.as_deref() == Some("with-locator"))
        .cloned()
        .expect("with-locator citation should exist");
    assert_eq!(
        processor.process_citation(&with_locator).unwrap(),
        "(Kuhn, 1962, p. 23)"
    );

    let no_date = citations
        .iter()
        .find(|c| c.id.as_deref() == Some("no-date-single"))
        .cloned()
        .expect("no-date-single citation should exist");
    assert_eq!(
        processor.process_citation(&no_date).unwrap(),
        "(Forthcoming, no date)"
    );
}

/// Tests style-specific no-date overrides without regressing n.d.-based styles.
#[test]
fn test_parsed_style_no_date_terms_match_expected_variants() {
    use std::{fs, path::Path};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bib_path = root.join("tests/fixtures/references-expanded.json");
    let cite_path = root.join("tests/fixtures/citations-expanded.json");
    let bibliography = crate::io::load_bibliography(&bib_path).expect("bib should load");
    let citations = crate::io::load_citations(&cite_path).expect("citations should load");
    let no_date = citations
        .iter()
        .find(|c| c.id.as_deref() == Some("no-date-single"))
        .cloned()
        .expect("no-date-single citation should exist");

    let load_style = |name: &str| -> Style {
        let style_path = root.join("styles").join(format!("{name}.yaml"));
        let style_yaml = fs::read_to_string(&style_path).expect("style should read");
        serde_yaml::from_str(&style_yaml).expect("style should parse")
    };

    let harvard = Processor::new(load_style("harvard-cite-them-right"), bibliography.clone());
    assert_eq!(
        harvard.process_citation(&no_date).unwrap(),
        "(Forthcoming, no date)"
    );

    let sage = Processor::new(load_style("sage-harvard"), bibliography);
    let sage_rendered = sage.process_citation(&no_date).unwrap();
    assert!(
        sage_rendered.contains("n.d."),
        "sage-harvard should keep the short no-date term: {sage_rendered}"
    );
    assert!(
        !sage_rendered.contains("no date"),
        "sage-harvard should not switch to the long no-date term: {sage_rendered}"
    );
}

/// Tests the behavior of test_render_bibliography.
#[test]
fn test_render_bibliography() {
    let style = make_style();
    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    let result = processor.render_bibliography();

    // Check it contains the key parts
    assert!(result.contains("Kuhn"));
    assert!(result.contains("(1962)"));
    assert!(result.contains("_The Structure of Scientific Revolutions_"));
}

/// Tests the behavior of test_disambiguation_hints.
#[test]
fn test_disambiguation_hints() {
    let style = make_style();
    let mut bib = make_bibliography();

    // Add another Kuhn 1962 reference to trigger disambiguation
    bib.insert(
        "kuhn1962b".to_string(),
        Reference::from(LegacyReference {
            id: "kuhn1962b".to_string(),
            ref_type: "article-journal".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas S.")]),
            title: Some("The Function of Measurement in Modern Physical Science".to_string()),
            issued: Some(DateVariable::year(1962)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let hints = &processor.hints;

    // Both should have disambiguation condition true
    assert!(hints.get("kuhn1962").unwrap().disamb_condition);
    assert!(hints.get("kuhn1962b").unwrap().disamb_condition);
}

/// Tests the behavior of test_disambiguation_givenname.
#[test]
fn test_disambiguation_givenname() {
    use citum_schema::options::{
        Disambiguation, Group, Processing, ProcessingCustom, Sort, SortKey, SortSpec,
    };

    // Style with add-givenname enabled
    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Custom(ProcessingCustom {
            sort: Some(citum_schema::options::SortEntry::Explicit(Sort {
                shorten_names: false,
                render_substitutions: false,
                template: vec![
                    SortSpec {
                        key: SortKey::Author,
                        ascending: true,
                    },
                    SortSpec {
                        key: SortKey::Year,
                        ascending: true,
                    },
                ],
            })),
            group: Some(Group {
                template: vec![SortKey::Author, SortKey::Year],
            }),
            disambiguate: Some(Disambiguation {
                names: true,
                add_givenname: true,
                year_suffix: true,
            }),
        })),
        contributors: Some(ContributorConfig {
            initialize_with: Some(". ".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "smith2020a".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020a".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "John")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );
    bib.insert(
        "smith2020b".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020b".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "Alice")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    let hints = &processor.hints;

    // Verify hints
    assert!(hints.get("smith2020a").unwrap().expand_given_names);
    assert!(hints.get("smith2020b").unwrap().expand_given_names);
    assert!(!hints.get("smith2020a").unwrap().disamb_condition); // No year suffix

    // Verify output
    let cit_a = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "smith2020a".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    let cit_b = processor
        .process_citation(&Citation {
            id: Some("c2".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "smith2020b".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    // Should expand to "J. Smith" and "A. Smith" (because initialized)
    assert!(cit_a.contains("J. Smith"));
    assert!(cit_b.contains("A. Smith"));
}

/// Tests the behavior of test_disambiguation_add_names.
#[test]
fn test_disambiguation_add_names() {
    use citum_schema::options::{
        Disambiguation, Group, Processing, ProcessingCustom, Sort, SortKey, SortSpec,
    };

    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Custom(ProcessingCustom {
            sort: Some(citum_schema::options::SortEntry::Explicit(Sort {
                shorten_names: false,
                render_substitutions: false,
                template: vec![
                    SortSpec {
                        key: SortKey::Author,
                        ascending: true,
                    },
                    SortSpec {
                        key: SortKey::Year,
                        ascending: true,
                    },
                ],
            })),
            group: Some(Group {
                template: vec![SortKey::Author, SortKey::Year],
            }),
            disambiguate: Some(Disambiguation {
                names: true, // disambiguate-add-names
                add_givenname: false,
                year_suffix: true,
            }),
        })),
        contributors: Some(ContributorConfig {
            shorten: Some(ShortenListOptions {
                min: 2,
                use_first: 1,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    });

    let mut bib = indexmap::IndexMap::new();
    // Two works by Smith & Jones and Smith & Brown
    // Both would be "Smith et al. (2020)"
    bib.insert(
        "ref1".to_string(),
        Reference::from(LegacyReference {
            id: "ref1".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![
                Name::new("Smith", "John"),
                Name::new("Jones", "Peter"),
            ]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );
    bib.insert(
        "ref2".to_string(),
        Reference::from(LegacyReference {
            id: "ref2".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![
                Name::new("Smith", "John"),
                Name::new("Brown", "Alice"),
            ]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    // Verify hints
    assert_eq!(
        processor.hints.get("ref1").unwrap().min_names_to_show,
        Some(2)
    );
    assert_eq!(
        processor.hints.get("ref2").unwrap().min_names_to_show,
        Some(2)
    );

    // Verify output
    let cit_1 = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref1".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    let cit_2 = processor
        .process_citation(&Citation {
            id: Some("c2".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref2".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    // Should expand to "Smith, Jones" and "Smith, Brown" (no et al. because only 2 names)
    assert!(cit_1.contains("Smith") && cit_1.contains("Jones"));
    assert!(cit_2.contains("Smith") && cit_2.contains("Brown"));
}

/// Tests the behavior of test_disambiguation_combined_expansion.
#[test]
fn test_disambiguation_combined_expansion() {
    use citum_schema::options::{
        Disambiguation, Group, Processing, ProcessingCustom, Sort, SortKey, SortSpec,
    };

    // This test simulates the "Sam Smith & Julie Smith" scenario but with
    // two items that remain ambiguous after name expansion alone.
    // Item 1: [Sam Smith, Julie Smith] 2020 -> "Smith & Smith" (base)
    // Item 2: [Sam Smith, Bob Smith] 2020   -> "Smith & Smith" (base)
    // Both would be "Smith et al." if min=3, but here they collide even as "Smith & Smith".
    // They need both expanded names AND expanded given names.

    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Custom(ProcessingCustom {
            sort: Some(citum_schema::options::SortEntry::Explicit(Sort {
                shorten_names: false,
                render_substitutions: false,
                template: vec![
                    SortSpec {
                        key: SortKey::Author,
                        ascending: true,
                    },
                    SortSpec {
                        key: SortKey::Year,
                        ascending: true,
                    },
                ],
            })),
            group: Some(Group {
                template: vec![SortKey::Author, SortKey::Year],
            }),
            disambiguate: Some(Disambiguation {
                names: true,
                add_givenname: true,
                year_suffix: true,
            }),
        })),
        contributors: Some(ContributorConfig {
            shorten: Some(ShortenListOptions {
                min: 2,
                use_first: 1,
                ..Default::default()
            }),
            initialize_with: Some(". ".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "ref1".to_string(),
        Reference::from(LegacyReference {
            id: "ref1".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "Sam"), Name::new("Smith", "Julie")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );
    bib.insert(
        "ref2".to_string(),
        Reference::from(LegacyReference {
            id: "ref2".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "Sam"), Name::new("Smith", "Bob")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    // Verify output
    let cit_1 = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref1".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    let cit_2 = processor
        .process_citation(&Citation {
            id: Some("c2".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref2".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    // Should expand to "S. Smith & J. Smith" and "S. Smith & B. Smith"
    assert!(
        cit_1.contains("S. Smith") && cit_1.contains("J. Smith"),
        "Output was: {}",
        cit_1
    );
    assert!(
        cit_2.contains("S. Smith") && cit_2.contains("B. Smith"),
        "Output was: {}",
        cit_2
    );
}

/// Tests the behavior of test_apa_titles_config.
#[test]
fn test_apa_titles_config() {
    use crate::reference::Reference;
    use citum_schema::options::{Config, TitleRendering, TitlesConfig};
    use citum_schema::template::{Rendering, TemplateTitle, TitleType};

    let config = Config {
        titles: Some(TitlesConfig {
            periodical: Some(TitleRendering {
                emph: Some(true),
                ..Default::default()
            }),
            monograph: Some(TitleRendering {
                emph: Some(true),
                ..Default::default()
            }),
            container_monograph: Some(TitleRendering {
                emph: Some(true),
                prefix: Some("In ".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let bib_template = vec![
        TemplateComponent::Title(TemplateTitle {
            title: TitleType::Primary,
            rendering: Rendering::default(),
            ..Default::default()
        }),
        TemplateComponent::Title(TemplateTitle {
            title: TitleType::ParentSerial,
            rendering: Rendering::default(),
            ..Default::default()
        }),
        TemplateComponent::Title(TemplateTitle {
            title: TitleType::ParentMonograph,
            rendering: Rendering::default(),
            ..Default::default()
        }),
    ];

    let style = Style {
        options: Some(config),
        bibliography: Some(citum_schema::BibliographySpec {
            template: Some(bib_template),
            ..Default::default()
        }),
        ..Default::default()
    };

    let references = vec![
        Reference::from(LegacyReference {
            id: "art1".to_string(),
            ref_type: "article-journal".to_string(),
            title: Some("A Title".to_string()),
            container_title: Some("Nature".to_string()),
            ..Default::default()
        }),
        Reference::from(LegacyReference {
            id: "ch1".to_string(),
            ref_type: "chapter".to_string(),
            title: Some("A Chapter".to_string()),
            container_title: Some("A Book".to_string()),
            ..Default::default()
        }),
        Reference::from(LegacyReference {
            id: "bk1".to_string(),
            ref_type: "book".to_string(),
            title: Some("A Global Book".to_string()),
            ..Default::default()
        }),
    ];

    let processor = Processor::new(
        style,
        references
            .into_iter()
            .map(|r| (r.id().unwrap().to_string(), r))
            .collect(),
    );

    let res = processor.render_bibliography();

    // Book Case: Primary title -> monograph category -> Italic, No "In "
    assert!(
        res.contains("_A Global Book_"),
        "Book title should be italicized: {}",
        res
    );
    assert!(
        !res.contains("In _A Global Book_"),
        "Book title should NOT have 'In ' prefix: {}",
        res
    );

    // Journal Article Case: ParentSerial -> periodical category -> Italic, No "In "
    assert!(
        res.contains("_Nature_"),
        "Journal title should be italicized: {}",
        res
    );
    assert!(
        !res.contains("In _Nature_"),
        "Journal title should NOT have 'In ' prefix: {}",
        res
    );

    // Chapter Case: ParentMonograph -> container_monograph category -> Italic, WITH "In "
    assert!(
        res.contains("In _A Book_"),
        "Chapter container title should have 'In ' prefix: {}",
        res
    );
}

/// Tests the behavior of test_numeric_citation_numbers_with_repeated_refs.
#[test]
fn test_numeric_citation_numbers_with_repeated_refs() {
    // Citation numbers should remain stable once assigned.
    // Citing ref1, ref2, ref1 again should give numbers 1, 2, 1.
    use citum_schema::CitationSpec;
    use citum_schema::options::{Config, Processing};
    use citum_schema::template::{NumberVariable, TemplateNumber};

    let style = Style {
        citation: Some(CitationSpec {
            wrap: Some(citum_schema::template::WrapPunctuation::Brackets),
            template: Some(vec![TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::CitationNumber,
                ..Default::default()
            })]),
            ..Default::default()
        }),
        options: Some(Config {
            processing: Some(Processing::Numeric),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut bib = Bibliography::new();
    bib.insert(
        "ref1".to_string(),
        Reference::from(LegacyReference {
            id: "ref1".to_string(),
            ref_type: "book".to_string(),
            title: Some("First Book".to_string()),
            ..Default::default()
        }),
    );
    bib.insert(
        "ref2".to_string(),
        Reference::from(LegacyReference {
            id: "ref2".to_string(),
            ref_type: "book".to_string(),
            title: Some("Second Book".to_string()),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    // Cite ref1 first
    let cit1 = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref1".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    // Cite ref2 second
    let cit2 = processor
        .process_citation(&Citation {
            id: Some("c2".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref2".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    // Cite ref1 again - should get the SAME number as before
    let cit3 = processor
        .process_citation(&Citation {
            id: Some("c3".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref1".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    assert_eq!(cit1, "[1]", "First citation of ref1 should be [1]");
    assert_eq!(cit2, "[2]", "First citation of ref2 should be [2]");
    assert_eq!(cit3, "[1]", "Second citation of ref1 should still be [1]");
}

/// Tests the behavior of test_numeric_citation_numbers_follow_registry_order.
#[test]
fn test_numeric_citation_numbers_follow_registry_order() {
    use citum_schema::CitationSpec;
    use citum_schema::options::{Config, Processing};
    use citum_schema::template::{NumberVariable, TemplateNumber};

    let style = Style {
        citation: Some(CitationSpec {
            wrap: Some(citum_schema::template::WrapPunctuation::Brackets),
            template: Some(vec![TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::CitationNumber,
                ..Default::default()
            })]),
            ..Default::default()
        }),
        options: Some(Config {
            processing: Some(Processing::Numeric),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut bib = Bibliography::new();
    bib.insert(
        "ref1".to_string(),
        Reference::from(LegacyReference {
            id: "ref1".to_string(),
            ref_type: "book".to_string(),
            title: Some("First Book".to_string()),
            ..Default::default()
        }),
    );
    bib.insert(
        "ref2".to_string(),
        Reference::from(LegacyReference {
            id: "ref2".to_string(),
            ref_type: "book".to_string(),
            title: Some("Second Book".to_string()),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    let cit = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![crate::reference::CitationItem {
                id: "ref2".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap();

    assert_eq!(
        cit, "[2]",
        "Numeric citation number should follow bibliography registry order"
    );
}

/// Tests the behavior of test_citation_grouping_same_author.
#[test]
fn test_citation_grouping_same_author() {
    // Test that adjacent citations by the same author are collapsed:
    // (Kuhn 1962a, 1962b) instead of (Kuhn 1962a; Kuhn 1962b)
    let style = make_style();
    let mut bib = make_bibliography();

    // Add second Kuhn 1962 with different title (triggers year-suffix)
    bib.insert(
        "kuhn1962b".to_string(),
        Reference::from(LegacyReference {
            id: "kuhn1962b".to_string(),
            ref_type: "article-journal".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas S.")]),
            title: Some("The Function of Measurement in Modern Physical Science".to_string()),
            issued: Some(DateVariable::year(1962)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    // Cite both Kuhn works in one citation - should group
    let result = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![
                crate::reference::CitationItem {
                    id: "kuhn1962b".to_string(), // "Function..." comes first alphabetically -> a
                    ..Default::default()
                },
                crate::reference::CitationItem {
                    id: "kuhn1962".to_string(), // "Structure..." comes second -> b
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .unwrap();

    // Should be grouped: "Kuhn, 1962a, 1962b" not "Kuhn, 1962a; Kuhn, 1962b"
    // Year suffix assigned by title order: "Function..." < "Structure..."
    assert!(
        result.contains("Kuhn, 1962a, 1962b") || result.contains("Kuhn, 1962b, 1962a"),
        "Same-author citations should be grouped. Got: {}",
        result
    );
    assert!(
        !result.contains("; Kuhn"),
        "Should not have semicolon between same-author citations. Got: {}",
        result
    );
}

/// Tests the behavior of test_label_mode_does_not_group_by_author.
#[test]
fn test_label_mode_does_not_group_by_author() {
    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Label(LabelConfig {
            preset: LabelPreset::Din,
            ..Default::default()
        })),
        ..Default::default()
    });
    style.citation = Some(CitationSpec {
        template: Some(vec![TemplateComponent::Number(TemplateNumber {
            number: NumberVariable::CitationLabel,
            ..Default::default()
        })]),
        wrap: Some(WrapPunctuation::Brackets),
        ..Default::default()
    });

    let mut bib = make_bibliography();
    bib.insert(
        "kuhn1962b".to_string(),
        Reference::from(LegacyReference {
            id: "kuhn1962b".to_string(),
            ref_type: "article-journal".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas S.")]),
            title: Some("The Function of Measurement in Modern Physical Science".to_string()),
            issued: Some(DateVariable::year(1962)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let result = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![
                crate::reference::CitationItem {
                    id: "kuhn1962b".to_string(),
                    ..Default::default()
                },
                crate::reference::CitationItem {
                    id: "kuhn1962".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .unwrap();

    assert!(
        !result.contains(", Kuhn"),
        "Label mode should not include grouped author text. Got: {}",
        result
    );
    assert!(
        result.contains(";"),
        "Label mode should render separate labels for multi-item citations. Got: {}",
        result
    );
}

/// Tests the behavior of test_citation_grouping_different_authors.
#[test]
fn test_citation_grouping_different_authors() {
    // Different authors should NOT be grouped
    let style = make_style();
    let mut bib = make_bibliography();

    bib.insert(
        "smith2020".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "John")]),
            title: Some("Another Book".to_string()),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    let result = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![
                crate::reference::CitationItem {
                    id: "kuhn1962".to_string(),
                    ..Default::default()
                },
                crate::reference::CitationItem {
                    id: "smith2020".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .unwrap();

    // Should have semicolon between different authors
    assert!(
        result.contains("Kuhn") && result.contains("Smith"),
        "Should contain both authors. Got: {}",
        result
    );
    assert!(
        result.contains("; "),
        "Different authors should be separated by semicolon. Got: {}",
        result
    );
}

/// Tests the behavior of test_sort_anonymous_work_by_title.
#[test]
fn test_sort_anonymous_work_by_title() {
    // Anonymous works (no author) should sort by title, with leading articles stripped
    let style = make_style();
    let mut bib = indexmap::IndexMap::new();

    // Add references in wrong alphabetical order to test sorting
    bib.insert(
        "smith".to_string(),
        Reference::from(LegacyReference {
            id: "smith".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "John")]),
            title: Some("A Book".to_string()),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    // Anonymous work - should sort by "Role" (stripping "The")
    bib.insert(
        "anon".to_string(),
        Reference::from(LegacyReference {
            id: "anon".to_string(),
            ref_type: "article-journal".to_string(),
            author: None, // No author!
            title: Some("The Role of Theory".to_string()),
            issued: Some(DateVariable::year(2018)),
            ..Default::default()
        }),
    );

    bib.insert(
        "jones".to_string(),
        Reference::from(LegacyReference {
            id: "jones".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Jones", "Alice")]),
            title: Some("Another Book".to_string()),
            issued: Some(DateVariable::year(2019)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // Order should be: Jones (J), anon/Role (R), Smith (S)
    let jones_pos = result.find("Jones").expect("Jones not found");
    let role_pos = result.find("Role of Theory").expect("Role not found");
    let smith_pos = result.find("Smith").expect("Smith not found");

    assert!(
        jones_pos < role_pos,
        "Jones should come before Role. Got:
{}",
        result
    );
    assert!(
        role_pos < smith_pos,
        "Role should come before Smith. Got:
{}",
        result
    );
}

/// Tests the behavior of test_whole_entry_linking_html.
#[test]
fn test_whole_entry_linking_html() {
    use crate::render::html::Html;
    use citum_schema::options::{LinkAnchor, LinkTarget, LinksConfig};

    let mut style = make_style();
    style.options.as_mut().unwrap().links = Some(LinksConfig {
        target: Some(LinkTarget::Url),
        anchor: Some(LinkAnchor::Entry),
        ..Default::default()
    });

    let mut bib = Bibliography::new();
    bib.insert(
        "link1".to_string(),
        Reference::from(LegacyReference {
            id: "link1".to_string(),
            ref_type: "webpage".to_string(),
            title: Some("Linked Page".to_string()),
            url: Some("https://example.com".to_string()),
            issued: Some(DateVariable::year(2023)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography_with_format::<Html>();

    // The whole entry content should be wrapped in an <a> tag inside the entry div
    assert!(result.contains(r#"id="ref-link1""#));
    assert!(result.contains(r#"<a href="https://example.com/">"#));
    assert!(result.contains("Linked Page"));
}

/// Tests the behavior of test_global_title_linking_html.
#[test]
fn test_global_title_linking_html() {
    use crate::render::html::Html;
    use citum_schema::options::{LinkAnchor, LinkTarget, LinksConfig};

    let mut style = make_style();
    style.options.as_mut().unwrap().links = Some(LinksConfig {
        target: Some(LinkTarget::Doi),
        anchor: Some(LinkAnchor::Title),
        ..Default::default()
    });

    let mut bib = Bibliography::new();
    bib.insert(
        "doi1".to_string(),
        Reference::from(LegacyReference {
            id: "doi1".to_string(),
            ref_type: "book".to_string(),
            title: Some("Linked Title".to_string()),
            doi: Some("10.1001/test".to_string()),
            issued: Some(DateVariable::year(2023)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography_with_format::<Html>();

    println!("Result: {}", result);

    // The title should be automatically hyperlinked because of global config.
    // Note: In this test, title substitutes for author, so it gets csln-author class.
    assert!(
        result.contains(r#"<span class="csln-author"><a href="https://doi.org/10.1001/test">"#)
    );
    assert!(result.contains("Linked Title"));
}

/// Tests the behavior of test_whole_entry_linking_typst.
#[test]
fn test_whole_entry_linking_typst() {
    use crate::render::typst::Typst;
    use citum_schema::options::{LinkAnchor, LinkTarget, LinksConfig};

    let mut style = make_style();
    style.options.as_mut().unwrap().links = Some(LinksConfig {
        target: Some(LinkTarget::Url),
        anchor: Some(LinkAnchor::Entry),
        ..Default::default()
    });

    let mut bib = Bibliography::new();
    bib.insert(
        "link1".to_string(),
        Reference::from(LegacyReference {
            id: "link1".to_string(),
            ref_type: "webpage".to_string(),
            title: Some("Linked Page".to_string()),
            url: Some("https://example.com".to_string()),
            issued: Some(DateVariable::year(2023)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography_with_format::<Typst>();

    assert!(result.contains(r#"#link("https://example.com/")["#));
    assert!(result.contains("<ref-link1>"));
    assert!(result.contains("Linked Page"));
}

/// Tests the behavior of test_typst_single_item_citation_links_to_bibliography_entry.
#[test]
fn test_typst_single_item_citation_links_to_bibliography_entry() {
    use crate::render::typst::Typst;

    let bib = make_bibliography();
    let processor = Processor::new(make_style(), bib);
    let citation = Citation {
        id: Some("cite-1".to_string()),
        items: vec![CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = processor
        .process_citation_with_format::<Typst>(&citation)
        .unwrap();
    assert!(result.contains("#link(<ref-kuhn1962>)"));
}

/// Tests the behavior of test_numeric_integral_citation_author_year.
#[test]
fn test_numeric_integral_citation_author_year() {
    use citum_schema::options::Processing;

    let mut style = make_style();
    // Override to numeric style
    style.options = Some(Config {
        processing: Some(Processing::Numeric),
        ..Default::default()
    });

    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    // Integral mode citation - should render author + citation number
    let citation = Citation {
        id: Some("c1".to_string()),
        mode: citum_schema::citation::CitationMode::Integral,
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = processor.process_citation(&citation).unwrap();
    // For numeric+integral, expect author + citation number (no outer parens)
    assert_eq!(result, "Kuhn [1]");
}

/// Tests the behavior of test_numeric_non_integral_citation_number.
#[test]
fn test_numeric_non_integral_citation_number() {
    use citum_schema::citation::CitationMode;
    use citum_schema::options::Processing;

    let mut style = make_style();
    // Override to numeric style with citation number template
    style.options = Some(Config {
        processing: Some(Processing::Numeric),
        ..Default::default()
    });
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![TemplateComponent::Number(
            citum_schema::template::TemplateNumber {
                number: citum_schema::template::NumberVariable::CitationNumber,
                form: None,
                rendering: Rendering::default(),
                ..Default::default()
            },
        )]),
        wrap: Some(WrapPunctuation::Brackets),
        ..Default::default()
    });

    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    // Non-integral mode citation - should render citation number
    let citation = Citation {
        id: Some("c1".to_string()),
        mode: CitationMode::NonIntegral,
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = processor.process_citation(&citation).unwrap();
    // For numeric+non-integral, expect number format: "[1]"
    assert_eq!(result, "[1]");
}

/// Verifies adjacent numeric citations collapse into ranges when requested by style.
#[test]
fn test_numeric_citation_number_collapse_enabled() {
    use citum_schema::citation::CitationMode;
    use citum_schema::options::Processing;

    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Numeric),
        ..Default::default()
    });
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![TemplateComponent::Number(
            citum_schema::template::TemplateNumber {
                number: citum_schema::template::NumberVariable::CitationNumber,
                ..Default::default()
            },
        )]),
        wrap: Some(WrapPunctuation::Brackets),
        multi_cite_delimiter: Some(",".to_string()),
        collapse: Some(citum_schema::CitationCollapse::CitationNumber),
        ..Default::default()
    });

    let bib = make_numeric_books(&[
        ("book-1", "Author A", 2001, "Book One"),
        ("book-2", "Author B", 2002, "Book Two"),
        ("book-3", "Author C", 2003, "Book Three"),
        ("book-4", "Author D", 2005, "Book Four"),
    ]);
    let processor = Processor::new(style, bib);

    let citation = Citation {
        id: Some("c1".to_string()),
        mode: CitationMode::NonIntegral,
        items: vec![
            crate::reference::CitationItem {
                id: "book-1".to_string(),
                ..Default::default()
            },
            crate::reference::CitationItem {
                id: "book-2".to_string(),
                ..Default::default()
            },
            crate::reference::CitationItem {
                id: "book-3".to_string(),
                ..Default::default()
            },
            crate::reference::CitationItem {
                id: "book-4".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert_eq!(processor.process_citation(&citation).unwrap(), "[1–4]");
}

/// Verifies numeric range collapse does not absorb affixed citations.
#[test]
fn test_numeric_citation_number_collapse_skips_affixed_items() {
    use citum_schema::citation::CitationMode;
    use citum_schema::options::Processing;

    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Numeric),
        ..Default::default()
    });
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![TemplateComponent::Number(
            citum_schema::template::TemplateNumber {
                number: citum_schema::template::NumberVariable::CitationNumber,
                ..Default::default()
            },
        )]),
        wrap: Some(WrapPunctuation::Brackets),
        multi_cite_delimiter: Some(",".to_string()),
        collapse: Some(citum_schema::CitationCollapse::CitationNumber),
        ..Default::default()
    });

    let bib = make_numeric_books(&[
        ("book-1", "Author A", 2001, "Book One"),
        ("book-2", "Author B", 2002, "Book Two"),
        ("book-3", "Author C", 2003, "Book Three"),
    ]);
    let processor = Processor::new(style, bib);

    let citation = Citation {
        id: Some("c2".to_string()),
        mode: CitationMode::NonIntegral,
        items: vec![
            crate::reference::CitationItem {
                id: "book-1".to_string(),
                ..Default::default()
            },
            crate::reference::CitationItem {
                id: "book-2".to_string(),
                suffix: Some("n. 12".to_string()),
                ..Default::default()
            },
            crate::reference::CitationItem {
                id: "book-3".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert_eq!(
        processor.process_citation(&citation).unwrap(),
        "[1,2 n. 12,3]"
    );
}

/// Tests the behavior of test_numeric_citation_numbers_follow_bibliography_sort.
#[test]
fn test_numeric_citation_numbers_follow_bibliography_sort() {
    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Numeric),
        ..Default::default()
    });
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![TemplateComponent::Number(
            citum_schema::template::TemplateNumber {
                number: citum_schema::template::NumberVariable::CitationNumber,
                ..Default::default()
            },
        )]),
        wrap: Some(WrapPunctuation::Brackets),
        ..Default::default()
    });
    style.bibliography = Some(BibliographySpec {
        sort: Some(citum_schema::grouping::GroupSortEntry::Explicit(
            citum_schema::grouping::GroupSort {
                template: vec![citum_schema::grouping::GroupSortKey {
                    key: citum_schema::grouping::SortKey::Author,
                    ascending: true,
                    order: None,
                    sort_order: None,
                }],
            },
        )),
        ..Default::default()
    });

    let mut bib = Bibliography::new();
    // Insert in reverse alphabetical order to verify numbering uses sort, not insertion.
    bib.insert(
        "smith2020".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "Jane")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );
    bib.insert(
        "adams2021".to_string(),
        Reference::from(LegacyReference {
            id: "adams2021".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Adams", "Amy")]),
            issued: Some(DateVariable::year(2021)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let citation = Citation {
        mode: citum_schema::citation::CitationMode::NonIntegral,
        items: vec![crate::reference::CitationItem {
            id: "adams2021".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = processor.process_citation(&citation).unwrap();
    assert_eq!(result, "[1]");
}

/// Tests the behavior of test_author_date_citations_preserve_input_order_without_explicit_sort.
#[test]
fn test_author_date_citations_preserve_input_order_without_explicit_sort() {
    let style = make_style();

    let mut bib = make_bibliography();
    bib.insert(
        "smith2020".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "Jane")]),
            title: Some("Another Book".to_string()),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);
    let result = processor
        .process_citation(&Citation {
            id: Some("c1".to_string()),
            items: vec![
                crate::reference::CitationItem {
                    id: "smith2020".to_string(),
                    ..Default::default()
                },
                crate::reference::CitationItem {
                    id: "kuhn1962".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .unwrap();

    assert!(result.find("Smith").unwrap() < result.find("Kuhn").unwrap());
}

/// Tests the behavior of test_numeric_integral_with_multiple_items.
#[test]
fn test_numeric_integral_with_multiple_items() {
    use citum_schema::options::Processing;

    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Numeric),
        ..Default::default()
    });

    let mut bib = make_bibliography();
    bib.insert(
        "smith2020".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "Jane")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(style, bib);

    // Integral mode with multiple items
    let citation = Citation {
        id: Some("c1".to_string()),
        mode: citum_schema::citation::CitationMode::Integral,
        items: vec![
            crate::reference::CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            },
            crate::reference::CitationItem {
                id: "smith2020".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let result = processor.process_citation(&citation).unwrap();
    // Should render both as author + citation number
    assert!(result.contains("Kuhn [1]"));
    assert!(result.contains("Smith [2]"));
}

/// Tests the behavior of test_label_integral_citation_uses_author_text.
#[test]
fn test_label_integral_citation_uses_author_text() {
    use citum_schema::options::Processing;

    let mut style = make_style();
    style.options = Some(Config {
        processing: Some(Processing::Label(LabelConfig {
            preset: LabelPreset::Din,
            ..Default::default()
        })),
        ..Default::default()
    });
    style.citation = Some(citum_schema::CitationSpec {
        template: Some(vec![TemplateComponent::Number(
            citum_schema::template::TemplateNumber {
                number: citum_schema::template::NumberVariable::CitationLabel,
                ..Default::default()
            },
        )]),
        wrap: Some(WrapPunctuation::Brackets),
        ..Default::default()
    });

    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    let citation = Citation {
        id: Some("c1".to_string()),
        mode: citum_schema::citation::CitationMode::Integral,
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = processor.process_citation(&citation).unwrap();
    assert_eq!(result, "Kuhn");
}

/// Tests the behavior of test_citation_visibility_modifiers.
#[test]
fn test_citation_visibility_modifiers() {
    use citum_schema::citation::CitationMode;

    let style = make_style();
    let bib = make_bibliography();
    let processor = Processor::new(style, bib);

    // 1. Suppress Author (citation-level flag applies to all items)
    let cit_suppress = Citation {
        suppress_author: true,
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let res_suppress = processor.process_citation(&cit_suppress).unwrap();
    // Default APA style: (Kuhn, 1962). Suppress Author: (1962).
    assert_eq!(res_suppress, "(1962)");

    // 2. Integral Citation
    let cit_integral = Citation {
        mode: CitationMode::Integral,
        items: vec![crate::reference::CitationItem {
            id: "kuhn1962".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let res_integral = processor.process_citation(&cit_integral).unwrap();
    // Integral mode for author-date styles: Kuhn (1962)
    assert_eq!(res_integral, "Kuhn (1962)");
}

/// Tests the behavior of test_bibliography_per_group_disambiguation.
#[test]
fn test_bibliography_per_group_disambiguation() {
    use citum_schema::grouping::{
        BibliographyGroup, DisambiguationScope, FieldMatcher, GroupHeading, GroupSelector,
    };

    let mut style = make_style();

    // Configure two groups, each with its own disambiguation scope
    style.bibliography.as_mut().unwrap().groups = Some(vec![
        BibliographyGroup {
            id: "group1".to_string(),
            heading: Some(GroupHeading::Literal {
                literal: "Group 1".to_string(),
            }),
            selector: GroupSelector {
                field: Some({
                    let mut map = HashMap::new();
                    map.insert("note".to_string(), FieldMatcher::Exact("g1".to_string()));
                    map
                }),
                ..Default::default()
            },
            sort: None,
            template: None,
            disambiguate: Some(DisambiguationScope::Locally),
        },
        BibliographyGroup {
            id: "group2".to_string(),
            heading: Some(GroupHeading::Literal {
                literal: "Group 2".to_string(),
            }),
            selector: GroupSelector {
                field: Some({
                    let mut map = HashMap::new();
                    map.insert("note".to_string(), FieldMatcher::Exact("g2".to_string()));
                    map
                }),
                ..Default::default()
            },
            sort: None,
            template: None,
            disambiguate: Some(DisambiguationScope::Locally),
        },
    ]);

    // Set up references that would normally disambiguate globally
    let mut bib = Bibliography::new();
    // Two Kuhn 1962 in Group 1
    bib.insert(
        "r1".to_string(),
        Reference::from(LegacyReference {
            id: "r1".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas")]),
            issued: Some(DateVariable::year(1962)),
            title: Some("B title".to_string()),
            note: Some("g1".to_string()),
            ..Default::default()
        }),
    );
    bib.insert(
        "r2".to_string(),
        Reference::from(LegacyReference {
            id: "r2".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas")]),
            issued: Some(DateVariable::year(1962)),
            title: Some("A title".to_string()),
            note: Some("g1".to_string()),
            ..Default::default()
        }),
    );
    // Same name/year in Group 2 - should RESTART suffixes if locally disambiguated
    bib.insert(
        "r3".to_string(),
        Reference::from(LegacyReference {
            id: "r3".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas")]),
            issued: Some(DateVariable::year(1962)),
            title: Some("C title".to_string()),
            note: Some("g2".to_string()),
            ..Default::default()
        }),
    );
    bib.insert(
        "r4".to_string(),
        Reference::from(LegacyReference {
            id: "r4".to_string(),
            author: Some(vec![Name::new("Kuhn", "Thomas")]),
            issued: Some(DateVariable::year(1962)),
            title: Some("D title".to_string()),
            note: Some("g2".to_string()),
            ..Default::default()
        }),
    );

    // Ensure year-suffix is enabled in style
    style.options.as_mut().unwrap().processing = Some(citum_schema::options::Processing::Custom(
        citum_schema::options::ProcessingCustom {
            disambiguate: Some(citum_schema::options::Disambiguation {
                year_suffix: true,
                ..Default::default()
            }),
            ..Default::default()
        },
    ));

    let processor = Processor::new(style, bib);
    let result =
        processor.render_grouped_bibliography_with_format::<crate::render::plain::PlainText>();

    assert!(result.contains("Group 2"));
    // Group 2 should have its own 1962a and 1962b
    let count_a = result.matches("1962a").count();
    assert_eq!(
        count_a, 2,
        "1962a should appear in both groups if disambiguated locally. Output: {}",
        result
    );
}

/// Tests the behavior of test_group_heading_localized_uses_processor_locale.
#[test]
fn test_group_heading_localized_uses_processor_locale() {
    use citum_schema::grouping::{BibliographyGroup, GroupHeading, GroupSelector};

    let mut style = make_style();
    style.bibliography.as_mut().unwrap().groups = Some(vec![BibliographyGroup {
        id: "all".to_string(),
        heading: Some(GroupHeading::Localized {
            localized: HashMap::from([
                ("en-US".to_string(), "English Sources".to_string()),
                ("vi".to_string(), "Tài liệu tiếng Việt".to_string()),
            ]),
        }),
        selector: GroupSelector::default(),
        sort: None,
        template: None,
        disambiguate: None,
    }]);

    let mut locale = citum_schema::Locale::en_us();
    locale.locale = "vi-VN".to_string();

    let processor = Processor::with_locale(style, make_bibliography(), locale);
    let output =
        processor.render_grouped_bibliography_with_format::<crate::render::plain::PlainText>();

    assert!(output.contains("# Tài liệu tiếng Việt"));
}

/// Tests the behavior of test_group_heading_term_resolves_from_locale.
#[test]
fn test_group_heading_term_resolves_from_locale() {
    use citum_schema::grouping::{BibliographyGroup, GroupHeading, GroupSelector};
    use citum_schema::locale::{GeneralTerm, TermForm};

    let mut style = make_style();
    style.bibliography.as_mut().unwrap().groups = Some(vec![BibliographyGroup {
        id: "all".to_string(),
        heading: Some(GroupHeading::Term {
            term: GeneralTerm::And,
            form: Some(TermForm::Long),
        }),
        selector: GroupSelector::default(),
        sort: None,
        template: None,
        disambiguate: None,
    }]);

    let processor = Processor::new(style, make_bibliography());
    let output =
        processor.render_grouped_bibliography_with_format::<crate::render::plain::PlainText>();

    assert!(output.contains("# and"));
}

/// Tests the behavior of test_position_detection_first.
#[test]
fn test_position_detection_first() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let mut citations = vec![Citation {
        items: vec![CitationItem {
            id: "smith2020".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }];

    processor.annotate_positions(&mut citations);

    assert_eq!(citations[0].position, Some(citum_schema::Position::First));
}

/// Tests the behavior of test_position_detection_subsequent.
#[test]
fn test_position_detection_subsequent() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let mut citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "jones2021".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    processor.annotate_positions(&mut citations);

    assert_eq!(citations[0].position, Some(citum_schema::Position::First));
    assert_eq!(citations[1].position, Some(citum_schema::Position::First));
    assert_eq!(
        citations[2].position,
        Some(citum_schema::Position::Subsequent)
    );
}

/// Tests the behavior of test_position_detection_ibid.
#[test]
fn test_position_detection_ibid() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let mut citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                locator: None,
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                locator: None,
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    processor.annotate_positions(&mut citations);

    assert_eq!(citations[0].position, Some(citum_schema::Position::First));
    assert_eq!(citations[1].position, Some(citum_schema::Position::Ibid));
}

/// Tests the behavior of test_position_detection_ibid_with_locator.
#[test]
fn test_position_detection_ibid_with_locator() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let mut citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                locator: Some("42".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                locator: Some("45".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    processor.annotate_positions(&mut citations);

    assert_eq!(citations[0].position, Some(citum_schema::Position::First));
    assert_eq!(
        citations[1].position,
        Some(citum_schema::Position::IbidWithLocator)
    );
}

/// Tests the behavior of test_position_detection_multi_item_no_ibid.
#[test]
fn test_position_detection_multi_item_no_ibid() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let mut citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "jones2021".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![
                CitationItem {
                    id: "smith2020".to_string(),
                    ..Default::default()
                },
                CitationItem {
                    id: "jones2021".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
    ];

    processor.annotate_positions(&mut citations);

    assert_eq!(citations[0].position, Some(citum_schema::Position::First));
    assert_eq!(citations[1].position, Some(citum_schema::Position::First));
    // Multi-item citations should never be ibid, even if all items appeared before
    assert_eq!(
        citations[2].position,
        Some(citum_schema::Position::Subsequent)
    );
}

/// Tests the behavior of test_position_detection_explicit_position_respected.
#[test]
fn test_position_detection_explicit_position_respected() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let mut citations = vec![Citation {
        items: vec![CitationItem {
            id: "smith2020".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::Position::Ibid),
        ..Default::default()
    }];

    processor.annotate_positions(&mut citations);

    // Explicit position should be preserved
    assert_eq!(citations[0].position, Some(citum_schema::Position::Ibid));
}

/// Tests annotate_positions via process_citations for ibid case.
///
/// Verifies that when the same item is cited consecutively with no locator,
/// the second citation is marked as Ibid position.
#[test]
fn test_annotate_positions_ibid_via_public_api() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    let mut citations_mut = citations;
    processor.annotate_positions(&mut citations_mut);

    assert_eq!(
        citations_mut[0].position,
        Some(citum_schema::Position::First)
    );
    assert_eq!(
        citations_mut[1].position,
        Some(citum_schema::Position::Ibid)
    );
}

/// Tests annotate_positions for ibid-with-locator case.
///
/// Verifies that when the same item is cited consecutively with different locators,
/// the second citation is marked as IbidWithLocator position.
#[test]
fn test_annotate_positions_ibid_with_locator_via_public_api() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let processor = Processor::new(make_style(), make_bibliography());
    let citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "kuhn1962".to_string(),
                locator: Some("50".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "kuhn1962".to_string(),
                locator: Some("75".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    let mut citations_mut = citations;
    processor.annotate_positions(&mut citations_mut);

    assert_eq!(
        citations_mut[0].position,
        Some(citum_schema::Position::First)
    );
    assert_eq!(
        citations_mut[1].position,
        Some(citum_schema::Position::IbidWithLocator)
    );
}

/// Tests annotate_positions for subsequent case.
///
/// Verifies that when the same item is cited non-consecutively (with another item in between),
/// the second citation is marked as Subsequent position (not Ibid).
#[test]
fn test_annotate_positions_subsequent_via_public_api() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let mut bib = make_bibliography();
    bib.insert(
        "smith2020".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "John")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );

    let processor = Processor::new(make_style(), bib);
    let citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    let mut citations_mut = citations;
    processor.annotate_positions(&mut citations_mut);

    assert_eq!(
        citations_mut[0].position,
        Some(citum_schema::Position::First)
    );
    assert_eq!(
        citations_mut[1].position,
        Some(citum_schema::Position::First)
    );
    assert_eq!(
        citations_mut[2].position,
        Some(citum_schema::Position::Subsequent)
    );
}

/// Tests annotate_positions for multi-item citations.
///
/// Verifies that positions are tracked correctly per-item in a multi-item citation group,
/// with multi-item citations never being marked as Ibid.
#[test]
fn test_annotate_positions_multi_item_via_public_api() {
    use crate::reference::CitationItem;
    use citum_schema::Citation;

    let mut bib = make_bibliography();
    bib.insert(
        "smith2020".to_string(),
        Reference::from(LegacyReference {
            id: "smith2020".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "John")]),
            issued: Some(DateVariable::year(2020)),
            ..Default::default()
        }),
    );
    let processor = Processor::new(make_style(), bib);
    let citations = vec![
        Citation {
            items: vec![CitationItem {
                id: "kuhn1962".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        Citation {
            items: vec![
                CitationItem {
                    id: "kuhn1962".to_string(),
                    ..Default::default()
                },
                CitationItem {
                    id: "smith2020".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
    ];

    let mut citations_mut = citations;
    processor.annotate_positions(&mut citations_mut);

    assert_eq!(
        citations_mut[0].position,
        Some(citum_schema::Position::First)
    );
    // Multi-item citations are First because at least one item is new
    assert_eq!(
        citations_mut[1].position,
        Some(citum_schema::Position::First)
    );
}

/// Tests that compound numeric mode assigns the same citation number to grouped refs.
#[test]
fn test_compound_numeric_number_assignment() {
    use citum_schema::options::bibliography::CompoundNumericConfig;
    use citum_schema::options::{BibliographyConfig, Config, Processing};
    use indexmap::IndexMap;

    let style = Style {
        options: Some(Config {
            processing: Some(Processing::Numeric),
            bibliography: Some(BibliographyConfig {
                compound_numeric: Some(CompoundNumericConfig::default()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Build bibliography and group membership using top-level sets.
    let refs_json = r#"[
        {
            "class": "monograph",
            "id": "ref-a",
            "type": "book",
            "title": "Book A",
            "issued": "2020"
        },
        {
            "class": "monograph",
            "id": "ref-b",
            "type": "book",
            "title": "Book B",
            "issued": "2021"
        },
        {
            "class": "monograph",
            "id": "ref-c",
            "type": "book",
            "title": "Book C",
            "issued": "2022"
        }
    ]"#;

    let refs: Vec<Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec!["ref-a".to_string(), "ref-b".to_string()],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);
    // Trigger number initialization via process_references
    let _ = processor.process_references();

    let numbers = processor.citation_numbers.borrow();
    // ref-a and ref-b share the same set membership -> same citation number.
    assert_eq!(
        numbers.get("ref-a"),
        numbers.get("ref-b"),
        "grouped refs should have the same citation number"
    );
    // ref-c has no set membership -> different number.
    assert_ne!(
        numbers.get("ref-a"),
        numbers.get("ref-c"),
        "ungrouped ref should have a different citation number"
    );
    assert_eq!(numbers.get("ref-a"), Some(&1), "first group should be 1");
    assert_eq!(numbers.get("ref-c"), Some(&2), "ungrouped ref should be 2");

    // Verify compound_groups tracking
    let groups = processor.compound_groups.borrow();
    assert!(
        groups.contains_key(&1),
        "compound_groups should track group 1"
    );
    let group1 = &groups[&1];
    assert!(group1.contains(&"ref-a".to_string()));
    assert!(group1.contains(&"ref-b".to_string()));
}

/// Verifies compound numeric bibliography rendering merges grouped entries.
#[test]
fn test_compound_numeric_bibliography_rendering() {
    use indexmap::IndexMap;

    let yaml = r#"
info:
  title: Test Compound Numeric
  id: test-compound-numeric
options:
  processing: numeric
  bibliography:
    compound-numeric:
      sub-label: alphabetic
      sub-label-suffix: ")"
      sub-delimiter: ", "
    entry-suffix: .
    separator: ". "
bibliography:
  template:
    - number: citation-number
      wrap: brackets
      suffix: " "
    - contributor: author
      form: long
    - title: primary
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();

    let refs_json = r#"[
        {
            "id": "ref-a",
            "class": "monograph",
            "type": "book",
            "title": "Article A",
            "author": [{"family": "Smith", "given": "A."}],
            "issued": "2020"
        },
        {
            "id": "ref-b",
            "class": "monograph",
            "type": "book",
            "title": "Article B",
            "author": [{"family": "Jones", "given": "B."}],
            "issued": "2021"
        },
        {
            "id": "ref-c",
            "class": "monograph",
            "type": "book",
            "title": "Standalone Article",
            "author": [{"family": "Brown", "given": "C."}],
            "issued": "2022"
        }
    ]"#;
    let refs: Vec<crate::reference::Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = crate::reference::Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec!["ref-a".to_string(), "ref-b".to_string()],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);
    let result = processor.render_bibliography();

    // Compound group should have one shared group label and sub-labels.
    assert_eq!(
        result.matches("[1]").count(),
        1,
        "Expected one group label: {result}"
    );
    assert!(
        result.contains("a)"),
        "Should contain sub-label a): {}",
        result
    );
    assert!(
        result.contains("b)"),
        "Should contain sub-label b): {}",
        result
    );
    // Should have 2 entries (1 compound + 1 standalone), not 3
    let entries: Vec<&str> = result.trim().split("\n\n").collect();
    assert_eq!(
        entries.len(),
        2,
        "Expected 2 entries (1 compound + 1 standalone), got {}: {:?}",
        entries.len(),
        entries
    );
    // Standalone entry should not have sub-labels
    let standalone = entries.iter().find(|e| e.contains("Brown")).unwrap();
    assert!(
        !standalone.contains("a)"),
        "Standalone should not have sub-labels"
    );
}

/// Verifies `subentry: false` keeps grouped citations at whole-group addressing.
#[test]
fn test_compound_numeric_citation_subentry_disabled() {
    use citum_schema::CitationSpec;
    use citum_schema::options::bibliography::CompoundNumericConfig;
    use citum_schema::options::{BibliographyConfig, Config, Processing};
    use citum_schema::template::{NumberVariable, TemplateNumber};
    use indexmap::IndexMap;

    let style = Style {
        citation: Some(CitationSpec {
            wrap: Some(WrapPunctuation::Brackets),
            template: Some(vec![TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::CitationNumber,
                ..Default::default()
            })]),
            ..Default::default()
        }),
        options: Some(Config {
            processing: Some(Processing::Numeric),
            bibliography: Some(BibliographyConfig {
                compound_numeric: Some(CompoundNumericConfig {
                    subentry: false,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let refs_json = r#"[
        {
            "class": "monograph",
            "id": "ref-a",
            "type": "book",
            "title": "Book A",
            "issued": "2020"
        },
        {
            "class": "monograph",
            "id": "ref-b",
            "type": "book",
            "title": "Book B",
            "issued": "2021"
        }
    ]"#;
    let refs: Vec<Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec!["ref-a".to_string(), "ref-b".to_string()],
    );

    let processor = Processor::try_with_compound_sets(style, bib, sets).unwrap();

    let citation = Citation {
        id: Some("c1".to_string()),
        items: vec![CitationItem {
            id: "ref-a".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let rendered = processor.process_citation(&citation).unwrap();
    assert_eq!(rendered, "[1]");
}

/// Verifies integral (narrative) citations include sub-labels for compound groups.
///
/// Regression test: render_author_number_for_numeric_integral_with_format was
/// using a bare citation number without consulting citation_sub_label_for_ref,
/// so grouped refs rendered "Smith [1]" instead of "Smith [1a]".
#[test]
fn test_compound_numeric_integral_citation_sub_label() {
    use citum_schema::options::bibliography::CompoundNumericConfig;
    use citum_schema::options::{BibliographyConfig, Config, Processing};
    use indexmap::IndexMap;

    let style = Style {
        options: Some(Config {
            processing: Some(Processing::Numeric),
            bibliography: Some(BibliographyConfig {
                compound_numeric: Some(CompoundNumericConfig::default()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let refs_json = r#"[
        {
            "class": "monograph",
            "id": "ref-a",
            "type": "book",
            "title": "Book A",
            "author": [{"family": "Smith", "given": "A."}],
            "issued": "2020"
        },
        {
            "class": "monograph",
            "id": "ref-b",
            "type": "book",
            "title": "Book B",
            "author": [{"family": "Jones", "given": "B."}],
            "issued": "2021"
        }
    ]"#;
    let refs: Vec<Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec!["ref-a".to_string(), "ref-b".to_string()],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);

    // First member should render "Smith [1a]", not "Smith [1]"
    let cite_a = Citation {
        id: Some("c-a".to_string()),
        items: vec![CitationItem {
            id: "ref-a".to_string(),
            ..Default::default()
        }],
        mode: citum_schema::citation::CitationMode::Integral,
        ..Default::default()
    };
    let rendered_a = processor.process_citation(&cite_a).unwrap();
    assert!(
        rendered_a.contains("[1a]"),
        "first compound member should show sub-label 'a': got '{}'",
        rendered_a
    );

    // Second member should render "Jones [1b]", not "Jones [1]"
    let cite_b = Citation {
        id: Some("c-b".to_string()),
        items: vec![CitationItem {
            id: "ref-b".to_string(),
            ..Default::default()
        }],
        mode: citum_schema::citation::CitationMode::Integral,
        ..Default::default()
    };
    let rendered_b = processor.process_citation(&cite_b).unwrap();
    assert!(
        rendered_b.contains("[1b]"),
        "second compound member should show sub-label 'b': got '{}'",
        rendered_b
    );
}

/// Verifies render_bibliography correctly merges compound groups when called
/// through the standard public API (regression guard for Bug 3).
///
/// The CLI's non-show_keys bibliography path was calling process_references()
/// directly and rendering entries one-by-one, bypassing merge_compound_entries.
/// This test ensures render_bibliography (the correct path) produces a single
/// merged entry — not two separate [1] entries.
#[test]
fn test_compound_numeric_bibliography_no_duplicate_labels() {
    use indexmap::IndexMap;

    let yaml = r#"
info:
  title: Test Compound Numeric Dedup
  id: test-compound-dedup
options:
  processing: numeric
  bibliography:
    compound-numeric: {}
    entry-suffix: .
bibliography:
  template:
    - number: citation-number
      wrap: brackets
      suffix: " "
    - contributor: author
      form: long
    - title: primary
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();

    let refs_json = r#"[
        {
            "id": "ref-a",
            "class": "monograph",
            "type": "book",
            "title": "Article A",
            "author": [{"family": "Smith", "given": "A."}],
            "issued": "2020"
        },
        {
            "id": "ref-b",
            "class": "monograph",
            "type": "book",
            "title": "Article B",
            "author": [{"family": "Jones", "given": "B."}],
            "issued": "2021"
        }
    ]"#;
    let refs: Vec<crate::reference::Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = crate::reference::Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec!["ref-a".to_string(), "ref-b".to_string()],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);
    let result = processor.render_bibliography();

    // Must have exactly one [1] label — compound group must be merged
    let label_1_count = result.matches("[1]").count();
    assert_eq!(
        label_1_count, 1,
        "expected exactly one [1] label for the merged group, got {}: {}",
        label_1_count, result
    );

    // Should contain exactly one compound group entry
    let entries: Vec<&str> = result.trim().split("\n\n").collect();
    assert_eq!(
        entries.len(),
        1,
        "expected 1 merged entry for the compound group, got {}: {:?}",
        entries.len(),
        entries
    );
}

/// Verifies merged HTML bibliography output does not nest bibliography wrappers.
#[test]
fn test_compound_numeric_bibliography_html_has_no_nested_wrappers() {
    use crate::render::html::Html;
    use indexmap::IndexMap;

    let yaml = r#"
info:
  title: Test Compound Numeric HTML
  id: test-compound-html
options:
  processing: numeric
  bibliography:
    compound-numeric: {}
    entry-suffix: .
    separator: ". "
bibliography:
  template:
    - number: citation-number
      wrap: brackets
      suffix: " "
    - contributor: author
      form: long
    - title: primary
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();

    let refs_json = r#"[
        {
            "id": "ref-a",
            "class": "monograph",
            "type": "book",
            "title": "Article A",
            "author": [{"family": "Smith", "given": "A."}],
            "issued": "2020"
        },
        {
            "id": "ref-b",
            "class": "monograph",
            "type": "book",
            "title": "Article B",
            "author": [{"family": "Jones", "given": "B."}],
            "issued": "2021"
        }
    ]"#;
    let refs: Vec<Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec!["ref-a".to_string(), "ref-b".to_string()],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);
    let result = processor.render_bibliography_with_format::<Html>();

    assert_eq!(
        result.matches("csln-bibliography").count(),
        1,
        "merged HTML should have exactly one bibliography wrapper: {result}"
    );
    assert_eq!(
        result.matches("csln-entry").count(),
        1,
        "merged HTML should have exactly one entry wrapper: {result}"
    );
}

/// Verifies subset bibliography rendering honors keys and does not force a merge.
#[test]
fn test_compound_numeric_selected_bibliography_subset_respects_keys() {
    use indexmap::IndexMap;

    let yaml = r#"
info:
  title: Test Compound Numeric Selection
  id: test-compound-selection
options:
  processing: numeric
  bibliography:
    compound-numeric:
      sub-label: alphabetic
      sub-label-suffix: ")"
    entry-suffix: .
    separator: ". "
bibliography:
  template:
    - number: citation-number
      wrap: brackets
      suffix: " "
    - contributor: author
      form: long
    - title: primary
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();

    let refs_json = r#"[
        {
            "id": "ref-a",
            "class": "monograph",
            "type": "book",
            "title": "Article A",
            "author": [{"family": "Smith", "given": "A."}],
            "issued": "2020"
        },
        {
            "id": "ref-b",
            "class": "monograph",
            "type": "book",
            "title": "Article B",
            "author": [{"family": "Jones", "given": "B."}],
            "issued": "2021"
        }
    ]"#;
    let refs: Vec<Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec!["ref-a".to_string(), "ref-b".to_string()],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);
    let result = processor
        .render_selected_bibliography_with_format::<crate::render::plain::PlainText, _>(vec![
            "ref-b".to_string(),
        ]);

    assert!(
        result.contains("Jones"),
        "selected entry should be rendered: {result}"
    );
    assert!(
        !result.contains("Smith"),
        "unselected entry should be omitted: {result}"
    );
    assert!(
        result.contains("[1]"),
        "selected entry should keep the group number: {result}"
    );
    assert!(
        !result.contains("a)"),
        "single selected member should not be merged: {result}"
    );
    assert!(
        !result.contains("b)"),
        "single selected member should not be merged: {result}"
    );
}

/// Verifies compound numeric citations remain explicit when collapse is disabled.
#[test]
fn test_compound_numeric_citation_subentry_collapse_disabled() {
    use citum_schema::CitationSpec;
    use citum_schema::options::bibliography::CompoundNumericConfig;
    use citum_schema::options::{BibliographyConfig, Config, Processing};
    use citum_schema::template::{NumberVariable, TemplateNumber};
    use indexmap::IndexMap;

    let style = Style {
        citation: Some(CitationSpec {
            wrap: Some(WrapPunctuation::Brackets),
            template: Some(vec![TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::CitationNumber,
                ..Default::default()
            })]),
            delimiter: Some(",".to_string()),
            multi_cite_delimiter: Some(",".to_string()),
            ..Default::default()
        }),
        options: Some(Config {
            processing: Some(Processing::Numeric),
            bibliography: Some(BibliographyConfig {
                compound_numeric: Some(CompoundNumericConfig {
                    subentry: true,
                    collapse_subentries: false,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let refs_json = r#"[
        {"class": "monograph", "id": "ref-a", "type": "book", "title": "Book A", "issued": "2020"},
        {"class": "monograph", "id": "ref-b", "type": "book", "title": "Book B", "issued": "2021"},
        {"class": "monograph", "id": "ref-c", "type": "book", "title": "Book C", "issued": "2022"}
    ]"#;
    let refs: Vec<Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec![
            "ref-a".to_string(),
            "ref-b".to_string(),
            "ref-c".to_string(),
        ],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);
    let citation = Citation {
        id: Some("c1".to_string()),
        items: vec![
            CitationItem {
                id: "ref-a".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ref-b".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ref-c".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let rendered = processor.process_citation(&citation).unwrap();
    assert_eq!(rendered, "[1a,1b,1c]");
}

/// Verifies compound numeric citations collapse adjacent grouped subentries.
#[test]
fn test_compound_numeric_citation_subentry_collapse_enabled() {
    use citum_schema::CitationSpec;
    use citum_schema::options::bibliography::CompoundNumericConfig;
    use citum_schema::options::{BibliographyConfig, Config, Processing};
    use citum_schema::template::{NumberVariable, TemplateNumber};
    use indexmap::IndexMap;

    let style = Style {
        citation: Some(CitationSpec {
            wrap: Some(WrapPunctuation::Brackets),
            template: Some(vec![TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::CitationNumber,
                ..Default::default()
            })]),
            delimiter: Some(",".to_string()),
            multi_cite_delimiter: Some(",".to_string()),
            ..Default::default()
        }),
        options: Some(Config {
            processing: Some(Processing::Numeric),
            bibliography: Some(BibliographyConfig {
                compound_numeric: Some(CompoundNumericConfig {
                    subentry: true,
                    collapse_subentries: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let refs_json = r#"[
        {"class": "monograph", "id": "ref-a", "type": "book", "title": "Book A", "issued": "2020"},
        {"class": "monograph", "id": "ref-b", "type": "book", "title": "Book B", "issued": "2021"},
        {"class": "monograph", "id": "ref-c", "type": "book", "title": "Book C", "issued": "2022"},
        {"class": "monograph", "id": "ref-d", "type": "book", "title": "Book D", "issued": "2023"}
    ]"#;
    let refs: Vec<Reference> = serde_json::from_str(refs_json).unwrap();
    let mut bib = Bibliography::new();
    for r in refs {
        if let Some(id) = r.id() {
            bib.insert(id, r);
        }
    }

    let mut sets = IndexMap::new();
    sets.insert(
        "group-1".to_string(),
        vec![
            "ref-a".to_string(),
            "ref-b".to_string(),
            "ref-c".to_string(),
            "ref-d".to_string(),
        ],
    );

    let processor = Processor::with_compound_sets(style, bib, sets);

    let contiguous = Citation {
        id: Some("c1".to_string()),
        items: vec![
            CitationItem {
                id: "ref-a".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ref-b".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ref-c".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert_eq!(processor.process_citation(&contiguous).unwrap(), "[1a-c]");

    let sparse = Citation {
        id: Some("c2".to_string()),
        items: vec![
            CitationItem {
                id: "ref-a".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ref-c".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert_eq!(processor.process_citation(&sparse).unwrap(), "[1a,c]");
}

/// Verifies checked constructors reject duplicate membership across sets.
#[test]
fn test_try_with_compound_sets_rejects_invalid_membership() {
    let style = Style::default();
    let mut bib = Bibliography::new();
    bib.insert(
        "ref-a".to_string(),
        Reference::from(LegacyReference {
            id: "ref-a".to_string(),
            ref_type: "book".to_string(),
            title: Some("Book A".to_string()),
            ..Default::default()
        }),
    );

    let mut sets = IndexMap::new();
    sets.insert("group-1".to_string(), vec!["ref-a".to_string()]);
    sets.insert("group-2".to_string(), vec!["ref-a".to_string()]);

    let err = Processor::try_with_compound_sets(style, bib, sets).expect_err("must reject sets");
    assert!(
        err.to_string()
            .contains("appears in both compound sets 'group-1' and 'group-2'"),
        "unexpected error: {err}"
    );
}
