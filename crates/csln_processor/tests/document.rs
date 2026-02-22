/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use std::{fs, path::PathBuf};

use csln_core::{
    BibliographySpec, Locale, Style, StyleInfo,
    options::{BibliographyConfig, Config, Disambiguation, Processing, ProcessingCustom},
};
use csln_processor::{
    Processor,
    io::load_bibliography,
    processor::document::{DocumentFormat, djot::DjotParser},
};

#[test]
fn test_document_html_output_contains_heading() {
    // Create a simple style
    let style = Style {
        info: StyleInfo {
            title: Some("Test Style".to_string()),
            id: Some("test".to_string()),
            ..Default::default()
        },
        templates: None,
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            bibliography: Some(BibliographyConfig {
                entry_suffix: Some(".".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        citation: None,
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                csln_core::tc_contributor!(Author, Long),
                csln_core::tc_date!(Issued, Year),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create a bibliography with one reference
    let mut bibliography = indexmap::IndexMap::new();
    let kuhn = make_book(
        "kuhn1962",
        "Kuhn",
        "Thomas S.",
        1962,
        "The Structure of Scientific Revolutions",
    );
    bibliography.insert("kuhn1962".to_string(), kuhn);

    // Create processor
    let processor = Processor::new(style, bibliography);

    // Create a simple document with a citation
    let document = "This is a test document with a citation [@kuhn1962].\n\nMore text here.";

    // Process document as HTML
    let parser = DjotParser;
    let html_output = processor.process_document::<_, csln_processor::render::html::Html>(
        document,
        &parser,
        DocumentFormat::Html,
    );

    // Verify that the output contains HTML heading
    assert!(
        html_output.contains("<h1>Bibliography</h1>"),
        "Output should contain <h1>Bibliography</h1>"
    );

    // Verify that the citation was replaced
    assert!(
        html_output.contains("kuhn1962") || html_output.contains("Kuhn"),
        "Output should contain reference to kuhn1962 or Kuhn. Got: {}",
        html_output
    );

    // Verify document structure is preserved
    assert!(
        html_output.contains("test document with a citation"),
        "Output should contain original document text"
    );
}

#[test]
fn test_document_djot_output_unmodified() {
    // Create a simple style
    let style = Style {
        info: StyleInfo {
            title: Some("Test Style".to_string()),
            id: Some("test".to_string()),
            ..Default::default()
        },
        templates: None,
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            bibliography: Some(BibliographyConfig {
                entry_suffix: Some(".".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        citation: None,
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                csln_core::tc_contributor!(Author, Long),
                csln_core::tc_date!(Issued, Year),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create a bibliography
    let mut bibliography = indexmap::IndexMap::new();
    let ref1 = make_book("ref1", "Author", "Name", 2020, "Title");
    bibliography.insert("ref1".to_string(), ref1);

    let processor = Processor::new(style, bibliography);
    let document = "Document with citation [@ref1].";

    // Process as Djot format
    let parser = DjotParser;
    let djot_output = processor.process_document::<_, csln_processor::render::djot::Djot>(
        document,
        &parser,
        DocumentFormat::Djot,
    );

    // Verify it contains Djot markdown (not HTML)
    assert!(
        djot_output.contains("# Bibliography"),
        "Djot output should contain # Bibliography markdown"
    );

    // Should not contain HTML tags
    assert!(
        !djot_output.contains("<h1>"),
        "Djot output should not contain HTML tags"
    );
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_style(path: &str) -> Style {
    let style_path = project_root().join(path);
    let bytes = fs::read(&style_path).expect("style fixture should be readable");
    serde_yaml::from_slice(&bytes).expect("style fixture should parse")
}

#[test]
fn test_process_document_renders_chicago_primary_secondary_groups() {
    let style = load_style("styles/chicago-author-date.yaml");
    let bibliography =
        load_bibliography(&project_root().join("tests/fixtures/grouping/primary-secondary.json"))
            .expect("grouping fixture should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let output = processor.process_document::<_, csln_processor::render::plain::PlainText>(
        "Grouping check [@interview-1978; @ms-archive-1901; @journal-2021].",
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.contains("# Primary Sources"),
        "missing primary heading: {output}"
    );
    assert!(
        output.contains("# Secondary Sources"),
        "missing secondary heading: {output}"
    );
    assert!(
        output.contains("Field Notes from the Delta Survey"),
        "missing primary-source entry: {output}"
    );
    assert!(
        output.contains("Trade Networks in the Early Modern Atlantic"),
        "missing secondary-source entry: {output}"
    );
}

#[test]
fn test_process_document_restarts_year_suffixes_per_group() {
    let mut style = load_style("styles/experimental/multilingual-academic.yaml");
    style
        .options
        .get_or_insert_with(Default::default)
        .processing = Some(Processing::Custom(ProcessingCustom {
        disambiguate: Some(Disambiguation {
            year_suffix: true,
            ..Default::default()
        }),
        ..Default::default()
    }));

    let bibliography =
        load_bibliography(&project_root().join("tests/fixtures/grouping/multilingual-groups.json"))
            .expect("multilingual grouping fixture should parse");
    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let output = processor.process_document::<_, csln_processor::render::plain::PlainText>(
        "Disambiguation check [@vi-kuhn-a; @vi-kuhn-b; @en-kuhn-a; @en-kuhn-b].",
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.contains("# Vietnamese Sources"),
        "missing vietnamese heading: {output}"
    );
    assert!(
        output.contains("# Western Sources"),
        "missing western heading: {output}"
    );

    // With per-group local disambiguation, each group should restart at 2020a.
    // Count only bibliography output because in-text citations can include extra suffixes.
    let bibliography_only = output
        .split("# Bibliography")
        .nth(1)
        .unwrap_or_default()
        .to_string();
    let count_2020a = bibliography_only.matches("2020a").count();
    assert_eq!(count_2020a, 2, "expected 2020a in both groups: {output}");
}

#[test]
fn test_process_document_renders_jm_legal_group_hierarchy() {
    let style = load_style("styles/experimental/jm-chicago-legal.yaml");
    let bibliography =
        load_bibliography(&project_root().join("tests/fixtures/grouping/legal-hierarchy.json"))
            .expect("legal hierarchy fixture should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let output = processor.process_document::<_, csln_processor::render::plain::PlainText>(
        "Legal grouping [@brown1954; @civilrights1964; @versailles1919; @hart1994].",
        &parser,
        DocumentFormat::Plain,
    );

    let cases = output
        .find("# Cases")
        .expect("missing cases heading in grouped bibliography");
    let statutes = output
        .find("# Statutes")
        .expect("missing statutes heading in grouped bibliography");
    let treaties = output
        .find("# Treaties and International Agreements")
        .expect("missing treaties heading in grouped bibliography");
    let secondary = output
        .find("# Secondary Sources")
        .expect("missing secondary heading in grouped bibliography");

    assert!(cases < statutes, "expected Cases before Statutes: {output}");
    assert!(
        statutes < treaties,
        "expected Statutes before Treaties: {output}"
    );
    assert!(
        treaties < secondary,
        "expected Treaties before Secondary: {output}"
    );
}

#[test]
fn test_process_document_group_heading_localization_falls_back_to_language_tag() {
    let style = load_style("styles/chicago-author-date.yaml");
    let bibliography =
        load_bibliography(&project_root().join("tests/fixtures/grouping/primary-secondary.json"))
            .expect("grouping fixture should parse");

    let mut locale = Locale::en_us();
    locale.locale = "en-GB".to_string();

    let processor = Processor::with_locale(style, bibliography, locale);
    let parser = DjotParser;
    let output = processor.process_document::<_, csln_processor::render::plain::PlainText>(
        "Locale fallback check [@interview-1978; @journal-2021].",
        &parser,
        DocumentFormat::Plain,
    );

    // chicago-author-date headings are localized with en-US + en.
    // en-GB should fall back to the language tag (en).
    assert!(
        output.contains("# Primary Sources"),
        "missing primary heading: {output}"
    );
    assert!(
        output.contains("# Secondary Sources"),
        "missing secondary heading: {output}"
    );
}
