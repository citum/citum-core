/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citation rendering orchestration.
//!
//! This module resolves the effective citation spec for each citation, prepares
//! renderer delimiters and affixes. Template-level rendering, including
//! sentence-initial note-start handling, lives in `rendering`.

use super::Processor;
use super::rendering::{CompoundRenderData, Renderer, RendererResources};
use crate::error::ProcessorError;
use crate::reference::Citation;
use citum_schema::NoteStartTextCase;
use citum_schema::locale::{GeneralTerm, Locale, TermForm};
use citum_schema::template::DelimiterPunctuation;

fn join_integral_groups(rendered_groups: Vec<String>, locale: &Locale) -> String {
    match rendered_groups.len() {
        0 => String::new(),
        1 => rendered_groups.into_iter().next().unwrap_or_default(),
        2 => {
            let conjunction = locale
                .resolved_general_term(&GeneralTerm::And, TermForm::Long)
                .unwrap_or_else(|| locale.and_term(false).to_string());
            rendered_groups.join(&format!(" {} ", conjunction.trim()))
        }
        _ => {
            let conjunction = locale
                .resolved_general_term(&GeneralTerm::And, TermForm::Long)
                .unwrap_or_else(|| locale.and_term(false).to_string());
            let final_delimiter = if locale.grammar_options.serial_comma {
                format!(", {} ", conjunction.trim())
            } else {
                format!(" {} ", conjunction.trim())
            };

            let mut rendered_groups = rendered_groups;
            let last = rendered_groups.pop().unwrap_or_default();
            format!("{}{}{}", rendered_groups.join(", "), final_delimiter, last)
        }
    }
}

impl Processor {
    fn sentence_initial_note_start_text_case(
        &self,
        citation: &Citation,
        effective_spec: &citum_schema::CitationSpec,
    ) -> Option<NoteStartTextCase> {
        let spec_prefix = effective_spec.prefix.as_deref().unwrap_or("");
        if self.is_note_style()
            && matches!(
                citation.position,
                Some(
                    citum_schema::citation::Position::Ibid
                        | citum_schema::citation::Position::IbidWithLocator
                )
            )
            && matches!(
                citation.mode,
                citum_schema::citation::CitationMode::NonIntegral
            )
            && citation.prefix.as_deref().unwrap_or("").is_empty()
            && spec_prefix.is_empty()
        {
            effective_spec.note_start_text_case
        } else {
            None
        }
    }

    fn resolve_positioned_citation_spec(
        &self,
        citation: &Citation,
    ) -> std::borrow::Cow<'_, citum_schema::CitationSpec> {
        self.style.citation.as_ref().map_or_else(
            || std::borrow::Cow::Owned(citum_schema::CitationSpec::default()),
            |spec| spec.resolve_for_position(citation.position.as_ref()),
        )
    }

    fn track_cited_ids_and_init_numbers(&self, citation: &Citation) {
        self.initialize_numeric_citation_numbers();
        let mut cited_ids = self.cited_ids.borrow_mut();
        for item in &citation.items {
            cited_ids.insert(item.id.clone());
        }
    }

    fn resolve_effective_citation_spec(&self, citation: &Citation) -> citum_schema::CitationSpec {
        self.resolve_positioned_citation_spec(citation)
            .into_owned()
            .resolve_for_mode(&citation.mode)
            .into_owned()
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
        note_start_text_case: Option<NoteStartTextCase>,
    ) -> Result<String, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let sorted_items = self.sort_citation_items(citation.items.clone(), effective_spec);
        let citation_config = self.get_citation_config();
        let renderer = Renderer::new(
            RendererResources {
                style: &self.style,
                bibliography: &self.bibliography,
                locale: &self.locale,
                config: &citation_config,
                bibliography_config: Some(self.get_bibliography_options().into_owned()),
            },
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
            self.show_semantics,
            self.inject_ast_indices,
        );
        let processing = citation_config.processing.clone().unwrap_or_default();
        let has_explicit_integral_multi_cite_delimiter = matches!(
            citation.mode,
            citum_schema::citation::CitationMode::Integral
        ) && self
            .resolve_positioned_citation_spec(citation)
            .integral
            .as_ref()
            .and_then(|spec| spec.multi_cite_delimiter.as_ref())
            .is_some();
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
                note_start_text_case,
            )?
        } else {
            renderer.render_grouped_citation_with_format::<F>(
                &sorted_items,
                effective_spec,
                &citation.mode,
                renderer_delimiter,
                citation.suppress_author,
                citation.position.as_ref(),
                note_start_text_case,
            )?
        };

        Ok(
            if matches!(
                citation.mode,
                citum_schema::citation::CitationMode::Integral
            ) && !has_explicit_integral_multi_cite_delimiter
            {
                join_integral_groups(rendered_groups, &self.locale)
            } else {
                F::default().join(rendered_groups, renderer_inter_delimiter)
            },
        )
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
                format!("{citation_prefix} ")
            } else {
                citation_prefix.to_string()
            };

        let formatted_suffix =
            if !citation_suffix.is_empty() && !citation_suffix.starts_with(char::is_whitespace) {
                format!(" {citation_suffix}")
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
        } else if let Some(wrap) = effective_spec.wrap.as_ref() {
            let inner_prefix = wrap.inner_prefix.as_deref().unwrap_or("");
            let inner_suffix = wrap.inner_suffix.as_deref().unwrap_or("");
            let inner_wrapped = if !inner_prefix.is_empty() || !inner_suffix.is_empty() {
                fmt.inner_affix(inner_prefix, output, inner_suffix)
            } else {
                output
            };
            fmt.wrap_punctuation(&wrap.punctuation, inner_wrapped)
        } else if !spec_prefix.is_empty() || !spec_suffix.is_empty() {
            fmt.affix(spec_prefix, output, spec_suffix)
        } else {
            output
        }
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
    /// position, renders the citation body, and applies input and style affixes.
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
        let note_start_text_case =
            self.sentence_initial_note_start_text_case(citation, &effective_spec);
        let (renderer_delimiter, renderer_inter_delimiter) =
            self.resolve_citation_delimiters(&effective_spec);
        let content = self.render_citation_content::<F>(
            citation,
            &effective_spec,
            renderer_delimiter,
            renderer_inter_delimiter,
            note_start_text_case,
        )?;
        let output = self.apply_citation_input_affixes(citation, content, &fmt);
        let wrapped = self.apply_spec_wrap_and_affixes(citation, &effective_spec, output, &fmt);

        Ok(fmt.finish(wrapped))
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
