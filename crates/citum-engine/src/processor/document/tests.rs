use crate::processor::Processor;
use crate::processor::document::{CitationParser, DocumentFormat, djot::DjotParser};
use crate::reference::{Bibliography, Reference};
use crate::render::plain::PlainText;
use crate::render::typst::Typst;
use citum_schema::options::{
    Config, IntegralNameConfig, IntegralNameContexts, IntegralNameForm, IntegralNameRule,
    IntegralNameScope, NoteConfig, NoteMarkerOrder, NoteNumberPlacement, NoteQuotePlacement,
    Processing,
};
use citum_schema::template::{
    ContributorForm, ContributorRole, DateForm, DateVariable, Rendering, TemplateComponent,
    TemplateContributor, TemplateDate, TemplateGroup, TemplateTerm, TemplateTitle, TitleType,
    WrapPunctuation,
};
use citum_schema::{BibliographySpec, CitationSpec, NoteStartTextCase, Style};
use csl_legacy::csl_json::{
    DateVariable as LegacyDateVariable, Name, Reference as LegacyReference,
};

fn make_test_bib() -> Bibliography {
    let mut bib = Bibliography::new();
    bib.insert(
        "item1".to_string(),
        Reference::from(LegacyReference {
            id: "item1".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Doe", "John")]),
            title: Some("Book One".to_string()),
            issued: Some(LegacyDateVariable::year(2020)),
            ..Default::default()
        }),
    );
    bib.insert(
        "item2".to_string(),
        Reference::from(LegacyReference {
            id: "item2".to_string(),
            ref_type: "book".to_string(),
            author: Some(vec![Name::new("Smith", "Jane")]),
            title: Some("Book Two".to_string()),
            issued: Some(LegacyDateVariable::year(2010)),
            ..Default::default()
        }),
    );
    bib
}

fn make_author_date_style() -> Style {
    Style {
        citation: Some(CitationSpec {
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
            ]),
            wrap: Some(WrapPunctuation::Parentheses.into()),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering {
                        prefix: Some(" (".to_string()),
                        suffix: Some(")".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_note_style() -> Style {
    Style {
        options: Some(Config {
            processing: Some(Processing::Note),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                rendering: Rendering::default(),
                ..Default::default()
            })]),
            suffix: Some(".".to_string()),
            subsequent: Some(Box::new(CitationSpec {
                template: Some(vec![TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    rendering: Rendering {
                        prefix: Some("sub: ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                })]),
                suffix: Some(".".to_string()),
                ..Default::default()
            })),
            ibid: Some(Box::new(CitationSpec {
                note_start_text_case: Some(NoteStartTextCase::CapitalizeFirst),
                template: Some(vec![TemplateComponent::Term(TemplateTerm {
                    term: citum_schema::locale::GeneralTerm::Ibid,
                    form: None,
                    gender: None,
                    rendering: Rendering::default(),
                    custom: None,
                })]),
                suffix: Some(".".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    ..Default::default()
                }),
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    rendering: Rendering {
                        prefix: Some(". ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_note_style_with_rules(notes: NoteConfig) -> Style {
    let mut style = make_note_style();
    style.options = Some(Config {
        processing: Some(Processing::Note),
        notes: Some(notes),
        ..Default::default()
    });
    style
}

fn make_integral_name_style(scope: IntegralNameScope, contexts: IntegralNameContexts) -> Style {
    Style {
        options: Some(Config {
            processing: Some(Processing::AuthorDate),
            integral_names: Some(IntegralNameConfig {
                rule: Some(IntegralNameRule::FullThenShort),
                scope: Some(scope),
                contexts: Some(contexts),
                subsequent_form: Some(IntegralNameForm::Short),
            }),
            ..Default::default()
        }),
        citation: Some(CitationSpec {
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
            ]),
            integral: Some(Box::new(CitationSpec {
                template: Some(vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    ..Default::default()
                })]),
                ..Default::default()
            })),
            wrap: Some(WrapPunctuation::Parentheses.into()),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering {
                        prefix: Some(" (".to_string()),
                        suffix: Some(")".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn test_author_date_documents_still_render_inline() {
    let bib = make_test_bib();
    let processor = Processor::new(make_author_date_style(), bib);
    let parser = DjotParser;

    let content = "Visible citation: [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Visible citation: (Doe, 2020).\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_note_style_prose_citation_generates_footnote() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Text.[^citum-auto-1]\n\n[^citum-auto-1]: Book One, Book One.\n\n\n# Bibliography\n\nJohn Doe. Book One"
    );
}

#[test]
fn test_manual_footnote_citations_render_in_place() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text[^m1].\n\n[^m1]: See [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Text[^m1].\n\n[^m1]: See Book One, Book One.\n\n# Bibliography\n\nJohn Doe. Book One"
    );
}

#[test]
fn test_manual_footnote_definition_is_not_duplicated() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text[^m1].\n\n[^m1]: See [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Text[^m1].\n\n[^m1]: See Book One, Book One.\n\n# Bibliography\n\nJohn Doe. Book One"
    );
}

#[test]
fn test_mixed_manual_and_auto_notes_share_sequence() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Manual[^m1]. Auto [@item2]. Later[^m2].\n\n[^m1]: First [@item1].\n\n[^m2]: Second [@item2].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Manual[^m1]. Auto.[^citum-auto-2] Later[^m2].\n\n[^m1]: First Book One, Book One.\n\n[^m2]: Second Ibid..\n\n[^citum-auto-2]: Book Two, Book Two.\n\n\n# Bibliography\n\nJohn Doe. Book One\n\nJane Smith. Book Two"
    );
}

#[test]
fn test_multiple_citations_in_manual_footnote_are_preserved() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text[^m1].\n\n[^m1]: See [@item1]. Compare [@item2].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Text[^m1].\n\n[^m1]: See Book One, Book One. Compare Book Two, Book Two.\n\n# Bibliography\n\nJohn Doe. Book One\n\nJane Smith. Book Two"
    );
}

#[test]
fn test_multi_cite_prose_marker_produces_one_generated_note() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1; @item2].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Text.[^citum-auto-1]\n\n[^citum-auto-1]: Book One, Book One; Book Two, Book Two.\n\n\n# Bibliography\n\nJohn Doe. Book One\n\nJane Smith. Book Two"
    );
}

#[test]
fn test_note_style_preserves_surrounding_punctuation() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Sentence [@item1]. Next, [@item2] (see [@item1]).";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Sentence.[^citum-auto-1] Next,[^citum-auto-2] (see[^citum-auto-3]).\n\n[^citum-auto-1]: Book One, Book One.\n[^citum-auto-2]: Book Two, Book Two.\n[^citum-auto-3]: Book Onesub: Book One.\n\n\n# Bibliography\n\nJohn Doe. Book One\n\nJane Smith. Book Two"
    );
}

#[test]
fn test_note_style_default_rule_places_marker_after_period() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Sentence [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Sentence.[^citum-auto-1]\n\n[^citum-auto-1]: Book One, Book One.\n\n\n# Bibliography\n\nJohn Doe. Book One"
    );
}

#[test]
fn test_note_style_config_can_place_marker_before_period() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_note_style_with_rules(NoteConfig {
            punctuation: Some(NoteQuotePlacement::Outside),
            number: Some(NoteNumberPlacement::Outside),
            order: Some(NoteMarkerOrder::Before),
        }),
        bib,
    );
    let parser = DjotParser;

    let content = "Sentence [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Sentence[^citum-auto-1].\n\n[^citum-auto-1]: Book One, Book One.\n\n\n# Bibliography\n\nJohn Doe. Book One"
    );
}

#[test]
fn test_note_style_config_moves_marker_inside_quotes() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_note_style_with_rules(NoteConfig {
            punctuation: Some(NoteQuotePlacement::Outside),
            number: Some(NoteNumberPlacement::Inside),
            order: Some(NoteMarkerOrder::After),
        }),
        bib,
    );
    let parser = DjotParser;

    let content = "\"Quoted [@item1].\"";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "\"Quoted[^citum-auto-1]\".\n\n[^citum-auto-1]: Book One, Book One.\n\n\n# Bibliography\n\nJohn Doe. Book One"
    );
}

#[test]
fn test_note_order_uses_manual_reference_order_not_definition_order() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Manual[^m1]. Later [@item1].\n\n[^m1]: See [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Manual[^m1]. Later.[^citum-auto-2]\n\n[^m1]: See Book One, Book One.\n\n[^citum-auto-2]: Ibid..\n\n\n# Bibliography\n\nJohn Doe. Book One"
    );
}

#[test]
fn test_note_style_html_output_contains_footnotes() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result = processor.process_document::<_, crate::render::html::Html>(
        content,
        &parser,
        DocumentFormat::Html,
    );

    assert_eq!(
        result,
        "<p>Text.<a id=\"fnref1\" href=\"#fn1\" role=\"doc-noteref\"><sup>1</sup></a></p>\n<section id=\"Bibliography\">\n<h1>Bibliography</h1>\n<div class=\"citum-bibliography\">\n<div class=\"citum-entry\" id=\"ref-item1\" data-author=\"Doe\" data-year=\"2020\" data-title=\"Book One\"><span class=\"citum-author\">John Doe</span><span class=\"citum-title\">. Book One</span></div>\n</div>\n</section>\n<section role=\"doc-endnotes\">\n<hr>\n<ol>\n<li id=\"fn1\">\n<p><span class=\"citum-citation\" data-ref=\"item1\">Book One, <span class=\"citum-title\">Book One</span></span>.<a href=\"#fnref1\" role=\"doc-backlink\">↩\u{fe0e}</a></p>\n</li>\n</ol>\n</section>\n"
    );
}

#[test]
fn test_note_style_integral_citation_keeps_prose_anchor() {
    let style = Style::from_yaml_str(include_str!(
        "../../../../../styles/embedded/chicago-shortened-notes-bibliography.yaml"
    ))
    .unwrap();
    let bib = make_test_bib();
    let processor = Processor::new(style, bib);
    let parser = DjotParser;

    let content = "Narrative [+@item1] continues.";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Narrative Doe[^citum-auto-1] continues.\n\n[^citum-auto-1]: Doe, _Book One_.\n\n\n# Bibliography\n\nDoe, John, _Book One_, 2020."
    );
}

#[test]
fn test_repro_djot_parsing() {
    use citum_schema::citation::CitationMode;

    let parser = DjotParser;
    let content = "Test [+@item1] and [-@item2]";
    let citations = parser.parse_citations(content, &citum_schema::Locale::en_us());
    assert_eq!(citations.len(), 2);
    assert_eq!(citations[0].2.mode, CitationMode::Integral);
    assert!(citations[1].2.suppress_author);

    let content2 = "Test @item1 and +@item2 and -@item3 and !@item4";
    let citations2 = parser.parse_citations(content2, &citum_schema::Locale::en_us());
    assert_eq!(citations2.len(), 0);
}

#[test]
fn test_repro_djot_rendering() {
    let style = Style {
        citation: Some(CitationSpec {
            template: Some(vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    ..Default::default()
                }),
            ]),
            delimiter: Some(", ".to_string()),
            wrap: Some(WrapPunctuation::Parentheses.into()),
            integral: Some(Box::new(citum_schema::CitationSpec {
                template: Some(vec![
                    TemplateComponent::Contributor(TemplateContributor {
                        contributor: ContributorRole::Author,
                        form: ContributorForm::Short,
                        ..Default::default()
                    }),
                    TemplateComponent::Group(TemplateGroup {
                        group: vec![TemplateComponent::Date(TemplateDate {
                            date: DateVariable::Issued,
                            form: DateForm::Year,
                            ..Default::default()
                        })],
                        rendering: Rendering {
                            wrap: Some(WrapPunctuation::Parentheses.into()),
                            ..Default::default()
                        },
                        delimiter: None,
                        custom: None,
                    }),
                ]),
                delimiter: Some(" ".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        }),
        ..Default::default()
    };

    let bib = make_test_bib();
    let processor = Processor::new(style, bib);
    let parser = DjotParser;

    let content = "Integral: [+@item1]. SuppressAuthor: [-@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(result, "Integral: Doe (2020). SuppressAuthor: (2020).");
}

#[test]
fn test_real_chicago_note_style_generates_djot_footnotes() {
    let style = Style::from_yaml_str(include_str!(
        "../../../../../styles/embedded/chicago-shortened-notes-bibliography.yaml"
    ))
    .unwrap();
    let bib = make_test_bib();
    let processor = Processor::new(style, bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Text.[^citum-auto-1]\n\n[^citum-auto-1]: Doe, _Book One_.\n\n\n# Bibliography\n\nDoe, John, _Book One_, 2020."
    );
}

#[test]
fn test_document_with_yaml_frontmatter_bibliography_groups() {
    let bib = make_test_bib();
    let processor = Processor::new(make_author_date_style(), bib);
    let parser = DjotParser;

    let content = r#"---
bibliography:
  - id: primary
    heading:
      literal: "Primary Sources"
    selector:
      cited: visible
  - id: secondary
    heading:
      literal: "Secondary Sources"
    selector: {}
---

Some text [@item1]."#;

    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    // Should have the frontmatter heading and bibliography groups rendered
    assert_eq!(
        result,
        "Some text (Doe, 2020).\n\n# Bibliography\n\n# Primary Sources\n\nJohn Doe (2020)\n\n# Secondary Sources\n\nJane Smith (2010)"
    );
}

#[test]
fn test_document_without_bibliography_blocks_uses_default() {
    let bib = make_test_bib();
    let processor = Processor::new(make_author_date_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    // Should have default bibliography heading
    assert_eq!(
        result,
        "Text (Doe, 2020).\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_document_without_bibliography_blocks_uses_typst_heading() {
    let bib = make_test_bib();
    let processor = Processor::new(make_author_date_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result = processor.process_document::<_, Typst>(content, &parser, DocumentFormat::Typst);

    assert_eq!(
        result,
        "Text (#link(<ref-item1>)[Doe, 2020]).\n\n= Bibliography\n\nJohn Doe (2020) <ref-item1>"
    );
}

#[test]
fn test_document_with_inline_bibliography_block() {
    let bib = make_test_bib();
    let processor = Processor::new(make_author_date_style(), bib);
    let parser = DjotParser;

    // Note: Since djot syntax with attributes is { key=value }, the test just checks
    // that citation processing still works. Full inline block attribute parsing
    // would require the block to use proper djot syntax.
    let content = "Text [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Text (Doe, 2020).\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_document_with_yaml_frontmatter_uses_typst_group_headings() {
    let bib = make_test_bib();
    let processor = Processor::new(make_author_date_style(), bib);
    let parser = DjotParser;

    let content = r#"---
bibliography:
  - id: primary
    heading:
      literal: "Primary Sources"
    selector:
      cited: visible
  - id: secondary
    heading:
      literal: "Secondary Sources"
    selector: {}
---

Some text [@item1]."#;

    let result = processor.process_document::<_, Typst>(content, &parser, DocumentFormat::Typst);

    assert_eq!(
        result,
        "Some text (#link(<ref-item1>)[Doe, 2020]).\n\n= Bibliography\n\n== Primary Sources\n\nJohn Doe (2020) <ref-item1>\n\n== Secondary Sources\n\nJane Smith (2010) <ref-item2>"
    );
}

#[test]
fn test_integral_name_memory_full_then_short_in_one_document() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Document,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = "First [+@item1]. Later [+@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "First John Doe. Later Doe.\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_integral_name_memory_chapter_reset_uses_full_name_again() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Chapter,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = "# One\n\n[+@item1]. [+@item1].\n\n# Two\n\n[+@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "# One\n\nJohn Doe. Doe.\n\n# Two\n\nJohn Doe.\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_integral_name_memory_section_reset_uses_full_name_again() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Section,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = "## One\n\n[+@item1]. [+@item1].\n\n## Two\n\n[+@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "## One\n\nJohn Doe. Doe.\n\n## Two\n\nJohn Doe.\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_integral_name_memory_body_only_ignores_note_mentions() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(IntegralNameScope::Document, IntegralNameContexts::BodyOnly),
        bib,
    );
    let parser = DjotParser;

    let content =
        "Lead[^n1]. First body [+@item1]. Later body [+@item1].\n\n[^n1]: Note [+@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Lead[^n1]. First body John Doe. Later body Doe.\n\n[^n1]: Note John Doe.\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_integral_name_memory_body_and_notes_keeps_body_first_after_note_first() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Document,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = "Lead[^n1]. First body [+@item1].\n\n[^n1]: First note [+@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Lead[^n1]. First body John Doe.\n\n[^n1]: First note John Doe.\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_integral_name_memory_repeated_note_then_body_transitions_correctly() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Document,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = "Lead[^n1] and again[^n2]. First body [+@item1]. Later[^n3].\n\n[^n1]: One [+@item1].\n\n[^n2]: Two [+@item1].\n\n[^n3]: Three [+@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "Lead[^n1] and again[^n2]. First body John Doe. Later[^n3].\n\n[^n1]: One John Doe.\n\n[^n2]: Two Doe.\n\n[^n3]: Three Doe.\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_document_frontmatter_can_disable_integral_name_memory() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Document,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = r"---
integral-names:
  enabled: false
---

First [+@item1]. Later [+@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "First John Doe. Later John Doe.\n\n# Bibliography\n\nJohn Doe (2020)"
    );
}

#[test]
fn test_document_frontmatter_can_override_integral_name_scope_for_typst() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Document,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = r"---
integral-names:
  enabled: true
  scope: chapter
---

# One

[+@item1]. [+@item1].

# Two

[+@item1].";
    let result = processor.process_document::<_, Typst>(content, &parser, DocumentFormat::Typst);

    assert_eq!(
        result,
        "= One\n\n#link(<ref-item1>)[John Doe]. #link(<ref-item1>)[Doe].\n\n= Two\n\n#link(<ref-item1>)[John Doe].\n\n= Bibliography\n\nJohn Doe (2020) <ref-item1>"
    );
}

#[test]
fn test_document_frontmatter_ignores_unknown_keys() {
    let bib = make_test_bib();
    let processor = Processor::new(
        make_integral_name_style(
            IntegralNameScope::Document,
            IntegralNameContexts::BodyAndNotes,
        ),
        bib,
    );
    let parser = DjotParser;

    let content = r#"---
title: Sample Essay
author: Example Author
bibliography:
  - id: cited
    heading:
      literal: "Cited Works"
    selector:
      cited: visible
integral-names:
  enabled: true
  scope: document
  contexts: body-and-notes
---

First [+@item1]. Later [+@item1]."#;
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert_eq!(
        result,
        "First John Doe. Later Doe.\n\n# Bibliography\n\n# Cited Works\n\nJohn Doe (2020)\n\nJane Smith (2010)"
    );
}
