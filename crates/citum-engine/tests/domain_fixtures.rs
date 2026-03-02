/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

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

fn single_item_citation(id: &str) -> Citation {
    Citation {
        items: vec![CitationItem {
            id: id.to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tests legal citation fixture rendering with APA style.
///
/// Verifies that legal references (court cases, legislation, treaties) render
/// correctly with proper case names, dates, and court/statute identification.
#[test]
fn test_legal_fixture_is_covered_in_processor_tests() {
    let root = project_root();
    let style = load_style(&root.join("styles/apa-7th.yaml"));
    let bibliography = load_bibliography(&root.join("tests/fixtures/references-legal.json"))
        .expect("legal fixture should parse");

    let processor = Processor::new(style, bibliography);
    let brown = processor
        .process_citation(&single_item_citation("brown1954"))
        .expect("brown citation should render");
    let civil = processor
        .process_citation(&single_item_citation("civilrights1964"))
        .expect("civil rights citation should render");
    let treaty = processor
        .process_citation(&single_item_citation("versailles1919"))
        .expect("treaty citation should render");
    let rendered_bib = processor.render_bibliography();

    // Verify Brown v. Board of Education case is rendered correctly
    assert_eq!(
        brown, "(_Brown v. Board of Education_, 1954)",
        "Brown case citation should have case name and year"
    );
    // Verify Civil Rights Act includes title and year
    assert!(
        civil.contains("Civil Rights Act of 1964")
            && civil.starts_with("(")
            && civil.ends_with(")"),
        "Civil Rights Act citation should include act name within parentheses"
    );
    // Verify Treaty has parties and date
    assert!(
        treaty.contains("Treaty of Versailles") && treaty.contains("1919"),
        "Treaty citation should include treaty name and date"
    );
    // Verify bibliography includes court information
    assert!(
        rendered_bib
            .contains("Brown v. Board of Education, 347 U.S. 483 (U.S. Supreme Court 1954)"),
        "Bibliography should include full Brown case citation with court"
    );
}

/// Tests scientific citation fixture rendering with APA style.
///
/// Verifies that specialized scientific references (patents, datasets, standards,
/// software) render correctly with proper authors/inventors and dates.
#[test]
fn test_scientific_fixture_is_covered_in_processor_tests() {
    let root = project_root();
    let style = load_style(&root.join("styles/apa-7th.yaml"));
    let bibliography = load_bibliography(&root.join("tests/fixtures/references-scientific.json"))
        .expect("scientific fixture should parse");

    let processor = Processor::new(style, bibliography);
    let patent = processor
        .process_citation(&single_item_citation("pavlovic2008"))
        .expect("patent citation should render");
    let dataset = processor
        .process_citation(&single_item_citation("irino2009"))
        .expect("dataset citation should render");
    let standard = processor
        .process_citation(&single_item_citation("ieee754-2008"))
        .expect("standard citation should render");
    let software = processor
        .process_citation(&single_item_citation("rcore2021"))
        .expect("software citation should render");
    let rendered_bib = processor.render_bibliography();

    // Verify patent includes inventor name and year
    assert!(
        patent.contains("Pavlovic") && patent.contains("2008"),
        "Patent citation should include inventor name and year"
    );
    // Verify dataset includes creator and year
    assert!(
        dataset.contains("Irino") && dataset.contains("2009"),
        "Dataset citation should include creator name and year"
    );
    // Verify standard includes standard name and year
    assert!(
        standard.contains("IEEE") && standard.contains("2008"),
        "Standard citation should include standards body and year"
    );
    // Verify software includes team/author and year
    assert!(
        software.contains("Core Team") && software.contains("2021"),
        "Software citation should include team name and year"
    );
    // Verify bibliography includes resource type labels
    assert!(
        rendered_bib.contains("[Dataset]"),
        "Bibliography should label dataset entries"
    );
    assert!(
        rendered_bib.contains("Patent"),
        "Bibliography should include patent information"
    );
}

/// Tests multilingual citation fixture rendering with APA style.
///
/// Verifies that references with multilingual names and content (Vietnamese, English, etc.)
/// render correctly with proper diacritics, names, and translated fields.
#[test]
fn test_multilingual_fixture_is_covered_in_processor_tests() {
    let root = project_root();
    let style = load_style(&root.join("styles/apa-7th.yaml"));
    let bibliography = load_bibliography(&root.join("tests/fixtures/references-multilingual.yaml"))
        .expect("multilingual fixture should parse");

    let processor = Processor::new(style, bibliography);
    let rendered_bib = processor.render_bibliography();

    // Verify Vietnamese names with diacritics are preserved
    assert!(
        rendered_bib.contains("Nguyễn"),
        "Bibliography should render Vietnamese names with diacritics"
    );
    assert!(
        rendered_bib.contains("Trần"),
        "Bibliography should include other Vietnamese names"
    );
    // Verify multilingual content is included
    assert!(
        rendered_bib.contains("Nhà xuất bản"),
        "Bibliography should include Vietnamese publisher names"
    );
    // Verify English-language references are also included
    assert!(
        rendered_bib.contains("Oxford University Press"),
        "Bibliography should include English publisher names"
    );
}
