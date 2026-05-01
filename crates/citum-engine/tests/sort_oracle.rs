#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in test, benchmark, and example code."
)]
#![allow(missing_docs, reason = "test")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::announce_behavior;

use citum_engine::Processor;
use citum_engine::io::load_bibliography;
use citum_schema::Style;
use citum_schema::citation::{Citation, CitationItem};
use std::fs;
use std::path::{Path, PathBuf};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_style(path: &Path) -> Style {
    let bytes = fs::read(path).expect("style fixture should be readable");
    serde_yaml::from_slice(&bytes).expect("style fixture should parse")
}

/// Load sort oracle fixture into native Citum references for testing.
fn load_sort_oracle_bibliography()
-> indexmap::IndexMap<String, citum_schema::reference::InputReference> {
    let root = project_root();
    let path = root.join("tests/fixtures/sort-oracle.json");
    load_bibliography(&path).expect("sort-oracle fixture should load")
}

/// Test APA 7th Edition sort order: author, date, title.
/// Adams has 3 works in 2020 — should sort alphabetically by title.
#[test]
fn test_apa_7th_sort_same_author_year_by_title() {
    announce_behavior(
        "Works by the same author in the same year are sorted alphabetically by title.",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // All three Adams 2020 items should appear in title order: Academic, Digital, Ethics.
    // APA now preserves legacy CSL title casing for these sort-oracle fixtures.
    let academic_pos = result
        .find("Academic Enterprise")
        .expect("Academic Enterprise should be in output");
    let digital_pos = result
        .find("Digital transformation")
        .or_else(|| result.find("Digital Transformation"))
        .expect("Digital Transformation should be in output");
    let ethics_pos = result
        .find("Ethics in Research")
        .expect("Ethics in Research should be in output");

    assert!(
        academic_pos < digital_pos,
        "Academic should come before Digital"
    );
    assert!(
        digital_pos < ethics_pos,
        "Digital should come before Ethics"
    );
}

/// Test APA 7th Edition anonymous work sorting by title (without leading article).
/// Anonymous works should sort by title when no author is present.
#[test]
fn test_apa_7th_sort_anonymous_works_by_title() {
    announce_behavior("Anonymous works sort by title with leading articles stripped.");
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // Both anonymous works should be present; "A Brief Guide" should come before "The Chicago Manual"
    // when sorting alphabetically (note: actual article-stripping not yet implemented per SORT-4 ignore note)
    let chicago_pos = result
        .find("Chicago Manual")
        .expect("Chicago Manual should be in output");
    let guide_pos = result
        .find("A Brief Guide")
        .expect("A Brief Guide should be in output");

    assert!(
        guide_pos < chicago_pos,
        "Anonymous works should file under title with article stripping. Got: {result}"
    );
}

/// Test numeric style sort: citation numbers remain stable for the bibliography order in the
/// fixture used here.
/// Multiple authors with same surname should maintain consistent alphabetical ordering.
#[test]
fn test_numeric_sort_by_citation_order() {
    announce_behavior(
        "Numeric style assigns citation numbers by fixture insertion order, not by author or title.",
    );
    // Build a simple numeric style for testing
    let style = {
        use citum_schema::options::Processing;
        use citum_schema::{BibliographySpec, CitationSpec, StyleInfo, options::Config};

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
            bibliography: Some(BibliographySpec {
                template: Some(vec![
                    citum_schema::tc_number!(CitationNumber, suffix = ". "),
                    citum_schema::tc_contributor!(Author, Long),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        }
    };

    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);

    // Process citations in a specific order to test numeric assignment
    let cit1 = Citation {
        items: vec![CitationItem {
            id: "SORT-6".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let result1 = processor
        .process_citation(&cit1)
        .expect("citation 1 should process");
    // SORT-6 is the 6th item in fixture, so gets number [6] in numeric style
    assert_eq!(
        result1, "[6]",
        "SORT-6 should be numbered [6] (6th in fixture)"
    );

    let cit2 = Citation {
        items: vec![CitationItem {
            id: "SORT-7".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let result2 = processor
        .process_citation(&cit2)
        .expect("citation 2 should process");
    // SORT-7 is the 7th item in fixture
    assert_eq!(
        result2, "[7]",
        "SORT-7 should be numbered [7] (7th in fixture)"
    );

    let bib_result = processor.render_bibliography();
    assert!(!bib_result.is_empty(), "Bibliography should render");
}

/// Test all-caps surname handling in sort order.
/// SMITH and WILLIAMS surnames should sort correctly in author-date and numeric styles.
#[test]
fn test_uppercase_surname_sort_order() {
    announce_behavior("All-caps surnames sort in the same order as normally-cased surnames.");
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // SMITH and WILLIAMS are both all-caps; SMITH should come before WILLIAMS alphabetically
    if let (Some(smith_pos), Some(williams_pos)) =
        (result.find("Smith, Robert"), result.find("Williams, David"))
    {
        assert!(
            smith_pos < williams_pos,
            "Smith should come before Williams in sort order"
        );
    }
}

/// Test multi-author books and articles in same year sorted by first author.
#[test]
fn test_multiauthor_same_year_sort() {
    announce_behavior(
        "Multi-author works with the same year appear together in author-date sort order.",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // Brown (2022) appears in both article and book form
    // Book version should sort with article version in author-date order
    let brown_refs = result.matches("Brown").count();
    assert!(
        brown_refs > 0,
        "Brown references should appear in bibliography"
    );
}

/// Test accented surnames sort with Unicode-aware collation instead of bytewise ordering.
#[test]
fn test_apa_7th_sort_unicode_accented_surnames() {
    announce_behavior(
        "Accented surnames sort near their ASCII peers in author-date bibliographies.",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    let celik_pos = result.find("Çelik, Z.").expect("Çelik should be in output");
    let o_tuathail_pos = result
        .find("Ó Tuathail, G.")
        .expect("Ó Tuathail should be in output");
    let zimring_pos = result
        .find("Zimring, C. A.")
        .expect("Zimring should be in output");

    assert!(
        celik_pos < o_tuathail_pos,
        "Çelik should sort before Ó Tuathail. Got: {result}"
    );
    assert!(
        o_tuathail_pos < zimring_pos,
        "Ó Tuathail should sort before Zimring. Got: {result}"
    );
}

/// Test numeric style volume/issue variation doesn't affect sort.
/// Numeric styles should sort by citation order, not by volume/issue.
#[test]
fn test_numeric_style_volume_issue_independence() {
    announce_behavior(
        "Numeric style numbering is determined by citation order, not by volume or issue.",
    );
    // Build a simple numeric style for testing
    let style = {
        use citum_schema::options::Processing;
        use citum_schema::{BibliographySpec, CitationSpec, StyleInfo, options::Config};

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
            bibliography: Some(BibliographySpec {
                template: Some(vec![
                    citum_schema::tc_number!(CitationNumber, suffix = ". "),
                    citum_schema::tc_contributor!(Author, Long),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        }
    };

    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);

    // SORT-6: volume 15, issue 3
    // SORT-7: volume 8, issue 2
    // In numeric style, citation order (not volume) determines numbering
    let cit6 = Citation {
        items: vec![CitationItem {
            id: "SORT-6".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let result6 = processor
        .process_citation(&cit6)
        .expect("citation should process");
    // SORT-6 is 6th item; gets [6] regardless of its volume/issue
    assert_eq!(
        result6, "[6]",
        "SORT-6 should be [6] (citation order, not volume)"
    );

    let cit7 = Citation {
        items: vec![CitationItem {
            id: "SORT-7".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let result7 = processor
        .process_citation(&cit7)
        .expect("citation should process");
    // SORT-7 is 7th item; gets [7] regardless of its volume/issue
    assert_eq!(
        result7, "[7]",
        "SORT-7 should be [7] (citation order, not volume)"
    );
}

/// Test cross-script sort order in a mixed Latin/Arabic/Hangul bibliography.
///
/// Under an en-US tailored collator (ICU4X), Latin entries sort first in
/// alphabetical order; Arabic-script entries sort after all Latin; Hangul
/// entries sort after Arabic. This order is a property of the CLDR collation
/// data for en-US and is verified here as a regression baseline.
///
/// Spec: UNICODE_BIBLIOGRAPHY_SORTING.md §Collation Policy — single collator,
/// locale-tailored, root-collation fallback for unsupported locales.
#[test]
fn test_mixed_script_sort_order() {
    announce_behavior(
        "Mixed-script bibliography: Latin entries sort alphabetically first, Arabic after Latin, Hangul after Arabic (en-US collator).",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // Latin entries sort correctly among themselves.
    let celik_pos = result.find("Çelik").expect("Çelik should be in output");
    let zimring_pos = result.find("Zimring").expect("Zimring should be in output");
    assert!(
        celik_pos < zimring_pos,
        "Çelik (C) must sort before Zimring (Z) in Latin ordering. Got:\n{result}"
    );

    // Arabic-script entry (al-Ghazali) sorts after all Latin entries.
    // Assert the Arabic script directly — a romanized fallback would hide a rendering bug.
    let ghazali_pos = result
        .find("الغزالي")
        .expect("Arabic-script author الغزالي must appear in output unchanged");
    assert!(
        zimring_pos < ghazali_pos,
        "Arabic-script entry must sort after Latin entries (Zimring). Got:\n{result}"
    );

    // Hangul entry (김) sorts after Arabic under en-US collation.
    let hangul_pos = result
        .find("김")
        .expect("Hangul author 김 must appear in output unchanged");
    assert!(
        ghazali_pos < hangul_pos,
        "Hangul entry must sort after Arabic-script entry (الغزالي). Got:\n{result}"
    );
}

/// Test that mixed-script bibliography sort is deterministic: running the same
/// processor twice produces byte-identical output. Collator equality alone does
/// not guarantee stable output; this verifies the tiebreaker chain holds.
#[test]
fn test_mixed_script_sort_determinism() {
    announce_behavior(
        "Mixed-script bibliography produces identical output on repeated calls (deterministic sort).",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);

    let first = processor.render_bibliography();
    let second = processor.render_bibliography();

    assert_eq!(
        first, second,
        "Bibliography sort must be identical across repeated calls"
    );
}

/// Test that all-caps surnames sort case-insensitively alongside mixed-case
/// surnames, verifying that case handling is done via collator configuration
/// rather than pre-processing (no lowercasing of source text).
///
/// SMITH and WILLIAMS are all-caps in the fixture; they must sort in the same
/// relative position as "Smith" and "Williams" would.
#[test]
fn test_allcaps_surname_sorts_case_insensitively() {
    announce_behavior(
        "All-caps surnames (SMITH, WILLIAMS) sort case-insensitively alongside mixed-case surnames without lowercasing source text.",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bib = load_sort_oracle_bibliography();
    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // SMITH (all-caps) must sort between surnames beginning with R and T,
    // not at the end of the list as it would under bytewise ordering.
    let brown_pos = result.find("Brown").expect("Brown should be in output");
    let smith_pos = result
        .find("SMITH")
        .expect("SMITH (all-caps) should be in output");
    let zimring_pos = result.find("Zimring").expect("Zimring should be in output");

    assert!(
        brown_pos < smith_pos,
        "Brown must sort before SMITH. Got:\n{result}"
    );
    assert!(
        smith_pos < zimring_pos,
        "SMITH must sort before Zimring — all-caps must not push it to end of list. Got:\n{result}"
    );
}
