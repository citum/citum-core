/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "test")]
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

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

mod common;
use common::*;

use citum_engine::Processor;
use citum_engine::processor::document::{
    CitationParser, DocumentFormat, djot::DjotParser, markdown::MarkdownParser,
};
use citum_io::load_bibliography;
use citum_schema::{
    BibliographySpec, Locale, Style, StyleInfo,
    options::{
        BibliographyOptions, Config, Disambiguation, LocatorPreset, OrgAbbreviationMemoryConfig,
        Processing, ProcessingCustom,
    },
    reference::{
        Contributor, EdtfString, InputReference as Reference, Monograph, MonographType, SimpleName,
        Title,
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
        html_output.contains(r#"<span class="citum-citation" data-ref="smith2010">"#),
        "citation markup should be real HTML: {html_output}"
    );
    assert!(
        html_output.contains(r#"<div class="citum-bibliography">"#),
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

    assert_eq!(
        output,
        "# Chapter One\n\nThis is a test document using the proposed Djot citation syntax.\nThis example overrides the MLA default `document` scope to `chapter`\nso the narrative-name reset is visible in one short sample.\n\n## Parenthetical Citations\n\nMulti-cite with locator: (Kuhn; Watson and Crick, ch. 2).\n\nStructured locator: (Kuhn, sec. 5).\n\nSimple parenthetical: (Watson and Crick).\n\n## Integral Citations\n\nIntroductory note[^narrative-note].\n\nFirst narrative mention: Anthony D. Smith (10) surveys the broader literature.\n\nLater in the same chapter: Smith (12) narrows the argument.\n\nIntegral with locator: Thomas S. Kuhn (10) argues...\n\n## Visibility Modifiers\n\nSuppress author with locator: (10).\n\n[^narrative-note]: Before the prose introduces him, Anthony D. Smith (3) already appears in a note.\n\n# Chapter Two\n\nThe chapter boundary resets the narrative name memory, so Anthony D. Smith (14)\nappears in full again here.\n\n## In-Document Bibliography Grouping\n\nCitum supports `::: bibliography :::` fenced divs to place and filter\nbibliography sections inline. Each block renders independently; the\ndefault appended bibliography is suppressed when any block is present.\n\n### Unfiltered (all references)\n\nBird, Arthur. _Ornithology_. Nature Press, 1987.\n\nBrown, Dorothy. _Methods of Surveying and Measuring Vegetation_. Commonwealth Agricultural Bureaux, 1954.\n\nDoe, Jane. “Silent Paper.” _Journal of Silence_, 2020.\n\nKuhn, Thomas S. “The Structure of Scientific Revolutions.” _International Encyclopedia of Unified Science_. 2, no. 2. University of Chicago Press, 1962. https://doi.org/10.1234/example\n\nSmith, Anthony D. _Nationalism: Theory, Ideology, History_. Polity, 2010.\n\nWatson, James D., and Francis H. C. Crick. “Molecular Structure of Nucleic Acids: A Structure for Deoxyribose Nucleic Acid.” _Nature_. 171, no. 4356, 25 Apr. 1953, pp. 737–38.\n\n### Filtered by type\n\n## Journal Articles\n\nDoe, Jane. “Silent Paper.” _Journal of Silence_, 2020.\n\nKuhn, Thomas S. “The Structure of Scientific Revolutions.” _International Encyclopedia of Unified Science_. 2, no. 2. University of Chicago Press, 1962. https://doi.org/10.1234/example\n\nWatson, James D., and Francis H. C. Crick. “Molecular Structure of Nucleic Acids: A Structure for Deoxyribose Nucleic Acid.” _Nature_. 171, no. 4356, 25 Apr. 1953, pp. 737–38.\n\n## Books\n\nBird, Arthur. _Ornithology_. Nature Press, 1987.\n\nBrown, Dorothy. _Methods of Surveying and Measuring Vegetation_. Commonwealth Agricultural Bureaux, 1954.\n\nSmith, Anthony D. _Nationalism: Theory, Ideology, History_. Polity, 2010.\n"
    );
}

fn given_two_authors_with_same_surname_when_both_cited_integrally_then_each_gets_first_form() {
    let mut style = load_style("styles/embedded/modern-language-association.yaml");
    style
        .options
        .get_or_insert_with(Config::default)
        .integral_name_memory = Some(citum_schema::options::IntegralNameMemoryConfig {
        contexts: Some(citum_schema::options::IntegralNameContexts::BodyOnly),
        subsequent_form: Some(citum_schema::options::SubsequentNameForm::Short),
        ..Default::default()
    });

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "john-smith".to_string(),
        Reference::Monograph(Box::new(Monograph {
            id: Some("john-smith".into()),
            r#type: MonographType::Book,
            title: Some(Title::Single("Book One".to_string())),
            author: Some(Contributor::StructuredName(
                citum_schema::reference::StructuredName {
                    family: citum_schema::reference::MultilingualString::Simple(
                        "Smith".to_string(),
                    ),
                    given: citum_schema::reference::MultilingualString::Simple("John".to_string()),
                    suffix: None,
                    dropping_particle: None,
                    non_dropping_particle: None,
                },
            )),
            issued: EdtfString("2010".to_string()),
            ..Default::default()
        })),
    );
    bib.insert(
        "jane-smith".to_string(),
        Reference::Monograph(Box::new(Monograph {
            id: Some("jane-smith".into()),
            r#type: MonographType::Book,
            title: Some(Title::Single("Book Two".to_string())),
            author: Some(Contributor::StructuredName(
                citum_schema::reference::StructuredName {
                    family: citum_schema::reference::MultilingualString::Simple(
                        "Smith".to_string(),
                    ),
                    given: citum_schema::reference::MultilingualString::Simple("Jane".to_string()),
                    suffix: None,
                    dropping_particle: None,
                    non_dropping_particle: None,
                },
            )),
            issued: EdtfString("2015".to_string()),
            ..Default::default()
        })),
    );

    let processor = citum_engine::Processor::new(style, bib);
    let parser = DjotParser;
    let doc = "[+@john-smith] wrote the first book. [+@jane-smith] wrote the second.";

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        doc,
        &parser,
        DocumentFormat::Plain,
    );

    // Both authors share surname "Smith" but are different people.
    // The second author's first integral mention must NOT be marked Subsequent.
    // A family-name-only tracking key conflates them — "Jane Smith" appears as
    // just "Smith" instead of "Jane Smith".
    assert!(
        output.contains("John Smith"),
        "first Smith should show full given+family name: {output}"
    );
    assert!(
        output.contains("Jane Smith"),
        "second Smith should show full given+family name (not just 'Smith'): {output}"
    );
}

fn given_org_with_short_name_when_org_abbreviation_memory_configured_and_cited_integrally_twice_then_first_shows_full_then_subsequent_shows_short()
 {
    let mut style = load_style("styles/embedded/modern-language-association.yaml");
    style
        .options
        .get_or_insert_with(Config::default)
        .org_abbreviation_memory = Some(OrgAbbreviationMemoryConfig {
        contexts: Some(citum_schema::options::IntegralNameContexts::BodyOnly),
        ..Default::default()
    });

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "who2020".to_string(),
        Reference::Monograph(Box::new(Monograph {
            id: Some("who2020".into()),
            r#type: MonographType::Book,
            title: Some(Title::Single("World Health Report".to_string())),
            author: Some(Contributor::SimpleName(SimpleName {
                name: citum_schema::reference::MultilingualString::Simple(
                    "World Health Organization".to_string(),
                ),
                short_name: Some("WHO".to_string()),
                location: None,
            })),
            issued: EdtfString("2020".to_string()),
            ..Default::default()
        })),
    );

    let processor = citum_engine::Processor::new(style, bib);
    let parser = DjotParser;
    // Two integral citations to the same org.
    let doc = "[+@who2020] released a report. Later, [+@who2020] followed up.";

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        doc,
        &parser,
        DocumentFormat::Plain,
    );

    // First integral mention: full name + abbreviation in parens.
    assert!(
        output.contains("World Health Organization (WHO)"),
        "first mention should show full name then abbreviation: {output}"
    );
    // Subsequent mention: abbreviation only.
    assert!(
        !output.contains("World Health Organization (WHO) released")
            || output.contains("WHO followed"),
        "second mention should use short form only: {output}"
    );
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

    // APA abbreviates given names: "A. D. Smith" (Long form, first integral mention)
    assert!(output.contains("First narrative mention: A. D. Smith (2010, p. 10)"));
    assert!(output.contains("Later in the same chapter: Smith (2010, p. 12)"));
    assert!(output.contains("Integral with locator: T. S. Kuhn (1962, p. 10) argues"));
    assert!(output.contains(
        "[^narrative-note]: Before the prose introduces him, A. D. Smith (2010, p. 3) already appears in a note."
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

    // Note-style in-text anchors show surname only; first-mention Long form applies to
    // note content. Smith's first body cite is IbidWithLocator (follows note cite),
    // so the fallback renders surname. Kuhn is Subsequent-position but first integral
    // mention → Long form → full name in anchor.
    assert!(output.contains("First narrative mention: Smith[^citum-auto-5] surveys"));
    assert!(output.contains("Later in the same chapter: Smith[^citum-auto-6] narrows"));
    assert!(output.contains("Integral with locator: Thomas S. Kuhn[^citum-auto-7] argues"));
    assert!(
        output.contains("[^narrative-note]: Before the prose introduces him, Smith, Anthony D."),
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

fn given_markdown_citation_inside_manual_footnote_when_rendered_with_note_style_then_it_renders_in_place()
 {
    // When the user writes their own [^n]: block containing a citation,
    // the citation should render inline inside that definition (ManualFootnote
    // placement) rather than generating a second auto-footnote.
    let processor =
        example_document_processor("styles/embedded/chicago-shortened-notes-bibliography.yaml");
    let parser = MarkdownParser;
    let document = "See note[^1].\n\n[^1]: Early work [@kuhn1962] supports this.";

    let output = processor.process_document::<_, citum_engine::render::plain::PlainText>(
        document,
        &parser,
        DocumentFormat::Plain,
    );

    // The manual footnote anchor must appear in prose.
    assert!(
        output.contains("[^1]"),
        "manual footnote reference should appear in prose: {output}"
    );
    // The rendered citation should appear inside the footnote definition, not
    // as a separate auto-generated note.
    assert!(
        output.contains("[^1]: Early work"),
        "footnote definition body should be preserved: {output}"
    );
    assert!(
        output.contains("Kuhn") || output.contains("Structure"),
        "citation inside manual footnote should be rendered in place: {output}"
    );
    // No auto-generated note should be created for this citation.
    assert!(
        !output.contains("[^citum-auto-"),
        "no auto-footnote should be generated for a ManualFootnote citation: {output}"
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
    fn two_authors_with_same_surname_both_get_first_form() {
        announce_behavior(
            "Two different integral authors sharing a family name must each render in full (First) form on their own first mention.",
        );
        super::given_two_authors_with_same_surname_when_both_cited_integrally_then_each_gets_first_form();
    }

    #[test]
    fn org_abbreviation_memory_renders_full_then_short_on_first_and_short_on_subsequent() {
        announce_behavior(
            "With org-abbreviation-memory configured, the first integral mention of an org shows full name + abbreviation; subsequent shows abbreviation only.",
        );
        super::given_org_with_short_name_when_org_abbreviation_memory_configured_and_cited_integrally_twice_then_first_shows_full_then_subsequent_shows_short();
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

fn given_markdown_document_with_pipe_table_when_rendered_as_markdown_then_body_passes_through_verbatim()
 {
    // A GFM pipe table, a fenced code block, and a citation in prose — body
    // markup must survive verbatim; only the [@key] marker is replaced.
    let processor = example_document_processor("styles/embedded/apa-7th.yaml");
    let parser = MarkdownParser;
    let pipe_table = "| Column A | Column B |\n|----------|----------|\n| cell 1   | cell 2   |";
    let code_block = "```rust\nfn hello() {}\n```";
    let document =
        format!("# Introduction\n\nAs argued in [@kuhn1962].\n\n{pipe_table}\n\n{code_block}\n");

    let output = processor.process_document::<_, citum_engine::render::markdown::Markdown>(
        &document,
        &parser,
        DocumentFormat::Markdown,
    );

    // Pipe table lines must be unchanged.
    assert!(
        output.contains("| Column A | Column B |"),
        "pipe table header missing: {output}"
    );
    assert!(
        output.contains("|----------|----------|"),
        "pipe table separator missing: {output}"
    );
    assert!(
        output.contains("| cell 1   | cell 2   |"),
        "pipe table row missing: {output}"
    );

    // Fenced code block must be unchanged.
    assert!(
        output.contains("```rust\nfn hello() {}\n```"),
        "fenced code block missing or modified: {output}"
    );

    // Citation marker replaced with rendered inline text.
    assert!(
        !output.contains("[@kuhn1962]"),
        "raw citation marker should be replaced: {output}"
    );
    assert!(
        output.contains("As argued in (Kuhn, 1962)."),
        "rendered citation should replace the marker with the full APA cite: {output}"
    );

    // Bibliography heading present.
    assert!(
        output.contains("# Bibliography"),
        "bibliography heading missing: {output}"
    );
}

fn given_note_style_markdown_document_when_rendered_as_markdown_then_commonmark_footnote_syntax_is_emitted()
 {
    // Note styles emit [^label] anchors in prose and [^label]: … definitions
    // at the end — the CommonMark+footnotes extension used by Pandoc/GFM.
    let processor =
        example_document_processor("styles/embedded/chicago-shortened-notes-bibliography.yaml");
    let parser = MarkdownParser;
    let document = "First claim [@kuhn1962]. Second claim [@smith2010].";

    let output = processor.process_document::<_, citum_engine::render::markdown::Markdown>(
        document,
        &parser,
        DocumentFormat::Markdown,
    );

    // Footnote anchors in prose.
    assert!(
        output.contains("[^citum-auto-1]"),
        "first footnote anchor missing: {output}"
    );
    assert!(
        output.contains("[^citum-auto-2]"),
        "second footnote anchor missing: {output}"
    );

    // Footnote definitions with rendered content (CommonMark emphasis).
    assert!(
        output.contains("[^citum-auto-1]:"),
        "first footnote definition missing: {output}"
    );
    assert!(
        output.contains("[^citum-auto-2]:"),
        "second footnote definition missing: {output}"
    );

    // No raw citation markers remain.
    assert!(
        !output.contains("[@kuhn1962]") && !output.contains("[@smith2010]"),
        "raw citation markers should be replaced: {output}"
    );
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

fn djot_note_preserves_italic_markup_in_html_bibliography() {
    use citum_engine::render::html::Html;
    use citum_schema::template::{SimpleVariable, TemplateVariable};

    let style = Style {
        info: StyleInfo {
            title: Some("Note Djot Test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![citum_schema::template::TemplateComponent::Variable(
                TemplateVariable {
                    variable: SimpleVariable::Note,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "ref1".to_string(),
        citum_schema::reference::InputReference::Monograph(Box::new(
            citum_schema::reference::Monograph {
                id: Some("ref1".into()),
                r#type: citum_schema::reference::MonographType::Book,
                title: Some(citum_schema::reference::Title::Single(
                    "Test Book".to_string(),
                )),
                issued: citum_schema::reference::EdtfString("2024".to_string()),
                note: Some(citum_schema::reference::RichText::Djot {
                    djot: "_italic_".to_string(),
                }),
                ..Default::default()
            },
        )),
    );

    let output = Processor::new(style, bib).render_bibliography_with_format::<Html>();
    assert!(
        output.contains("<em>italic</em>"),
        "Djot _italic_ in note should render as <em>italic</em> in HTML, got: {output}"
    );
}

fn djot_note_sentence_case_does_not_restart_across_markup_boundaries() {
    use citum_engine::render::html::Html;
    use citum_schema::options::titles::TextCase;
    use citum_schema::template::{Rendering, SimpleVariable, TemplateComponent, TemplateVariable};

    let style = Style {
        info: StyleInfo {
            title: Some("Note Djot Sentence Case Test".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            template: Some(vec![TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Note,
                rendering: Rendering {
                    text_case: Some(TextCase::Sentence),
                    ..Default::default()
                },
                ..Default::default()
            })]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut bib = indexmap::IndexMap::new();
    bib.insert(
        "ref1".to_string(),
        citum_schema::reference::InputReference::Monograph(Box::new(
            citum_schema::reference::Monograph {
                id: Some("ref1".into()),
                r#type: citum_schema::reference::MonographType::Book,
                title: Some(citum_schema::reference::Title::Single(
                    "Test Book".to_string(),
                )),
                issued: citum_schema::reference::EdtfString("2024".to_string()),
                note: Some(citum_schema::reference::RichText::Djot {
                    djot: "foo _BAR_ baz".to_string(),
                }),
                ..Default::default()
            },
        )),
    );

    let output = Processor::new(style, bib).render_bibliography_with_format::<Html>();
    assert!(
        output.contains("Foo <em>bar</em> baz"),
        "Djot sentence case should not restart inside markup, got: {output}"
    );
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

    #[test]
    fn note_preserves_italic_markup_in_html_bibliography() {
        announce_behavior("Djot note preserves italic markup in HTML bibliography.");
        super::djot_note_preserves_italic_markup_in_html_bibliography();
    }

    #[test]
    fn note_sentence_case_does_not_restart_across_markup_boundaries() {
        announce_behavior("Djot note sentence case does not restart across markup boundaries.");
        super::djot_note_sentence_case_does_not_restart_across_markup_boundaries();
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

// --- Body markup conversion for terminal formats (#824) ---

fn given_markdown_block_quote_when_rendered_as_typst_then_quote_block_is_emitted() {
    let processor = example_document_processor("styles/embedded/apa-7th.yaml");
    let parser = MarkdownParser;
    let document = concat!(
        "> This is a block quote with *italic* text,\n",
        "> and **strong** text. So is __this__.\n",
    );

    let output = processor.process_document::<_, citum_engine::render::typst::Typst>(
        document,
        &parser,
        DocumentFormat::Typst,
    );

    assert!(
        output.contains("#quote(block: true)"),
        "markdown block quote should produce Typst #quote(block: true), got: {output}"
    );
    assert!(
        output.contains("#emph[italic]"),
        "markdown *italic* should produce Typst #emph[…], got: {output}"
    );
    assert!(
        !output.starts_with('>'),
        "raw markdown block-quote syntax should not appear in Typst output, got: {output}"
    );
}

fn given_djot_block_quote_when_rendered_as_typst_then_quote_block_is_emitted() {
    let processor = example_document_processor("styles/embedded/apa-7th.yaml");
    let parser = DjotParser;
    let document = concat!(
        "> This is a block quote with _italic_ text,\n",
        "> and *bold*.\n",
    );

    let output = processor.process_document::<_, citum_engine::render::typst::Typst>(
        document,
        &parser,
        DocumentFormat::Typst,
    );

    assert!(
        output.contains("#quote(block: true)"),
        "djot block quote should produce Typst #quote(block: true), got: {output}"
    );
    assert!(
        output.contains("#emph[italic]"),
        "djot _italic_ should produce Typst #emph[…], got: {output}"
    );
}

fn given_markdown_block_quote_when_rendered_as_latex_then_quote_environment_is_emitted() {
    let processor = example_document_processor("styles/embedded/apa-7th.yaml");
    let parser = MarkdownParser;
    let document = concat!(
        "> This is a block quote with *italic* text\n",
        "> and **strong** text.\n",
    );

    let output = processor.process_document::<_, citum_engine::render::latex::Latex>(
        document,
        &parser,
        DocumentFormat::Latex,
    );

    assert!(
        output.contains("\\begin{quote}"),
        "markdown block quote should produce LaTeX \\begin{{quote}}, got: {output}"
    );
    assert!(
        output.contains("\\emph{italic}"),
        "markdown *italic* should produce LaTeX \\emph{{}}, got: {output}"
    );
    assert!(
        output.contains("\\textbf{strong}"),
        "markdown **strong** should produce LaTeX \\textbf{{}}, got: {output}"
    );
}

fn given_markdown_citation_inside_prose_when_rendered_as_typst_then_citation_and_markup_both_appear()
 {
    let processor = example_document_processor("styles/embedded/apa-7th.yaml");
    let parser = MarkdownParser;
    let document = "A paragraph with *emphasis* and a citation [@kuhn1962].";

    let output = processor.process_document::<_, citum_engine::render::typst::Typst>(
        document,
        &parser,
        DocumentFormat::Typst,
    );

    assert!(
        output.contains("#emph[emphasis]"),
        "markdown *emphasis* in paragraph should produce Typst #emph[…], got: {output}"
    );
    assert!(
        output.contains("Kuhn") && output.contains("1962"),
        "citation should render in Typst output, got: {output}"
    );
}

mod body_markup_terminal_formats {
    use super::announce_behavior;

    #[test]
    fn markdown_block_quote_renders_as_typst_quote_block() {
        announce_behavior(
            "A Markdown block quote rendered to Typst should produce #quote(block: true) with inline emphasis correctly mapped — not raw '>' syntax (fixes #824).",
        );
        super::given_markdown_block_quote_when_rendered_as_typst_then_quote_block_is_emitted();
    }

    #[test]
    fn djot_block_quote_renders_as_typst_quote_block() {
        announce_behavior(
            "A Djot block quote rendered to Typst should produce #quote(block: true) with correctly mapped inline markup.",
        );
        super::given_djot_block_quote_when_rendered_as_typst_then_quote_block_is_emitted();
    }

    #[test]
    fn markdown_block_quote_renders_as_latex_quote_environment() {
        announce_behavior(
            "A Markdown block quote rendered to LaTeX should produce a \\begin{quote} environment with \\emph and \\textbf for inline markup.",
        );
        super::given_markdown_block_quote_when_rendered_as_latex_then_quote_environment_is_emitted(
        );
    }

    #[test]
    fn markdown_citation_in_prose_renders_alongside_converted_markup() {
        announce_behavior(
            "A Markdown paragraph with a citation and inline emphasis rendered to Typst should produce both a converted citation and #emph[…] markup.",
        );
        super::given_markdown_citation_inside_prose_when_rendered_as_typst_then_citation_and_markup_both_appear();
    }
}
