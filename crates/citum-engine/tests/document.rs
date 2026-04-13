#![allow(missing_docs, reason = "test")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::{
    Processor,
    io::load_bibliography,
    processor::document::{
        CitationParser, DocumentFormat, djot::DjotParser, markdown::MarkdownParser,
    },
};
use citum_schema::{
    BibliographySpec, Locale, Style, StyleInfo,
    options::{
        BibliographyOptions, Config, Disambiguation, LocatorPreset, Processing, ProcessingCustom,
    },
};

// --- Document Rendering Scenarios ---

fn given_simple_author_date_document_when_rendered_as_html_then_a_bibliography_heading_is_appended()
{
    // Create a simple style
    let style = Style {
        info: StyleInfo {
            title: Some("Test Style".to_string()),
            id: Some("test".into()),
            ..Default::default()
        },
        templates: None,
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            ..Default::default()
        }),
        citation: None,
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
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
        "Output should contain reference to kuhn1962 or Kuhn. Got: {html_output}"
    );

    // Verify document structure is preserved
    assert!(
        html_output.contains("test document with a citation"),
        "Output should contain original document text"
    );
}

fn given_simple_author_date_document_when_rendered_as_djot_then_html_tags_are_not_emitted() {
    // Create a simple style
    let style = Style {
        info: StyleInfo {
            title: Some("Test Style".to_string()),
            id: Some("test".into()),
            ..Default::default()
        },
        templates: None,
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            ..Default::default()
        }),
        citation: None,
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
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

fn given_example_mla_document_when_rendered_as_html_then_citation_markup_is_not_escaped() {
    let processor = example_document_processor("styles/embedded/modern-language-association.yaml");
    let parser = DjotParser;
    let document = load_example_document("examples/document.djot");

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

fn given_example_mla_document_when_rendered_as_plain_text_then_integral_name_memory_is_visible() {
    let processor = example_document_processor("styles/embedded/modern-language-association.yaml");
    let parser = DjotParser;
    let document = load_example_document("examples/document.djot");

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        &document,
        &parser,
        DocumentFormat::Plain,
    );

    assert!(output.contains("First narrative mention: Anthony D. Smith (10)"));
    assert!(output.contains("Later in the same chapter: Smith (12) narrows"));
    assert!(output.contains("Integral with locator: Thomas S. Kuhn (10) argues"));
    assert!(output.contains(
        "[^narrative-note]: Before the prose introduces him, Anthony D. Smith (3) already appears in a note."
    ));
    assert!(output.contains("Suppress author with locator: (10)."));
    assert!(output.contains("# Chapter Two"));
    assert!(output.contains("so Anthony D. Smith (14)"));
}

fn given_example_apa_document_when_rendered_as_plain_text_then_integral_citations_include_locators()
{
    let processor = example_document_processor("styles/embedded/apa-7th.yaml");
    let parser = DjotParser;
    let document = load_example_document("examples/document.djot");

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

fn given_example_chicago_note_document_when_rendered_as_plain_text_then_integral_mentions_keep_their_note_anchor()
 {
    let processor =
        example_document_processor("styles/embedded/chicago-shortened-notes-bibliography.yaml");
    let parser = DjotParser;
    let document = load_example_document("examples/document.djot");

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
        output.contains("_Nationalism: Theory, Ideology, History_, 3"),
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

// --- Note Flow Scenarios ---

fn given_chicago_note_flow_document_when_ibid_is_rendered_then_it_does_not_concatenate_with_the_narrative_anchor()
 {
    let processor = example_document_processor("styles/embedded/chicago-notes-18th.yaml");
    let parser = DjotParser;
    let document = load_example_document("examples/document-citation-flow.djot");

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

fn given_chicago_note_locator_repeat_when_integral_ibid_is_rendered_then_anchor_and_locator_are_preserved()
 {
    let processor = example_document_processor("styles/embedded/chicago-notes-18th.yaml");
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
        output.contains("Brown (ibid., 12)"),
        "integral ibid-with-locator should preserve the reduced locator fragment: {output}"
    );
}

fn given_page_labels_are_configured_when_integral_ibid_is_rendered_then_the_labeled_page_locator_is_preserved()
 {
    let mut style = load_style("styles/embedded/chicago-notes-18th.yaml").into_resolved();
    style.options.get_or_insert_with(Default::default).locators =
        Some(LocatorPreset::AuthorDate.config());
    let processor = Processor::new(style, load_example_bibliography());
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
        output.contains("p. 12"),
        "configured page labels should be preserved in reduced manual notes: {output}"
    );
}

fn given_chapter_locator_repeat_when_integral_ibid_is_rendered_then_the_labeled_chapter_locator_is_preserved()
 {
    let processor = example_document_processor("styles/embedded/chicago-notes-18th.yaml");
    let parser = DjotParser;
    let document = concat!(
        "Text.[^n1]\n\n",
        "[^n1]:\n\n",
        "  - [+@brown1954, chap. 2] argues that...\n",
        "  - [+@brown1954, chap. 3] also argues that...\n",
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
        output.contains("ch. 3"),
        "chapter locators should retain their localized label in reduced manual notes: {output}"
    );
}

fn given_locale_specific_ibid_term_when_the_style_has_no_ibid_override_then_the_locale_term_is_used_without_base_suffix_punctuation()
 {
    let mut style = load_style("styles/embedded/chicago-notes-18th.yaml").into_resolved();
    if let Some(citation) = style.citation.as_mut() {
        citation.suffix = Some(".".to_string());
        citation.ibid = None;
    }

    let mut locale = Locale::en_us();
    locale.terms.ibid = Some("ibid".to_string());

    let processor = Processor::with_locale(style, load_example_bibliography(), locale);
    let parser = DjotParser;
    let document = load_example_document("examples/document-citation-flow.djot");

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

fn given_explicit_style_ibid_suffix_when_locale_also_defines_ibid_then_the_style_suffix_wins() {
    let mut style = load_style("styles/embedded/chicago-notes-18th.yaml").into_resolved();
    if let Some(citation) = style.citation.as_mut()
        && let Some(ibid) = citation.ibid.as_mut()
    {
        ibid.suffix = Some("IBIDX".to_string());
    }

    let mut locale = Locale::en_us();
    locale.terms.ibid = Some("ibid".to_string());

    let processor = Processor::with_locale(style, load_example_bibliography(), locale);
    let parser = DjotParser;
    let document = load_example_document("examples/document-citation-flow.djot");

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

fn given_missing_note_anchor_when_integral_ibid_is_rendered_then_the_reduced_citation_still_appears_without_concatenation()
 {
    let processor = example_document_processor("styles/embedded/chicago-notes-18th.yaml");
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

fn given_chicago_note_flow_document_when_no_bibliography_entries_are_needed_then_no_heading_is_emitted()
 {
    let processor = example_document_processor("styles/embedded/chicago-notes-18th.yaml");
    let parser = DjotParser;
    let document = load_example_document("examples/document-citation-flow.djot");

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

fn given_non_note_styles_when_rendering_the_note_flow_example_then_ibid_is_never_emitted() {
    let parser = DjotParser;
    let document = load_example_document("examples/document-citation-flow.djot");

    for style_path in [
        "styles/embedded/apa-7th.yaml",
        "styles/embedded/ieee.yaml",
        "styles/alpha.yaml",
    ] {
        let style = load_style(style_path);
        let processor = Processor::new(style, load_example_bibliography());

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

fn given_pandoc_markdown_author_date_syntax_when_rendered_then_integral_and_cluster_citations_are_preserved()
 {
    let processor = example_document_processor("styles/embedded/apa-7th.yaml");
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

fn given_markdown_integral_note_citation_when_rendered_with_a_note_style_then_a_generated_note_is_emitted()
 {
    let processor =
        example_document_processor("styles/embedded/chicago-shortened-notes-bibliography.yaml");
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
        output.contains("[^citum-auto-1]: Smith, _Nationalism: Theory, Ideology, History_."),
        "generated note missing for markdown citation: {output}"
    );
}

// --- Grouped Bibliography Scenarios ---

fn given_grouped_primary_and_secondary_sources_when_rendered_then_both_group_headings_and_entries_appear()
 {
    let style = load_style("styles/embedded/chicago-author-date-18th.yaml");
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

fn given_group_local_disambiguation_when_rendering_multilingual_groups_then_year_suffixes_restart_within_each_group()
 {
    let mut style = load_style("styles/experimental/multilingual-academic.yaml");
    style
        .options
        .get_or_insert_with(Default::default)
        .processing = Some(Processing::Custom(ProcessingCustom {
        disambiguate: Some(Disambiguation {
            year_suffix: false,
            ..Default::default()
        }),
        ..Default::default()
    }));
    style
        .bibliography
        .get_or_insert_with(Default::default)
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

fn given_juris_m_legal_grouping_when_rendered_then_headings_follow_the_expected_legal_hierarchy() {
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

fn given_an_english_locale_variant_when_group_headings_are_localized_then_the_language_tag_fallback_is_used()
 {
    let style = load_style("styles/embedded/chicago-author-date-18th.yaml");
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

mod rendering_formats {
    use super::announce_behavior;

    #[test]
    fn simple_author_date_html_appends_a_bibliography_heading() {
        announce_behavior(
            "Rendering a simple author-date document as HTML should append a Bibliography heading and preserve the prose.",
        );
        super::given_simple_author_date_document_when_rendered_as_html_then_a_bibliography_heading_is_appended();
    }

    #[test]
    fn simple_author_date_djot_does_not_emit_html_tags() {
        announce_behavior(
            "Rendering the same document as Djot should produce markdown headings rather than HTML tags.",
        );
        super::given_simple_author_date_document_when_rendered_as_djot_then_html_tags_are_not_emitted();
    }
}

mod example_documents {
    use super::announce_behavior;

    #[test]
    fn mla_html_keeps_citation_markup_unescaped() {
        announce_behavior(
            "The MLA example document should emit real citation and bibliography HTML instead of escaped markup.",
        );
        super::given_example_mla_document_when_rendered_as_html_then_citation_markup_is_not_escaped(
        );
    }

    #[test]
    fn mla_plain_text_shows_integral_name_memory() {
        announce_behavior(
            "The MLA plain-text example should shorten repeated narrative citations after the first integral mention.",
        );
        super::given_example_mla_document_when_rendered_as_plain_text_then_integral_name_memory_is_visible();
    }

    #[test]
    fn apa_plain_text_integral_citations_keep_locators() {
        announce_behavior(
            "The APA plain-text example should keep locators inside integral citations throughout the document.",
        );
        super::given_example_apa_document_when_rendered_as_plain_text_then_integral_citations_include_locators();
    }

    #[test]
    fn chicago_note_plain_text_keeps_integral_note_anchors() {
        announce_behavior(
            "The Chicago note example should preserve narrative note anchors and keep manual-note content intact.",
        );
        super::given_example_chicago_note_document_when_rendered_as_plain_text_then_integral_mentions_keep_their_note_anchor();
    }
}

mod note_flow {
    use super::announce_behavior;

    #[test]
    fn chicago_note_flow_does_not_concatenate_ibid_with_the_narrative_anchor() {
        announce_behavior(
            "A Chicago note-flow narrative mention should not concatenate the generated ibid text onto the prose anchor.",
        );
        super::given_chicago_note_flow_document_when_ibid_is_rendered_then_it_does_not_concatenate_with_the_narrative_anchor();
    }

    #[test]
    fn locator_repeats_keep_the_anchor_and_locator() {
        announce_behavior(
            "A repeated note citation with a locator should keep both the narrative anchor and the locator.",
        );
        super::given_chicago_note_locator_repeat_when_integral_ibid_is_rendered_then_anchor_and_locator_are_preserved();
    }

    #[test]
    fn configured_page_labels_are_preserved_in_manual_note_ibid_with_locator() {
        announce_behavior(
            "If locator rendering is configured to show page labels, a repeated manual note should keep the labeled page locator in the reduced ibid text.",
        );
        super::given_page_labels_are_configured_when_integral_ibid_is_rendered_then_the_labeled_page_locator_is_preserved();
    }

    #[test]
    fn chapter_locator_repeats_keep_the_labeled_locator() {
        announce_behavior(
            "A repeated manual note with a chapter locator should keep the chapter label in the reduced ibid text.",
        );
        super::given_chapter_locator_repeat_when_integral_ibid_is_rendered_then_the_labeled_chapter_locator_is_preserved();
    }

    #[test]
    fn locale_ibid_term_is_used_when_the_style_has_no_override() {
        announce_behavior(
            "If a note style does not override ibid, the localized term should be used without extra base punctuation.",
        );
        super::given_locale_specific_ibid_term_when_the_style_has_no_ibid_override_then_the_locale_term_is_used_without_base_suffix_punctuation();
    }

    #[test]
    fn explicit_style_ibid_suffix_overrides_the_locale_term() {
        announce_behavior(
            "If the style defines its own ibid suffix, that style-specific suffix should override the locale term.",
        );
        super::given_explicit_style_ibid_suffix_when_locale_also_defines_ibid_then_the_style_suffix_wins();
    }

    #[test]
    fn missing_note_anchor_falls_back_to_reduced_citation_text() {
        announce_behavior(
            "If a repeated note cite has no reusable anchor, the reduced citation text should still appear cleanly.",
        );
        super::given_missing_note_anchor_when_integral_ibid_is_rendered_then_the_reduced_citation_still_appears_without_concatenation();
    }

    #[test]
    fn empty_note_flow_does_not_emit_a_bibliography_heading() {
        announce_behavior(
            "A note-flow document with no bibliography entries should not emit an empty bibliography heading.",
        );
        super::given_chicago_note_flow_document_when_no_bibliography_entries_are_needed_then_no_heading_is_emitted();
    }

    #[test]
    fn non_note_styles_never_emit_ibid_in_the_note_flow_example() {
        announce_behavior(
            "Running the note-flow example under non-note styles should never emit ibid.",
        );
        super::given_non_note_styles_when_rendering_the_note_flow_example_then_ibid_is_never_emitted();
    }
}

mod markdown_documents {
    use super::announce_behavior;

    #[test]
    fn pandoc_author_date_syntax_preserves_integral_and_cluster_citations() {
        announce_behavior(
            "Pandoc markdown citations should preserve both integral citations and citation clusters through rendering.",
        );
        super::given_pandoc_markdown_author_date_syntax_when_rendered_then_integral_and_cluster_citations_are_preserved();
    }

    #[test]
    fn note_style_markdown_integral_citations_emit_generated_notes() {
        announce_behavior(
            "Markdown integral citations rendered with a note style should generate note content instead of inline prose cites.",
        );
        super::given_markdown_integral_note_citation_when_rendered_with_a_note_style_then_a_generated_note_is_emitted();
    }
}

// --- Djot Adapter & Pipeline Tests ---

fn djot_parser_extracts_citations_from_simple_document() {
    let document = "A citation [@kuhn1962] appears here.";
    let parser = DjotParser;

    let parsed = parser.parse_document(document, &Locale::en_us());
    assert_eq!(parsed.citations.len(), 1, "Should extract one citation");
    assert_eq!(parsed.citations[0].citation.items[0].id, "kuhn1962");
}

fn djot_parser_respects_manual_footnotes() {
    let document = "Text[^m1].\n\n[^m1]: See [@kuhn1962].";
    let parser = DjotParser;

    let parsed = parser.parse_document(document, &Locale::en_us());
    assert_eq!(
        parsed.manual_note_order.len(),
        1,
        "Should track one manual note"
    );
    assert_eq!(parsed.manual_note_order[0], "m1");
    assert_eq!(
        parsed.citations.len(),
        1,
        "Should extract citation in footnote"
    );
}

fn djot_parsing_handles_multiple_citations() {
    let document = "First [@smith2020] and second [@jones2021] citations.";
    let parser = DjotParser;

    let parsed = parser.parse_document(document, &Locale::en_us());
    assert_eq!(
        parsed.citations.len(),
        2,
        "Should extract two separate citations"
    );
    assert_eq!(parsed.citations[0].citation.items[0].id, "smith2020");
    assert_eq!(parsed.citations[1].citation.items[0].id, "jones2021");
}

mod djot_adapter {
    use super::announce_behavior;

    #[test]
    fn simple_document_citation_extraction() {
        announce_behavior(
            "The Djot parser adapter should extract citations from simple documents.",
        );
        super::djot_parser_extracts_citations_from_simple_document();
    }

    #[test]
    fn manual_footnotes_are_tracked() {
        announce_behavior(
            "The Djot parser should track manual footnotes and citations within them.",
        );
        super::djot_parser_respects_manual_footnotes();
    }

    #[test]
    fn multiple_citations_extraction() {
        announce_behavior(
            "The Djot parser should extract multiple citations from a single document.",
        );
        super::djot_parsing_handles_multiple_citations();
    }
}

mod grouped_bibliography {
    use super::announce_behavior;

    #[test]
    fn primary_and_secondary_sources_render_both_headings_and_entries() {
        announce_behavior(
            "A grouped bibliography should render both primary and secondary headings along with their entries.",
        );
        super::given_grouped_primary_and_secondary_sources_when_rendered_then_both_group_headings_and_entries_appear();
    }

    #[test]
    fn group_local_disambiguation_restarts_year_suffixes_per_group() {
        announce_behavior(
            "Group-local disambiguation should restart year suffixes inside each bibliography group.",
        );
        super::given_group_local_disambiguation_when_rendering_multilingual_groups_then_year_suffixes_restart_within_each_group();
    }

    #[test]
    fn juris_m_legal_grouping_follows_the_expected_hierarchy() {
        announce_behavior(
            "Juris-M legal bibliography grouping should follow the expected legal hierarchy and headings.",
        );
        super::given_juris_m_legal_grouping_when_rendered_then_headings_follow_the_expected_legal_hierarchy();
    }

    #[test]
    fn english_locale_variants_fall_back_to_the_language_tag_for_group_headings() {
        announce_behavior(
            "English locale variants should fall back to their language tag when no localized group heading term exists.",
        );
        super::given_an_english_locale_variant_when_group_headings_are_localized_then_the_language_tag_fallback_is_used();
    }
}
