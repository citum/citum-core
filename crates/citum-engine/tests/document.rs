/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use std::{fs, path::PathBuf};

use citum_engine::{
    Processor,
    io::load_bibliography,
    processor::document::{DocumentFormat, djot::DjotParser, markdown::MarkdownParser},
};
use citum_schema::{
    BibliographySpec, Locale, Style, StyleInfo,
    options::{BibliographyConfig, Config, Disambiguation, Processing, ProcessingCustom},
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
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_date!(Issued, Year),
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
    let html_output = processor.process_document::<_, citum_engine::render::html::Html>(
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
                citum_schema::tc_contributor!(Author, Long),
                citum_schema::tc_date!(Issued, Year),
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
    let djot_output = processor.process_document::<_, citum_engine::render::djot::Djot>(
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

#[test]
fn test_document_html_output_renders_raw_markup() {
    let style = load_style("styles/modern-language-association.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document.djot"))
        .expect("example document should be readable");

    let html_output = processor.process_document::<_, citum_engine::render::html::Html>(
        &document,
        &parser,
        DocumentFormat::Html,
    );

    assert!(
        html_output.contains(r#"<span class="csln-citation" data-ref="smith2010">"#),
        "citation markup should be real HTML: {html_output}"
    );
    assert!(
        html_output.contains(r#"<div class="csln-bibliography">"#),
        "bibliography markup should be real HTML: {html_output}"
    );
    assert!(
        !html_output.contains("&lt;span class="),
        "citation markup should not be escaped: {html_output}"
    );
    assert!(
        !html_output.contains("&lt;div class="),
        "bibliography markup should not be escaped: {html_output}"
    );
}

#[test]
fn test_example_document_renders_integral_name_memory() {
    let style = load_style("styles/modern-language-association.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document.djot"))
        .expect("example document should be readable");

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        &document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(output.contains("First narrative mention: John Smith (10)"));
    assert!(output.contains("Later in the same chapter: Smith (12) narrows"));
    assert!(output.contains("Integral with locator: Thomas S. Kuhn (10) argues"));
    assert!(output.contains(
        "[^narrative-note]: Before the prose introduces him, John Smith (3) already appears in a note."
    ));
    assert!(output.contains("Suppress author with locator: (10)."));
    assert!(output.contains("# Chapter Two"));
    assert!(output.contains("so John Smith (14)"));
}

#[test]
fn test_example_document_renders_author_date_integral_citations() {
    let style = load_style("styles/apa-7th.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document.djot"))
        .expect("example document should be readable");

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        &document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(output.contains("First narrative mention: Smith (2010, p. 10)"));
    assert!(output.contains("Later in the same chapter: Smith (2010, p. 12)"));
    assert!(output.contains("Integral with locator: Kuhn (1962, p. 10) argues"));
    assert!(output.contains(
        "[^narrative-note]: Before the prose introduces him, Smith (2010, p. 3) already appears in a note."
    ));
    assert!(output.contains("Suppress author with locator: (1962, p. 10)."));
}

#[test]
fn test_example_document_renders_note_style_integral_anchor_and_notes() {
    let style = load_style("styles/chicago-shortened-notes-bibliography.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document.djot"))
        .expect("example document should be readable");

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        &document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(output.contains("First narrative mention: Smith[^citum-auto-5] surveys"));
    assert!(output.contains("Later in the same chapter: Smith[^citum-auto-6] narrows"));
    assert!(output.contains("Integral with locator: Kuhn[^citum-auto-7] argues"));
    assert!(
        output.contains("[^narrative-note]: Before the prose introduces him, Smith ("),
        "manual note should preserve the authored anchor: {output}"
    );
    assert!(
        output.contains("_A Great Book_, 3"),
        "manual note should preserve the reduced note content and locator: {output}"
    );
    assert_eq!(
        output.matches("[^narrative-note]:").count(),
        1,
        "manual note should not be duplicated: {output}"
    );
    assert!(
        !output.contains("[+@smith2010]"),
        "raw citation leaked: {output}"
    );
}

#[test]
fn test_chicago_notes_document_renders_ibid_without_author_concatenation() {
    let style = load_style("styles/chicago-notes.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document-citation-flow.djot"))
        .expect("citation flow example should be readable");

    let output = processor.process_document::<_, citum_engine::render::djot::Djot>(
        &document,
        &parser,
        DocumentFormat::Djot,
    );

    assert!(
        output.to_lowercase().contains("ibid"),
        "note style should render ibid in this scenario: {output}"
    );
    assert!(
        !output.contains("KuhnIbid"),
        "ibid should not concatenate with author token: {output}"
    );
    assert!(
        !output.contains("SmithIbid"),
        "ibid should not concatenate with author token: {output}"
    );
    assert!(
        output.contains("Brown ("),
        "integral ibid should preserve the authored anchor: {output}"
    );
    assert!(
        output.to_lowercase().contains("ibid"),
        "integral ibid should remain reduced in authored notes: {output}"
    );
    assert!(
        !output.to_lowercase().contains("brown ibid"),
        "integral ibid should not concatenate the anchor and reduced form: {output}"
    );
    assert!(
        !output.to_lowercase().contains("ibid also argues that..."),
        "integral ibid should not replace the narrative anchor in authored notes: {output}"
    );
}

#[test]
fn test_chicago_notes_document_integral_ibid_with_locator_keeps_anchor_and_locator() {
    let style = load_style("styles/chicago-notes.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = concat!(
        "Text.[^n1]\n\n",
        "[^n1]:\n\n",
        "  - [+@brown1954, p. 10] argues that...\n",
        "  - [+@brown1954, p. 12] also argues that...\n",
    );

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.contains("Brown ("),
        "integral ibid-with-locator should preserve the authored anchor: {output}"
    );
    assert!(
        output.to_lowercase().contains("ibid"),
        "integral ibid-with-locator should remain reduced: {output}"
    );
    assert!(
        output.contains("12"),
        "integral ibid-with-locator should preserve the locator: {output}"
    );
}

#[test]
fn test_chicago_notes_document_integral_ibid_uses_locale_term_without_period() {
    let mut style = load_style("styles/chicago-notes.yaml");
    if let Some(citation) = style.citation.as_mut() {
        citation.suffix = Some(".".to_string());
        citation.ibid = None;
    }

    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let mut locale = Locale::en_us();
    locale.terms.ibid = Some("ibid".to_string());

    let processor = Processor::with_locale(style, bibliography, locale);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document-citation-flow.djot"))
        .expect("citation flow example should be readable");

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        &document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.contains("Brown (ibid) also argues that..."),
        "ibid term should come from locale data and preserve locale punctuation: {output}"
    );
    assert!(
        !output.contains("Brown (.) also argues that..."),
        "ibid fallback must not use base citation suffix: {output}"
    );
}

#[test]
fn test_chicago_notes_document_integral_ibid_style_suffix_overrides_locale_term() {
    let mut style = load_style("styles/chicago-notes.yaml");
    if let Some(citation) = style.citation.as_mut()
        && let Some(ibid) = citation.ibid.as_mut()
    {
        ibid.suffix = Some("IBIDX".to_string());
    }

    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let mut locale = Locale::en_us();
    locale.terms.ibid = Some("ibid".to_string());

    let processor = Processor::with_locale(style, bibliography, locale);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document-citation-flow.djot"))
        .expect("citation flow example should be readable");

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        &document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.contains("Brown (IBIDX) also argues that..."),
        "explicit style ibid suffix should override locale ibid term: {output}"
    );
}

#[test]
fn test_chicago_notes_document_integral_ibid_anchor_failure_falls_back_to_reduced_only() {
    let style = load_style("styles/chicago-notes.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = concat!(
        "Text.[^n1]\n\n",
        "[^n1]:\n\n",
        "  - [+@missingref, p. 10] argues that...\n",
        "  - [+@missingref, p. 12] also argues that...\n",
    );

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.to_lowercase().contains("ibid"),
        "when anchor cannot render, integral ibid should still render reduced citation text: {output}"
    );
    assert!(
        !output.contains("missingrefIbid"),
        "anchor failure path must not concatenate fallback and ibid text: {output}"
    );
}

#[test]
fn test_chicago_notes_document_omits_empty_bibliography_heading() {
    let style = load_style("styles/chicago-notes.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document-citation-flow.djot"))
        .expect("citation flow example should be readable");

    let output = processor.process_document::<_, citum_engine::render::djot::Djot>(
        &document,
        &parser,
        DocumentFormat::Djot,
    );

    assert!(
        !output.contains("# Bibliography"),
        "empty bibliography should not emit heading: {output}"
    );

    let html_output = processor.process_document::<_, citum_engine::render::html::Html>(
        &document,
        &parser,
        DocumentFormat::Html,
    );
    assert!(
        !html_output.contains("<h1>Bibliography</h1>"),
        "empty bibliography should not emit HTML heading: {html_output}"
    );
}

#[test]
fn test_document_citation_flow_non_note_styles_do_not_render_ibid() {
    let parser = DjotParser;
    let document = fs::read_to_string(project_root().join("examples/document-citation-flow.djot"))
        .expect("citation flow example should be readable");

    for style_path in [
        "styles/apa-7th.yaml",
        "styles/ieee.yaml",
        "styles/alpha.yaml",
    ] {
        let style = load_style(style_path);
        let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
            .expect("example bibliography should parse");
        let processor = Processor::new(style, bibliography);

        let output = processor.process_document::<_, citum_engine::render::djot::Djot>(
            &document,
            &parser,
            DocumentFormat::Djot,
        );
        assert!(
            !output.contains("Ibid"),
            "non-note style unexpectedly rendered ibid for {style_path}: {output}"
        );
    }
}

#[test]
fn test_markdown_document_renders_pandoc_author_date_citations() {
    let style = load_style("styles/apa-7th.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = MarkdownParser;
    let document = concat!(
        "Kuhn argued that @kuhn1962 [p. 10] changed science.\n\n",
        "Later work supports this [see @smith2010, p. 12; @kuhn1962, ch. 3].",
    );

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.contains("Kuhn (1962, p. 10) changed science."),
        "integral markdown citation did not render: {output}"
    );
    assert!(
        output.contains("Later work supports this ("),
        "bracketed markdown cite cluster did not render: {output}"
    );
    assert!(
        output.contains("Kuhn, 1962, ch. 3"),
        "markdown locator cite missing from cluster: {output}"
    );
    assert!(
        output.contains("see Smith, 2010, p. 12"),
        "markdown prefix cite missing from cluster: {output}"
    );
}

#[test]
fn test_markdown_document_generates_notes_for_note_styles() {
    let style = load_style("styles/chicago-shortened-notes-bibliography.yaml");
    let bibliography = load_bibliography(&project_root().join("examples/document-refs.json"))
        .expect("example bibliography should parse");

    let processor = Processor::new(style, bibliography);
    let parser = MarkdownParser;
    let document = "Narrative mention @smith2010 introduces the argument.";

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(
        output.contains("Narrative mention Smith[^citum-auto-1] introduces the argument."),
        "note-style markdown integral citation did not anchor correctly: {output}"
    );
    assert!(
        output.contains("[^citum-auto-1]: Smith, _A Great Book_."),
        "generated note missing for markdown citation: {output}"
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
    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
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
    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
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
    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
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
    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
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
