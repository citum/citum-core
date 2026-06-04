/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Stateful document session API for interactive adapters.

use crate::render::djot::Djot;
use crate::render::html::Html;
use crate::render::latex::Latex;
use crate::render::markdown::Markdown;
use crate::render::plain::PlainText;
use crate::render::typst::Typst;
use citum_schema::Style;
use citum_schema::options::Processing;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::document::{format_bibliography, format_by_kind};
use super::{
    CitationOccurrence, CitationOccurrenceItem, DocumentOptions, FormatDocumentError,
    FormattedBibliography, FormattedCitation, OutputFormatKind, RefsInput, StyleInput, Warning,
    WarningLevel, unknown_enum_warnings, unknown_reference_class_warnings,
};
use crate::processor::Processor;
use crate::reference::Citation;

/// Position context for inserting or moving a citation in a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CitationInsertPosition {
    /// Citation ID that should precede the inserted citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_citation_id: Option<String>,
    /// Citation ID that should follow the inserted citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_citation_id: Option<String>,
}

/// Result returned when a new interactive session is opened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSessionResult {
    /// Opaque session identifier used by transport adapters.
    pub session_id: String,
}

/// Result returned by mutation methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMutationResult {
    /// Monotonic session version after the mutation.
    pub version: u64,
    /// Complete set of citations whose rendered output changed.
    pub affected_citations: Vec<FormattedCitation>,
    /// Current bibliography after the mutation.
    pub bibliography: FormattedBibliography,
    /// True when numeric citation labels or note numbers shifted.
    pub renumbering_occurred: bool,
    /// Non-fatal diagnostics encountered during rendering.
    pub warnings: Vec<Warning>,
}

/// Result returned by citation preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewCitationResult {
    /// Rendered preview text.
    pub preview: String,
    /// Non-fatal diagnostics encountered during preview rendering.
    pub warnings: Vec<Warning>,
}

/// Errors returned by the stateful session API.
#[derive(Debug)]
pub enum DocumentSessionError {
    /// The requested citation does not exist in the session.
    CitationNotFound(String),
    /// The requested insertion position is invalid.
    InvalidPosition(String),
    /// Rendering failed while recomputing session output.
    Format(FormatDocumentError),
}

impl std::fmt::Display for DocumentSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CitationNotFound(id) => write!(f, "citation not found: {id}"),
            Self::InvalidPosition(msg) => write!(f, "invalid citation position: {msg}"),
            Self::Format(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for DocumentSessionError {}

impl From<FormatDocumentError> for DocumentSessionError {
    fn from(err: FormatDocumentError) -> Self {
        Self::Format(err)
    }
}

/// Stateful facade over whole-document citation rendering.
#[derive(Debug, Clone)]
pub struct DocumentSession {
    style: Style,
    locale: Option<String>,
    output_format: OutputFormatKind,
    document_options: Option<DocumentOptions>,
    refs: Option<RefsInput>,
    citations: Vec<CitationOccurrence>,
    version: u64,
    formatted_citations: Vec<FormattedCitation>,
    bibliography: Option<FormattedBibliography>,
    warnings: Vec<Warning>,
}

impl DocumentSession {
    /// Create a session with an already-resolved style.
    pub fn new(
        style: Style,
        _style_input: StyleInput,
        locale: Option<String>,
        output_format: OutputFormatKind,
        document_options: Option<DocumentOptions>,
    ) -> Self {
        Self {
            style,
            locale,
            output_format,
            document_options,
            refs: None,
            citations: Vec::new(),
            version: 0,
            formatted_citations: Vec::new(),
            bibliography: None,
            warnings: Vec::new(),
        }
    }

    /// Return the current session version.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Replace the full reference set used by this session.
    pub fn put_references(&mut self, refs: RefsInput) {
        self.refs = Some(refs);
    }

    /// Replace the full ordered citation list.
    ///
    /// # Errors
    ///
    /// Returns an error when recomputing the formatted session output fails.
    pub fn insert_citations_batch(
        &mut self,
        citations: Vec<CitationOccurrence>,
    ) -> Result<SessionMutationResult, DocumentSessionError> {
        let old_citations = self.citations.clone();
        let old_formatted = self.formatted_citations.clone();
        self.citations = citations;
        self.commit_render(old_citations, old_formatted)
    }

    /// Insert a citation at the requested position.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested position is invalid or rendering fails.
    pub fn insert_citation(
        &mut self,
        citation: CitationOccurrence,
        position: Option<CitationInsertPosition>,
    ) -> Result<SessionMutationResult, DocumentSessionError> {
        let old_citations = self.citations.clone();
        let old_formatted = self.formatted_citations.clone();
        let index = self.resolve_insert_index(position.as_ref())?;
        self.citations.insert(index, citation);
        self.commit_render(old_citations, old_formatted)
    }

    /// Update an existing citation, optionally moving it to a new position.
    ///
    /// # Errors
    ///
    /// Returns an error when the citation does not exist, the requested
    /// position is invalid, or rendering fails.
    pub fn update_citation(
        &mut self,
        citation_id: &str,
        mut citation: CitationOccurrence,
        position: Option<CitationInsertPosition>,
    ) -> Result<SessionMutationResult, DocumentSessionError> {
        let current_index = self
            .citation_index(citation_id)
            .ok_or_else(|| DocumentSessionError::CitationNotFound(citation_id.to_string()))?;
        let old_citations = self.citations.clone();
        let old_formatted = self.formatted_citations.clone();
        citation.id = citation_id.to_string();
        self.citations.remove(current_index);
        let index = if let Some(position) = position.as_ref() {
            self.resolve_insert_index(Some(position))?
        } else {
            current_index.min(self.citations.len())
        };
        self.citations.insert(index, citation);
        self.commit_render(old_citations, old_formatted)
    }

    /// Delete a citation by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the citation does not exist or rendering fails.
    pub fn delete_citation(
        &mut self,
        citation_id: &str,
    ) -> Result<SessionMutationResult, DocumentSessionError> {
        let index = self
            .citation_index(citation_id)
            .ok_or_else(|| DocumentSessionError::CitationNotFound(citation_id.to_string()))?;
        let old_citations = self.citations.clone();
        let old_formatted = self.formatted_citations.clone();
        self.citations.remove(index);
        self.commit_render(old_citations, old_formatted)
    }

    /// Render a citation preview without mutating session state.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested preview position is invalid or
    /// rendering fails.
    pub fn preview_citation(
        &self,
        items: Vec<CitationOccurrenceItem>,
        position: Option<CitationInsertPosition>,
    ) -> Result<PreviewCitationResult, DocumentSessionError> {
        let mut citations = self.citations.clone();
        let index = self.resolve_insert_index_in(&citations, position.as_ref())?;
        let preview_id = "__citum_preview__".to_string();
        citations.insert(
            index,
            CitationOccurrence {
                id: preview_id.clone(),
                items,
                mode: None,
                note_number: None,
                suppress_author: None,
                grouped: None,
                prefix: None,
                suffix: None,
                sentence_start: None,
            },
        );
        let rendered = self.render_citations(&citations)?;
        let preview = rendered
            .formatted_citations
            .iter()
            .find(|citation| citation.id == preview_id)
            .map(|citation| citation.text.clone())
            .unwrap_or_default();
        Ok(PreviewCitationResult {
            preview,
            warnings: rendered.warnings,
        })
    }

    /// Return the current formatted citations.
    pub fn get_citations(&self) -> Vec<FormattedCitation> {
        self.formatted_citations.clone()
    }

    /// Return the current bibliography, if a mutation has rendered one.
    pub fn get_bibliography(&self) -> Option<FormattedBibliography> {
        self.bibliography.clone()
    }

    fn commit_render(
        &mut self,
        old_citations: Vec<CitationOccurrence>,
        old_formatted: Vec<FormattedCitation>,
    ) -> Result<SessionMutationResult, DocumentSessionError> {
        let rendered = self.render_citations(&self.citations)?;
        let affected_citations =
            diff_formatted_citations(&old_formatted, &rendered.formatted_citations);
        let renumbering_occurred = renumbering_occurred(
            &self.style,
            &old_citations,
            &self.citations,
            &old_formatted,
            &rendered.formatted_citations,
        );
        self.version += 1;
        self.formatted_citations = rendered.formatted_citations;
        self.bibliography = Some(rendered.bibliography.clone());
        self.warnings = rendered.warnings.clone();
        Ok(SessionMutationResult {
            version: self.version,
            affected_citations,
            bibliography: rendered.bibliography,
            renumbering_occurred,
            warnings: rendered.warnings,
        })
    }

    #[allow(
        clippy::too_many_lines,
        reason = "session rendering mirrors Tier 1 setup and format dispatch"
    )]
    fn render_citations(
        &self,
        citations: &[CitationOccurrence],
    ) -> Result<SessionRenderResult, FormatDocumentError> {
        let mut warnings = Vec::new();
        if let Some(tag) = &self.locale
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

        let bibliography = self
            .refs
            .clone()
            .unwrap_or_else(|| RefsInput::Json(serde_json::json!({})))
            .resolve_local()?;
        let mut processor = Processor::new(self.style.clone(), bibliography);
        warnings.extend(unknown_reference_class_warnings(&processor.bibliography));
        warnings.extend(unknown_enum_warnings(&processor));

        if let Some(opts) = &self.document_options {
            if let Some(show_semantics) = opts.show_semantics {
                processor.show_semantics = show_semantics;
            }
            if let Some(inject_ast) = opts.inject_ast_indices {
                processor.set_inject_ast_indices(inject_ast);
            }
            if let Some(abbr_map) = opts.abbreviation_map.clone() {
                processor.abbreviation_map = Some(abbr_map);
            }
            if opts.integral_name_memory.is_some() {
                warnings.push(Warning {
                    level: WarningLevel::Warning,
                    code: "integral_name_memory_not_applied".to_string(),
                    citation_id: None,
                    ref_id: None,
                    message: "document_options.integral_name_memory is accepted but not yet wired through the processor; tracked in csl26-ktq6.".to_string(),
                });
            }
        }

        let mut processor_citations: Vec<Citation> = Vec::new();
        for occ in citations.iter().cloned() {
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
            processor_citations.push(citation);
        }

        let formatted_citations = match self.output_format {
            OutputFormatKind::Plain => {
                format_by_kind::<PlainText>(&processor, &processor_citations)?
            }
            OutputFormatKind::Html => format_by_kind::<Html>(&processor, &processor_citations)?,
            OutputFormatKind::Djot => format_by_kind::<Djot>(&processor, &processor_citations)?,
            OutputFormatKind::Latex => format_by_kind::<Latex>(&processor, &processor_citations)?,
            OutputFormatKind::Typst => format_by_kind::<Typst>(&processor, &processor_citations)?,
            OutputFormatKind::Markdown => {
                format_by_kind::<Markdown>(&processor, &processor_citations)?
            }
        };
        let bibliography = match self.output_format {
            OutputFormatKind::Plain => format_bibliography::<PlainText>(
                &processor,
                self.output_format,
                self.document_options.as_ref(),
            )?,
            OutputFormatKind::Html => format_bibliography::<Html>(
                &processor,
                self.output_format,
                self.document_options.as_ref(),
            )?,
            OutputFormatKind::Djot => format_bibliography::<Djot>(
                &processor,
                self.output_format,
                self.document_options.as_ref(),
            )?,
            OutputFormatKind::Latex => format_bibliography::<Latex>(
                &processor,
                self.output_format,
                self.document_options.as_ref(),
            )?,
            OutputFormatKind::Typst => format_bibliography::<Typst>(
                &processor,
                self.output_format,
                self.document_options.as_ref(),
            )?,
            OutputFormatKind::Markdown => format_bibliography::<Markdown>(
                &processor,
                self.output_format,
                self.document_options.as_ref(),
            )?,
        };

        Ok(SessionRenderResult {
            formatted_citations,
            bibliography,
            warnings,
        })
    }

    fn citation_index(&self, citation_id: &str) -> Option<usize> {
        self.citations
            .iter()
            .position(|citation| citation.id == citation_id)
    }

    fn resolve_insert_index(
        &self,
        position: Option<&CitationInsertPosition>,
    ) -> Result<usize, DocumentSessionError> {
        self.resolve_insert_index_in(&self.citations, position)
    }

    fn resolve_insert_index_in(
        &self,
        citations: &[CitationOccurrence],
        position: Option<&CitationInsertPosition>,
    ) -> Result<usize, DocumentSessionError> {
        let Some(position) = position else {
            return Ok(citations.len());
        };
        match (&position.after_citation_id, &position.before_citation_id) {
            (None, None) => Ok(citations.len()),
            (Some(after), None) => citations
                .iter()
                .position(|citation| citation.id == *after)
                .map(|index| index + 1)
                .ok_or_else(|| {
                    DocumentSessionError::InvalidPosition(format!(
                        "unknown after_citation_id '{after}'"
                    ))
                }),
            (None, Some(before)) => citations
                .iter()
                .position(|citation| citation.id == *before)
                .ok_or_else(|| {
                    DocumentSessionError::InvalidPosition(format!(
                        "unknown before_citation_id '{before}'"
                    ))
                }),
            (Some(after), Some(before)) => {
                let after_index = citations
                    .iter()
                    .position(|citation| citation.id == *after)
                    .ok_or_else(|| {
                        DocumentSessionError::InvalidPosition(format!(
                            "unknown after_citation_id '{after}'"
                        ))
                    })?;
                let before_index = citations
                    .iter()
                    .position(|citation| citation.id == *before)
                    .ok_or_else(|| {
                        DocumentSessionError::InvalidPosition(format!(
                            "unknown before_citation_id '{before}'"
                        ))
                    })?;
                if after_index + 1 == before_index {
                    Ok(before_index)
                } else {
                    Err(DocumentSessionError::InvalidPosition(format!(
                        "after_citation_id '{after}' and before_citation_id '{before}' are not adjacent"
                    )))
                }
            }
        }
    }
}

#[derive(Debug)]
struct SessionRenderResult {
    formatted_citations: Vec<FormattedCitation>,
    bibliography: FormattedBibliography,
    warnings: Vec<Warning>,
}

fn diff_formatted_citations(
    old: &[FormattedCitation],
    new: &[FormattedCitation],
) -> Vec<FormattedCitation> {
    let old_by_id: HashMap<&str, &FormattedCitation> = old
        .iter()
        .map(|citation| (citation.id.as_str(), citation))
        .collect();
    new.iter()
        .filter(|citation| {
            old_by_id.get(citation.id.as_str()).is_none_or(|previous| {
                previous.text != citation.text || previous.ref_ids != citation.ref_ids
            })
        })
        .cloned()
        .collect()
}

fn renumbering_occurred(
    style: &Style,
    old_citations: &[CitationOccurrence],
    new_citations: &[CitationOccurrence],
    old_formatted: &[FormattedCitation],
    new_formatted: &[FormattedCitation],
) -> bool {
    if note_numbers_shifted(old_citations, new_citations) {
        return true;
    }
    if !uses_numeric_labels(style) {
        return false;
    }
    let old_by_id: HashMap<&str, &FormattedCitation> = old_formatted
        .iter()
        .map(|citation| (citation.id.as_str(), citation))
        .collect();
    let old_occurrences_by_id: HashMap<&str, &CitationOccurrence> = old_citations
        .iter()
        .map(|citation| (citation.id.as_str(), citation))
        .collect();
    let new_occurrences_by_id: HashMap<&str, &CitationOccurrence> = new_citations
        .iter()
        .map(|citation| (citation.id.as_str(), citation))
        .collect();
    new_formatted.iter().any(|citation| {
        let Some(previous) = old_by_id.get(citation.id.as_str()) else {
            return false;
        };
        if previous.text == citation.text {
            return false;
        }
        let Some(old_occurrence) = old_occurrences_by_id.get(citation.id.as_str()) else {
            return false;
        };
        let Some(new_occurrence) = new_occurrences_by_id.get(citation.id.as_str()) else {
            return false;
        };
        *old_occurrence == *new_occurrence
    })
}

fn note_numbers_shifted(
    old_citations: &[CitationOccurrence],
    new_citations: &[CitationOccurrence],
) -> bool {
    let old_by_id: HashMap<&str, Option<u32>> = old_citations
        .iter()
        .map(|citation| (citation.id.as_str(), citation.note_number))
        .collect();
    new_citations.iter().any(|citation| {
        old_by_id
            .get(citation.id.as_str())
            .is_some_and(|old_note_number| *old_note_number != citation.note_number)
    })
}

fn uses_numeric_labels(style: &Style) -> bool {
    matches!(
        style
            .options
            .as_ref()
            .and_then(|options| options.processing.as_ref()),
        Some(Processing::Numeric | Processing::Label(_))
    )
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
    use crate::reference::Bibliography;
    use crate::{
        Config, Contributor, ContributorForm, ContributorList, ContributorRole, DateForm,
        MultilingualString, Processing, Rendering, StructuredName, TemplateDateVariable,
    };
    use citum_schema::reference::{EdtfString, InputReference, Monograph, MonographType, Title};
    use citum_schema::{
        CitationSpec, StyleInfo, TemplateComponent, TemplateContributor, TemplateDate,
        WrapPunctuation,
    };

    fn style() -> Style {
        Style {
            info: StyleInfo {
                title: Some("Session Test Style".to_string()),
                id: Some("session-test".into()),
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
                        rendering: Rendering {
                            prefix: Some(", ".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ]),
                wrap: Some(WrapPunctuation::Parentheses.into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn numeric_style() -> Style {
        Style {
            info: StyleInfo {
                title: Some("Numeric Session Test Style".to_string()),
                id: Some("numeric-session-test".into()),
                ..Default::default()
            },
            options: Some(Config {
                processing: Some(Processing::Numeric),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn refs() -> RefsInput {
        let mut refs = Bibliography::new();
        refs.insert(
            "smith2020".to_string(),
            reference("smith2020", "Smith", "2020"),
        );
        refs.insert("doe2021".to_string(), reference("doe2021", "Doe", "2021"));
        refs.insert("roe2022".to_string(), reference("roe2022", "Roe", "2022"));
        RefsInput::Json(serde_json::to_value(refs).expect("refs should serialize"))
    }

    fn reference(id: &str, family: &str, issued: &str) -> InputReference {
        InputReference::Monograph(Box::new(Monograph {
            id: Some(id.into()),
            r#type: MonographType::Book,
            title: Some(Title::Single(format!("{family} Work"))),
            author: Some(Contributor::ContributorList(ContributorList(vec![
                Contributor::StructuredName(StructuredName {
                    family: MultilingualString::Simple(family.to_string()),
                    given: MultilingualString::Simple("Alex".to_string()),
                    suffix: None,
                    dropping_particle: None,
                    non_dropping_particle: None,
                }),
            ]))),
            issued: EdtfString(issued.to_string()),
            ..Default::default()
        }))
    }

    fn citation(citation_id: &str, ref_id: &str) -> CitationOccurrence {
        CitationOccurrence {
            id: citation_id.to_string(),
            items: vec![CitationOccurrenceItem {
                id: ref_id.to_string(),
                locator: None,
                prefix: None,
                suffix: None,
                integral_name_state: None,
                org_abbreviation_state: None,
            }],
            mode: None,
            note_number: None,
            suppress_author: None,
            grouped: None,
            prefix: None,
            suffix: None,
            sentence_start: None,
        }
    }

    fn formatted(citation_id: &str, text: &str) -> FormattedCitation {
        FormattedCitation {
            id: citation_id.to_string(),
            text: text.to_string(),
            ref_ids: vec!["smith2020".to_string()],
        }
    }

    fn session() -> DocumentSession {
        let mut session = DocumentSession::new(
            style(),
            StyleInput::Yaml(String::new()),
            None,
            OutputFormatKind::Plain,
            None,
        );
        session.put_references(refs());
        session
    }

    #[test]
    fn session_batch_insert_returns_complete_changed_set() {
        let mut session = session();
        let result = session
            .insert_citations_batch(vec![citation("c1", "smith2020"), citation("c2", "doe2021")])
            .expect("batch insert should render");

        assert_eq!(result.version, 1);
        assert_eq!(result.affected_citations.len(), 2);
        assert_eq!(session.get_citations().len(), 2);
        assert!(!result.renumbering_occurred);
    }

    #[test]
    fn author_date_insert_does_not_report_renumbering() {
        let mut session = session();
        session
            .insert_citations_batch(vec![citation("c1", "smith2020"), citation("c2", "doe2021")])
            .expect("batch insert should render");
        let result = session
            .insert_citation(
                citation("c0", "roe2022"),
                Some(CitationInsertPosition {
                    after_citation_id: None,
                    before_citation_id: Some("c1".to_string()),
                }),
            )
            .expect("insert should render");

        assert!(!result.renumbering_occurred);
        assert_eq!(
            result
                .affected_citations
                .iter()
                .map(|citation| citation.id.as_str())
                .collect::<Vec<_>>(),
            vec!["c0"]
        );
    }

    #[test]
    fn note_number_shift_reports_renumbering() {
        let mut session = session();
        let mut first = citation("c1", "smith2020");
        first.note_number = Some(1);
        session
            .insert_citations_batch(vec![first])
            .expect("batch insert should render");
        let mut updated = citation("c1", "smith2020");
        updated.note_number = Some(2);
        let result = session
            .update_citation("c1", updated, None)
            .expect("update should render");

        assert!(result.renumbering_occurred);
    }

    #[test]
    fn numeric_own_payload_edit_does_not_report_renumbering() {
        let old = citation("c1", "smith2020");
        let mut new = old.clone();
        new.suffix = Some(", p. 12".to_string());

        assert!(!renumbering_occurred(
            &numeric_style(),
            &[old],
            &[new],
            &[formatted("c1", "[1]")],
            &[formatted("c1", "[1], p. 12")],
        ));
    }

    #[test]
    fn numeric_unchanged_existing_output_shift_reports_renumbering() {
        let unchanged = citation("c1", "smith2020");

        assert!(renumbering_occurred(
            &numeric_style(),
            std::slice::from_ref(&unchanged),
            std::slice::from_ref(&unchanged),
            &[formatted("c1", "[1]")],
            &[formatted("c1", "[2]")],
        ));
    }

    #[test]
    fn preview_does_not_mutate_session() {
        let mut session = session();
        session
            .insert_citations_batch(vec![citation("c1", "smith2020")])
            .expect("batch insert should render");
        let before_version = session.version();
        let before_citations = session.get_citations();

        let preview = session
            .preview_citation(citation("preview", "doe2021").items, None)
            .expect("preview should render");

        assert!(!preview.preview.is_empty());
        assert_eq!(session.version(), before_version);
        assert_eq!(session.get_citations().len(), before_citations.len());
    }

    /// Two sessions opened from the same base style but with different overrides
    /// must produce divergent output for the same two-author citation.
    #[test]
    fn session_style_override_produces_divergent_output() {
        use crate::api::apply_style_overrides;
        use citum_schema::options::{AndOptions, ContributorConfig};

        // base style with explicit `and: text`
        let mut base_style = style();
        assert!(
            base_style.options.is_some(),
            "style() must return options: Some(...) for this test's contributor setup to take effect"
        );
        if let Some(opts) = base_style.options.as_mut() {
            opts.contributors = Some(ContributorConfig {
                and: Some(AndOptions::Text),
                ..Default::default()
            });
        }

        // two-author reference via inline YAML
        let two_author_refs = RefsInput::Yaml(
            r#"duo2024:
  class: monograph
  id: duo2024
  type: book
  title: Duo Work
  issued: "2024"
  author:
    - family: Smith
      given: Alice
    - family: Jones
      given: Bob
"#
            .to_string(),
        );

        // session 1: no override — uses "and" text
        let mut session_base = DocumentSession::new(
            base_style.clone(),
            StyleInput::Yaml(String::new()),
            None,
            OutputFormatKind::Plain,
            None,
        );
        session_base.put_references(two_author_refs.clone());
        let result_base = session_base
            .insert_citations_batch(vec![citation("c1", "duo2024")])
            .expect("base session should render");
        let text_base = result_base.affected_citations[0].text.clone();

        // session 2: override switches to "&" symbol
        let mut style_overridden = base_style.clone();
        apply_style_overrides(
            &mut style_overridden,
            "options:\n  contributors:\n    and: symbol\n",
        )
        .expect("override should parse");
        let mut session_override = DocumentSession::new(
            style_overridden,
            StyleInput::Yaml(String::new()),
            None,
            OutputFormatKind::Plain,
            None,
        );
        session_override.put_references(two_author_refs);
        let result_override = session_override
            .insert_citations_batch(vec![citation("c1", "duo2024")])
            .expect("override session should render");
        let text_override = result_override.affected_citations[0].text.clone();

        assert!(
            text_base.contains("and"),
            "base session should use text 'and', got: {text_base:?}"
        );
        assert!(
            text_override.contains('&'),
            "override session should use '&', got: {text_override:?}"
        );
        assert_ne!(
            text_base, text_override,
            "sessions with different overrides should produce different output"
        );
    }
}
