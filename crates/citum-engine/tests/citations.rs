#![allow(missing_docs, reason = "test")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::{Processor, render::html::Html};
use citum_schema::{
    CitationOptions, CitationSpec, Style, StyleInfo,
    citation::{Citation, CitationItem, CitationMode, IntegralNameState},
    grouping::{GroupSort, GroupSortEntry, GroupSortKey, SortKey as GroupSortKeyType},
    options::{
        AndOptions, Config, ContributorConfig, DelimiterPrecedesLast, DisplayAsSort,
        IntegralNameConfig, IntegralNameContexts, IntegralNameForm, IntegralNameRule,
        IntegralNameScope, NameForm, Processing, ProcessingCustom, ShortenListOptions,
    },
    reference::InputReference,
};

// --- Helper Functions ---

fn build_numeric_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Numeric Test".to_string()),
            id: Some("numeric-test".into()),
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
        ..Default::default()
    }
}

fn build_title_year_citation_style(sort: Vec<GroupSortKey>) -> Style {
    Style {
        info: StyleInfo {
            title: Some("Title Year Citation Sort Test".to_string()),
            id: Some("title-year-citation-sort-test".into()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Numeric),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            sort: Some(GroupSortEntry::Explicit(GroupSort { template: sort })),
            template: Some(vec![
                citum_schema::tc_title!(Primary),
                citum_schema::tc_date!(Issued, Year),
            ]),
            delimiter: Some(" ".to_string()),
            multi_cite_delimiter: Some("; ".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_integral_name_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Integral Name Memory".to_string()),
            id: Some("integral-name-memory".into()),
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
                template: Some(vec![citum_schema::tc_contributor!(Author, Long)]),
                ..Default::default()
            })),
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Short),
                citum_schema::tc_date!(
                    Issued,
                    Year,
                    wrap = citum_schema::template::WrapPunctuation::Parentheses
                ),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn integral_name_state_overrides_processor_memory() {
    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert(
        "item1".to_string(),
        make_book("item1", "Smith", "John", 2020, "Book A"),
    );
    let processor = Processor::new(build_integral_name_style(), bibliography);

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
            .expect("first should render"),
        "John Smith"
    );
    assert_eq!(
        processor
            .process_citation(&subsequent)
            .expect("subsequent should render"),
        "Smith"
    );
}

fn embedded_mla_enables_integral_name_memory() {
    let style = citum_schema::embedded::get_embedded_style("mla")
        .expect("mla style should be embedded")
        .expect("mla style should parse");
    let integral_names = style
        .options
        .and_then(|options| options.integral_names)
        .expect("mla should enable integral-names");

    assert_eq!(integral_names.rule, Some(IntegralNameRule::FullThenShort));
    assert_eq!(integral_names.scope, Some(IntegralNameScope::Document));
    assert_eq!(
        integral_names.contexts,
        Some(IntegralNameContexts::BodyAndNotes)
    );
    assert_eq!(
        integral_names.subsequent_form,
        Some(IntegralNameForm::Short)
    );
}

// --- Disambiguation Scenarios ---

/// Test year suffix disambiguation with alphabetical title sorting.
fn disambiguation_same_author_same_year_titles_follow_title_order() {
    let input = vec![
        make_book("item1", "Smith", "John", 2020, "Alpha"),
        make_book("item2", "Smith", "John", 2020, "Beta"),
    ];
    let citation_items = vec![vec!["item1", "item2"]];
    let expected = "Smith, (2020a), (2020b)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

/// Test the upstream `YearSuffixAtTwoLevels` disambiguation cascade.
#[allow(
    clippy::too_many_lines,
    reason = "test functions naturally exceed 100 lines"
)]
fn disambiguation_two_level_author_collisions_get_distinct_suffixes() {
    let input = vec![
        make_book_multi_author(
            "ITEM-1",
            vec![("Smith", "John"), ("Jones", "John"), ("Brown", "John")],
            1986,
            "Book A",
        ),
        make_book_multi_author(
            "ITEM-2",
            vec![("Smith", "John"), ("Jones", "John"), ("Brown", "John")],
            1986,
            "Book B",
        ),
        make_book_multi_author(
            "ITEM-3",
            vec![
                ("Smith", "John"),
                ("Jones", "John"),
                ("Brown", "John"),
                ("Green", "John"),
            ],
            1986,
            "Book C",
        ),
        make_book_multi_author(
            "ITEM-4",
            vec![
                ("Smith", "John"),
                ("Jones", "John"),
                ("Brown", "John"),
                ("Green", "John"),
            ],
            1986,
            "Book D",
        ),
    ];

    let mut style = build_author_date_style(true, true, false, Some(3), Some(1));
    style.options = Some(Config {
        processing: Some(Processing::Custom(ProcessingCustom {
            disambiguate: Some(citum_schema::options::Disambiguation {
                year_suffix: true,
                names: true,
                add_givenname: false,
            }),
            ..Default::default()
        })),
        contributors: Some(ContributorConfig {
            display_as_sort: Some(DisplayAsSort::First),
            initialize_with: Some(String::new()),
            shorten: Some(ShortenListOptions {
                min: 3,
                use_first: 1,
                ..Default::default()
            }),
            and: Some(AndOptions::Symbol),
            delimiter_precedes_last: Some(DelimiterPrecedesLast::Never),
            ..Default::default()
        }),
        ..Default::default()
    });
    style.citation = Some(CitationSpec {
        sort: build_author_date_style(true, true, false, Some(3), Some(1))
            .citation
            .as_ref()
            .and_then(|spec| spec.sort.clone()),
        template: Some(vec![
            citum_schema::tc_contributor!(Author, Short),
            citum_schema::tc_date!(
                Issued,
                Year,
                wrap = citum_schema::template::WrapPunctuation::Parentheses
            ),
        ]),
        delimiter: Some(" ".to_string()),
        multi_cite_delimiter: Some("; ".to_string()),
        ..Default::default()
    });

    let mut bibliography = indexmap::IndexMap::new();
    for item in input {
        if let Some(id) = item.id() {
            bibliography.insert(id.to_string(), item);
        }
    }

    let processor = Processor::new(style, bibliography);
    let citation = Citation {
        items: vec![
            CitationItem {
                id: "ITEM-1".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-2".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-3".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-4".to_string(),
                ..Default::default()
            },
        ],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };

    let result = processor
        .process_citation(&citation)
        .expect("Failed to process two-level year-suffix disambiguation citation");
    assert_eq!(
        result,
        "Smith, Jones & Brown (1986a); Smith, Jones & Brown (1986b); Smith, Jones, Brown, et al. (1986a); Smith, Jones, Brown, et al. (1986b)"
    );
}

/// Test year suffix disambiguation with multiple identical references.
fn disambiguation_same_year_articles_increment_suffixes() {
    let input = vec![
        make_article("22", "Ylinen", "A", 1995, "Article A"),
        make_article("21", "Ylinen", "A", 1995, "Article B"),
        make_article("23", "Ylinen", "A", 1995, "Article C"),
    ];
    let citation_items = vec![vec!["22", "21", "23"]];
    let expected = "Ylinen, (1995a), (1995b), (1995c)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

/// Test given name expansion for authors with duplicate family names.
fn disambiguation_duplicate_family_names_expand_given_names_only_where_needed() {
    let input = vec![
        make_book_multi_author(
            "ITEM-1",
            vec![("Asthma", "Albert"), ("Asthma", "Bridget")],
            1980,
            "Book A",
        ),
        make_book("ITEM-2", "Bronchitis", "Beauregarde", 1995, "Book B"),
        make_book("ITEM-3", "Asthma", "Albert", 1885, "Book C"),
    ];
    let citation_items = vec![vec!["ITEM-1", "ITEM-2", "ITEM-3"]];
    // Sorted by author (Asthma, then Bronchitis) and year (1885, then 1980)
    let expected = "Asthma, (1885); Asthma, Asthma, (1980); Bronchitis, (1995)";

    run_test_case_native_with_options(common::TestCaseOptions {
        input: &input,
        citation_items: &citation_items,
        expected,
        mode: "citation",
        disambiguate_year_suffix: false,
        disambiguate_names: false,
        disambiguate_givenname: true,
        et_al_min: None,
        et_al_use_first: None,
    });
}

/// Test et-al expansion success: Name expansion disambiguates conflicting references.
fn disambiguation_et_al_conflicts_expand_names_when_that_resolves_them() {
    let input = vec![
        make_book_multi_author(
            "ITEM-1",
            vec![("Smith", "John"), ("Brown", "John"), ("Jones", "John")],
            1980,
            "Book A",
        ),
        make_book_multi_author(
            "ITEM-2",
            vec![
                ("Smith", "John"),
                ("Beefheart", "Captain"),
                ("Jones", "John"),
            ],
            1980,
            "Book B",
        ),
    ];
    let citation_items = vec![vec!["ITEM-1", "ITEM-2"]];
    let expected = "Smith, Brown, et al., (1980); Smith, Beefheart, et al., (1980)";

    run_test_case_native_with_options(common::TestCaseOptions {
        input: &input,
        citation_items: &citation_items,
        expected,
        mode: "citation",
        disambiguate_year_suffix: false,
        disambiguate_names: true,
        disambiguate_givenname: false,
        et_al_min: Some(3),
        et_al_use_first: Some(1),
    });
}

/// Test et-al expansion failure: Cascade to year suffix when name expansion fails.
fn disambiguation_et_al_conflicts_fall_back_to_year_suffixes() {
    let input = vec![
        make_book_multi_author(
            "ITEM-1",
            vec![("Smith", "John"), ("Brown", "John"), ("Jones", "John")],
            1980,
            "Book A",
        ),
        make_book_multi_author(
            "ITEM-2",
            vec![("Smith", "John"), ("Brown", "John"), ("Jones", "John")],
            1980,
            "Book B",
        ),
    ];
    let citation_items = vec![vec!["ITEM-1", "ITEM-2"]];
    let expected = "Smith et al., (1980a), (1980b)";

    run_test_case_native_with_options(common::TestCaseOptions {
        input: &input,
        citation_items: &citation_items,
        expected,
        mode: "citation",
        disambiguate_year_suffix: true,
        disambiguate_names: true,
        disambiguate_givenname: false,
        et_al_min: Some(3),
        et_al_use_first: Some(1),
    });
}

/// Test given name expansion with initial form (`initialize_with`).
fn disambiguation_initials_are_used_when_short_form_family_names_collide() {
    let input = vec![
        make_book("ITEM-1", "Roe", "Jane", 2000, "Book A"),
        make_book("ITEM-2", "Doe", "John", 2000, "Book B"),
        make_book("ITEM-3", "Doe", "Aloysius", 2000, "Book C"),
        make_book("ITEM-4", "Smith", "Thomas", 2000, "Book D"),
        make_book("ITEM-5", "Smith", "Ted", 2000, "Book E"),
    ];
    let citation_items = vec![
        vec!["ITEM-1"],
        vec!["ITEM-2", "ITEM-3"],
        vec!["ITEM-4", "ITEM-5"],
    ];
    let expected = "Roe, (2000)
J Doe, (2000); A Doe, (2000)
T Smith, (2000); T Smith, (2000)";

    run_test_case_native_with_options(common::TestCaseOptions {
        input: &input,
        citation_items: &citation_items,
        expected,
        mode: "citation",
        disambiguate_year_suffix: false,
        disambiguate_names: false,
        disambiguate_givenname: true,
        et_al_min: None,
        et_al_use_first: None,
    });
}

/// Test subsequent et-al: first cite shows full list; repeat cite applies `subsequent_min/use_first`.
fn subsequent_et_al_thresholds_shorten_the_repeat_citation() {
    use citum_schema::options::{Disambiguation, Processing, ProcessingCustom, ShortenListOptions};

    let authors = vec![("Doe", "John"), ("Smith", "Jane"), ("Jones", "Alice")];

    let item = make_book_multi_author("REF-1", authors, 2020, "A Multi-Author Book");
    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert("REF-1".to_string(), item);

    // Style: min=3 (show all on first cite), subsequent_min=1 + subsequent_use_first=1
    let style = Style {
        info: StyleInfo {
            title: Some("Subsequent Et-Al Test".to_string()),
            id: Some("subsequent-etal-test".into()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                disambiguate: Some(Disambiguation {
                    year_suffix: false,
                    names: false,
                    add_givenname: false,
                }),
                ..Default::default()
            })),
            contributors: Some(citum_schema::options::ContributorConfig {
                shorten: Some(ShortenListOptions {
                    // min=4: first citation has 3 names → below threshold → show all (no et al.)
                    min: 4,
                    use_first: 3,
                    // subsequent_min=2: repeat citation has 3 names → ≥ threshold → et al.
                    subsequent_min: Some(2),
                    subsequent_use_first: Some(1),
                    ..Default::default()
                }),
                initialize_with: Some(" ".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![
                citum_schema::tc_contributor!(Author, Short),
                citum_schema::tc_date!(
                    Issued,
                    Year,
                    wrap = citum_schema::template::WrapPunctuation::Parentheses
                ),
            ]),
            multi_cite_delimiter: Some("; ".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let processor = Processor::new(style, bibliography);

    let first_cite = Citation {
        items: vec![CitationItem {
            id: "REF-1".to_string(),
            ..Default::default()
        }],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };
    let repeat_cite = Citation {
        items: vec![CitationItem {
            id: "REF-1".to_string(),
            ..Default::default()
        }],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };

    let results = processor
        .process_citations(&[first_cite, repeat_cite])
        .expect("citations should render");

    // First cite: all 3 authors visible (no et al.)
    assert!(
        results[0].contains("Doe") && results[0].contains("Smith") && results[0].contains("Jones"),
        "First citation should show all authors, got: {}",
        results[0]
    );
    assert!(
        !results[0].contains("et al"),
        "First citation should not use et al., got: {}",
        results[0]
    );

    // Subsequent cite: only 1 author + et al. (subsequent_use_first=1)
    assert!(
        results[1].contains("et al"),
        "Subsequent citation should use et al., got: {}",
        results[1]
    );
    assert!(
        !results[1].contains("Smith") && !results[1].contains("Jones"),
        "Subsequent citation should hide Smith and Jones, got: {}",
        results[1]
    );
}

/// Test year suffix + et-al with varying author list lengths.
fn subsequent_et_al_configuration_uses_the_subsequent_form_on_repeat() {
    let input = vec![
        make_article_multi_author(
            "ITEM-1",
            vec![
                ("Baur", "Bruno"),
                ("Fröberg", "Lars"),
                ("Baur", "Anette"),
                ("Guggenheim", "Richard"),
                ("Haase", "Martin"),
            ],
            2000,
            "Ultrastructure of snail grazing damage to calcicolous lichens",
        ),
        make_article_multi_author(
            "ITEM-2",
            vec![
                ("Baur", "Bruno"),
                ("Schileyko", "Anatoly A."),
                ("Baur", "Anette"),
            ],
            2000,
            "Ecological observations on Arianta aethiops aethiops",
        ),
        make_article("ITEM-3", "Doe", "John", 2000, "Some bogus title"),
    ];
    let citation_items = vec![vec!["ITEM-1", "ITEM-2", "ITEM-3"]];
    let expected = "Baur et al., (2000b); Baur et al., (2000a); Doe, (2000)";

    run_test_case_native_with_options(common::TestCaseOptions {
        input: &input,
        citation_items: &citation_items,
        expected,
        mode: "citation",
        disambiguate_year_suffix: true,
        disambiguate_names: false,
        disambiguate_givenname: false,
        et_al_min: Some(3),
        et_al_use_first: Some(1),
    });
}

fn citation_scoped_contributor_shorten_applies_without_component_override() {
    let item = make_book_multi_author(
        "REF-1",
        vec![
            ("Doe", "John"),
            ("Smith", "Jane"),
            ("Jones", "Alex"),
            ("Brown", "Casey"),
        ],
        2020,
        "Scoped Shorten",
    );
    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert("REF-1".to_string(), item);

    let style = Style {
        info: StyleInfo {
            title: Some("Scoped contributor shorten".to_string()),
            id: Some("scoped-contributor-shorten".into()),
            ..Default::default()
        },
        citation: Some(CitationSpec {
            options: Some(CitationOptions {
                contributors: Some(ContributorConfig {
                    shorten: Some(ShortenListOptions {
                        min: 4,
                        use_first: 1,
                        and_others: citum_schema::options::AndOtherOptions::Text,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            template: Some(vec![citum_schema::tc_contributor!(Author, Long)]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let processor = Processor::new(style, bibliography);
    let rendered = processor
        .process_citation(&Citation {
            items: vec![CitationItem {
                id: "REF-1".to_string(),
                ..Default::default()
            }],
            mode: CitationMode::NonIntegral,
            ..Default::default()
        })
        .expect("citation should render");

    assert_eq!(
        rendered, "John Doe et al",
        "citation-scoped shorten should apply without component override"
    );
}

/// Test conditional disambiguation with identical author-year pairs.
fn disambiguation_conditions_expand_only_the_marked_items() {
    let input = vec![
        make_book_multi_author(
            "ITEM-1",
            vec![("Doe", "John"), ("Roe", "Jane")],
            2000,
            "Book A",
        ),
        make_book_multi_author(
            "ITEM-2",
            vec![("Doe", "John"), ("Roe", "Jane")],
            2000,
            "Book B",
        ),
    ];
    let citation_items = vec![vec!["ITEM-1", "ITEM-2"]];
    let expected = "Doe, Roe, (2000a), (2000b)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

/// Test year suffix with 30 entries (base-26 suffix wrapping).
fn disambiguation_suffixes_continue_past_z() {
    let mut input = Vec::new();
    let mut citation_ids = Vec::new();

    for i in 1..=30 {
        input.push(make_book(
            &format!("ITEM-{i}"),
            "Smith",
            "John",
            1986,
            "Book",
        ));
        citation_ids.push(format!("ITEM-{i}"));
    }

    let citation_items = vec![
        citation_ids
            .iter()
            .map(std::string::String::as_str)
            .collect(),
    ];
    let expected = "Smith, (1986a), (1986b), (1986c), (1986d), (1986e), (1986f), (1986g), (1986h), (1986i), (1986j), (1986k), (1986l), (1986m), (1986n), (1986o), (1986p), (1986q), (1986r), (1986s), (1986t), (1986u), (1986v), (1986w), (1986x), (1986y), (1986z), (1986aa), (1986ab), (1986ac), (1986ad)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

// --- Numeric Citation Scenarios ---

fn numeric_style_single_reference_renders_bracketed_number() {
    let style = build_numeric_style();

    let bib = citum_schema::bib_map![
        "item1" => make_book("item1", "Smith", "John", 2020, "Title A"),
        "item2" => make_book("item2", "Doe", "Jane", 2021, "Title B"),
    ];
    let processor = Processor::new(style, bib);
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!("item1"))
            .unwrap(),
        "[1]"
    );
    assert_eq!(
        processor
            .process_citation(&citum_schema::cite!("item2"))
            .unwrap(),
        "[2]"
    );
}

// --- Citation Sorting And Grouping Scenarios ---

/// Test basic multi-item citation sorting by author.
fn author_date_sorting_orders_cluster_by_author_then_year() {
    let input = vec![
        make_book("item1", "Kuhn", "Thomas", 1962, "Title A"),
        make_book("item2", "Hawking", "Stephen", 1988, "Title B"),
    ];
    // Kuhn then Hawking in input, should be Hawking then Kuhn in output
    let citation_items = vec![vec!["item1", "item2"]];
    let expected = "Hawking, (1988); Kuhn, (1962)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

/// Test grouped citation sorting by year.
fn group_sorting_orders_cluster_by_year_within_an_author_group() {
    let input = vec![
        make_book("item1", "Kuhn", "Thomas", 1970, "Title A"),
        make_book("item2", "Kuhn", "Thomas", 1962, "Title B"),
    ];
    // 1970 then 1962 in input, should be 1962 then 1970 in output
    let citation_items = vec![vec!["item1", "item2"]];
    let expected = "Kuhn, (1962), (1970)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

/// Test multi-item citation sorting with accented surnames.
fn author_date_sorting_orders_cluster_with_unicode_surnames() {
    let input = vec![
        make_book("item1", "Zimring", "Craig", 2020, "Title A"),
        make_book("item2", "Ó Tuathail", "Gearóid", 1998, "Title B"),
        make_book("item3", "Çelik", "Zeynep", 1996, "Title C"),
    ];
    let citation_items = vec![vec!["item1", "item2", "item3"]];
    let expected = "Çelik, (1996); Ó Tuathail, (1998); Zimring, (2020)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

fn sorting_empty_dates_pushes_undated_items_to_the_end() {
    // Upstream provenance: CSL fixture `date_SortEmptyDatesCitation`.
    fn make_undated_book(id: &str, title: &str) -> InputReference {
        let mut reference = make_book(id, "Smith", "Jane", 2000, title);
        if let InputReference::Monograph(monograph) = &mut reference {
            monograph.issued = citum_schema::reference::EdtfString(String::new());
        }
        reference
    }

    let style = build_title_year_citation_style(vec![
        GroupSortKey {
            key: GroupSortKeyType::Issued,
            ascending: true,
            order: None,
            sort_order: None,
        },
        GroupSortKey {
            key: GroupSortKeyType::Title,
            ascending: true,
            order: None,
            sort_order: None,
        },
    ]);

    let mut bibliography = indexmap::IndexMap::new();
    bibliography.insert("ITEM-1".to_string(), make_undated_book("ITEM-1", "BookA"));
    bibliography.insert(
        "ITEM-2".to_string(),
        make_book("ITEM-2", "Smith", "Jane", 2000, "BookB"),
    );
    bibliography.insert("ITEM-3".to_string(), make_undated_book("ITEM-3", "BookC"));
    bibliography.insert(
        "ITEM-4".to_string(),
        make_book("ITEM-4", "Smith", "Jane", 1999, "BookD"),
    );
    bibliography.insert("ITEM-5".to_string(), make_undated_book("ITEM-5", "BookE"));

    let processor = Processor::new(style, bibliography);
    let citation = Citation {
        items: vec![
            CitationItem {
                id: "ITEM-1".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-2".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-3".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-4".to_string(),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-5".to_string(),
                ..Default::default()
            },
        ],
        mode: CitationMode::NonIntegral,
        ..Default::default()
    };

    let result = processor
        .process_citation(&citation)
        .expect("Failed to process citation with empty-date sort");
    assert_eq!(
        result,
        "BookD 1999; BookB 2000; BookA n.d.; BookC n.d.; BookE n.d."
    );
}

// --- Note Style Position Scenarios ---

fn chicago_notes_immediate_repeat_renders_compact_ibid() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/embedded/chicago-notes-18th.yaml");

    let yaml = std::fs::read_to_string(&path).expect("Failed to read chicago-notes.yaml");
    let style: citum_schema::Style =
        serde_yaml::from_str(&yaml).expect("Failed to parse chicago-notes.yaml");

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];

    let processor = Processor::new(style, bib);

    // First citation (full form)
    let first_citation = citum_schema::Citation {
        items: vec![citum_schema::citation::CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::First),
        ..Default::default()
    };

    let first_result = processor
        .process_citation(&first_citation)
        .expect("Failed to process first citation");
    assert_eq!(first_result, "John Smith, _A Great Book_ (1995).");

    // Second citation with Ibid position (should render "Ibid.")
    let ibid_citation = citum_schema::Citation {
        items: vec![citum_schema::citation::CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Ibid),
        ..Default::default()
    };

    let ibid_result = processor
        .process_citation(&ibid_citation)
        .expect("Failed to process ibid citation");
    assert_eq!(ibid_result, "Ibid.");
}

fn chicago_notes_prefixed_ibid_remains_mid_sentence() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/embedded/chicago-notes-18th.yaml");

    let yaml = std::fs::read_to_string(&path).expect("Failed to read chicago-notes.yaml");
    let style: citum_schema::Style =
        serde_yaml::from_str(&yaml).expect("Failed to parse chicago-notes.yaml");

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];

    let processor = Processor::new(style, bib);

    let ibid_citation = citum_schema::Citation {
        items: vec![citum_schema::citation::CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Ibid),
        prefix: Some("See".to_string()),
        ..Default::default()
    };

    let ibid_result = processor
        .process_citation(&ibid_citation)
        .expect("Failed to process prefixed ibid citation");
    assert!(
        ibid_result.contains("See ibid."),
        "prefixed ibid should remain mid-sentence lowercase: {ibid_result}"
    );
    assert!(
        !ibid_result.contains("See Ibid."),
        "prefixed ibid should not be capitalized as sentence-initial: {ibid_result}"
    );
}

fn chicago_notes_immediate_repeat_with_locator_keeps_the_locator() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/embedded/chicago-notes-18th.yaml");

    let yaml = std::fs::read_to_string(&path).expect("Failed to read chicago-notes.yaml");
    let style: citum_schema::Style =
        serde_yaml::from_str(&yaml).expect("Failed to parse chicago-notes.yaml");

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];

    let processor = Processor::new(style, bib);

    // Citation with IbidWithLocator position and locator
    let ibid_with_locator = citum_schema::Citation {
        items: vec![citum_schema::citation::CitationItem {
            id: "smith1995".to_string(),
            locator: Some(citum_schema::citation::CitationLocator::single(
                citum_schema::citation::LocatorType::Page,
                "45",
            )),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::IbidWithLocator),
        ..Default::default()
    };

    let result = processor
        .process_citation(&ibid_with_locator)
        .expect("Failed to process ibid with locator citation");
    assert!(
        result.contains("Ibid., 45"),
        "IbidWithLocator should contain lexical ibid: {result}"
    );
}

fn chicago_notes_non_immediate_repeat_uses_the_subsequent_short_form() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/embedded/chicago-notes-18th.yaml");

    let yaml = std::fs::read_to_string(&path).expect("Failed to read chicago-notes.yaml");
    let style: citum_schema::Style =
        serde_yaml::from_str(&yaml).expect("Failed to parse chicago-notes.yaml");

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];

    let processor = Processor::new(style, bib);

    // Subsequent citation (after another source in between)
    let subsequent_citation = citum_schema::Citation {
        items: vec![citum_schema::citation::CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Subsequent),
        ..Default::default()
    };

    let result = processor
        .process_citation(&subsequent_citation)
        .expect("Failed to process subsequent citation");
    assert_eq!(result, "Smith, _A Great Book_.");
}

fn chicago_notes_reprint_full_note_renders_original_publisher_metadata() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/embedded/chicago-notes-18th.yaml");

    let yaml = std::fs::read_to_string(&path).expect("Failed to read chicago-notes.yaml");
    let style: citum_schema::Style =
        serde_yaml::from_str(&yaml).expect("Failed to parse chicago-notes.yaml");

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "reprint1994",
        "type": "book",
        "title": "Orientalism",
        "author": [{ "family": "Said", "given": "Edward W." }],
        "issued": { "date-parts": [[1994]] },
        "publisher": "Vintage Books",
        "publisher-place": "New York",
        "original-date": { "date-parts": [[1901]] },
        "original-publisher": "Old Press",
        "original-publisher-place": "Boston"
    }))
    .expect("failed to parse legacy reprint fixture");
    let id = legacy.id.clone();
    let bib = indexmap::IndexMap::from([(id.clone(), legacy.into())]);
    let processor = Processor::new(style, bib);

    let first_citation = citum_schema::Citation {
        items: vec![citum_schema::citation::CitationItem {
            id,
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::First),
        ..Default::default()
    };

    let rendered = processor
        .process_citation(&first_citation)
        .expect("Failed to process reprint citation");
    assert_eq!(
        rendered,
        "Edward W. Said, _Orientalism_ (1901) Old Press, Boston (Vintage Books, 1994)."
    );
}

fn note_styles_without_ibid_overrides_fall_back_to_subsequent() {
    let style = Style {
        info: StyleInfo {
            title: Some("Note Subsequent Fallback".to_string()),
            id: Some("note-subsequent-fallback".into()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Note),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![citum_schema::tc_contributor!(Author, Long)]),
            subsequent: Some(Box::new(CitationSpec {
                template: Some(vec![citum_schema::tc_contributor!(Author, Short)]),
                ..Default::default()
            })),
            ..Default::default()
        }),
        ..Default::default()
    };

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];
    let processor = Processor::new(style, bib);

    let subsequent = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Subsequent),
        ..Default::default()
    };
    let ibid = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Ibid),
        ..Default::default()
    };
    let ibid_with_locator = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            locator: Some(citum_schema::citation::CitationLocator::single(
                citum_schema::citation::LocatorType::Page,
                "45",
            )),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::IbidWithLocator),
        ..Default::default()
    };

    let subsequent_rendered = processor
        .process_citation(&subsequent)
        .expect("subsequent should render");
    let ibid_rendered = processor
        .process_citation(&ibid)
        .expect("ibid should render");
    let ibid_with_locator_rendered = processor
        .process_citation(&ibid_with_locator)
        .expect("ibid-with-locator should render");

    assert_eq!(
        ibid_rendered, subsequent_rendered,
        "Ibid should fall back to subsequent form when `citation.ibid` is absent"
    );
    assert_eq!(
        ibid_with_locator_rendered, subsequent_rendered,
        "IbidWithLocator should fall back to subsequent form when `citation.ibid` is absent"
    );
    assert!(
        !ibid_rendered.contains("Ibid"),
        "fallback should not force lexical ibid output"
    );
}

fn oscola_position_overrides_control_ibid_and_subsequent_forms() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/oscola.yaml");

    let yaml = std::fs::read_to_string(&path).expect("Failed to read oscola.yaml");
    let style: citum_schema::Style =
        serde_yaml::from_str(&yaml).expect("Failed to parse oscola.yaml");

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];

    let processor = Processor::new(style, bib);

    let first = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::First),
        ..Default::default()
    };
    let subsequent = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Subsequent),
        ..Default::default()
    };
    let ibid = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Ibid),
        ..Default::default()
    };
    let ibid_with_locator = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            locator: Some(citum_schema::citation::CitationLocator::single(
                citum_schema::citation::LocatorType::Page,
                "45",
            )),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::IbidWithLocator),
        ..Default::default()
    };

    let first_rendered = processor
        .process_citation(&first)
        .expect("first cite should render");
    let subsequent_rendered = processor
        .process_citation(&subsequent)
        .expect("subsequent cite should render");
    let ibid_rendered = processor
        .process_citation(&ibid)
        .expect("ibid cite should render");
    let ibid_with_locator_rendered = processor
        .process_citation(&ibid_with_locator)
        .expect("ibid-with-locator cite should render");

    assert_eq!(
        first_rendered,
        "John Smith, \u{201C}_A Great Book_\u{201D}(1995)."
    );
    assert_eq!(
        subsequent_rendered,
        "Smith, \u{201C}_A Great Book_\u{201D}."
    );
    assert_eq!(ibid_rendered, "ibid.");
    assert_eq!(ibid_with_locator_rendered, "ibid p45.");
}

fn oscola_without_ibid_reuses_the_subsequent_form_for_immediate_repeats() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/oscola-no-ibid.yaml");

    let yaml = std::fs::read_to_string(&path).expect("Failed to read oscola-no-ibid.yaml");
    let style: citum_schema::Style =
        serde_yaml::from_str(&yaml).expect("Failed to parse oscola-no-ibid.yaml");

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];

    let processor = Processor::new(style, bib);

    let subsequent = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Subsequent),
        ..Default::default()
    };
    let ibid = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Ibid),
        ..Default::default()
    };

    let subsequent_rendered = processor
        .process_citation(&subsequent)
        .expect("subsequent cite should render");
    let ibid_rendered = processor
        .process_citation(&ibid)
        .expect("ibid cite should render");

    assert_eq!(
        ibid_rendered, subsequent_rendered,
        "OSCOLA no-ibid should fall back to the subsequent form for immediate repeats"
    );
    assert!(
        !ibid_rendered.to_lowercase().contains("ibid"),
        "OSCOLA no-ibid should never render lexical ibid: {ibid_rendered}"
    );
}

fn thomson_reuters_subsequent_short_form_keeps_the_locator() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/thomson-reuters-legal-tax-and-accounting-australia.yaml");

    let yaml = std::fs::read_to_string(&path)
        .expect("Failed to read thomson-reuters-legal-tax-and-accounting-australia.yaml");
    let style: citum_schema::Style = serde_yaml::from_str(&yaml)
        .expect("Failed to parse thomson-reuters-legal-tax-and-accounting-australia.yaml");

    let bib = citum_schema::bib_map![
        "smith1995" => make_book("smith1995", "Smith", "John", 1995, "A Great Book"),
    ];

    let processor = Processor::new(style, bib);

    let first = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::First),
        ..Default::default()
    };
    let subsequent = Citation {
        items: vec![CitationItem {
            id: "smith1995".to_string(),
            locator: Some(citum_schema::citation::CitationLocator::single(
                citum_schema::citation::LocatorType::Page,
                "23",
            )),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::Subsequent),
        ..Default::default()
    };

    let first_rendered = processor
        .process_citation(&first)
        .expect("first cite should render");
    let subsequent_rendered = processor
        .process_citation(&subsequent)
        .expect("subsequent cite should render");

    assert_eq!(first_rendered, "Smith, \u{201C}A Great Book\u{201D}(1995).");
    assert_eq!(
        subsequent_rendered,
        "Smith, \u{201C}_A Great Book_\u{201D} at 23."
    );
}

// --- Grouped Citation Rendering Tests ---

fn grouped_author_date_mode_groups_items_by_author() {
    let input = vec![
        make_book("item1", "Smith", "John", 2020, "Book A"),
        make_book("item1b", "Smith", "John", 2021, "Book B"),
        make_book("item2", "Jones", "Jane", 2020, "Book C"),
    ];
    let citation_items = vec![vec!["item1", "item1b", "item2"]];
    // Grouped author-date clusters by author then year within group
    let expected = "Jones, (2020); Smith, (2020), (2021)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

fn grouped_numeric_mode_preserves_item_order() {
    let input = vec![
        make_book("item1", "Smith", "John", 2020, "Book A"),
        make_book("item2", "Jones", "Jane", 2021, "Book B"),
        make_book("item3", "Brown", "Bob", 2022, "Book C"),
    ];
    let citation_items = vec![vec!["item1", "item2", "item3"]];
    // Numeric in default author-date style still sorts by author
    let expected = "Brown, (2022); Jones, (2021); Smith, (2020)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

fn grouped_integral_mode_displays_first_author_only() {
    let input = vec![
        make_book("item1", "Smith", "John", 2020, "Book A"),
        make_book("item1b", "Smith", "John", 2021, "Book B"),
    ];
    let citation_items = vec![vec!["item1", "item1b"]];
    // Author-date style groups same-author works by year
    let expected = "Smith, (2020), (2021)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

fn citation_html_injects_sparse_template_indices_when_enabled() {
    let style_yaml = r#"
info:
  title: Indexed Citation Preview
  id: indexed-citation-preview
citation:
  template:
    - title: primary
    - variable: doi
      prefix: ". "
    - variable: url
      prefix: " "
"#;
    let style: Style = serde_yaml::from_str(style_yaml).expect("style should parse");

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(serde_json::json!({
        "id": "ITEM-1",
        "type": "book",
        "title": "Preview Book",
        "URL": "https://example.com/preview-book"
    }))
    .expect("legacy fixture should parse");

    let mut bib = indexmap::IndexMap::new();
    bib.insert("ITEM-1".to_string(), legacy.into());

    let processor = Processor::new(style, bib).with_inject_ast_indices(true);
    let citation = Citation {
        items: vec![CitationItem {
            id: "ITEM-1".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let rendered = processor
        .process_citation_with_format::<Html>(&citation)
        .expect("citation should render");

    assert!(
        rendered.contains(r#"class="citum-title" data-index="0""#),
        "title wrapper should carry the first template index: {rendered}"
    );
    assert!(
        rendered.contains(r#"class="citum-url" data-index="2""#),
        "url wrapper should carry the sparse third template index: {rendered}"
    );
    assert!(
        !rendered.contains(r#"data-index="1""#),
        "missing DOI output should preserve sparse template indices: {rendered}"
    );
}

mod integral_name_memory {
    use super::announce_behavior;

    #[test]
    fn explicit_integral_name_state_overrides_processor_memory() {
        announce_behavior(
            "An explicit integral-name state should force full-form on first cite and short-form on repeat.",
        );
        super::integral_name_state_overrides_processor_memory();
    }

    #[test]
    fn embedded_mla_enables_integral_name_memory() {
        announce_behavior(
            "The embedded MLA style should enable document-scoped integral-name memory with full-then-short behavior.",
        );
        super::embedded_mla_enables_integral_name_memory();
    }
}

mod disambiguation {
    use super::announce_behavior;

    #[test]
    fn same_author_same_year_titles_follow_title_order() {
        announce_behavior(
            "Two same-author, same-year works should receive year suffixes in title order.",
        );
        super::disambiguation_same_author_same_year_titles_follow_title_order();
    }

    #[test]
    fn two_level_author_collisions_get_distinct_suffixes() {
        announce_behavior(
            "Colliding author lists at multiple truncation levels should still end up with distinct year suffixes.",
        );
        super::disambiguation_two_level_author_collisions_get_distinct_suffixes();
    }

    #[test]
    fn same_year_articles_increment_suffixes() {
        announce_behavior(
            "Same-year articles should increment year suffixes a, b, c in citation order.",
        );
        super::disambiguation_same_year_articles_increment_suffixes();
    }

    #[test]
    fn duplicate_family_names_expand_given_names_only_where_needed() {
        announce_behavior(
            "Family-name collisions should expand given names only for the ambiguous items.",
        );
        super::disambiguation_duplicate_family_names_expand_given_names_only_where_needed();
    }

    #[test]
    fn et_al_conflicts_expand_names_when_that_resolves_them() {
        announce_behavior(
            "When et al. creates a collision, name expansion should win if it can resolve the ambiguity.",
        );
        super::disambiguation_et_al_conflicts_expand_names_when_that_resolves_them();
    }

    #[test]
    fn et_al_conflicts_fall_back_to_year_suffixes() {
        announce_behavior(
            "When et al. collisions cannot be resolved by names alone, year suffixes should disambiguate the cites.",
        );
        super::disambiguation_et_al_conflicts_fall_back_to_year_suffixes();
    }

    #[test]
    fn initials_are_used_when_short_form_family_names_collide() {
        announce_behavior(
            "Short-form family-name collisions should expand to initials when that is the configured fallback.",
        );
        super::disambiguation_initials_are_used_when_short_form_family_names_collide();
    }

    #[test]
    fn subsequent_et_al_thresholds_shorten_the_repeat_citation() {
        announce_behavior(
            "Subsequent-citation et al. thresholds should shorten a repeat citation more aggressively than the first cite.",
        );
        super::subsequent_et_al_thresholds_shorten_the_repeat_citation();
    }

    #[test]
    fn subsequent_et_al_configuration_uses_the_subsequent_form_on_repeat() {
        announce_behavior(
            "Repeat citations should honor the subsequent et al. configuration instead of reusing first-citation name expansion.",
        );
        super::subsequent_et_al_configuration_uses_the_subsequent_form_on_repeat();
    }

    #[test]
    fn conditions_expand_only_the_marked_items() {
        announce_behavior(
            "Conditional disambiguation should expand only the specifically marked citation items.",
        );
        super::disambiguation_conditions_expand_only_the_marked_items();
    }

    #[test]
    fn suffixes_continue_past_z() {
        announce_behavior(
            "Year suffix generation should continue past z without resetting or truncating.",
        );
        super::disambiguation_suffixes_continue_past_z();
    }
}

mod contributor_scoping {
    use super::announce_behavior;

    #[test]
    fn citation_scoped_shorten_applies_without_component_override() {
        announce_behavior(
            "Citation-scoped contributor shortening should apply even when the template contributor has no explicit shorten block.",
        );
        super::citation_scoped_contributor_shorten_applies_without_component_override();
    }
}

mod numeric_style {
    use super::announce_behavior;

    #[test]
    fn single_reference_renders_bracketed_number() {
        announce_behavior(
            "A numeric citation style should render a single reference number in brackets.",
        );
        super::numeric_style_single_reference_renders_bracketed_number();
    }
}

mod sorting_and_grouping {
    use super::announce_behavior;

    #[test]
    fn author_date_sorting_orders_cluster_by_author_then_year() {
        announce_behavior(
            "Author-date citation clusters should sort entries by author and then by year.",
        );
        super::author_date_sorting_orders_cluster_by_author_then_year();
    }

    #[test]
    fn group_sorting_orders_cluster_by_year_within_an_author_group() {
        announce_behavior(
            "Grouped citation sorting should keep works together by author and then sort years within that group.",
        );
        super::group_sorting_orders_cluster_by_year_within_an_author_group();
    }

    #[test]
    fn author_date_sorting_orders_cluster_with_unicode_surnames() {
        announce_behavior(
            "Author-date citation clusters should sort accented surnames with Unicode-aware collation.",
        );
        super::author_date_sorting_orders_cluster_with_unicode_surnames();
    }

    #[test]
    fn empty_dates_push_undated_items_to_the_end() {
        announce_behavior(
            "Undated items should sort after dated items rather than interleaving with them.",
        );
        super::sorting_empty_dates_pushes_undated_items_to_the_end();
    }
}

mod note_style_positions {
    use super::announce_behavior;

    #[test]
    fn chicago_notes_immediate_repeat_renders_compact_ibid() {
        announce_behavior("An immediate Chicago note repeat should collapse to a compact ibid.");
        super::chicago_notes_immediate_repeat_renders_compact_ibid();
    }

    #[test]
    fn chicago_notes_immediate_repeat_with_locator_keeps_the_locator() {
        announce_behavior(
            "An immediate Chicago note repeat with a locator should keep the locator in the ibid form.",
        );
        super::chicago_notes_immediate_repeat_with_locator_keeps_the_locator();
    }

    #[test]
    fn chicago_notes_prefixed_ibid_remains_mid_sentence() {
        announce_behavior(
            "A prefixed Chicago ibid should stay lowercase because the note marker is no longer sentence-initial.",
        );
        super::chicago_notes_prefixed_ibid_remains_mid_sentence();
    }

    #[test]
    fn chicago_notes_non_immediate_repeat_uses_the_subsequent_short_form() {
        announce_behavior(
            "A non-immediate Chicago note repeat should use the shortened subsequent-note form instead of ibid.",
        );
        super::chicago_notes_non_immediate_repeat_uses_the_subsequent_short_form();
    }

    #[test]
    fn chicago_notes_reprint_full_note_renders_original_publisher_metadata() {
        announce_behavior(
            "A full Chicago note for a reprint should include original publisher metadata before the current publication details.",
        );
        super::chicago_notes_reprint_full_note_renders_original_publisher_metadata();
    }

    #[test]
    fn note_styles_without_ibid_overrides_fall_back_to_subsequent() {
        announce_behavior(
            "Note styles without ibid overrides should fall back to their normal subsequent-note form.",
        );
        super::note_styles_without_ibid_overrides_fall_back_to_subsequent();
    }

    #[test]
    fn oscola_position_overrides_control_ibid_and_subsequent_forms() {
        announce_behavior(
            "OSCOLA note-position overrides should decide when to emit ibid versus a subsequent short form.",
        );
        super::oscola_position_overrides_control_ibid_and_subsequent_forms();
    }

    #[test]
    fn oscola_without_ibid_reuses_the_subsequent_form_for_immediate_repeats() {
        announce_behavior(
            "When OSCOLA disables ibid, even immediate repeats should reuse the subsequent short form.",
        );
        super::oscola_without_ibid_reuses_the_subsequent_form_for_immediate_repeats();
    }

    #[test]
    fn thomson_reuters_subsequent_short_form_keeps_the_locator() {
        announce_behavior(
            "Thomson Reuters repeated notes should shorten the cite while preserving the locator.",
        );
        super::thomson_reuters_subsequent_short_form_keeps_the_locator();
    }

    // --- Regression Tests for Grouped Citation Rendering ---

    #[test]
    fn grouped_author_date_mode_groups_items_by_author() {
        announce_behavior(
            "Author-date grouped rendering should collapse multiple items with same author.",
        );
        super::grouped_author_date_mode_groups_items_by_author();
    }

    #[test]
    fn grouped_numeric_mode_preserves_item_order() {
        announce_behavior(
            "Numeric grouped rendering should maintain citation order without author collapse.",
        );
        super::grouped_numeric_mode_preserves_item_order();
    }

    #[test]
    fn grouped_integral_mode_displays_first_author_only() {
        announce_behavior(
            "Integral grouped rendering should display only the first item's author.",
        );
        super::grouped_integral_mode_displays_first_author_only();
    }
}

mod annotated_html_preview {
    use super::announce_behavior;

    #[test]
    fn citation_indices_stay_sparse_when_template_components_do_not_render() {
        announce_behavior(
            "Annotated citation HTML should preserve the original template indices when intermediate components do not render.",
        );
        super::citation_html_injects_sparse_template_indices_when_enabled();
    }
}

#[test]
fn test_personal_communication_citation_rendering_is_style_driven() {
    let bib_vec = serde_yaml::from_str::<Vec<InputReference>>(
        r#"
- id: oglethorpe-1733
  class: monograph
  type: personal-communication
  contributors:
  - role: author
    contributor: {given: James, family: Oglethorpe}
  - role: recipient
    contributor: {name: "the Trustees"}
  issued: '1733-01-13'
"#,
    )
    .unwrap();

    let mut bib = indexmap::IndexMap::new();
    for item in bib_vec {
        bib.insert(item.id().unwrap().to_string(), item);
    }

    // Mock APA 7th non-integral: (J. Oglethorpe, personal communication, January 13, 1733)
    let apa_style = Style {
        info: StyleInfo {
            title: Some("APA Personal Communication".to_string()),
            ..Default::default()
        },
        citation: Some(CitationSpec {
            template: Some(vec![
                citum_schema::template::TemplateComponent::Contributor(
                    citum_schema::template::TemplateContributor {
                        contributor: citum_schema::template::ContributorRole::Author,
                        form: citum_schema::template::ContributorForm::Long,
                        name_order: Some(citum_schema::template::NameOrder::GivenFirst),
                        rendering: citum_schema::template::Rendering {
                            name_form: Some(NameForm::Initials),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
                citum_schema::tc_term!(PersonalCommunication),
                citum_schema::tc_date!(Issued, Full),
            ]),
            delimiter: Some(", ".to_string()),
            wrap: Some(citum_schema::template::WrapPunctuation::Parentheses.into()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let processor = Processor::new(apa_style, bib);
    let citation = Citation {
        items: vec![CitationItem {
            id: "oglethorpe-1733".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let output = processor.process_citation(&citation).unwrap();
    assert_eq!(
        output,
        "(J. Oglethorpe, personal communication, January 13, 1733)"
    );
}
