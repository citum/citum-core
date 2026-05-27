/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Integral-name state handling for document processing.

#[rustfmt::skip]
use super::note_support::{NoteOccurrence, build_note_order_indices};
use super::types::{DocumentIntegralNameOverride, DocumentOrgAbbreviationOverride};
use super::{CitationPlacement, CitationStructure, ParsedDocument};
use crate::reference::Contributor;
use crate::{Citation, Processor};
use citum_schema::options::{IntegralNameContexts, IntegralNameScope};
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AnnotateKind {
    Personal,
    OrgAbbreviation,
}

impl Processor {
    #[allow(
        clippy::indexing_slicing,
        reason = "indices are verified within bounds"
    )]
    pub(super) fn normalize_integral_name_citations(
        &self,
        parsed: &ParsedDocument,
    ) -> Vec<Citation> {
        let mut normalized: Vec<Citation> = parsed
            .citations
            .iter()
            .map(|parsed| parsed.citation.clone())
            .collect();
        let ordered_indices = build_integral_name_order_indices(parsed);
        #[allow(
            clippy::indexing_slicing,
            reason = "index derived from citations collection"
        )]
        let mut ordered_citations: Vec<Citation> = ordered_indices
            .iter()
            .map(|index| normalized[*index].clone())
            .collect();
        #[allow(
            clippy::indexing_slicing,
            reason = "index derived from citations collection"
        )]
        let ordered_contexts: Vec<_> = ordered_indices
            .iter()
            .map(|index| IntegralNameContext {
                placement: parsed.citations[*index].placement.clone(),
                structure: parsed.citations[*index].structure.clone(),
            })
            .collect();
        self.annotate_integral_name_states(&mut ordered_citations, &ordered_contexts);
        #[allow(
            clippy::indexing_slicing,
            reason = "index derived from citations collection"
        )]
        for (citation, index) in ordered_citations.into_iter().zip(ordered_indices) {
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
            .and_then(|options| options.integral_name_memory.as_ref());
        let applied = override_config.apply_to(base);
        style
            .options
            .get_or_insert_with(Default::default)
            .integral_name_memory = applied;
        // The style cloned from self.style is already resolved — bypass
        // into_resolved() to prevent a second preset application that would
        // restore null-cleared fields (e.g. type-variants: ~).
        Some(Self::build_processor_pre_resolved(
            style,
            self.bibliography.clone(),
            self.locale.clone(),
            self.compound_sets.clone(),
        ))
    }

    pub(super) fn processor_with_document_org_abbreviation_override(
        &self,
        override_config: Option<&DocumentOrgAbbreviationOverride>,
    ) -> Option<Self> {
        let override_config = override_config?;
        let mut style = self.style.clone();
        let base = style
            .options
            .as_ref()
            .and_then(|options| options.org_abbreviation_memory.as_ref());
        let applied = override_config.apply_to(base);
        style
            .options
            .get_or_insert_with(Default::default)
            .org_abbreviation_memory = applied;
        Some(Self::build_processor_pre_resolved(
            style,
            self.bibliography.clone(),
            self.locale.clone(),
            self.compound_sets.clone(),
        ))
    }

    fn annotate_name_states_for_kind(
        &self,
        citations: &mut [Citation],
        contexts: &[IntegralNameContext],
        kind: AnnotateKind,
        scope: IntegralNameScope,
        name_contexts: IntegralNameContexts,
    ) {
        let mut seen: HashMap<(String, String), SeenIntegralNameState> = HashMap::new();
        for (citation, context) in citations.iter_mut().zip(contexts.iter()) {
            let counts_as_integral = matches!(
                citation.mode,
                citum_schema::citation::CitationMode::Integral
            ) || citation.suppress_author;
            if !counts_as_integral {
                continue;
            }

            let is_body = matches!(context.placement, CitationPlacement::InlineProse);
            if matches!(name_contexts, IntegralNameContexts::BodyOnly) && !is_body {
                continue;
            }

            let scope_key = match scope {
                IntegralNameScope::Document => "document".to_string(),
                IntegralNameScope::Chapter => context.structure.chapter_scope.clone(),
                IntegralNameScope::Section => context.structure.section_scope.clone(),
            };

            for item in &mut citation.items {
                // Skip if this item's author type doesn't match the kind
                let is_org = first_author_is_org(&self.bibliography, &item.id);
                match kind {
                    AnnotateKind::Personal if is_org => continue,
                    AnnotateKind::OrgAbbreviation if !is_org => continue,
                    _ => {}
                }

                // Skip items with an explicit override already set
                let state_already_set = match kind {
                    AnnotateKind::Personal => item.integral_name_state.is_some(),
                    AnnotateKind::OrgAbbreviation => item.org_abbreviation_state.is_some(),
                };
                if state_already_set {
                    continue;
                }

                let author_key = match kind {
                    AnnotateKind::Personal => {
                        personal_author_key_for_item(&self.bibliography, &item.id)
                    }
                    AnnotateKind::OrgAbbreviation => {
                        first_author_key_for_item(&self.bibliography, &item.id)
                    }
                };
                let key = (scope_key.clone(), author_key);
                let state = seen
                    .get(&key)
                    .copied()
                    .unwrap_or(SeenIntegralNameState::Unseen);
                let derived = match name_contexts {
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
                match kind {
                    AnnotateKind::Personal => item.integral_name_state = Some(derived),
                    AnnotateKind::OrgAbbreviation => item.org_abbreviation_state = Some(derived),
                }
            }
        }
    }

    pub(super) fn annotate_integral_name_states(
        &self,
        citations: &mut [Citation],
        contexts: &[IntegralNameContext],
    ) {
        let citation_config = self.get_citation_config();

        if let Some(config) = citation_config
            .integral_name_memory
            .as_ref()
            .map(citum_schema::options::IntegralNameMemoryConfig::resolve)
        {
            self.annotate_name_states_for_kind(
                citations,
                contexts,
                AnnotateKind::Personal,
                config.scope,
                config.contexts,
            );
        }

        if let Some(config) = citation_config
            .org_abbreviation_memory
            .as_ref()
            .map(citum_schema::options::OrgAbbreviationMemoryConfig::resolve)
        {
            self.annotate_name_states_for_kind(
                citations,
                contexts,
                AnnotateKind::OrgAbbreviation,
                config.scope,
                config.contexts,
            );
        }
    }
}

/// Returns a name-based tracking key for integral name-memory state.
///
/// Uses the first author's family name so that two works by the same author
/// share a single First/Subsequent slot, regardless of citation key.
/// Falls back to `item_id` when no author or family name is available.
fn first_author_key_for_item(bibliography: &crate::Bibliography, item_id: &str) -> String {
    bibliography
        .get(item_id)
        .and_then(|r| r.author())
        .and_then(|c| contributor_first_family(&c))
        .unwrap_or_else(|| item_id.to_string())
}

fn contributor_first_family(contributor: &Contributor) -> Option<String> {
    match contributor {
        Contributor::StructuredName(n) => Some(n.family.to_string()),
        Contributor::Multilingual(m) => Some(m.original.family.to_string()),
        Contributor::SimpleName(n) => Some(n.name.to_string()),
        Contributor::ContributorList(l) => l.0.first().and_then(contributor_first_family),
    }
}

fn personal_author_key_for_item(bibliography: &crate::Bibliography, item_id: &str) -> String {
    bibliography
        .get(item_id)
        .and_then(|r| r.author())
        .and_then(|c| contributor_personal_key(&c))
        .unwrap_or_else(|| item_id.to_string())
}

fn contributor_personal_key(contributor: &Contributor) -> Option<String> {
    match contributor {
        Contributor::StructuredName(n) => {
            let given = n.given.to_string();
            let family = n.family.to_string();
            let suffix = n.suffix.as_deref().unwrap_or("");
            Some(format!("{given}|{family}|{suffix}"))
        }
        Contributor::Multilingual(m) => {
            let given = m.original.given.to_string();
            let family = m.original.family.to_string();
            let suffix = m.original.suffix.as_deref().unwrap_or("");
            Some(format!("{given}|{family}|{suffix}"))
        }
        Contributor::SimpleName(n) => Some(n.name.to_string()),
        Contributor::ContributorList(l) => l.0.first().and_then(contributor_personal_key),
    }
}

fn first_author_is_org(bibliography: &crate::Bibliography, item_id: &str) -> bool {
    bibliography
        .get(item_id)
        .and_then(|r| r.author())
        .is_some_and(|c| first_contributor_is_org(&c))
}

fn first_contributor_is_org(contributor: &Contributor) -> bool {
    match contributor {
        Contributor::SimpleName(_) => true,
        Contributor::StructuredName(_) | Contributor::Multilingual(_) => false,
        Contributor::ContributorList(l) => l.0.first().is_some_and(first_contributor_is_org),
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
        #[allow(
            clippy::indexing_slicing,
            reason = "index derived from citations collection"
        )]
        indices.sort_by_key(|index| parsed.citations[*index].start);
    }

    note_occurrences.sort_by_key(NoteOccurrence::start);
    build_note_order_indices(&note_occurrences, &manual_citations)
}
