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
mod ranker;
mod savings;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let styles_dir = &args[1];
    let json_output = args.contains(&"--json".to_string());
    let rank_parents = args.contains(&"--rank-parents".to_string());
    let quantify_savings = args.contains(&"--quantify-savings".to_string());

    // Check for format filter (--format author-date, --format numeric, etc.)
    let format_filter = args
        .iter()
        .position(|a| a == "--format")
        .and_then(|i| args.get(i + 1))
        .map(std::string::String::as_str);

    if quantify_savings {
        savings::run_savings_report(styles_dir, json_output);
    } else if rank_parents {
        ranker::run_parent_ranker(styles_dir, json_output, format_filter);
    } else {
        analyzer::run_style_analyzer(styles_dir, json_output);
    }
}

fn print_usage() {
    eprintln!("CSL Style Analyzer");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  citum_analyze <styles_dir> [--json]");
    eprintln!("      Analyze all .csl files and report feature statistics.");
    eprintln!();
    eprintln!("  citum_analyze <styles_dir> --rank-parents [--json] [--format <format>]");
    eprintln!("      Rank parent styles by how many dependent styles reference them.");
    eprintln!(
        "      Use --format to filter by citation format (author-date, numeric, note, label)."
    );
    eprintln!();
    eprintln!("  citum_analyze <styles_dir> --quantify-savings [--json]");
    eprintln!("      Estimate how many CSL styles presets and locale overrides can replace.");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  citum_analyze styles-legacy/");
    eprintln!("  citum_analyze styles-legacy/ --rank-parents");
    eprintln!("  citum_analyze styles-legacy/ --rank-parents --format author-date --json");
    eprintln!("  citum_analyze styles-legacy/ --quantify-savings --json");
}
