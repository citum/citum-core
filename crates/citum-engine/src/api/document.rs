/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Document-level batch formatting API.

use crate::api::AnnotationStyle;
use crate::error::ProcessorError;
use crate::processor::Processor;
use crate::reference::{Bibliography, Citation};
use crate::render::djot::Djot;
use crate::render::format::OutputFormat;
use crate::render::html::Html;
use crate::render::latex::Latex;
use crate::render::plain::PlainText;
use crate::render::typst::Typst;
use citum_schema::Style;
use citum_schema::locale::{GeneralTerm, TermForm};
use citum_schema::reference::{
    ClassExtension, CollectionType, ContributorRole as ReferenceRole, MonographComponentType,
    MonographType, ReferenceClass, SerialComponentType,
};
use citum_schema::template::ContributorRole as TemplateRole;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    BibliographyEntry, CitationOccurrence, DocumentOptions, EntryMetadata, FormattedBibliography,
    FormattedCitation, OutputFormatKind, StyleInput, Warning, WarningLevel,
};

/// A request to format a complete document's citations and bibliography.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatDocumentRequest {
    /// The style to use (may be resolved locally or by an adapter).
    pub style: StyleInput,
    /// Optional locale override as a BCP 47 language tag (e.g. `en-US`).
    /// When omitted or set to en-US the engine uses its built-in en-US locale;
    /// other locales emit a warning and fall back to en-US until adapter-side
    /// locale resolution is wired through.
    pub locale: Option<String>,
    /// Output format (plain, html, djot, latex, typst). Defaults to plain
    /// when omitted from the request.
    #[serde(default)]
    pub output_format: OutputFormatKind,
    /// Bibliography (references indexed by ID).
    pub refs: Bibliography,
    /// Ordered citations as they appear in the document.
    pub citations: Vec<CitationOccurrence>,
    /// Optional document-level configuration.
    pub document_options: Option<DocumentOptions>,
}

/// The result of formatting a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatDocumentResult {
    /// Formatted citations in document order.
    pub formatted_citations: Vec<FormattedCitation>,
    /// Formatted bibliography.
    pub bibliography: FormattedBibliography,
    /// Non-fatal warnings encountered during processing.
    pub warnings: Vec<Warning>,
}

/// Errors that can occur during document formatting.
#[derive(Debug)]
pub enum FormatDocumentError {
    /// The style ID or URI requires a resolver chain not available in the engine.
    UnresolvedInput(String),
    /// Failed to parse the style YAML.
    StyleParse(String),
    /// Failed to read or locate the style file.
    StylePath(String),
    /// The processor encountered an error during rendering.
    Processing(ProcessorError),
}

impl std::fmt::Display for FormatDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnresolvedInput(msg) => write!(f, "Unresolved style input: {}", msg),
            Self::StyleParse(msg) => write!(f, "Style parse error: {}", msg),
            Self::StylePath(msg) => write!(f, "Style path error: {}", msg),
            Self::Processing(err) => write!(f, "Processing error: {}", err),
        }
    }
}

impl std::error::Error for FormatDocumentError {}

impl From<ProcessorError> for FormatDocumentError {
    fn from(err: ProcessorError) -> Self {
        Self::Processing(err)
    }
}

/// Format a complete document's citations and bibliography (convenience wrapper).
///
/// This function resolves the style locally using `StyleInput::resolve_local`.
/// For styles requiring a resolver chain (Id or Uri), use `format_document_with_style`
/// after pre-resolving.
///
/// # Errors
///
/// Returns an error if the style cannot be resolved, parsed, or if rendering fails.
pub fn format_document(
    request: FormatDocumentRequest,
) -> Result<FormatDocumentResult, FormatDocumentError> {
    let style = request.style.resolve_local()?;
    format_document_with_style(style, request)
}

/// Format a document using an already-resolved style.
///
/// This is the primary entry point for adapters (citum-server, citum-bindings)
/// that have a resolver chain and can pre-resolve style IDs and URIs.
///
/// # Errors
///
/// Returns an error if rendering fails.
pub fn format_document_with_style(
    style: Style,
    request: FormatDocumentRequest,
) -> Result<FormatDocumentResult, FormatDocumentError> {
    let mut warnings = Vec::new();

    // Locale: the engine has no resolver chain for non-en-US locales.
    // Adapters with a citum_store dep can pre-resolve and call
    // Processor::with_locale directly; for now, emit a warning when a
    // non-en-US tag is requested and fall back to en-US.
    if let Some(tag) = &request.locale
        && !tag.is_empty()
        && !tag.eq_ignore_ascii_case("en-us")
    {
        warnings.push(Warning {
            level: WarningLevel::Warning,
            code: "locale_fallback".to_string(),
            citation_id: None,
            ref_id: None,
            message: format!(
                "Requested locale '{tag}' could not be loaded by the engine; falling back to en-US. Adapter-side locale resolution is not yet wired through."
            ),
        });
    }

    let mut processor = Processor::new(style, request.refs);
    warnings.extend(unknown_reference_class_warnings(&processor.bibliography));
    warnings.extend(unknown_enum_warnings(&processor));

    if let Some(opts) = &request.document_options {
        if let Some(show_semantics) = opts.show_semantics {
            processor.show_semantics = show_semantics;
        }
        if let Some(inject_ast) = opts.inject_ast_indices {
            processor.set_inject_ast_indices(inject_ast);
        }
        if let Some(abbr_map) = opts.abbreviation_map.clone() {
            processor.abbreviation_map = Some(abbr_map);
        }
        if opts.integral_names.is_some() {
            warnings.push(Warning {
                level: WarningLevel::Warning,
                code: "integral_names_not_applied".to_string(),
                citation_id: None,
                ref_id: None,
                message: "document_options.integral_names is accepted but not yet wired through the processor; tracked in csl26-wq0y.".to_string(),
            });
        }
    }

    // Convert citations, recording missing-ref warnings and dropping items
    // whose reference IDs are absent from the bibliography. Citations with no
    // surviving items are kept as empty placeholders so the output preserves
    // input order and length.
    let mut citations: Vec<Citation> = Vec::new();
    for occ in request.citations {
        let mut citation: Citation = occ.into();
        citation.items.retain(|item| {
            if processor.bibliography.contains_key(&item.id) {
                true
            } else {
                warnings.push(Warning {
                    level: WarningLevel::Warning,
                    code: "missing_ref".to_string(),
                    citation_id: citation.id.clone(),
                    ref_id: Some(item.id.clone()),
                    message: format!("Reference '{}' not found in bibliography", item.id),
                });
                false
            }
        });
        citations.push(citation);
    }

    // Process citations
    let formatted_citations = match request.output_format {
        OutputFormatKind::Plain => format_by_kind::<PlainText>(&processor, &citations)?,
        OutputFormatKind::Html => format_by_kind::<Html>(&processor, &citations)?,
        OutputFormatKind::Djot => format_by_kind::<Djot>(&processor, &citations)?,
        OutputFormatKind::Latex => format_by_kind::<Latex>(&processor, &citations)?,
        OutputFormatKind::Typst => format_by_kind::<Typst>(&processor, &citations)?,
    };

    // Process bibliography
    let bibliography = match request.output_format {
        OutputFormatKind::Plain => format_bibliography::<PlainText>(
            &processor,
            request.output_format,
            request.document_options.as_ref(),
        )?,
        OutputFormatKind::Html => format_bibliography::<Html>(
            &processor,
            request.output_format,
            request.document_options.as_ref(),
        )?,
        OutputFormatKind::Djot => format_bibliography::<Djot>(
            &processor,
            request.output_format,
            request.document_options.as_ref(),
        )?,
        OutputFormatKind::Latex => format_bibliography::<Latex>(
            &processor,
            request.output_format,
            request.document_options.as_ref(),
        )?,
        OutputFormatKind::Typst => format_bibliography::<Typst>(
            &processor,
            request.output_format,
            request.document_options.as_ref(),
        )?,
    };

    Ok(FormatDocumentResult {
        formatted_citations,
        bibliography,
        warnings,
    })
}

/// Scan the bibliography for unknown reference classes and return compatibility warnings.
pub fn unknown_reference_class_warnings(bibliography: &Bibliography) -> Vec<Warning> {
    bibliography
        .iter()
        .filter_map(|(ref_id, reference)| {
            let ReferenceClass::Unknown(class) = reference.class() else {
                return None;
            };
            Some(Warning {
                level: WarningLevel::Warning,
                code: "unknown_reference_class".to_string(),
                citation_id: None,
                ref_id: Some(ref_id.clone()),
                message: format!(
                    "Reference '{ref_id}' uses unknown class '{class}'; rendering will use only fields this engine understands."
                ),
            })
        })
        .collect()
}

/// Scan the style and bibliography for unknown enum variants and term keys.
///
/// Returns a list of structured compatibility warnings for encounter of
/// unknown variants that were captured via the tolerant-enum mechanism.
pub fn unknown_enum_warnings(processor: &Processor) -> Vec<Warning> {
    let mut warnings = Vec::new();

    // 1. Scan bibliography
    for (ref_id, reference) in &processor.bibliography {
        match reference.extension() {
            ClassExtension::Monograph(r) => {
                if let MonographType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown monograph type '{s}'; rendering will use default monograph formatting."),
                    });
                }
            }
            ClassExtension::Collection(r) => {
                if let CollectionType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown collection type '{s}'; rendering will use default collection formatting."),
                    });
                }
            }
            ClassExtension::CollectionComponent(r) => {
                if let MonographComponentType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown monograph component type '{s}'; rendering will use default chapter formatting."),
                    });
                }
            }
            ClassExtension::SerialComponent(r) => {
                if let SerialComponentType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown serial component type '{s}'; rendering will use default article formatting."),
                    });
                }
            }
            _ => {}
        }

        for contributor in reference.all_contributor_entries() {
            if let ReferenceRole::Unknown(s) = &contributor.role {
                warnings.push(Warning {
                    level: WarningLevel::Warning,
                    code: "unknown_enum_variant".to_string(),
                    citation_id: None,
                    ref_id: Some(ref_id.clone()),
                    message: format!("Reference '{ref_id}' uses unknown contributor role '{s}'; this role may be ignored during rendering."),
                });
            }
        }
    }

    // 2. Scan Style
    if let Some(templates) = &processor.style.templates {
        for (name, template) in templates {
            scan_template_for_unknowns(template, &format!("template '{name}'"), &mut warnings);
        }
    }
    if let Some(citation) = &processor.style.citation
        && let Some(template) = &citation.template
    {
        scan_template_for_unknowns(template, "citation layout", &mut warnings);
    }
    if let Some(bib) = &processor.style.bibliography
        && let Some(template) = &bib.template
    {
        scan_template_for_unknowns(template, "bibliography layout", &mut warnings);
    }

    warnings
}

fn scan_template_for_unknowns(
    components: &[citum_schema::template::TemplateComponent],
    location: &str,
    warnings: &mut Vec<Warning>,
) {
    use citum_schema::template::TemplateComponent;
    for component in components {
        match component {
            TemplateComponent::Term(t) => {
                if let GeneralTerm::Unknown(s) = &t.term {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown locale term key '{s}'; this term may render as empty."),
                    });
                }
                if let Some(TermForm::Unknown(s)) = &t.form {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown term form '{s}'; falling back to long form."),
                    });
                }
            }
            TemplateComponent::Contributor(c) => {
                if let TemplateRole::Unknown(s) = &c.contributor {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown contributor role '{s}'; this role may be ignored."),
                    });
                }
            }
            TemplateComponent::Date(d) => {
                if let citum_schema::template::DateForm::Unknown(s) = &d.form {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown date form '{s}'; falling back to year only."),
                    });
                }
            }
            TemplateComponent::Group(g) => {
                scan_template_for_unknowns(&g.group, location, warnings);
            }
            _ => {}
        }
    }
}

/// Process citations and return formatted text.
fn format_by_kind<F>(
    processor: &Processor,
    citations: &[Citation],
) -> Result<Vec<FormattedCitation>, FormatDocumentError>
where
    F: OutputFormat<Output = String>,
{
    let texts = processor.process_citations_with_format::<F>(citations)?;

    let formatted = citations
        .iter()
        .zip(texts.iter())
        .map(|(citation, text)| {
            let ref_ids = citation.items.iter().map(|item| item.id.clone()).collect();
            FormattedCitation {
                id: citation.id.clone().unwrap_or_default(),
                text: text.clone(),
                ref_ids,
            }
        })
        .collect();

    Ok(formatted)
}

/// Format the bibliography by output kind.
fn format_bibliography<F>(
    processor: &Processor,
    format_kind: OutputFormatKind,
    doc_opts: Option<&DocumentOptions>,
) -> Result<FormattedBibliography, FormatDocumentError>
where
    F: OutputFormat<Output = String>,
{
    // Extract annotation map and style if present
    let (annotations, annotation_style) = if let Some(opts) = doc_opts {
        if let Some(anns) = &opts.annotations {
            let style = opts.annotation_format.as_ref().map(|fmt| AnnotationStyle {
                format: fmt.clone(),
            });
            (anns.clone(), style)
        } else {
            (HashMap::new(), None)
        }
    } else {
        (HashMap::new(), None)
    };

    // Render bibliography as string
    let content = if annotations.is_empty() {
        processor
            .render_bibliography_with_format_and_annotations::<F>(None, annotation_style.as_ref())
    } else {
        processor.render_bibliography_with_format_and_annotations::<F>(
            Some(&annotations),
            annotation_style.as_ref(),
        )
    };

    // Extract per-entry text in the requested output format and capture metadata.
    let proc_entries = processor.process_references().bibliography;
    let entries = proc_entries
        .into_iter()
        .map(|entry| {
            let entry_anns = if annotations.is_empty() {
                None
            } else {
                Some(&annotations)
            };
            let text = crate::render::bibliography::refs_to_string_with_format::<F>(
                vec![entry.clone()],
                entry_anns,
                annotation_style.as_ref(),
            );
            let metadata = EntryMetadata {
                author: entry.metadata.author.unwrap_or_default(),
                year: entry.metadata.year.unwrap_or_default(),
                title: entry.metadata.title.unwrap_or_default(),
            };
            BibliographyEntry {
                id: entry.id,
                text,
                metadata,
            }
        })
        .collect();

    Ok(FormattedBibliography {
        format: format_kind,
        content,
        entries,
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code uses assertions and panic"
)]
mod tests {
    use super::*;
    use crate::api::CitationOccurrenceItem;
    use crate::{
        Config, ContributorForm, ContributorRole, DateForm, Processing, Rendering,
        TemplateComponent, TemplateContributor, TemplateDate, TemplateDateVariable,
        WrapPunctuation,
    };
    use citum_schema::reference::{EdtfString, InputReference, Monograph, MonographType, Title};
    use citum_schema::{CitationSpec, StyleInfo};

    fn make_test_style() -> Style {
        Style {
            info: StyleInfo {
                title: Some("Test Style".to_string()),
                id: Some("test".into()),
                ..Default::default()
            },
            options: Some(Config {
                processing: Some(Processing::AuthorDate),
                ..Default::default()
            }),
            citation: Some(CitationSpec {
                template: Some(vec![
                    TemplateComponent::Contributor(TemplateContributor {
                        contributor: ContributorRole::Author,
                        form: ContributorForm::Short,
                        rendering: Rendering::default(),
                        ..Default::default()
                    }),
                    TemplateComponent::Date(TemplateDate {
                        date: TemplateDateVariable::Issued,
                        form: DateForm::Year,
                        rendering: Rendering::default(),
                        ..Default::default()
                    }),
                ]),
                wrap: Some(WrapPunctuation::Parentheses.into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_test_bibliography() -> Bibliography {
        let mut refs = Bibliography::new();
        refs.insert(
            "smith2020".to_string(),
            InputReference::Monograph(Box::new(Monograph {
                id: Some("smith2020".into()),
                r#type: MonographType::Book,
                title: Some(Title::Single("Sample Work".to_string())),
                issued: EdtfString("2020".to_string()),
                ..Default::default()
            })),
        );
        refs
    }

    #[test]
    fn format_document_with_style_empty_citations() {
        let style = make_test_style();
        let refs = make_test_bibliography();
        let request = FormatDocumentRequest {
            style: StyleInput::Yaml("dummy".to_string()),
            locale: None,
            output_format: OutputFormatKind::Plain,
            refs,
            citations: vec![],
            document_options: None,
        };

        let result = format_document_with_style(style, request);
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.formatted_citations.len(), 0);
    }

    #[test]
    fn format_document_missing_ref_warning() {
        let style = make_test_style();
        let refs = make_test_bibliography();

        let citation_occ = CitationOccurrence {
            id: "cite1".to_string(),
            items: vec![CitationOccurrenceItem {
                id: "unknown_ref".to_string(),
                locator: None,
                prefix: None,
                suffix: None,
                integral_name_state: None,
            }],
            mode: None,
            note_number: None,
            suppress_author: None,
            grouped: None,
            prefix: None,
            suffix: None,
        };

        let request = FormatDocumentRequest {
            style: StyleInput::Yaml("dummy".to_string()),
            locale: None,
            output_format: OutputFormatKind::Plain,
            refs,
            citations: vec![citation_occ],
            document_options: None,
        };

        let result = format_document_with_style(style, request);
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.warnings.iter().any(|w| w.code == "missing_ref"));
    }

    #[test]
    fn format_document_unknown_reference_class_warning() {
        let style = make_test_style();
        let mut refs = Bibliography::new();
        let unknown_ref: InputReference = serde_json::from_str(
            r#"{
                "class": "dance-performance",
                "id": "pina2011",
                "title": "Pina",
                "issued": "2011",
                "venue": "Berlin"
            }"#,
        )
        .expect("unknown class should parse through the compatibility path");
        refs.insert("pina2011".to_string(), unknown_ref);

        let citation_occ = CitationOccurrence {
            id: "cite1".to_string(),
            items: vec![CitationOccurrenceItem {
                id: "pina2011".to_string(),
                locator: None,
                prefix: None,
                suffix: None,
                integral_name_state: None,
            }],
            mode: None,
            note_number: None,
            suppress_author: None,
            grouped: None,
            prefix: None,
            suffix: None,
        };

        let request = FormatDocumentRequest {
            style: StyleInput::Yaml("dummy".to_string()),
            locale: None,
            output_format: OutputFormatKind::Plain,
            refs,
            citations: vec![citation_occ],
            document_options: None,
        };

        let result = format_document_with_style(style, request).unwrap();
        let warning = result
            .warnings
            .iter()
            .find(|w| w.code == "unknown_reference_class")
            .expect("unknown class warning should be emitted");
        assert_eq!(warning.ref_id.as_deref(), Some("pina2011"));
        assert!(warning.message.contains("dance-performance"));
    }

    #[test]
    fn format_document_yaml_style_input() {
        let style = make_test_style();
        let yaml_style = serde_yaml::to_string(&style).expect("serialize test style");

        let mut refs = Bibliography::new();
        refs.insert(
            "test2024".to_string(),
            InputReference::Monograph(Box::new(Monograph {
                id: Some("test2024".into()),
                r#type: MonographType::Book,
                title: Some(Title::Single("Test Work".to_string())),
                issued: EdtfString("2024".to_string()),
                ..Default::default()
            })),
        );

        let citation_occ = CitationOccurrence {
            id: "c1".to_string(),
            items: vec![CitationOccurrenceItem {
                id: "test2024".to_string(),
                locator: None,
                prefix: None,
                suffix: None,
                integral_name_state: None,
            }],
            mode: None,
            note_number: None,
            suppress_author: None,
            grouped: None,
            prefix: None,
            suffix: None,
        };

        let request = FormatDocumentRequest {
            style: StyleInput::Yaml(yaml_style),
            locale: None,
            output_format: OutputFormatKind::Plain,
            refs,
            citations: vec![citation_occ],
            document_options: None,
        };

        let result = format_document(request);
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.formatted_citations.len(), 1);
        assert!(!res.formatted_citations[0].text.is_empty());
    }

    #[test]
    fn format_document_uri_input_unresolved() {
        let request = FormatDocumentRequest {
            style: StyleInput::Uri("https://example.com/style.yaml".to_string()),
            locale: None,
            output_format: OutputFormatKind::Plain,
            refs: Bibliography::new(),
            citations: vec![],
            document_options: None,
        };

        let result = format_document(request);
        match result {
            Err(FormatDocumentError::UnresolvedInput(_)) => {
                // Expected
            }
            _ => panic!("Expected UnresolvedInput error"),
        }
    }
}
