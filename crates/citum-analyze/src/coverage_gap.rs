/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Corpus-wide coverage-gap analysis: which CSL features does migrate drop?
//!
//! Walks all CSL files in a styles directory, runs each through the
//! `citum-migrate` XML compilation pipeline in-process, and compares the raw
//! CSL feature references in the legacy source against the compiled Citum
//! template output. The difference is the **converter gap** — constructs
//! present in the CSL source that do not appear in migrate's output.
//!
//! Produces two output reports:
//! - **`prioritized_gaps`**: features ranked by how many styles are affected —
//!   the actionable feed for `citum-migrate` converter work.
//! - **`preset_families`**: independent styles whose compiled output closely
//!   matches a Citum base style (Jaccard ≥ threshold) — data-driven discovery
//!   of preset/alias candidates.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write as _;
use std::path::Path;

use walkdir::WalkDir;

use crate::semantic::{
    SemanticItem, collect_base_semantic_sets, jaccard_similarity, semantic_to_legacy_key,
    template_to_set,
};
use citum_migrate::{OptionsExtractor, compilation, fixups, provenance::ProvenanceTracker};
use citum_schema::StyleBase;
use citum_schema::options::Processing;
use csl_legacy::model::CslNode;
use csl_legacy::parser::parse_style;

/// Similarity threshold for including a style in a preset-family cluster.
const PRESET_SIMILARITY_THRESHOLD: f32 = 0.65;

/// Maximum example style slugs to include per gap entry.
const MAX_EXAMPLES: usize = 5;

/// An entry in the prioritized converter-gap list.
#[derive(Debug, serde::Serialize)]
pub struct ConverterGapEntry {
    /// Feature key, e.g. `"var:author"`, `"num:volume"`, `"date:issued"`.
    pub feature: String,
    /// Independent styles where this feature is present in legacy but absent from compiled output.
    pub corpus_count: u32,
    /// Up to [`MAX_EXAMPLES`] style slugs exhibiting this gap.
    pub example_styles: Vec<String>,
}

/// A single style within a preset-family cluster.
#[derive(Debug, serde::Serialize)]
pub struct MatchedStyle {
    /// Style file slug, e.g. `"taylor-and-francis-chicago-author-date"`.
    pub slug: String,
    /// Average of `bibliography_similarity` and `citation_similarity`.
    pub combined_similarity: f32,
    /// Bibliography-only Jaccard similarity.
    pub bibliography_similarity: f32,
    /// Citation-only Jaccard similarity.
    pub citation_similarity: f32,
}

/// A cluster of independent styles that closely match a Citum base.
#[derive(Debug, serde::Serialize)]
pub struct PresetFamilyEntry {
    /// The Citum base style key, e.g. `"apa"`, `"chicago-author-date"`.
    pub base: String,
    /// Matched independent styles, sorted by similarity descending.
    pub matched_styles: Vec<MatchedStyle>,
}

/// Full coverage-gap analysis output.
#[derive(Debug, Default, serde::Serialize)]
pub struct CoverageGapReport {
    /// Number of CSL styles analyzed.
    pub total_analyzed: u32,
    /// Number of styles that failed to parse or compile.
    pub parse_errors: u32,
    /// Converter gaps ranked by corpus weight (highest first).
    pub prioritized_gaps: Vec<ConverterGapEntry>,
    /// Preset-family clusters (combined similarity ≥ threshold).
    pub preset_families: Vec<PresetFamilyEntry>,
}

/// Run the corpus-wide coverage-gap analysis and print or emit the report.
pub fn run_coverage_gap(styles_dir: &str, json_output: bool) {
    let report = analyze_coverage_gap(Path::new(styles_dir));
    if json_output {
        match serde_json::to_string_pretty(&report) {
            Ok(json) => writeln!(std::io::stdout(), "{json}").unwrap_or(()),
            Err(err) => eprintln!("Error: serializing coverage-gap report: {err}"),
        }
    } else {
        print_coverage_gap_report(&report);
    }
}

fn analyze_coverage_gap(styles_dir: &Path) -> CoverageGapReport {
    let tracker = ProvenanceTracker::new(false);
    let bases = collect_base_semantic_sets();

    // Collect upfront so we can show progress against a known total.
    let entries: Vec<_> = WalkDir::new(styles_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "csl"))
        .collect();
    let corpus_size = entries.len();
    eprintln!("Analyzing {corpus_size} styles...");

    let mut gap_counts: HashMap<String, (u32, Vec<String>)> = HashMap::new();
    let mut family_matches: HashMap<String, Vec<MatchedStyle>> = HashMap::new();
    let mut total_analyzed = 0u32;
    let mut parse_errors = 0u32;

    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let slug = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        match analyze_one_style(path, &tracker) {
            Ok((legacy_features, compiled_features, bib_set, cit_set)) => {
                total_analyzed += 1;
                for gap_feature in legacy_features.difference(&compiled_features) {
                    let (count, examples) = gap_counts
                        .entry(gap_feature.clone())
                        .or_insert_with(|| (0, Vec::new()));
                    *count += 1;
                    if examples.len() < MAX_EXAMPLES {
                        examples.push(slug.clone());
                    }
                }
                match_against_bases(&slug, &bib_set, &cit_set, &bases, &mut family_matches);
            }
            Err(_) => parse_errors += 1,
        }

        if (i + 1) % 500 == 0 {
            eprintln!("  {}/{corpus_size}", i + 1);
        }
    }
    eprintln!("  done ({corpus_size} styles).");

    build_report(total_analyzed, parse_errors, gap_counts, family_matches)
}

fn match_against_bases(
    slug: &str,
    bib_set: &HashSet<SemanticItem>,
    cit_set: &HashSet<SemanticItem>,
    bases: &[(StyleBase, HashSet<SemanticItem>, Vec<HashSet<SemanticItem>>)],
    family_matches: &mut HashMap<String, Vec<MatchedStyle>>,
) {
    for (base, base_bib_set, base_cit_sets) in bases {
        let bib_sim = jaccard_similarity(bib_set, base_bib_set);
        let cit_sim = if base_cit_sets.is_empty() {
            1.0
        } else {
            base_cit_sets
                .iter()
                .map(|s| jaccard_similarity(cit_set, s))
                .fold(0.0_f32, f32::max)
        };
        let combined = f32::midpoint(bib_sim, cit_sim);
        if combined >= PRESET_SIMILARITY_THRESHOLD {
            family_matches
                .entry(base.key().to_string())
                .or_default()
                .push(MatchedStyle {
                    slug: slug.to_string(),
                    combined_similarity: combined,
                    bibliography_similarity: bib_sim,
                    citation_similarity: cit_sim,
                });
        }
    }
}

fn analyze_one_style(
    path: &Path,
    tracker: &ProvenanceTracker,
) -> Result<
    (
        HashSet<String>,
        HashSet<String>,
        HashSet<SemanticItem>,
        HashSet<SemanticItem>,
    ),
    String,
> {
    let content = fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
    let doc = roxmltree::Document::parse(&content).map_err(|e| format!("xml: {e}"))?;
    let legacy = parse_style(doc.root_element()).map_err(|e| format!("csl: {e}"))?;

    let legacy_features = collect_legacy_features(&legacy);

    let opts = OptionsExtractor::extract_migration_options(&legacy);
    let mut options = opts.options;
    let output = compilation::compile_from_xml(&legacy, &mut options, false, tracker);

    let mut citation_template = output.citation.clone();
    if matches!(options.processing, Some(Processing::Numeric)) {
        fixups::ensure_numeric_locator_citation_component(
            &legacy.citation.layout,
            &mut citation_template,
        );
    } else if legacy.class == "in-text" {
        fixups::normalize_author_date_locator_citation_component(
            &legacy.citation.layout,
            &legacy.macros,
            &mut citation_template,
        );
    }

    let bib_set = template_to_set(&output.bibliography);
    let cit_set = template_to_set(&citation_template);
    let compiled_features: HashSet<String> = bib_set
        .iter()
        .chain(cit_set.iter())
        .map(semantic_to_legacy_key)
        .collect();

    Ok((legacy_features, compiled_features, bib_set, cit_set))
}

/// Collect all CSL feature keys referenced anywhere in a legacy style's macros and layouts.
///
/// Walking all macros rather than only reachable ones may include a small number of
/// "dead macro" features; these show up as false positives in the gap list. Known-benign
/// fixup normalizations (e.g. locator citation injection) are also visible here.
pub fn collect_legacy_features(style: &csl_legacy::model::Style) -> HashSet<String> {
    let mut features = HashSet::new();
    collect_features_from_nodes(&style.citation.layout.children, &mut features);
    if let Some(ref bib) = style.bibliography {
        collect_features_from_nodes(&bib.layout.children, &mut features);
    }
    for macro_ in &style.macros {
        collect_features_from_nodes(&macro_.children, &mut features);
    }
    features
}

fn collect_features_from_nodes(nodes: &[CslNode], features: &mut HashSet<String>) {
    for node in nodes {
        match node {
            CslNode::Text(t) => {
                if let Some(var) = &t.variable {
                    features.insert(format!("var:{var}"));
                }
                if let Some(term) = &t.term {
                    features.insert(format!("term:{term}"));
                }
            }
            CslNode::Number(n) => {
                features.insert(format!("num:{}", n.variable));
            }
            CslNode::Date(d) => {
                features.insert(format!("date:{}", d.variable));
            }
            CslNode::Names(n) => {
                for var in n.variable.split_whitespace() {
                    features.insert(format!("names:{var}"));
                }
                collect_features_from_nodes(&n.children, features);
            }
            CslNode::Label(l) => {
                if let Some(var) = &l.variable {
                    features.insert(format!("term:{var}"));
                }
            }
            CslNode::Group(g) => collect_features_from_nodes(&g.children, features),
            CslNode::Choose(c) => {
                collect_features_from_nodes(&c.if_branch.children, features);
                for branch in &c.else_if_branches {
                    collect_features_from_nodes(&branch.children, features);
                }
                if let Some(else_nodes) = &c.else_branch {
                    collect_features_from_nodes(else_nodes, features);
                }
            }
            CslNode::Name(_) | CslNode::EtAl(_) | CslNode::Substitute(_) => {}
        }
    }
}

fn build_report(
    total_analyzed: u32,
    parse_errors: u32,
    gap_counts: HashMap<String, (u32, Vec<String>)>,
    family_matches: HashMap<String, Vec<MatchedStyle>>,
) -> CoverageGapReport {
    let mut prioritized_gaps: Vec<ConverterGapEntry> = gap_counts
        .into_iter()
        .map(
            |(feature, (corpus_count, example_styles))| ConverterGapEntry {
                feature,
                corpus_count,
                example_styles,
            },
        )
        .collect();
    prioritized_gaps.sort_by_key(|e| std::cmp::Reverse(e.corpus_count));

    let mut preset_families: Vec<PresetFamilyEntry> = family_matches
        .into_iter()
        .map(|(base, mut styles)| {
            styles.sort_by(|a, b| {
                b.combined_similarity
                    .partial_cmp(&a.combined_similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            PresetFamilyEntry {
                base,
                matched_styles: styles,
            }
        })
        .collect();
    preset_families.sort_by_key(|f| std::cmp::Reverse(f.matched_styles.len()));

    CoverageGapReport {
        total_analyzed,
        parse_errors,
        prioritized_gaps,
        preset_families,
    }
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
fn print_coverage_gap_report(report: &CoverageGapReport) {
    println!("=== Coverage-Gap Report ===\n");
    println!("Styles analyzed:  {}", report.total_analyzed);
    println!("Parse errors:     {}", report.parse_errors);
    println!();

    println!("=== Prioritized Converter Gaps (top 40) ===\n");
    println!("{:>4}  {:<44} {:>6}  Examples", "Rank", "Feature", "Styles");
    println!("{}", "-".repeat(80));
    for (i, entry) in report.prioritized_gaps.iter().take(40).enumerate() {
        let examples = entry.example_styles.join(", ");
        let examples_trunc = if examples.chars().count() > 40 {
            format!("{}…", examples.chars().take(39).collect::<String>())
        } else {
            examples
        };
        println!(
            "{:>4}  {:<44} {:>6}  {}",
            i + 1,
            entry.feature,
            entry.corpus_count,
            examples_trunc,
        );
    }

    println!("\n=== Preset-Family Clusters (similarity ≥ {PRESET_SIMILARITY_THRESHOLD:.2}) ===\n");
    for family in report.preset_families.iter().take(20) {
        println!(
            "  {} ({} styles):",
            family.base,
            family.matched_styles.len()
        );
        for style in family.matched_styles.iter().take(5) {
            println!(
                "    - {} (combined {:.2}, bib {:.2}, cit {:.2})",
                style.slug,
                style.combined_similarity,
                style.bibliography_similarity,
                style.citation_similarity
            );
        }
        if family.matched_styles.len() > 5 {
            println!("    … and {} more", family.matched_styles.len() - 5);
        }
    }
}
