/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::Processor;
use citum_schema::{
    CitationSpec, Style, StyleInfo,
    citation::{Citation, CitationItem, CitationMode},
    options::{
        AndOptions, Config, ContributorConfig, DelimiterPrecedesLast, DisplayAsSort, Processing,
        ProcessingCustom, ShortenListOptions,
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
        ..Default::default()
    }
}

// --- Disambiguation Tests ---

/// Test year suffix disambiguation with alphabetical title sorting.
#[test]
fn test_disambiguate_yearsuffixandsort() {
    let input = vec![
        make_book("item1", "Smith", "John", 2020, "Alpha"),
        make_book("item2", "Smith", "John", 2020, "Beta"),
    ];
    let citation_items = vec![vec!["item1", "item2"]];
    let expected = "Smith, (2020a), (2020b)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

/// Test the upstream YearSuffixAtTwoLevels disambiguation cascade.
#[test]
fn test_disambiguate_yearsuffixattwolevels() {
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
            initialize_with: Some("".to_string()),
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
            bibliography.insert(id, item);
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
#[test]
fn test_disambiguate_yearsuffixmixeddates() {
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
#[test]
fn test_disambiguate_bycitetwoauthorssamefamilyname() {
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

    run_test_case_native_with_options(
        &input,
        &citation_items,
        expected,
        "citation",
        false,
        false,
        true,
        None,
        None,
    );
}

/// Test et-al expansion success: Name expansion disambiguates conflicting references.
#[test]
fn test_disambiguate_addnamessuccess() {
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

    run_test_case_native_with_options(
        &input,
        &citation_items,
        expected,
        "citation",
        false,
        true,
        false,
        Some(3),
        Some(1),
    );
}

/// Test et-al expansion failure: Cascade to year suffix when name expansion fails.
#[test]
fn test_disambiguate_addnamesfailure() {
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

    run_test_case_native_with_options(
        &input,
        &citation_items,
        expected,
        "citation",
        true,
        true,
        false,
        Some(3),
        Some(1),
    );
}

/// Test given name expansion with initial form (initialize_with).
#[test]
fn test_disambiguate_bycitegivennameshortforminitializewith() {
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

    run_test_case_native_with_options(
        &input,
        &citation_items,
        expected,
        "citation",
        false,
        false,
        true,
        None,
        None,
    );
}

/// Test year suffix + et-al with varying author list lengths.
#[test]
fn test_disambiguate_basedonetalsubsequent() {
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

    run_test_case_native_with_options(
        &input,
        &citation_items,
        expected,
        "citation",
        true,
        false,
        false,
        Some(3),
        Some(1),
    );
}

/// Test conditional disambiguation with identical author-year pairs.
#[test]
fn test_disambiguate_bycitedisambiguatecondition() {
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
#[test]
fn test_disambiguate_yearsuffixfiftytwoentries() {
    let mut input = Vec::new();
    let mut citation_ids = Vec::new();

    for i in 1..=30 {
        input.push(make_book(
            &format!("ITEM-{}", i),
            "Smith",
            "John",
            1986,
            "Book",
        ));
        citation_ids.push(format!("ITEM-{}", i));
    }

    let citation_items = vec![citation_ids.iter().map(|s| s.as_str()).collect()];
    let expected = "Smith, (1986a), (1986b), (1986c), (1986d), (1986e), (1986f), (1986g), (1986h), (1986i), (1986j), (1986k), (1986l), (1986m), (1986n), (1986o), (1986p), (1986q), (1986r), (1986s), (1986t), (1986u), (1986v), (1986w), (1986x), (1986y), (1986z), (1986aa), (1986ab), (1986ac), (1986ad)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

// --- Numeric Citation Tests ---

#[test]
fn test_numeric_citation() {
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

// --- Sorting and Grouping Tests ---

/// Test basic multi-item citation sorting by author.
#[test]
fn test_citation_sorting_by_author() {
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
#[test]
fn test_grouped_citation_sorting_by_year() {
    let input = vec![
        make_book("item1", "Kuhn", "Thomas", 1970, "Title A"),
        make_book("item2", "Kuhn", "Thomas", 1962, "Title B"),
    ];
    // 1970 then 1962 in input, should be 1962 then 1970 in output
    let citation_items = vec![vec!["item1", "item2"]];
    let expected = "Kuhn, (1962), (1970)";

    run_test_case_native(&input, &citation_items, expected, "citation");
}

// --- Position-Based Citation Tests (Note Styles) ---

#[test]
fn test_chicago_notes_ibid_renders_compact() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/chicago-notes.yaml");

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
    assert!(
        first_result.contains("Smith"),
        "First citation should contain author name"
    );

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
    assert!(
        ibid_result.contains("Ibid."),
        "Ibid citation should contain 'Ibid.': got {}",
        ibid_result
    );
    // The ibid position is being respected - the citation should be shorter
    // than the full first citation because it uses the ibid spec
    assert!(
        ibid_result.len() < first_result.len(),
        "Ibid citation should be shorter than full first citation"
    );
}

#[test]
fn test_chicago_notes_ibid_with_locator() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/chicago-notes.yaml");

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
            label: Some(citum_schema::citation::LocatorType::Page),
            locator: Some("45".to_string()),
            ..Default::default()
        }],
        position: Some(citum_schema::citation::Position::IbidWithLocator),
        ..Default::default()
    };

    let result = processor
        .process_citation(&ibid_with_locator)
        .expect("Failed to process ibid with locator citation");
    assert!(
        result.contains("Ibid."),
        "IbidWithLocator should contain 'Ibid.'"
    );
    assert!(
        result.contains("45"),
        "IbidWithLocator should contain locator value"
    );
}

#[test]
fn test_chicago_notes_subsequent_renders_short() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("styles/chicago-notes.yaml");

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
    assert!(
        result.contains("Smith"),
        "Subsequent citation should contain shortened author"
    );
    assert!(
        result.contains("Great Book"),
        "Subsequent citation should contain shortened title"
    );
}
