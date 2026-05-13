/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#![allow(missing_docs, reason = "bin/main")]
/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! CSL Style Analyzer
//!
//! Analyzes CSL 1.0 styles in a directory to collect statistics
//! and identify patterns for guiding migration development.

mod analyzer;
mod profile_discovery;
mod ranker;
mod savings;
mod util;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    #[allow(clippy::expect_used, reason = "fatal bootstrap error")]
    let styles_dir = args
        .get(1)
        .expect("Error: styles directory path required as first argument");
    let json_output = args.iter().any(|arg| arg == "--json");
    let rank_parents = args.iter().any(|arg| arg == "--rank-parents");
    let quantify_savings = args.iter().any(|arg| arg == "--quantify-savings");
    let identify_profiles = args.iter().any(|arg| arg == "--identify-profiles");

    // Check for format filter (--format author-date, --format numeric, etc.)
    let format_filter = args
        .iter()
        .position(|a| a == "--format")
        .and_then(|i| args.get(i + 1))
        .map(std::string::String::as_str);

    if identify_profiles {
        profile_discovery::run_profile_discovery(styles_dir, json_output);
    } else if quantify_savings {
        savings::run_savings_report(styles_dir, json_output);
    } else if rank_parents {
        ranker::run_parent_ranker(styles_dir, json_output, format_filter);
    } else {
        analyzer::run_style_analyzer(styles_dir, json_output);
    }
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
fn print_usage() {
    tracing::debug!("CSL Style Analyzer");
    tracing::debug!("");
    tracing::debug!("Usage:");
    tracing::debug!("  citum_analyze <styles_dir> [--json]");
    tracing::debug!("      Analyze all .csl files and report feature statistics.");
    tracing::debug!("");
    tracing::debug!("  citum_analyze <styles_dir> --rank-parents [--json] [--format <format>]");
    tracing::debug!("      Rank parent styles by how many dependent styles reference them.");
    tracing::debug!(
        "      Use --format to filter by citation format (author-date, numeric, note, label)."
    );
    tracing::debug!("");
    tracing::debug!("  citum_analyze <styles_dir> --quantify-savings [--json]");
    tracing::debug!("      Estimate how many CSL styles presets and locale overrides can replace.");
    tracing::debug!("");
    tracing::debug!("  citum_analyze <styles_dir> --identify-profiles [--json]");
    tracing::debug!(
        "      Audit the current journal-profile candidate shortlist with normalized IDs and repo evidence."
    );
    tracing::debug!("");
    tracing::debug!("Examples:");
    tracing::debug!("  citum_analyze styles-legacy/");
    tracing::debug!("  citum_analyze styles-legacy/ --rank-parents");
    tracing::debug!("  citum_analyze styles-legacy/ --rank-parents --format author-date --json");
    tracing::debug!("  citum_analyze styles-legacy/ --quantify-savings --json");
    tracing::debug!("  citum_analyze styles-legacy/ --identify-profiles --json");
}
