/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Note-context normalization and citation position inference.
//!
//! These helpers prepare citation numbering for note styles by filling in
//! missing note numbers and assigning positions such as `First`, `Subsequent`,
//! and `Ibid` before citation rendering begins.

use super::Processor;
use crate::reference::{Citation, CitationItem};
use citum_schema::citation::Position;

/// Get a canonical locator string for ibid comparison.
///
/// Accounts for both single and compound locator forms.
/// Returns `None` when no locator is present.
fn effective_locator_string(item: &CitationItem) -> Option<String> {
    item.locator
        .as_ref()
        .map(citum_schema::citation::CitationLocator::canonical_string)
}

impl Processor {
    /// Detect and annotate citation positions.
    ///
    /// Analyzes citations in order and assigns positions based on whether an item
    /// has been cited before:
    /// - First: Item not cited before
    /// - Subsequent: Item cited before but not immediately preceding
    /// - Ibid: Same single item as immediately preceding citation with same locator context
    /// - `IbidWithLocator`: Same single item as preceding, different locators
    ///
    /// Multi-item citations are never marked as Ibid (only First or Subsequent).
    /// Only sets position if currently None (respects explicit caller values).
    pub(crate) fn annotate_positions(&self, citations: &mut [Citation]) {
        let mut seen_items: std::collections::HashMap<String, Option<String>> =
            std::collections::HashMap::new();
        let mut previous_items: Option<Vec<(String, Option<String>)>> = None;

        for citation in citations.iter_mut() {
            if citation.position.is_some() {
                let current_items: Vec<(String, Option<String>)> = citation
                    .items
                    .iter()
                    .map(|item| (item.id.clone(), effective_locator_string(item)))
                    .collect();
                previous_items = Some(current_items);
                for item in &citation.items {
                    seen_items.insert(item.id.clone(), effective_locator_string(item));
                }
                continue;
            }

            if citation.items.len() == 1 {
                #[allow(clippy::indexing_slicing, reason = "citation.items.len() == 1")]
                let current_id = &citation.items[0].id;
                #[allow(clippy::indexing_slicing, reason = "citation.items.len() == 1")]
                let current_locator = effective_locator_string(&citation.items[0]);

                if let Some(previous) = previous_items.as_ref()
                    && previous.len() == 1
                    && let Some(prev_item) = previous.first()
                    && prev_item.0 == *current_id
                {
                    let previous_locator = &prev_item.1;
                    citation.position = Some(if previous_locator == &current_locator {
                        Position::Ibid
                    } else {
                        Position::IbidWithLocator
                    });
                }

                if citation.position.is_none() {
                    citation.position = Some(if seen_items.contains_key(current_id) {
                        Position::Subsequent
                    } else {
                        Position::First
                    });
                }

                seen_items.insert(current_id.clone(), current_locator);
            } else {
                let all_seen = citation
                    .items
                    .iter()
                    .all(|item| seen_items.contains_key(&item.id));

                citation.position = Some(if all_seen {
                    Position::Subsequent
                } else {
                    Position::First
                });

                for item in &citation.items {
                    seen_items.insert(item.id.clone(), effective_locator_string(item));
                }
            }

            previous_items = Some(
                citation
                    .items
                    .iter()
                    .map(|item| (item.id.clone(), effective_locator_string(item)))
                    .collect(),
            );
        }
    }

    /// Normalize citation note context for note styles.
    ///
    /// Document/plugin layers should provide explicit `note_number` values.
    /// When missing, this method assigns sequential note numbers in citation order.
    pub fn normalize_note_context(&self, citations: &[Citation]) -> Vec<Citation> {
        if !self.is_note_style() {
            return citations.to_vec();
        }

        let mut next_note = 1_u32;
        citations
            .iter()
            .cloned()
            .map(|mut citation| {
                if let Some(note_number) = citation.note_number {
                    if note_number >= next_note {
                        next_note = note_number.saturating_add(1);
                    }
                } else {
                    citation.note_number = Some(next_note);
                    next_note = next_note.saturating_add(1);
                }
                citation
            })
            .collect()
    }
}
