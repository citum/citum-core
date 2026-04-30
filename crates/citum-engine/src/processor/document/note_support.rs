/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Shared note-processing state and helper functions.

use super::integral_names::IntegralNameContext;
use super::{CitationPlacement, ParsedDocument};
use crate::Citation;
use citum_schema::options::{
    NoteConfig as StyleNoteConfig, NoteMarkerOrder, NoteNumberPlacement, NoteQuotePlacement,
};
use std::collections::{HashMap, HashSet};

const GENERATED_NOTE_LABEL_PREFIX: &str = "citum-auto-";
const MOVABLE_PUNCTUATION: [char; 6] = ['.', ',', ';', ':', '!', '?'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QuoteSide {
    Inside,
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NoteOrder {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PunctuationRule {
    Inside,
    Outside,
    Adaptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NumberRule {
    Inside,
    Outside,
    Same,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NoteRule {
    pub(super) punctuation: PunctuationRule,
    pub(super) number: NumberRule,
    pub(super) order: NoteOrder,
}

#[derive(Debug, Default)]
struct LeftContext {
    punctuation: Option<char>,
    quote: Option<char>,
}

#[derive(Debug, Default)]
struct RightContext {
    punctuation: Option<char>,
    quote: Option<char>,
    consumed_len: usize,
}

#[derive(Debug, Clone)]
pub(super) struct GeneratedNote {
    pub(super) citation_index: usize,
    pub(super) label: String,
    pub(super) note_number: u32,
}

#[derive(Debug, Clone)]
pub(super) enum NoteOccurrence {
    Manual { label: String, start: usize },
    Generated { citation_index: usize, start: usize },
}

impl NoteOccurrence {
    pub(super) fn start(&self) -> usize {
        match self {
            Self::Manual { start, .. } | Self::Generated { start, .. } => *start,
        }
    }
}

pub(super) fn collect_note_occurrences(
    parsed: &ParsedDocument,
) -> (Vec<NoteOccurrence>, HashMap<String, Vec<usize>>) {
    let mut note_occurrences: Vec<NoteOccurrence> = parsed
        .manual_note_references
        .iter()
        .map(|note| NoteOccurrence::Manual {
            label: note.label.clone(),
            start: note.start,
        })
        .collect();
    let mut manual_citations: HashMap<String, Vec<usize>> = HashMap::new();

    for (index, parsed_citation) in parsed.citations.iter().enumerate() {
        match &parsed_citation.placement {
            CitationPlacement::InlineProse => {
                note_occurrences.push(NoteOccurrence::Generated {
                    citation_index: index,
                    start: parsed_citation.start,
                });
            }
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

    (note_occurrences, manual_citations)
}

pub(super) fn assign_note_numbers(
    parsed: &mut ParsedDocument,
    note_occurrences: &[NoteOccurrence],
    manual_citations: &HashMap<String, Vec<usize>>,
) -> Vec<GeneratedNote> {
    let mut used_labels = parsed.manual_note_labels.clone();
    let mut manual_numbers: HashMap<String, u32> = HashMap::new();
    let mut next_note = 1_u32;
    let mut generated_notes = assign_note_occurrence_numbers(
        parsed,
        note_occurrences,
        &mut used_labels,
        &mut manual_numbers,
        &mut next_note,
    );
    assign_orphan_manual_note_numbers(
        parsed,
        manual_citations,
        &mut manual_numbers,
        &mut next_note,
    );
    apply_manual_note_numbers(parsed, manual_citations, &manual_numbers);
    generated_notes.sort_by_key(|note| note.note_number);
    generated_notes
}

fn assign_note_occurrence_numbers(
    parsed: &mut ParsedDocument,
    note_occurrences: &[NoteOccurrence],
    used_labels: &mut HashSet<String>,
    manual_numbers: &mut HashMap<String, u32>,
    next_note: &mut u32,
) -> Vec<GeneratedNote> {
    let mut generated_notes = Vec::new();

    for occurrence in note_occurrences {
        match occurrence {
            NoteOccurrence::Manual { label, .. } => {
                manual_numbers
                    .entry(label.clone())
                    .or_insert_with(|| take_next_note_number(next_note));
            }
            NoteOccurrence::Generated { citation_index, .. } => {
                let note_number = take_next_note_number(next_note);
                #[allow(
                    clippy::indexing_slicing,
                    reason = "index derived from citations collection"
                )]
                {
                    parsed.citations[*citation_index].citation.note_number = Some(note_number);
                }
                generated_notes.push(GeneratedNote {
                    citation_index: *citation_index,
                    label: next_generated_note_label(used_labels, note_number),
                    note_number,
                });
            }
        }
    }

    generated_notes
}

fn assign_orphan_manual_note_numbers(
    parsed: &ParsedDocument,
    manual_citations: &HashMap<String, Vec<usize>>,
    manual_numbers: &mut HashMap<String, u32>,
    next_note: &mut u32,
) {
    let mut orphan_labels: Vec<_> = manual_citations
        .keys()
        .filter(|label| !manual_numbers.contains_key(*label))
        .cloned()
        .collect();
    orphan_labels.sort_by_key(|label| {
        manual_citations
            .get(label)
            .and_then(|indices| indices.first())
            .map_or(usize::MAX, |index| {
                #[allow(
                    clippy::indexing_slicing,
                    reason = "index derived from citations collection"
                )]
                let start = parsed.citations[*index].start;
                start
            })
    });

    for label in orphan_labels {
        manual_numbers.insert(label, take_next_note_number(next_note));
    }
}

fn apply_manual_note_numbers(
    parsed: &mut ParsedDocument,
    manual_citations: &HashMap<String, Vec<usize>>,
    manual_numbers: &HashMap<String, u32>,
) {
    for (label, indices) in manual_citations {
        if let Some(note_number) = manual_numbers.get(label).copied() {
            #[allow(
                clippy::indexing_slicing,
                reason = "index derived from citations collection"
            )]
            for index in indices {
                parsed.citations[*index].citation.note_number = Some(note_number);
            }
        }
    }
}

pub(super) fn ordered_note_citations_and_contexts(
    parsed: &ParsedDocument,
    ordered_indices: &[usize],
) -> (Vec<Citation>, Vec<IntegralNameContext>) {
    #[allow(
        clippy::indexing_slicing,
        reason = "index derived from citations collection"
    )]
    let citations = ordered_indices
        .iter()
        .map(|index| parsed.citations[*index].citation.clone())
        .collect();
    #[allow(
        clippy::indexing_slicing,
        reason = "index derived from citations collection"
    )]
    let contexts = ordered_indices
        .iter()
        .map(|index| IntegralNameContext {
            placement: parsed.citations[*index].placement.clone(),
            structure: parsed.citations[*index].structure.clone(),
        })
        .collect();
    (citations, contexts)
}

fn take_next_note_number(next_note: &mut u32) -> u32 {
    let current = *next_note;
    *next_note = (*next_note).saturating_add(1);
    current
}

pub(super) fn merge_note_rule(default: NoteRule, config: &StyleNoteConfig) -> NoteRule {
    NoteRule {
        punctuation: config
            .punctuation
            .map_or(default.punctuation, map_quote_placement),
        number: config.number.map_or(default.number, map_number_placement),
        order: config.order.map_or(default.order, map_note_order),
    }
}

fn map_quote_placement(value: NoteQuotePlacement) -> PunctuationRule {
    match value {
        NoteQuotePlacement::Inside => PunctuationRule::Inside,
        NoteQuotePlacement::Outside => PunctuationRule::Outside,
        NoteQuotePlacement::Adaptive => PunctuationRule::Adaptive,
    }
}

fn map_number_placement(value: NoteNumberPlacement) -> NumberRule {
    match value {
        NoteNumberPlacement::Inside => NumberRule::Inside,
        NoteNumberPlacement::Outside => NumberRule::Outside,
        NoteNumberPlacement::Same => NumberRule::Same,
    }
}

fn map_note_order(value: NoteMarkerOrder) -> NoteOrder {
    match value {
        NoteMarkerOrder::Before => NoteOrder::Before,
        NoteMarkerOrder::After => NoteOrder::After,
    }
}

pub(super) fn language_tag(locale: &str) -> &str {
    locale.split('-').next().unwrap_or(locale)
}

pub(super) fn render_note_reference_in_prose(
    result: &mut String,
    right: &str,
    note_ref: &str,
    rule: NoteRule,
) -> usize {
    let left = pop_left_context(result);
    let right_ctx = inspect_right_context(right);

    let quote = left.quote.or(right_ctx.quote);
    if let Some(quote_char) = quote {
        let mut inside_punctuation = if left.quote.is_some() {
            left.punctuation
        } else {
            None
        };
        let mut outside_punctuation = if right_ctx.quote.is_some() || left.quote.is_some() {
            right_ctx.punctuation
        } else {
            None
        };

        if inside_punctuation.is_some() ^ outside_punctuation.is_some() {
            let punctuation = inside_punctuation.take().or(outside_punctuation.take());
            match desired_punctuation_side(rule, left.punctuation.is_some() && left.quote.is_some())
            {
                QuoteSide::Inside => inside_punctuation = punctuation,
                QuoteSide::Outside => outside_punctuation = punctuation,
            }
        }

        let note_side = desired_note_side(rule, inside_punctuation, outside_punctuation);
        let inside = side_content(
            note_side == QuoteSide::Inside,
            inside_punctuation,
            rule.order,
            note_ref,
        );
        let outside = side_content(
            note_side == QuoteSide::Outside,
            outside_punctuation,
            rule.order,
            note_ref,
        );

        result.push_str(&inside);
        result.push(quote_char);
        result.push_str(&outside);
        right_ctx.consumed_len
    } else {
        let punctuation = right_ctx.punctuation.or(left.punctuation);
        result.push_str(&side_content(true, punctuation, rule.order, note_ref));
        right_ctx.consumed_len
    }
}

fn desired_punctuation_side(rule: NoteRule, punctuation_inside_quote: bool) -> QuoteSide {
    match rule.punctuation {
        PunctuationRule::Inside => QuoteSide::Inside,
        PunctuationRule::Outside => QuoteSide::Outside,
        PunctuationRule::Adaptive => {
            if punctuation_inside_quote {
                QuoteSide::Inside
            } else {
                QuoteSide::Outside
            }
        }
    }
}

fn desired_note_side(
    rule: NoteRule,
    inside_punctuation: Option<char>,
    outside_punctuation: Option<char>,
) -> QuoteSide {
    match rule.number {
        NumberRule::Inside => QuoteSide::Inside,
        NumberRule::Outside => QuoteSide::Outside,
        NumberRule::Same => match (inside_punctuation.is_some(), outside_punctuation.is_some()) {
            (true, false) => QuoteSide::Inside,
            (false, true) => QuoteSide::Outside,
            _ => QuoteSide::Outside,
        },
    }
}

fn side_content(
    include_note: bool,
    punctuation: Option<char>,
    order: NoteOrder,
    note_ref: &str,
) -> String {
    match (include_note, punctuation) {
        (true, Some(punctuation)) => match order {
            NoteOrder::Before => format!("{note_ref}{punctuation}"),
            NoteOrder::After => format!("{punctuation}{note_ref}"),
        },
        (true, None) => note_ref.to_string(),
        (false, Some(punctuation)) => punctuation.to_string(),
        (false, None) => String::new(),
    }
}

fn pop_left_context(result: &mut String) -> LeftContext {
    while result.ends_with(char::is_whitespace) {
        result.pop();
    }

    let mut context = LeftContext::default();
    if let Some(last) = result.chars().last()
        && is_quote(last)
    {
        result.pop();
        context.quote = Some(last);
    }
    if let Some(last) = result.chars().last()
        && is_movable_punctuation(last)
    {
        result.pop();
        context.punctuation = Some(last);
    }
    context
}

fn inspect_right_context(right: &str) -> RightContext {
    let mut chars = right.char_indices();
    let mut context = RightContext::default();

    if let Some((idx, ch)) = chars.next() {
        if is_movable_punctuation(ch) {
            context.punctuation = Some(ch);
            context.consumed_len = idx + ch.len_utf8();
            if let Some((next_idx, next)) = chars.next()
                && is_quote(next)
            {
                context.quote = Some(next);
                context.consumed_len = next_idx + next.len_utf8();
            }
            return context;
        }
        if is_quote(ch) {
            context.quote = Some(ch);
            context.consumed_len = idx + ch.len_utf8();
            if let Some((next_idx, next)) = chars.next()
                && is_movable_punctuation(next)
            {
                context.punctuation = Some(next);
                context.consumed_len = next_idx + next.len_utf8();
            }
        }
    }
    context
}

fn is_movable_punctuation(ch: char) -> bool {
    MOVABLE_PUNCTUATION.contains(&ch)
}

fn is_quote(ch: char) -> bool {
    matches!(ch, '"' | '\'' | '”' | '’' | '»')
}

pub(super) fn build_note_order_indices(
    note_occurrences: &[NoteOccurrence],
    manual_citations: &HashMap<String, Vec<usize>>,
) -> Vec<usize> {
    let mut ordered = Vec::new();
    let mut seen_manual = HashSet::new();

    for occurrence in note_occurrences {
        match occurrence {
            NoteOccurrence::Manual { label, .. } => {
                if seen_manual.insert(label.clone())
                    && let Some(indices) = manual_citations.get(label)
                {
                    ordered.extend(indices.iter().copied());
                }
            }
            NoteOccurrence::Generated { citation_index, .. } => ordered.push(*citation_index),
        }
    }

    let mut orphan_manual: Vec<_> = manual_citations
        .iter()
        .filter(|(label, _)| !seen_manual.contains(*label))
        .collect();
    orphan_manual.sort_by_key(|(_, indices)| indices.first().copied().unwrap_or(usize::MAX));
    for (_, indices) in orphan_manual {
        ordered.extend(indices.iter().copied());
    }

    ordered
}

fn next_generated_note_label(used_labels: &mut HashSet<String>, note_number: u32) -> String {
    let mut candidate = note_number;
    loop {
        let label = format!("{GENERATED_NOTE_LABEL_PREFIX}{candidate}");
        if used_labels.insert(label.clone()) {
            return label;
        }
        candidate = candidate.saturating_add(1);
    }
}

pub(super) fn adjust_manual_note_citation_rendering(rendered: &str, right: &str) -> String {
    let trimmed_right = right.trim_start_matches([' ', '\t']);
    if trimmed_right.is_empty() || trimmed_right.starts_with('\n') {
        return rendered.to_string();
    }

    trim_visible_terminal_period(rendered)
}

fn trim_visible_terminal_period(rendered: &str) -> String {
    let mut cut = rendered.len();
    while let Some(slice) = rendered.get(..cut) {
        cut = slice.trim_end().len();

        let Some(trimmed) = rendered.get(..cut) else {
            break;
        };
        if !trimmed.ends_with('>') {
            break;
        }

        let Some(tag_start) = trimmed.rfind("</") else {
            break;
        };
        #[allow(
            clippy::string_slice,
            reason = "indices come from valid char boundaries"
        )]
        let tag = &trimmed[tag_start..];
        if let Some(inner) = tag.strip_prefix("</").and_then(|s| s.strip_suffix('>'))
            && !inner.contains('<')
        {
            cut = tag_start;
            continue;
        }
        break;
    }

    let Some(slice) = rendered.get(..cut) else {
        return rendered.to_string();
    };
    let Some((period_idx, '.')) = slice.char_indices().next_back() else {
        return rendered.to_string();
    };

    format!(
        "{}{}",
        {
            #[allow(
                clippy::string_slice,
                reason = "indices come from valid char boundaries"
            )]
            let prefix = &rendered[..period_idx];
            prefix
        },
        {
            #[allow(
                clippy::string_slice,
                reason = "indices come from valid char boundaries"
            )]
            let suffix = &rendered[cut..];
            suffix
        }
    )
}
