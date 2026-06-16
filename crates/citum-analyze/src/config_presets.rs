/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Corpus-wide config-preset discovery: which per-concern config shapes are unnamed?
//!
//! Walks all independent CSL styles, extracts the per-concern config blocks that
//! migration populates (`contributors`, `dates`, `titles`, `locators`), and checks
//! each observed shape against the named presets in `citum-schema-style`.
//!
//! Only **unnamed** shapes — recurring ≥ [`MIN_FREQUENCY`] times and matching no
//! existing preset exactly — are emitted. These are candidates for new named presets
//! in `citum-schema-style`. Matched shapes are tallied in the per-concern summary so
//! coverage context is preserved even though individual matches are suppressed.
//!
//! Invoke via `citum-analyze <styles_dir> --config-presets [--json]`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write as _;
use std::path::Path;

use walkdir::WalkDir;

use citum_migrate::OptionsExtractor;
use citum_schema::options::{
    ContributorConfig, DateConfig, LocatorConfig, LocatorPreset, TitlesConfig,
};
use citum_schema::presets::{ContributorPreset, DatePreset, TitlePreset};
use csl_legacy::parser::parse_style;

/// Minimum number of distinct styles an unnamed config shape must appear in to be reported.
const MIN_FREQUENCY: u32 = 3;

/// Maximum example style slugs per candidate entry.
const MAX_EXAMPLES: usize = 5;

/// A recurring config shape that does not match any existing named preset.
#[derive(Debug, serde::Serialize)]
pub struct PresetCandidate {
    /// Number of corpus styles sharing this exact config shape.
    pub corpus_count: u32,
    /// Up to [`MAX_EXAMPLES`] style slugs that use this config.
    pub example_styles: Vec<String>,
    /// Serialized config block: the literal shape a new preset would encode.
    pub canonical_config: serde_json::Value,
}

/// Per-concern summary: preset coverage count and unnamed-candidate list.
#[derive(Debug, serde::Serialize)]
pub struct ConcernReport {
    /// Concern name: `"contributors"`, `"dates"`, `"titles"`, or `"locators"`.
    pub concern: String,
    /// Styles whose non-default config for this concern matched a named preset exactly.
    pub matched_style_count: u32,
    /// Styles with a non-default config that did not match any preset.
    pub unmatched_style_count: u32,
    /// Unnamed shapes above [`MIN_FREQUENCY`], ranked by corpus count descending.
    pub candidates: Vec<PresetCandidate>,
}

/// Full config-preset analysis report.
#[derive(Debug, Default, serde::Serialize)]
pub struct ConfigPresetReport {
    /// Total CSL styles analyzed.
    pub total_analyzed: u32,
    /// Styles that failed to parse or load options.
    pub parse_errors: u32,
    /// Per-concern results in order: contributors, dates, titles, locators.
    pub concerns: Vec<ConcernReport>,
}

/// Run the config-preset analysis and emit the report to stdout or stderr.
pub fn run_config_presets(styles_dir: &str, json_output: bool) {
    let report = analyze_config_presets(Path::new(styles_dir));
    if json_output {
        match serde_json::to_string_pretty(&report) {
            Ok(json) => writeln!(std::io::stdout(), "{json}").unwrap_or(()),
            Err(err) => eprintln!("Error: serializing config-preset report: {err}"),
        }
    } else {
        print_config_preset_report(&report);
    }
}

fn analyze_config_presets(styles_dir: &Path) -> ConfigPresetReport {
    let entries: Vec<_> = WalkDir::new(styles_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "csl"))
        .collect();
    let corpus_size = entries.len();
    eprintln!("Analyzing {corpus_size} styles for config-preset gaps...");

    // concern name → (canonical_key → (count, examples, display value))
    let mut concern_maps: [HashMap<String, (u32, Vec<String>, serde_json::Value)>; 4] =
        Default::default();

    let mut total_analyzed = 0u32;
    let mut parse_errors = 0u32;

    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let slug = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        let Ok(content) = std::fs::read_to_string(path) else {
            parse_errors += 1;
            continue;
        };
        let Ok(doc) = roxmltree::Document::parse(&content) else {
            parse_errors += 1;
            continue;
        };
        let Ok(legacy) = parse_style(doc.root_element()) else {
            parse_errors += 1;
            continue;
        };

        // Options extraction: XML attribute parsing only — no compile or engine call.
        let config = OptionsExtractor::extract_migration_options(&legacy).options;
        total_analyzed += 1;

        if let Some(v) = config.contributors
            && v != ContributorConfig::default()
        {
            accumulate(&mut concern_maps[0], &slug, &v);
        }
        if let Some(v) = config.dates
            && v != DateConfig::default()
        {
            accumulate(&mut concern_maps[1], &slug, &v);
        }
        if let Some(v) = config.titles
            && v != TitlesConfig::default()
        {
            accumulate(&mut concern_maps[2], &slug, &v);
        }
        if let Some(v) = config.locators
            && v != LocatorConfig::default()
        {
            accumulate(&mut concern_maps[3], &slug, &v);
        }

        if (i + 1) % 500 == 0 {
            eprintln!("  {}/{corpus_size}", i + 1);
        }
    }
    eprintln!("  done ({corpus_size} styles).");

    let [contributor_map, date_map, title_map, locator_map] = concern_maps;

    let concerns = vec![
        build_concern("contributors", contributor_map, &contributor_named_keys()),
        build_concern("dates", date_map, &date_named_keys()),
        build_concern("titles", title_map, &title_named_keys()),
        build_concern("locators", locator_map, &locator_named_keys()),
    ];

    ConfigPresetReport {
        total_analyzed,
        parse_errors,
        concerns,
    }
}

/// Accumulate a serializable config value into a frequency map.
fn accumulate(
    map: &mut HashMap<String, (u32, Vec<String>, serde_json::Value)>,
    slug: &str,
    value: &impl serde::Serialize,
) {
    let raw = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    let sorted = sort_json_keys(&raw);
    let key = sorted.to_string();
    let entry = map.entry(key).or_insert((0, Vec::new(), sorted));
    entry.0 += 1;
    if entry.1.len() < MAX_EXAMPLES {
        entry.1.push(slug.to_string());
    }
}

/// Build a [`ConcernReport`] by comparing accumulated keys against the named-preset set.
fn build_concern(
    name: &str,
    counts: HashMap<String, (u32, Vec<String>, serde_json::Value)>,
    named_keys: &HashSet<String>,
) -> ConcernReport {
    let mut matched = 0u32;
    let mut unmatched = 0u32;
    let mut candidates = Vec::new();

    for (key, (count, examples, canonical_config)) in counts {
        if named_keys.contains(&key) {
            matched += count;
        } else {
            unmatched += count;
            if count >= MIN_FREQUENCY {
                candidates.push(PresetCandidate {
                    corpus_count: count,
                    example_styles: examples,
                    canonical_config,
                });
            }
        }
    }
    candidates.sort_by_key(|c| std::cmp::Reverse(c.corpus_count));

    ConcernReport {
        concern: name.to_string(),
        matched_style_count: matched,
        unmatched_style_count: unmatched,
        candidates,
    }
}

/// Recursively sort JSON object keys for a stable canonical representation.
fn sort_json_keys(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map.iter().collect();
            let new_map: serde_json::Map<_, _> = sorted
                .into_iter()
                .map(|(k, v)| (k.clone(), sort_json_keys(v)))
                .collect();
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sort_json_keys).collect())
        }
        other => other.clone(),
    }
}

// ── Preset enumerators ──────────────────────────────────────────────────────
//
// These lists enumerate all current variants of each preset enum so the report
// can reverse-match observed configs against them. Keep in sync with the enum
// definitions in `citum-schema-style/src/presets.rs` and
// `citum-schema-style/src/options/locators.rs`.

fn preset_keys<P: serde::Serialize>(
    presets: impl IntoIterator<Item = (P, impl serde::Serialize)>,
) -> HashSet<String> {
    presets
        .into_iter()
        .map(|(_, config)| {
            let raw = serde_json::to_value(&config).unwrap_or(serde_json::Value::Null);
            sort_json_keys(&raw).to_string()
        })
        .collect()
}

fn contributor_named_keys() -> HashSet<String> {
    // keep in sync with ContributorPreset in presets.rs
    let variants = [
        ContributorPreset::Apa,
        ContributorPreset::Chicago,
        ContributorPreset::Vancouver,
        ContributorPreset::Ieee,
        ContributorPreset::Harvard,
        ContributorPreset::Springer,
        ContributorPreset::NumericCompact,
        ContributorPreset::NumericMedium,
        ContributorPreset::NumericTight,
        ContributorPreset::NumericLarge,
        ContributorPreset::NumericAllAuthors,
        ContributorPreset::NumericGivenDot,
        ContributorPreset::AnnualReviews,
        ContributorPreset::MathPhys,
        ContributorPreset::SocSciFirst,
        ContributorPreset::PhysicsNumeric,
    ];
    preset_keys(variants.iter().map(|p| (p, p.config())))
}

fn date_named_keys() -> HashSet<String> {
    // keep in sync with DatePreset in presets.rs
    let variants = [
        DatePreset::Long,
        DatePreset::Short,
        DatePreset::Numeric,
        DatePreset::Iso,
    ];
    preset_keys(variants.iter().map(|p| (p, p.config())))
}

fn title_named_keys() -> HashSet<String> {
    // keep in sync with TitlePreset in presets.rs
    let variants = [
        TitlePreset::Apa,
        TitlePreset::Chicago,
        TitlePreset::Ieee,
        TitlePreset::Humanities,
        TitlePreset::JournalEmphasis,
        TitlePreset::Scientific,
    ];
    preset_keys(variants.iter().map(|p| (p, p.config())))
}

fn locator_named_keys() -> HashSet<String> {
    // keep in sync with LocatorPreset in options/locators.rs
    let variants = [LocatorPreset::Note, LocatorPreset::AuthorDate];
    preset_keys(variants.into_iter().map(|p| (p, p.config())))
}

// ── Human-readable output ───────────────────────────────────────────────────

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
fn print_config_preset_report(report: &ConfigPresetReport) {
    println!("=== Config-Preset Discovery Report ===\n");
    println!("Styles analyzed: {}", report.total_analyzed);
    println!("Parse errors:    {}", report.parse_errors);
    println!();

    for concern in &report.concerns {
        println!("=== {} ===", concern.concern.to_ascii_uppercase());
        println!(
            "  {} styles matched existing presets, {} styles unmatched",
            concern.matched_style_count, concern.unmatched_style_count
        );
        if concern.candidates.is_empty() {
            println!("  (no unnamed shapes ≥ {MIN_FREQUENCY} styles above threshold)");
        } else {
            println!(
                "  {} unnamed shapes (≥ {MIN_FREQUENCY} styles):\n",
                concern.candidates.len()
            );
            println!("  {:>6}  Examples", "Styles");
            println!("  {}", "-".repeat(60));
            for (rank, c) in concern.candidates.iter().enumerate() {
                let examples = c.example_styles.join(", ");
                let examples_trunc = if examples.chars().count() > 44 {
                    format!("{}…", examples.chars().take(43).collect::<String>())
                } else {
                    examples
                };
                println!("  {:>6}  {}", c.corpus_count, examples_trunc);
                let config_str = serde_json::to_string(&c.canonical_config)
                    .unwrap_or_else(|_| String::from("{?}"));
                let config_trunc = if config_str.chars().count() > 100 {
                    format!("{}…", config_str.chars().take(99).collect::<String>())
                } else {
                    config_str
                };
                println!("         Config[{}]: {config_trunc}", rank + 1);
                println!();
            }
        }
        println!();
    }
}
