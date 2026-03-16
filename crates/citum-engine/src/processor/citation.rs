/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citation rendering orchestration.
//!
//! This module resolves the effective citation spec for each citation, prepares
//! renderer delimiters and affixes, and applies note-style casing rules to the
//! final output. Template-level rendering still lives in `rendering`.

use super::Processor;
use super::rendering::{CompoundRenderData, Renderer};
use crate::error::ProcessorError;
use crate::reference::Citation;
use citum_schema::NoteStartTextCase;
use citum_schema::template::{DelimiterPunctuation, WrapPunctuation};

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn apply_note_start_text_case(value: &str, text_case: NoteStartTextCase) -> String {
    match text_case {
        NoteStartTextCase::CapitalizeFirst => capitalize_first(value),
        NoteStartTextCase::Lowercase => value.to_lowercase(),
    }
}

/// Apply note-start casing to the first visible text node in rendered citation output.
///
/// This preserves any leading markup so note-style case adjustments only touch
/// the user-visible text content.
pub(crate) fn apply_note_start_text_case_to_leading_text_node(
    rendered: &str,
    text_case: NoteStartTextCase,
) -> String {
    let mut in_tag = false;
    let mut text_start = None;

    for (index, ch) in rendered.char_indices() {
        match ch {
            '<' if !in_tag => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag && !ch.is_whitespace() => {
                text_start = Some(index);
                break;
            }
            _ => {}
        }
    }

    let Some(text_start) = text_start else {
        return rendered.to_string();
    };

    let text_end = rendered[text_start..]
        .find('<')
        .map(|offset| text_start + offset)
        .unwrap_or(rendered.len());

    let mut result = String::with_capacity(rendered.len());
    result.push_str(&rendered[..text_start]);
    result.push_str(&apply_note_start_text_case(
        &rendered[text_start..text_end],
        text_case,
    ));
    result.push_str(&rendered[text_end..]);
    result
}

impl Processor {
    fn track_cited_ids_and_init_numbers(&self, citation: &Citation) {
        self.initialize_numeric_citation_numbers();
        let mut cited_ids = self.cited_ids.borrow_mut();
        for item in &citation.items {
            cited_ids.insert(item.id.clone());
        }
    }

    fn resolve_effective_citation_spec(&self, citation: &Citation) -> citum_schema::CitationSpec {
        self.style
            .citation
            .as_ref()
            .map_or_else(citum_schema::CitationSpec::default, |spec| {
                spec.resolve_for_position(citation.position.as_ref())
                    .into_owned()
                    .resolve_for_mode(&citation.mode)
                    .into_owned()
            })
    }

    fn resolve_citation_delimiters<'a>(
        &self,
        effective_spec: &'a citum_schema::CitationSpec,
    ) -> (&'a str, &'a str) {
        let intra_delimiter = effective_spec.delimiter.as_deref().unwrap_or(", ");
        let inter_delimiter = effective_spec
            .multi_cite_delimiter
            .as_deref()
            .unwrap_or("; ");

        (
            if matches!(
                DelimiterPunctuation::from_csl_string(intra_delimiter),
                DelimiterPunctuation::None
            ) {
                ""
            } else {
                intra_delimiter
            },
            if matches!(
                DelimiterPunctuation::from_csl_string(inter_delimiter),
                DelimiterPunctuation::None
            ) {
                ""
            } else {
                inter_delimiter
            },
        )
    }

    fn render_citation_content<F>(
        &self,
        citation: &Citation,
        effective_spec: &citum_schema::CitationSpec,
        renderer_delimiter: &str,
        renderer_inter_delimiter: &str,
    ) -> Result<String, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let sorted_items = self.sort_citation_items(citation.items.clone(), effective_spec);
        let citation_config = self.get_citation_config();
        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            &citation_config,
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
            self.show_semantics,
        );
        let processing = citation_config.processing.clone().unwrap_or_default();
        let rendered_groups = if matches!(
            processing,
            citum_schema::options::Processing::Numeric
                | citum_schema::options::Processing::Label(_)
        ) {
            renderer.render_ungrouped_citation_with_format::<F>(
                &sorted_items,
                effective_spec,
                &citation.mode,
                renderer_delimiter,
                citation.suppress_author,
                citation.position.as_ref(),
            )?
        } else {
            renderer.render_grouped_citation_with_format::<F>(
                &sorted_items,
                effective_spec,
                &citation.mode,
                renderer_delimiter,
                citation.suppress_author,
                citation.position.as_ref(),
            )?
        };

        Ok(F::default().join(rendered_groups, renderer_inter_delimiter))
    }

    fn apply_citation_input_affixes<F>(
        &self,
        citation: &Citation,
        content: String,
        fmt: &F,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let citation_prefix = citation.prefix.as_deref().unwrap_or("");
        let citation_suffix = citation.suffix.as_deref().unwrap_or("");

        if citation_prefix.is_empty() && citation_suffix.is_empty() {
            return content;
        }

        let formatted_prefix =
            if !citation_prefix.is_empty() && !citation_prefix.ends_with(char::is_whitespace) {
                format!("{} ", citation_prefix)
            } else {
                citation_prefix.to_string()
            };

        let formatted_suffix =
            if !citation_suffix.is_empty() && !citation_suffix.starts_with(char::is_whitespace) {
                format!(" {}", citation_suffix)
            } else {
                citation_suffix.to_string()
            };

        fmt.affix(&formatted_prefix, content, &formatted_suffix)
    }

    fn apply_spec_wrap_and_affixes<F>(
        &self,
        citation: &Citation,
        effective_spec: &citum_schema::CitationSpec,
        output: String,
        fmt: &F,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let wrap = effective_spec
            .wrap
            .as_ref()
            .unwrap_or(&WrapPunctuation::None);
        let spec_prefix = effective_spec.prefix.as_deref().unwrap_or("");
        let spec_suffix = effective_spec.suffix.as_deref().unwrap_or("");

        if matches!(
            citation.mode,
            citum_schema::citation::CitationMode::Integral
        ) {
            if !spec_prefix.is_empty() || !spec_suffix.is_empty() {
                fmt.affix(spec_prefix, output, spec_suffix)
            } else {
                output
            }
        } else if *wrap != WrapPunctuation::None {
            fmt.wrap_punctuation(wrap, output)
        } else if !spec_prefix.is_empty() || !spec_suffix.is_empty() {
            fmt.affix(spec_prefix, output, spec_suffix)
        } else {
            output
        }
    }

    fn apply_note_start_case_if_needed(
        &self,
        citation: &Citation,
        effective_spec: &citum_schema::CitationSpec,
        rendered: String,
    ) -> String {
        let spec_prefix = effective_spec.prefix.as_deref().unwrap_or("");
        if self.is_note_style()
            && matches!(
                citation.position,
                Some(citum_schema::citation::Position::Ibid)
                    | Some(citum_schema::citation::Position::IbidWithLocator)
            )
            && matches!(
                citation.mode,
                citum_schema::citation::CitationMode::NonIntegral
            )
            && citation.prefix.as_deref().unwrap_or("").is_empty()
            && spec_prefix.is_empty()
            && let Some(text_case) = effective_spec.note_start_text_case
        {
            return apply_note_start_text_case_to_leading_text_node(&rendered, text_case);
        }

        rendered
    }

    /// Render a single citation to plain text.
    ///
    /// This is the primary entry point for citation processing. It handles:
    /// 1. Looking up references in the bibliography.
    /// 2. Annotating positions (ibid, subsequent, etc.).
    /// 3. Resolving disambiguation (name expansion, year suffixes).
    /// 4. Applying the style's citation template.
    ///
    /// Returns the formatted citation string or an error if processing fails.
    ///
    /// # Errors
    ///
    /// Returns an error when referenced items are missing or rendering fails.
    pub fn process_citation(&self, citation: &Citation) -> Result<String, ProcessorError> {
        self.process_citation_with_format::<crate::render::plain::PlainText>(citation)
    }

    /// Render a citation to a string using a specific output format.
    ///
    /// This resolves the effective citation spec for the citation's mode and
    /// position, renders the citation body, applies input and style affixes,
    /// and finally applies note-style casing when required.
    ///
    /// # Errors
    ///
    /// Returns an error when referenced items are missing or rendering fails.
    pub fn process_citation_with_format<F>(
        &self,
        citation: &Citation,
    ) -> Result<String, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        self.track_cited_ids_and_init_numbers(citation);

        let effective_spec = self.resolve_effective_citation_spec(citation);
        let (renderer_delimiter, renderer_inter_delimiter) =
            self.resolve_citation_delimiters(&effective_spec);
        let content = self.render_citation_content::<F>(
            citation,
            &effective_spec,
            renderer_delimiter,
            renderer_inter_delimiter,
        )?;
        let output = self.apply_citation_input_affixes(citation, content, &fmt);
        let wrapped = self.apply_spec_wrap_and_affixes(citation, &effective_spec, output, &fmt);

        Ok(self.apply_note_start_case_if_needed(citation, &effective_spec, fmt.finish(wrapped)))
    }

    /// Render multiple citations in document order.
    ///
    /// For note-based styles, normalizes context and assigns citation positions.
    ///
    /// # Errors
    ///
    /// Returns an error when any citation in the sequence fails to render.
    pub fn process_citations(&self, citations: &[Citation]) -> Result<Vec<String>, ProcessorError> {
        self.process_citations_with_format::<crate::render::plain::PlainText>(citations)
    }

    /// Render multiple citations with a custom output format.
    ///
    /// # Errors
    ///
    /// Returns an error when any citation in the sequence fails to render.
    pub fn process_citations_with_format<F>(
        &self,
        citations: &[Citation],
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut normalized = self.normalize_note_context(citations);
        self.annotate_positions(&mut normalized);
        normalized
            .iter()
            .map(|citation| self.process_citation_with_format::<F>(citation))
            .collect()
    }
}
