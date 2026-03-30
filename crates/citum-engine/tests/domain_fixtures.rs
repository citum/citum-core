#![allow(missing_docs, reason = "test")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use citum_engine::Processor;
use citum_engine::io::load_bibliography;
use citum_schema::Style;
use citum_schema::citation::{Citation, CitationItem, CitationLocator, LocatorType};
use citum_schema::reference::{InputReference, MultilingualString};
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

fn single_item_citation_with_locator(id: &str, locator: &str) -> Citation {
    Citation {
        items: vec![CitationItem {
            id: id.to_string(),
            locator: Some(CitationLocator::single(LocatorType::Page, locator)),
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
            && civil.starts_with('(')
            && civil.ends_with(')'),
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

#[test]
fn test_humanities_note_fixture_preserves_archive_and_interview_fields() {
    let root = project_root();
    let style = load_style(&root.join("styles/chicago-notes.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/references-humanities-note.json"))
            .expect("humanities-note fixture should parse");
    let manuscript_ref = bibliography
        .get("dead-sea-scrolls")
        .cloned()
        .expect("manuscript fixture should exist");

    let processor = Processor::new(style, bibliography);
    let manuscript = processor
        .process_citation(&single_item_citation("dead-sea-scrolls"))
        .expect("manuscript citation should render");
    let interview = processor
        .process_citation(&single_item_citation_with_locator(
            "foucault-interview",
            "115",
        ))
        .expect("interview citation should render");
    let letter = processor
        .process_citation(&single_item_citation("derrida-letter"))
        .expect("personal communication citation should render");

    let InputReference::Monograph(manuscript_record) = manuscript_ref else {
        panic!("dead-sea-scrolls should deserialize as a monograph");
    };
    let archive_info = manuscript_record
        .archive_info
        .as_ref()
        .expect("manuscript fixture should preserve structured archive info");

    assert!(
        matches!(
            archive_info.name.as_ref(),
            Some(MultilingualString::Simple(name)) if name == "Israel Antiquities Authority"
        ) && archive_info.location.as_deref() == Some("Shrine of the Book")
            && archive_info.place.as_deref() == Some("Jerusalem"),
        "manuscript fixture should preserve structured archive name, location, and place"
    );
    assert!(
        manuscript.contains("The Community Rule (1QS)")
            && manuscript.contains("Manuscript scroll")
            && manuscript.contains("101 BC"),
        "manuscript citation should continue rendering the manuscript reference"
    );
    assert!(
        interview.contains("interview by Alessandro Fontana")
            && interview.contains("Power/Knowledge: Selected Interviews and Other Writings")
            && interview.contains("115"),
        "interview citation should include interviewer, container title, and locator"
    );
    assert!(
        letter.contains("to Paul de Man")
            && letter.contains("University of California, Irvine, Critical Theory Archive"),
        "personal communication citation should include recipient and archive"
    );
}

#[test]
fn test_taylor_and_francis_author_date_wrapper_preserves_prefixed_multi_cites() {
    let root = project_root();
    let style = load_style(&root.join("styles/taylor-and-francis-chicago-author-date.yaml"));
    let bibliography = load_bibliography(&root.join("tests/fixtures/references-expanded.json"))
        .expect("expanded fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = Citation {
        items: vec![
            CitationItem {
                id: "ITEM-1".to_string(),
                locator: Some(CitationLocator::single(LocatorType::Page, "44")),
                ..Default::default()
            },
            CitationItem {
                id: "ITEM-3".to_string(),
                prefix: Some("cf. ".to_string()),
                locator: Some(CitationLocator::single(LocatorType::Page, "437")),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let rendered = processor
        .process_citation(&citation)
        .expect("prefixed multi-cite should render");

    assert!(
        rendered.contains("cf. LeCun, Bengio, and Hinton 2015")
            && !rendered.contains("cf. LeCun et al"),
        "prefixed multi-cites should retain the full three-author form: {rendered}"
    );
}

#[test]
fn test_taylor_and_francis_author_date_wrapper_preserves_media_and_translation_details() {
    let root = project_root();
    let style = load_style(&root.join("styles/taylor-and-francis-chicago-author-date.yaml"));
    let bibliography = load_bibliography(&root.join("tests/fixtures/references-expanded.json"))
        .expect("expanded fixture should parse");

    let processor = Processor::new(style, bibliography);
    let rendered_bib = processor.render_bibliography();

    assert!(
        rendered_bib.contains("The Arrival of a Train at La Ciotat Station")
            && rendered_bib.contains("Short film")
            && rendered_bib.contains("Directed by Louis Lumière"),
        "motion pictures should retain genre and director detail"
    );
    assert!(
        rendered_bib.contains("The Future of Artificial Intelligence")
            && rendered_bib.contains("Interview by Stephen Colbert")
            && rendered_bib.contains("Video interview")
            && rendered_bib.contains("https://example.com/interview"),
        "interviews should retain interviewer, genre, and url detail"
    );
    assert!(
        rendered_bib.contains("Metamorphosis")
            && rendered_bib.contains("Translated by David Wyllie")
            && rendered_bib.contains("Kurt Wolff Verlag"),
        "translated books should retain translator detail"
    );
}
