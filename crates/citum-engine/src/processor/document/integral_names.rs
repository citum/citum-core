/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Integral-name state handling for document processing.

use super::note_support::{NoteOccurrence, build_note_order_indices};
use super::{CitationPlacement, CitationStructure, DocumentIntegralNameOverride, ParsedDocument};
use crate::Citation;
use crate::processor::Processor;
use citum_schema::options::{IntegralNameContexts, IntegralNameRule, IntegralNameScope};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(super) struct IntegralNameContext {
    pub(super) placement: CitationPlacement,
    pub(super) structure: CitationStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeenIntegralNameState {
    Unseen,
    NoteOnlySeen,
    BodySeen,
}

impl Processor {
    pub(super) fn normalize_inline_document_citations(
        &self,
        parsed: &ParsedDocument,
    ) -> Vec<Citation> {
        let mut normalized: Vec<Citation> = parsed
            .citations
            .iter()
            .map(|parsed| parsed.citation.clone())
            .collect();
        let ordered_indices = build_integral_name_order_indices(parsed);
        let mut ordered_citations: Vec<Citation> = ordered_indices
            .iter()
            .map(|index| normalized[*index].clone())
            .collect();
        let ordered_contexts: Vec<_> = ordered_indices
            .iter()
            .map(|index| IntegralNameContext {
                placement: parsed.citations[*index].placement.clone(),
                structure: parsed.citations[*index].structure.clone(),
            })
            .collect();
        self.annotate_integral_name_states(&mut ordered_citations, &ordered_contexts);
        for (citation, index) in ordered_citations
            .into_iter()
            .zip(ordered_indices.into_iter())
        {
            normalized[index] = citation;
        }
        self.normalize_note_context(&normalized)
    }

    pub(super) fn processor_with_document_integral_name_override(
        &self,
        override_config: Option<&DocumentIntegralNameOverride>,
    ) -> Option<Self> {
        let override_config = override_config?;
        let mut style = self.style.clone();
        let base = style
            .options
            .as_ref()
            .and_then(|options| options.integral_names.as_ref());
        let applied = override_config.apply_to(base);
        style
            .options
            .get_or_insert_with(Default::default)
            .integral_names = applied;
        Some(Self::with_locale_and_compound_sets(
            style,
            self.bibliography.clone(),
            self.locale.clone(),
            self.compound_sets.clone(),
        ))
    }

    pub(super) fn annotate_integral_name_states(
        &self,
        citations: &mut [Citation],
        contexts: &[IntegralNameContext],
    ) {
        let citation_config = self.get_citation_config();
        let Some(config) = citation_config
            .integral_names
            .as_ref()
            .map(citum_schema::options::IntegralNameConfig::resolve)
        else {
            return;
        };
        if !matches!(config.rule, IntegralNameRule::FullThenShort) {
            return;
        }

        let mut seen: HashMap<(String, String), SeenIntegralNameState> = HashMap::new();
        for (citation, context) in citations.iter_mut().zip(contexts.iter()) {
            if !matches!(
                citation.mode,
                citum_schema::citation::CitationMode::Integral
            ) {
                continue;
            }

            let is_body = matches!(context.placement, CitationPlacement::InlineProse);
            if matches!(config.contexts, IntegralNameContexts::BodyOnly) && !is_body {
                continue;
            }

            let scope_key = match config.scope {
                IntegralNameScope::Document => "document".to_string(),
                IntegralNameScope::Chapter => context.structure.chapter_scope.clone(),
                IntegralNameScope::Section => context.structure.section_scope.clone(),
            };

            for item in &mut citation.items {
                if item.integral_name_state.is_some() {
                    continue;
                }

                let key = (scope_key.clone(), item.id.clone());
                let state = seen
                    .get(&key)
                    .copied()
                    .unwrap_or(SeenIntegralNameState::Unseen);
                let derived = match config.contexts {
                    IntegralNameContexts::BodyOnly => {
                        if matches!(state, SeenIntegralNameState::BodySeen) {
                            citum_schema::citation::IntegralNameState::Subsequent
                        } else {
                            seen.insert(key, SeenIntegralNameState::BodySeen);
                            citum_schema::citation::IntegralNameState::First
                        }
                    }
                    IntegralNameContexts::BodyAndNotes => {
                        if is_body {
                            match state {
                                SeenIntegralNameState::BodySeen => {
                                    citum_schema::citation::IntegralNameState::Subsequent
                                }
                                SeenIntegralNameState::Unseen
                                | SeenIntegralNameState::NoteOnlySeen => {
                                    seen.insert(key, SeenIntegralNameState::BodySeen);
                                    citum_schema::citation::IntegralNameState::First
                                }
                            }
                        } else {
                            match state {
                                SeenIntegralNameState::Unseen => {
                                    seen.insert(key, SeenIntegralNameState::NoteOnlySeen);
                                    citum_schema::citation::IntegralNameState::First
                                }
                                SeenIntegralNameState::NoteOnlySeen
                                | SeenIntegralNameState::BodySeen => {
                                    citum_schema::citation::IntegralNameState::Subsequent
                                }
                            }
                        }
                    }
                };
                item.integral_name_state = Some(derived);
            }
        }
    }
}

pub(super) fn build_integral_name_order_indices(parsed: &ParsedDocument) -> Vec<usize> {
    let mut manual_citations: HashMap<String, Vec<usize>> = HashMap::new();
    let mut note_occurrences: Vec<NoteOccurrence> = parsed
        .manual_note_references
        .iter()
        .map(|note| NoteOccurrence::Manual {
            label: note.label.clone(),
            start: note.start,
        })
        .collect();

    for (index, parsed_citation) in parsed.citations.iter().enumerate() {
        match &parsed_citation.placement {
            CitationPlacement::InlineProse => note_occurrences.push(NoteOccurrence::Generated {
                citation_index: index,
                start: parsed_citation.start,
            }),
            CitationPlacement::ManualFootnote { label } => {
                manual_citations
                    .entry(label.clone())
                    .or_default()
                    .push(index);
            }
        }
    }

    for indices in manual_citations.values_mut() {
        indices.sort_by_key(|index| parsed.citations[*index].start);
    }

    note_occurrences.sort_by_key(NoteOccurrence::start);
    build_note_order_indices(&note_occurrences, &manual_citations)
}
