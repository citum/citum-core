use crate::processor::Processor;
use crate::processor::document::{CitationParser, DocumentFormat, djot::DjotParser};
use crate::reference::{Bibliography, Reference};
use crate::render::plain::PlainText;
use citum_schema::options::{Config, Processing};
use citum_schema::template::{
    ContributorForm, ContributorRole, DateForm, DateVariable, Rendering, TemplateComponent,
    TemplateContributor, TemplateDate, TemplateList, TemplateTerm, TemplateTitle, TitleType,
    WrapPunctuation,
};
use citum_schema::{BibliographySpec, CitationSpec, Style};
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
                template: Some(vec![TemplateComponent::Term(TemplateTerm {
                    term: citum_schema::locale::GeneralTerm::Ibid,
                    form: None,
                    rendering: Rendering::default(),
                    overrides: None,
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

    assert!(result.contains("Text [^citum-auto-1]."));
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
fn test_mixed_manual_and_auto_notes_share_sequence() {
    let bib = make_test_bib();
    let processor = Processor::new(make_note_style(), bib);
    let parser = DjotParser;

    let content = "Manual[^m1]. Auto [@item2]. Later[^m2].\n\n[^m1]: First [@item1].\n\n[^m2]: Second [@item2].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Auto [^citum-auto-2]."));
    assert!(result.contains("[^m1]: First"));
    assert!(result.contains("[^m2]: Second"));
    assert!(result.contains("[^citum-auto-2]:"));
    assert!(result.contains("ibid"));
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

    assert!(result.contains("Text [^citum-auto-1]."));
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

    assert!(result.contains("Sentence [^citum-auto-1]."));
    assert!(result.contains("Next, [^citum-auto-2] (see [^citum-auto-3])."));
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
    assert!(result.contains("ibid"));
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
fn test_repro_djot_parsing() {
    use citum_schema::citation::CitationMode;

    let parser = DjotParser;
    let content = "Test [+@item1] and [-@item2]";
    let citations = parser.parse_citations(content);
    assert_eq!(citations.len(), 2);
    assert_eq!(citations[0].2.mode, CitationMode::Integral);
    assert!(citations[1].2.suppress_author);

    let content2 = "Test @item1 and +@item2 and -@item3 and !@item4";
    let citations2 = parser.parse_citations(content2);
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
                    TemplateComponent::List(TemplateList {
                        items: vec![TemplateComponent::Date(TemplateDate {
                            date: DateVariable::Issued,
                            form: DateForm::Year,
                            ..Default::default()
                        })],
                        rendering: Rendering {
                            wrap: Some(WrapPunctuation::Parentheses),
                            ..Default::default()
                        },
                        delimiter: None,
                        overrides: None,
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
    let style: Style = serde_yaml::from_str(include_str!(
        "../../../../../styles/chicago-shortened-notes-bibliography.yaml"
    ))
    .unwrap();
    let bib = make_test_bib();
    let processor = Processor::new(style, bib);
    let parser = DjotParser;

    let content = "Text [@item1].";
    let result =
        processor.process_document::<_, PlainText>(content, &parser, DocumentFormat::Plain);

    assert!(result.contains("Text [^citum-auto-1]."));
    assert!(result.contains("[^citum-auto-1]:"));
}
