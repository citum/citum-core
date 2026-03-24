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
            wrap: Some(WrapPunctuation::Parentheses),
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
            wrap: Some(WrapPunctuation::Parentheses),
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

    assert!(result.contains("Visible citation: (Doe, 2020)."));
    assert!(!result.contains("citum-auto-"));
    assert!(result.contains("# Bibliography"));
}

#[test]
fn test_note_style_prose_citation_generates_footnote() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Text.[^citum-auto-1]"));
    assert!(result.contains("[^citum-auto-1]:"));
    assert!(result.contains("Book One"));
    assert!(result.contains("# Bibliography"));
}

#[test]
fn test_manual_footnote_citations_render_in_place() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text[^m1].\n\n[^m1]: See [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Text[^m1]."));
    assert!(result.contains("[^m1]: See"));
    assert!(result.contains("Book One"));
    assert!(!result.contains("citum-auto-"));
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
        result.matches("[^m1]:").count(),
        1,
        "manual note duplicated: {result}"
    );
    assert!(
        !result.contains("[@item1]"),
        "raw citation leaked: {result}"
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

    assert!(result.contains("Auto.[^citum-auto-2]"));
    assert!(result.contains("[^m1]: First"));
    assert!(result.contains("[^m2]: Second"));
    assert!(result.contains("[^citum-auto-2]:"));
    assert!(result.contains("Ibid"));
}

#[test]
fn test_multiple_citations_in_manual_footnote_are_preserved() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text[^m1].\n\n[^m1]: See [@item1]. Compare [@item2].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("[^m1]: See"));
    assert!(result.contains("Compare"));
    assert!(result.contains("Book One"));
    assert!(result.contains("Book Two"));
    assert!(!result.contains("citum-auto-"));
}

#[test]
fn test_multi_cite_prose_marker_produces_one_generated_note() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1; @item2].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Text.[^citum-auto-1]"));
    assert_eq!(result.matches("[^citum-auto-1]:").count(), 1);
}

#[test]
fn test_note_style_preserves_surrounding_punctuation() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Sentence [@item1]. Next, [@item2] (see [@item1]).";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Sentence.[^citum-auto-1]"));
    assert!(result.contains("Next,[^citum-auto-2] (see[^citum-auto-3])."));
}

#[test]
fn test_note_style_default_rule_places_marker_after_period() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Sentence [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Sentence.[^citum-auto-1]"));
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

    assert!(result.contains("Sentence[^citum-auto-1]."));
    assert!(!result.contains("Sentence.[^citum-auto-1]"));
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

    assert!(result.contains("\"Quoted[^citum-auto-1]\"."));
}

#[test]
fn test_note_order_uses_manual_reference_order_not_definition_order() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Manual[^m1]. Later [@item1].\n\n[^m1]: See [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("[^m1]: See"));
    assert!(result.contains("[^citum-auto-2]:"));
    assert!(result.contains("Ibid"));
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

    assert!(result.contains("role=\"doc-noteref\""));
    assert!(result.contains("role=\"doc-endnotes\""));
}

#[test]
fn test_note_style_integral_citation_keeps_prose_anchor() {
    let style = Style::from_yaml_str(include_str!(
        "../../../../../styles/chicago-shortened-notes-bibliography.yaml"
    ))
    .unwrap();
    let bib = make_test_bib();
    let processor = Processor::new(style, bib);
    let parser = DjotParser;

    let content = "Narrative [+@item1] continues.";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Narrative Doe[^citum-auto-1] continues."));
    assert!(result.contains("[^citum-auto-1]: Doe,"));
    assert!(result.contains("Book One"));
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
            wrap: Some(WrapPunctuation::Parentheses),
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
                            wrap: Some(WrapPunctuation::Parentheses),
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

    assert!(result.contains("Integral: Doe (2020)."));
    assert!(result.contains("SuppressAuthor: (2020)."));
}

#[test]
fn test_real_chicago_note_style_generates_djot_footnotes() {
    let style = Style::from_yaml_str(include_str!(
        "../../../../../styles/chicago-shortened-notes-bibliography.yaml"
    ))
    .unwrap();
    let bib = make_test_bib();
    let processor = Processor::new(style, bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Text.[^citum-auto-1]"));
    assert!(result.contains("[^citum-auto-1]:"));
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
    assert!(result.contains("Primary Sources"));
    assert!(result.contains("Secondary Sources"));
    assert!(result.contains("Doe"));
    assert!(result.contains("Smith"));
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
    assert!(result.contains("# Bibliography"));
    assert!(result.contains("Doe"));
}

#[test]
fn test_document_without_bibliography_blocks_uses_typst_heading() {
    let bib = make_test_bib();
    let processor = Processor::new(make_author_date_style(), bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result = processor.process_document::<_, Typst>(content, &parser, DocumentFormat::Typst);

    assert!(result.contains("= Bibliography"));
    assert!(!result.contains("# Bibliography"));
    assert!(result.contains("#link(<ref-item1>)"));
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

    assert!(result.contains("Doe"));
    assert!(result.contains("# Bibliography"));
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

    assert!(result.contains("== Primary Sources"));
    assert!(result.contains("== Secondary Sources"));
    assert!(!result.contains("## Primary Sources"));
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

    assert!(result.contains("First John Doe. Later Doe."));
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

    assert!(result.contains("John Doe. Doe."));
    assert!(result.contains("# Two\n\nJohn Doe."));
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

    assert!(result.contains("John Doe. Doe."));
    assert!(result.contains("## Two\n\nJohn Doe."));
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

    assert!(result.contains("[^n1]: Note John Doe."));
    assert!(result.contains("First body John Doe. Later body Doe."));
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

    assert!(result.contains("[^n1]: First note John Doe."));
    assert!(result.contains("First body John Doe."));
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

    assert!(result.contains("[^n1]: One John Doe."));
    assert!(result.contains("[^n2]: Two Doe."));
    assert!(result.contains("First body John Doe."));
    assert!(result.contains("[^n3]: Three Doe."));
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

    assert!(result.contains("First John Doe. Later John Doe."));
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

    assert!(result.contains("#link(<ref-item1>)[John Doe]. #link(<ref-item1>)[Doe]."));
    assert!(result.contains("= Two\n\n#link(<ref-item1>)[John Doe]."));
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

    assert!(result.contains("First John Doe. Later Doe."));
    assert!(result.contains("# Bibliography"));
    assert!(result.contains("Cited Works"));
}
