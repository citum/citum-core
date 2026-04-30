/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citation number and compound subentry collapsing.

use super::Renderer;
use crate::values::range::{ConsecutiveSegment, consecutive_segments};

impl Renderer<'_> {
    #[allow(clippy::indexing_slicing, reason = "loop-guaranteed indices")]
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

    #[allow(clippy::indexing_slicing, reason = "loop-guaranteed indices")]
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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;
    use crate::reference::Bibliography;
    use crate::values::ProcHints;
    use citum_schema::locale::Locale;
    use citum_schema::options::Config;
    use citum_schema::options::bibliography::BibliographyConfig;
    use indexmap::IndexMap;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[allow(clippy::too_many_arguments, reason = "test helper")]
    fn make_renderer<'a>(
        style: &'a citum_schema::Style,
        bib: &'a Bibliography,
        loc: &'a Locale,
        cfg: &'a Config,
        hints: &'a HashMap<String, ProcHints>,
        citation_numbers: &'a RefCell<HashMap<String, usize>>,
        compound_set_by_ref: &'a HashMap<String, String>,
        compound_member_index: &'a HashMap<String, usize>,
        compound_sets: &'a IndexMap<String, Vec<String>>,
        bibliography_config: Option<BibliographyConfig>,
    ) -> Renderer<'a> {
        Renderer {
            style,
            bibliography: bib,
            locale: loc,
            config: cfg,
            bibliography_config,
            hints,
            citation_numbers,
            compound_set_by_ref,
            compound_member_index,
            compound_sets,
            show_semantics: false,
            inject_ast_indices: false,
            filtered_to_original_index: RefCell::new(None),
        }
    }

    #[test]
    fn test_collapse_numeric() {
        let style = citum_schema::Style::default();
        let bib = Bibliography::default();
        let loc = Locale::default();
        let cfg = Config::default();
        let hints = HashMap::new();

        let mut nums = HashMap::new();
        nums.insert("A".to_string(), 1);
        nums.insert("B".to_string(), 2);
        nums.insert("C".to_string(), 3);
        nums.insert("D".to_string(), 4);
        let citation_numbers = RefCell::new(nums);
        let empty_map_string = HashMap::new();
        let empty_map_usize = HashMap::new();
        let empty_index = IndexMap::new();

        let renderer = make_renderer(
            &style,
            &bib,
            &loc,
            &cfg,
            &hints,
            &citation_numbers,
            &empty_map_string,
            &empty_map_usize,
            &empty_index,
            None,
        );

        let cases = [
            (
                vec![
                    (vec!["A".to_string()], "1".to_string()),
                    (vec!["B".to_string()], "2".to_string()),
                    (vec!["C".to_string()], "3".to_string()),
                ],
                vec![(
                    vec!["A".to_string(), "B".to_string(), "C".to_string()],
                    "1–3".to_string(),
                )],
            ),
            (
                vec![
                    (vec!["A".to_string()], "1".to_string()),
                    (vec!["B".to_string()], "2".to_string()),
                    (vec!["D".to_string()], "4".to_string()),
                ],
                vec![
                    (vec!["A".to_string(), "B".to_string()], "1–2".to_string()),
                    (vec!["D".to_string()], "4".to_string()),
                ],
            ),
        ];

        for (chunks, expected) in cases {
            assert_eq!(renderer.collapse_numeric_citation_chunks(chunks), expected);
        }
    }
}
