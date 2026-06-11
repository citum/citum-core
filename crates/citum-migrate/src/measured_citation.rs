/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Measured inferred-vs-XML citation template selection.
//!
//! The template inferrer's confidence score is computed against its own
//! reconstruction surface, so it can rate a citation template highly even
//! when the template renders badly. This module settles the choice
//! empirically at migration time: render both candidate styles with the
//! Citum engine over the embedded fixture items, compare each output to the
//! citeproc-js reference rendering of the original CSL style, and keep the
//! candidate that matches better.
//!
//! The per-item criterion mirrors the fidelity oracle
//! (`scripts/oracle-utils.js`): bag-of-words Jaccard similarity over
//! lowercased alphanumeric tokens, with a pass at `>= 0.60`. Selection
//! therefore optimizes the same measure the oracle reports.

use crate::js_runtime::{self, EmbeddedTemplateRuntime};
use citum_engine::Processor;
use citum_engine::reference::{Bibliography, Citation};
use citum_schema::Style;
use std::collections::BTreeMap;
use std::path::Path;

/// Outcome of measured citation-candidate scoring.
#[derive(Debug, Clone)]
pub struct MeasuredCitationSelection {
    /// Whether the XML-compiled candidate beat the inferred candidate.
    pub use_xml: bool,
    /// Items the inferred candidate passed at the oracle threshold.
    pub inferred_passes: usize,
    /// Items the XML-compiled candidate passed at the oracle threshold.
    pub xml_passes: usize,
    /// Number of fixture items that produced a citeproc reference citation.
    pub items: usize,
}

/// Per-item pass threshold, mirroring the oracle's `similarityThreshold`.
const PASS_THRESHOLD: f64 = 0.60;

/// Margin (in summed similarity) the XML candidate must clear to win a
/// pass-count tie, so noise cannot flip the inferred status quo.
const TIE_MARGIN: f64 = 0.5;

/// Score both candidate styles against citeproc-js reference citations.
///
/// `inferred_style` is the standalone style assembled with the inferred
/// citation template; `xml_style` is the same style assembled down the
/// XML-compilation path. The XML candidate wins only when it passes strictly
/// more items, or ties on passes with a clearly higher summed similarity.
///
/// # Errors
///
/// Returns an error when the embedded runtime, fixture data, or reference
/// rendering is unavailable; callers should treat that as "keep the
/// inferred candidate".
pub fn select(
    inferred_style: &Style,
    xml_style: &Style,
    style_name: &str,
    style_xml: &str,
    workspace_root: &Path,
) -> Result<MeasuredCitationSelection, String> {
    let mut runtime = EmbeddedTemplateRuntime::new(workspace_root)?;
    let reference_json = runtime.render_citation_strings(style_name, style_xml)?;
    let references: BTreeMap<String, Vec<Option<String>>> =
        serde_json::from_str(&reference_json)
            .map_err(|err| format!("failed to parse citeproc citation references: {err}"))?;

    let bibliography = fixture_bibliography(workspace_root)?;
    let inferred = score_candidate(inferred_style, &bibliography, &references);
    let xml = score_candidate(xml_style, &bibliography, &references);

    let use_xml = xml.passes > inferred.passes
        || (xml.passes == inferred.passes
            && xml.similarity_sum > inferred.similarity_sum + TIE_MARGIN);

    Ok(MeasuredCitationSelection {
        use_xml,
        inferred_passes: inferred.passes,
        xml_passes: xml.passes,
        items: inferred.items,
    })
}

/// Aggregate score for one candidate over the fixture items.
struct CandidateScore {
    passes: usize,
    similarity_sum: f64,
    items: usize,
}

/// Load the embedded fixture items as an engine bibliography.
fn fixture_bibliography(workspace_root: &Path) -> Result<Bibliography, String> {
    let fixtures = js_runtime::load_fixtures(workspace_root)?;
    let map = fixtures
        .as_object()
        .ok_or_else(|| "embedded fixture file is not a JSON object".to_string())?;

    let mut bibliography = Bibliography::new();
    for (id, item) in map {
        let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(item.clone())
            .map_err(|err| format!("fixture item {id} failed to parse as CSL JSON: {err}"))?;
        bibliography.insert(id.clone(), legacy.into());
    }
    Ok(bibliography)
}

/// Render the citation scenarios for one candidate and score each against the
/// citeproc references at the oracle's similarity criterion.
///
/// Scenario index 0 is a bare single-item citation; index 1 adds a page
/// locator, so locator placement failures count against a candidate too.
fn score_candidate(
    style: &Style,
    bibliography: &Bibliography,
    references: &BTreeMap<String, Vec<Option<String>>>,
) -> CandidateScore {
    let processor = Processor::new(style.clone(), bibliography.clone());
    let mut score = CandidateScore {
        passes: 0,
        similarity_sum: 0.0,
        items: 0,
    };
    for (id, scenario_references) in references {
        if !bibliography.contains_key(id) {
            continue;
        }
        for (scenario_index, reference) in scenario_references.iter().enumerate() {
            let Some(reference) = reference else {
                continue;
            };
            if reference.trim().is_empty() {
                continue;
            }
            score.items += 1;
            let rendered = processor
                .process_citation(&scenario_citation(id, scenario_index))
                .unwrap_or_default();
            let similarity = token_jaccard(&rendered, reference);
            if similarity >= PASS_THRESHOLD {
                score.passes += 1;
            }
            score.similarity_sum += similarity;
        }
    }
    score
}

/// Build the engine-side citation matching a citeproc reference scenario.
///
/// Index 0 is a bare single-item citation; any other index carries the same
/// page locator the JS side renders (`p. 23`).
fn scenario_citation(id: &str, scenario_index: usize) -> Citation {
    let mut citation = Citation::simple(id);
    if scenario_index > 0
        && let Some(item) = citation.items.first_mut()
    {
        item.locator = Some(citum_schema::citation::CitationLocator::single(
            citum_schema::citation::LocatorType::Page,
            "23",
        ));
    }
    citation
}

/// Bag-of-words Jaccard similarity over alphanumeric tokens, mirroring the
/// oracle's `textSimilarity`: lowercase, non-alphanumeric characters split
/// tokens, single-character tokens dropped, multiset intersection over union.
fn token_jaccard(left_text: &str, right_text: &str) -> f64 {
    let left = tokenize(left_text);
    let right = tokenize(right_text);
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }

    let mut counts: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for token in &left {
        counts.entry(token).or_insert((0, 0)).0 += 1;
    }
    for token in &right {
        counts.entry(token).or_insert((0, 0)).1 += 1;
    }

    let mut intersection = 0usize;
    let mut union = 0usize;
    for (left_count, right_count) in counts.values() {
        intersection += left_count.min(right_count);
        union += left_count.max(right_count);
    }

    if union == 0 {
        return 0.0;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "citation token counts are far below f64 integer precision"
    )]
    let ratio = intersection as f64 / union as f64;
    ratio
}

/// Split text into lowercased alphanumeric tokens, dropping single-character
/// tokens, matching the oracle's `tokenizeForSimilarity`.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.chars().count() > 1)
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "Panicking and exact comparison are acceptable in tests."
)]
mod tests {
    use super::{PASS_THRESHOLD, token_jaccard, tokenize};

    #[test]
    fn tokenize_splits_on_punctuation_and_drops_short_tokens() {
        let given = "T.S. Kuhn, ‘The Structure’ (1962)";
        let tokens = tokenize(given);
        assert_eq!(tokens, vec!["kuhn", "the", "structure", "1962"]);
    }

    #[test]
    fn token_jaccard_is_one_for_equal_bags_regardless_of_order() {
        let left = "Kuhn, The Structure of Scientific Revolutions (1962)";
        let right = "1962 Kuhn: of Scientific Revolutions, The Structure";
        assert_eq!(token_jaccard(left, right), 1.0);
    }

    #[test]
    fn token_jaccard_punishes_run_on_components() {
        let reference =
            "Kuhn, ‘The Structure of Scientific Revolutions’, International Encyclopedia (1962)";
        let run_on =
            "Kuhn, ‘The Structure of Scientific RevolutionsInternational Encyclopedia’ (1962)";
        let separated =
            "Kuhn, 1962, ‘The Structure of Scientific Revolutions’, International Encyclopedia";
        assert!(token_jaccard(separated, reference) > token_jaccard(run_on, reference));
    }

    #[test]
    fn token_jaccard_threshold_separates_wrong_shape_from_near_miss() {
        let reference =
            "Thomas Kuhn, The Structure of Scientific Revolutions, Chicago (1962), p. 23";
        let near_miss = "Thomas Kuhn, The Structure of Scientific Revolutions, Chicago (1962)";
        let wrong_shape = "Kuhn 1962";
        assert!(token_jaccard(near_miss, reference) >= PASS_THRESHOLD);
        assert!(token_jaccard(wrong_shape, reference) < PASS_THRESHOLD);
    }

    #[test]
    fn token_jaccard_handles_empty_inputs() {
        assert_eq!(token_jaccard("", ""), 1.0);
        assert_eq!(token_jaccard("Kuhn 1962", ""), 0.0);
        assert_eq!(token_jaccard("", "Kuhn 1962"), 0.0);
    }
}
