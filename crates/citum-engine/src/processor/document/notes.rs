/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Note-style document rendering and numbering helpers.

use super::note_support::{
    GeneratedNote, NoteOccurrence, NoteRule, adjust_manual_note_citation_rendering,
    assign_note_numbers, build_note_order_indices, collect_note_occurrences, language_tag,
    merge_note_rule, ordered_note_citations_and_contexts, render_note_reference_in_prose,
};
use super::output::HtmlPlaceholderRegistry;
use super::{CitationPlacement, ParsedDocument};
use crate::Citation;
use crate::processor::Processor;
use crate::processor::rendering::{CompoundRenderData, Renderer, RendererResources};
use std::collections::HashMap;

impl Processor {
    pub(super) fn process_note_document<F>(
        &self,
        content: &str,
        mut parsed: ParsedDocument,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let (generated_notes, rendered_notes) =
            self.prepare_note_citations::<F>(content, &mut parsed);
        let generated_note_by_index: HashMap<usize, &GeneratedNote> = generated_notes
            .iter()
            .map(|note| (note.citation_index, note))
            .collect();
        let note_rule = self.note_rule();

        let mut result = String::new();
        let mut last_idx = 0;
        for (index, parsed_citation) in parsed.citations.iter().enumerate() {
            result.push_str(&content[last_idx..parsed_citation.start]);
            match &parsed_citation.placement {
                CitationPlacement::ManualFootnote { .. } => {
                    if let Some(rendered) = rendered_notes.get(&index) {
                        result.push_str(rendered);
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                    }
                    last_idx = parsed_citation.end;
                }
                CitationPlacement::InlineProse => {
                    if let Some(note) = generated_note_by_index.get(&index) {
                        if matches!(
                            parsed_citation.citation.mode,
                            citum_schema::citation::CitationMode::Integral
                        ) && let Ok(anchor) = self
                            .render_note_integral_anchor_with_format::<F>(&parsed_citation.citation)
                        {
                            result.push_str(&anchor);
                        }
                        let consumed_right = render_note_reference_in_prose(
                            &mut result,
                            &content[parsed_citation.end..],
                            &format!("[^{}]", note.label),
                            note_rule,
                        );
                        last_idx = parsed_citation.end + consumed_right;
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                        last_idx = parsed_citation.end;
                    }
                }
            }
        }
        result.push_str(&content[last_idx..]);

        if !generated_notes.is_empty() {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');

            for note in &generated_notes {
                if let Some(rendered) = rendered_notes.get(&note.citation_index) {
                    result.push_str(&format!("[^{}]: {}\n", note.label, rendered));
                }
            }
        }

        result
    }

    pub(super) fn process_note_document_html(
        &self,
        content: &str,
        mut parsed: ParsedDocument,
        placeholders: &mut HtmlPlaceholderRegistry,
    ) -> String {
        let (generated_notes, rendered_notes) =
            self.prepare_note_citations_html(content, &mut parsed, placeholders);
        let generated_note_by_index: HashMap<usize, &GeneratedNote> = generated_notes
            .iter()
            .map(|note| (note.citation_index, note))
            .collect();
        let note_rule = self.note_rule();

        let mut result = String::new();
        let mut last_idx = 0;
        for (index, parsed_citation) in parsed.citations.iter().enumerate() {
            result.push_str(&content[last_idx..parsed_citation.start]);
            match &parsed_citation.placement {
                CitationPlacement::ManualFootnote { .. } => {
                    if let Some(rendered) = rendered_notes.get(&index) {
                        result.push_str(rendered);
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                    }
                    last_idx = parsed_citation.end;
                }
                CitationPlacement::InlineProse => {
                    if let Some(note) = generated_note_by_index.get(&index) {
                        if matches!(
                            parsed_citation.citation.mode,
                            citum_schema::citation::CitationMode::Integral
                        ) && let Ok(anchor) = self
                            .render_note_integral_anchor_with_format::<crate::render::html::Html>(
                                &parsed_citation.citation,
                            )
                        {
                            result.push_str(&placeholders.push_inline(anchor));
                        }
                        let consumed_right = render_note_reference_in_prose(
                            &mut result,
                            &content[parsed_citation.end..],
                            &format!("[^{}]", note.label),
                            note_rule,
                        );
                        last_idx = parsed_citation.end + consumed_right;
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                        last_idx = parsed_citation.end;
                    }
                }
            }
        }
        result.push_str(&content[last_idx..]);

        if !generated_notes.is_empty() {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');

            for note in &generated_notes {
                if let Some(rendered) = rendered_notes.get(&note.citation_index) {
                    result.push_str(&format!("[^{}]: {}\n", note.label, rendered));
                }
            }
        }

        result
    }

    fn prepare_note_citation_state(
        &self,
        parsed: &mut ParsedDocument,
    ) -> (Vec<GeneratedNote>, HashMap<String, Vec<usize>>) {
        let (note_occurrences, manual_citations) = collect_note_occurrences(parsed);
        let generated_notes = assign_note_numbers(parsed, &note_occurrences, &manual_citations);
        self.apply_note_citation_annotations(parsed, &note_occurrences, &manual_citations);
        (generated_notes, manual_citations)
    }

    fn apply_note_citation_annotations(
        &self,
        parsed: &mut ParsedDocument,
        note_occurrences: &[NoteOccurrence],
        manual_citations: &HashMap<String, Vec<usize>>,
    ) {
        let ordered_indices = build_note_order_indices(note_occurrences, manual_citations);
        let (mut ordered_citations, ordered_contexts) =
            ordered_note_citations_and_contexts(parsed, &ordered_indices);
        self.annotate_integral_name_states(&mut ordered_citations, &ordered_contexts);
        ordered_citations = self.normalize_note_context(&ordered_citations);
        self.annotate_positions(&mut ordered_citations);

        for (citation, index) in ordered_citations.into_iter().zip(ordered_indices) {
            parsed.citations[index].citation = citation;
        }
    }

    fn prepare_note_citations<F>(
        &self,
        content: &str,
        parsed: &mut ParsedDocument,
    ) -> (Vec<GeneratedNote>, HashMap<usize, String>)
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let (generated_notes, manual_citations) = self.prepare_note_citation_state(parsed);
        let mut rendered_notes: HashMap<usize, String> = HashMap::new();

        for generated in &generated_notes {
            let note_citation = self.note_render_citation_for_generated(
                &parsed.citations[generated.citation_index].citation,
            );
            let rendered = self
                .process_citation_with_format::<F>(&note_citation)
                .unwrap_or_else(|_| {
                    content[parsed.citations[generated.citation_index].start
                        ..parsed.citations[generated.citation_index].end]
                        .to_string()
                });
            rendered_notes.insert(generated.citation_index, rendered);
        }

        for indices in manual_citations.values() {
            for index in indices {
                let rendered = self
                    .render_manual_note_citation_with_format::<F>(
                        &parsed.citations[*index].citation,
                    )
                    .unwrap_or_else(|_| {
                        content[parsed.citations[*index].start..parsed.citations[*index].end]
                            .to_string()
                    });
                rendered_notes.insert(
                    *index,
                    adjust_manual_note_citation_rendering(
                        &rendered,
                        &content[parsed.citations[*index].end..],
                    ),
                );
            }
        }

        (generated_notes, rendered_notes)
    }

    fn prepare_note_citations_html(
        &self,
        content: &str,
        parsed: &mut ParsedDocument,
        placeholders: &mut HtmlPlaceholderRegistry,
    ) -> (Vec<GeneratedNote>, HashMap<usize, String>) {
        let (generated_notes, manual_citations) = self.prepare_note_citation_state(parsed);
        let mut rendered_notes = HashMap::new();

        for generated in &generated_notes {
            let note_citation = self.note_render_citation_for_generated(
                &parsed.citations[generated.citation_index].citation,
            );
            let rendered = self
                .process_citation_with_format::<crate::render::html::Html>(&note_citation)
                .unwrap_or_else(|_| {
                    content[parsed.citations[generated.citation_index].start
                        ..parsed.citations[generated.citation_index].end]
                        .to_string()
                });
            rendered_notes.insert(generated.citation_index, placeholders.push_inline(rendered));
        }

        for indices in manual_citations.values() {
            for index in indices {
                let rendered = self
                    .render_manual_note_citation_with_format::<crate::render::html::Html>(
                        &parsed.citations[*index].citation,
                    )
                    .unwrap_or_else(|_| {
                        content[parsed.citations[*index].start..parsed.citations[*index].end]
                            .to_string()
                    });
                rendered_notes.insert(
                    *index,
                    placeholders.push_inline(adjust_manual_note_citation_rendering(
                        &rendered,
                        &content[parsed.citations[*index].end..],
                    )),
                );
            }
        }

        (generated_notes, rendered_notes)
    }

    fn render_manual_note_citation_with_format<F>(
        &self,
        citation: &Citation,
    ) -> Result<String, crate::error::ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        if self.should_compose_integral_ibid_in_manual_notes(citation) {
            return self
                .render_manual_note_integral_ibid_with_format::<F>(citation)
                .or_else(|_| self.process_citation_with_format::<F>(citation));
        }
        self.process_citation_with_format::<F>(citation)
    }

    fn should_compose_integral_ibid_in_manual_notes(&self, citation: &Citation) -> bool {
        if !matches!(
            citation.mode,
            citum_schema::citation::CitationMode::Integral
        ) {
            return false;
        }

        if !matches!(
            citation.position,
            Some(
                citum_schema::citation::Position::Ibid
                    | citum_schema::citation::Position::IbidWithLocator
            )
        ) {
            return false;
        }

        !self.has_explicit_position_integral_template(citation.position.as_ref())
    }

    fn has_explicit_position_integral_template(
        &self,
        position: Option<&citum_schema::citation::Position>,
    ) -> bool {
        let Some(citation_spec) = self.style.citation.as_ref() else {
            return false;
        };

        let position_spec = match position {
            Some(
                citum_schema::citation::Position::Ibid
                | citum_schema::citation::Position::IbidWithLocator,
            ) => citation_spec.ibid.as_ref(),
            Some(citum_schema::citation::Position::Subsequent) => citation_spec.subsequent.as_ref(),
            _ => None,
        };

        position_spec
            .and_then(|spec| spec.integral.as_ref())
            .is_some_and(|spec| spec.template.is_some() || spec.locales.is_some())
    }

    fn render_manual_note_integral_ibid_with_format<F>(
        &self,
        citation: &Citation,
    ) -> Result<String, crate::error::ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let anchor = self
            .render_note_integral_anchor_with_format::<F>(citation)
            .unwrap_or_default();
        let reduced = self.render_default_note_ibid_text_with_format::<F>(citation)?;
        if anchor.trim().is_empty() {
            return Ok(reduced);
        }
        if reduced.trim().is_empty() {
            return Ok(anchor);
        }
        Ok(format!("{anchor} ({reduced})"))
    }

    fn render_default_note_ibid_text_with_format<F>(
        &self,
        citation: &Citation,
    ) -> Result<String, crate::error::ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let ibid_term = self
            .style
            .citation
            .as_ref()
            .and_then(|spec| spec.ibid.as_ref())
            .and_then(|ibid| ibid.suffix.clone())
            .filter(|suffix| !suffix.trim().is_empty())
            .or_else(|| {
                self.locale.resolved_general_term(
                    &citum_schema::locale::GeneralTerm::Ibid,
                    citum_schema::locale::TermForm::Long,
                )
            })
            .unwrap_or_else(|| "ibid.".to_string());

        if matches!(
            citation.position,
            Some(citum_schema::citation::Position::IbidWithLocator)
        ) {
            let locator = citation
                .items
                .first()
                .and_then(|item| item.locator.as_ref().map(|locator| (item, locator)))
                .map(|(item, locator)| {
                    let citation_config = self.get_citation_config();
                    let derived_locator_config;
                    let locator_config = if let Some(config) = citation_config.locators.as_ref() {
                        config
                    } else {
                        derived_locator_config = if matches!(
                            citation_config.processing,
                            Some(citum_schema::options::Processing::Note)
                        ) {
                            citum_schema::options::LocatorPreset::Note.config()
                        } else {
                            citum_schema::options::LocatorConfig::default()
                        };
                        &derived_locator_config
                    };
                    let ref_type = self
                        .bibliography
                        .get(&item.id)
                        .map(|reference| reference.ref_type())
                        .unwrap_or_default();
                    crate::values::locator::render_locator(
                        locator,
                        &ref_type,
                        locator_config,
                        &self.locale,
                    )
                })
                .filter(|value| !value.trim().is_empty());

            if let Some(locator) = locator {
                let fmt = F::default();
                return Ok(fmt.join(vec![ibid_term.trim().to_string(), locator], ", "));
            }
        }

        Ok(ibid_term)
    }

    fn note_render_citation_for_generated(&self, citation: &Citation) -> Citation {
        let mut note_citation = citation.clone();
        if matches!(
            note_citation.mode,
            citum_schema::citation::CitationMode::Integral
        ) {
            note_citation.mode = citum_schema::citation::CitationMode::NonIntegral;
        }
        note_citation
    }

    fn render_note_integral_anchor_with_format<F>(
        &self,
        citation: &Citation,
    ) -> Result<String, crate::error::ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let default_spec = citum_schema::CitationSpec::default();
        let effective_spec = self.style.citation.as_ref().map_or_else(
            || std::borrow::Cow::Borrowed(&default_spec),
            |cs| {
                let position_resolved = cs.resolve_for_position(citation.position.as_ref());
                let spec_for_mode = position_resolved.into_owned();
                std::borrow::Cow::Owned(
                    spec_for_mode
                        .resolve_for_mode(&citum_schema::citation::CitationMode::Integral)
                        .into_owned(),
                )
            },
        );

        let sorted_items = self.sort_citation_items(citation.items.clone(), &effective_spec);
        let inter_delimiter = effective_spec
            .multi_cite_delimiter
            .as_deref()
            .unwrap_or("; ");

        let renderer = Renderer::new(
            RendererResources {
                style: &self.style,
                bibliography: &self.bibliography,
                locale: &self.locale,
                config: self.get_config(),
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
        let anchor_position = match citation.position.as_ref() {
            Some(
                citum_schema::citation::Position::Ibid
                | citum_schema::citation::Position::IbidWithLocator,
            ) => None,
            other => other,
        };

        renderer.render_integral_anchor_with_format::<F>(
            &sorted_items,
            &effective_spec,
            inter_delimiter,
            citation.suppress_author,
            anchor_position,
        )
    }

    fn note_rule(&self) -> NoteRule {
        if let Some(notes) = self.get_config().notes.as_ref() {
            return merge_note_rule(self.locale_note_rule(), notes);
        }

        self.locale_note_rule()
    }

    fn locale_note_rule(&self) -> NoteRule {
        let locale = self
            .style
            .info
            .default_locale
            .as_deref()
            .unwrap_or(self.locale.locale.as_str())
            .to_ascii_lowercase();
        match locale.as_str() {
            "en-us" => NoteRule {
                punctuation: super::note_support::PunctuationRule::Inside,
                number: super::note_support::NumberRule::Outside,
                order: super::note_support::NoteOrder::After,
            },
            tag if language_tag(tag) == "fr" => NoteRule {
                punctuation: super::note_support::PunctuationRule::Adaptive,
                number: super::note_support::NumberRule::Same,
                order: super::note_support::NoteOrder::Before,
            },
            _ => NoteRule {
                punctuation: super::note_support::PunctuationRule::Adaptive,
                number: super::note_support::NumberRule::Outside,
                order: super::note_support::NoteOrder::After,
            },
        }
    }
}
