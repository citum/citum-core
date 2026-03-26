/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citation number and compound subentry collapsing.

use super::Renderer;
use crate::values::range::{ConsecutiveSegment, consecutive_segments};

impl Renderer<'_> {
    pub(super) fn collapse_numeric_citation_chunks(
        &self,
        chunks: Vec<(Vec<String>, String)>,
    ) -> Vec<(Vec<String>, String)> {
        let citation_numbers = self.citation_numbers.borrow();
        let mut collapsed = Vec::new();
        let mut i = 0;

        while i < chunks.len() {
            let Some(ref_id) = chunks[i].0.first() else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            if chunks[i].0.len() != 1 {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }
            let Some(&citation_number) = citation_numbers.get(ref_id) else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            if chunks[i].1 != citation_number.to_string() {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }

            let mut j = i;
            let mut block_ids = Vec::new();
            let mut end_number = citation_number;

            while j < chunks.len() {
                let Some(candidate_id) = chunks[j].0.first() else {
                    break;
                };
                if chunks[j].0.len() != 1 {
                    break;
                }
                let Some(&candidate_number) = citation_numbers.get(candidate_id) else {
                    break;
                };
                if chunks[j].1 != candidate_number.to_string() {
                    break;
                }
                if !block_ids.is_empty() && candidate_number != end_number + 1 {
                    break;
                }

                block_ids.push(candidate_id.clone());
                end_number = candidate_number;
                j += 1;
            }

            if block_ids.len() < 2 {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }

            collapsed.push((block_ids, format!("{citation_number}–{end_number}")));
            i = j;
        }

        collapsed
    }

    pub(super) fn collapse_compound_citation_chunks(
        &self,
        chunks: Vec<(Vec<String>, String)>,
    ) -> Vec<(Vec<String>, String)> {
        let Some(compound) = self
            .bibliography_config
            .as_ref()
            .and_then(|b| b.compound_numeric.as_ref())
        else {
            return chunks;
        };

        if !matches!(
            compound.sub_label,
            citum_schema::options::bibliography::SubLabelStyle::Alphabetic
        ) {
            return chunks;
        }

        let citation_numbers = self.citation_numbers.borrow();
        let mut collapsed = Vec::new();
        let mut i = 0;

        while i < chunks.len() {
            let Some(ref_id) = chunks[i].0.first() else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            let Some(group_id) = self.compound_set_by_ref.get(ref_id) else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };
            let Some(&citation_number) = citation_numbers.get(ref_id) else {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            };

            let mut j = i;
            let mut block_ids = Vec::new();
            let mut member_ordinals = Vec::new();

            while j < chunks.len() {
                let Some(candidate_id) = chunks[j].0.first() else {
                    break;
                };
                if chunks[j].0.len() != 1
                    || self.compound_set_by_ref.get(candidate_id) != Some(group_id)
                    || citation_numbers.get(candidate_id).copied() != Some(citation_number)
                {
                    break;
                }

                let Some(member_index) = self.compound_member_index.get(candidate_id).copied()
                else {
                    break;
                };
                let expected = format!(
                    "{}{}",
                    citation_number,
                    self.citation_sub_label_for_ref(candidate_id)
                        .unwrap_or_default()
                );
                if chunks[j].1 != expected {
                    break;
                }

                block_ids.push(candidate_id.clone());
                member_ordinals.push((member_index + 1) as u32);
                j += 1;
            }

            if block_ids.len() < 2 {
                collapsed.push(chunks[i].clone());
                i += 1;
                continue;
            }

            let labels = consecutive_segments(&member_ordinals)
                .into_iter()
                .map(|segment| match segment {
                    ConsecutiveSegment::Single(value) => {
                        crate::values::int_to_letter(value).unwrap_or_default()
                    }
                    ConsecutiveSegment::Range { start, end } => {
                        let start_label = crate::values::int_to_letter(start).unwrap_or_default();
                        let end_label = crate::values::int_to_letter(end).unwrap_or_default();
                        format!("{start_label}-{end_label}")
                    }
                })
                .collect::<Vec<_>>()
                .join(",");

            collapsed.push((block_ids, format!("{citation_number}{labels}")));
            i = j;
        }

        collapsed
    }
}
