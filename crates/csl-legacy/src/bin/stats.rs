#![allow(missing_docs, reason = "bin crate")]

//! Utility binary for scanning a style directory and summarizing parse failures.

use csl_legacy::parser::parse_style;
use roxmltree::Document;
use std::fs;

fn main() {
    let styles_dir = "styles";
    let entries = match fs::read_dir(styles_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading styles directory: {e}");
            return;
        }
    };

    let mut total = 0;
    let mut success = 0;
    let mut errors = 0;
    let mut error_types = std::collections::HashMap::new();

    println!("Starting analysis of styles in {styles_dir}...");

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("csl") {
            total += 1;

            // Read file
            let Ok(text) = fs::read_to_string(&path) else {
                *error_types
                    .entry("File read error".to_string())
                    .or_insert(0) += 1;
                errors += 1;
                continue;
            };

            // Parse XML
            let doc = match Document::parse(&text) {
                Ok(d) => d,
                Err(e) => {
                    *error_types
                        .entry(format!("XML Parse Error: {e}"))
                        .or_insert(0) += 1;
                    errors += 1;
                    continue;
                }
            };

            // Parse CSL
            match parse_style(doc.root_element()) {
                Ok(_) => success += 1,
                Err(e) => {
                    // Simplify error message to group them
                    let simple_err = if e.contains("missing") {
                        e.split_whitespace().collect::<Vec<_>>().join(" ")
                    } else {
                        e.clone()
                    };
                    *error_types.entry(simple_err).or_insert(0) += 1;
                    errors += 1;
                    // Optional: Print first few failures
                    // if errors <= 5 {
                    //     println!("Failed: {:?} -> {}", path.file_name().unwrap(), e);
                    // }
                }
            }
        }
    }

    println!("\n=== ANALYSIS COMPLETE ===");
    println!("Total Styles: {total}");
    println!(
        "Success:      {} ({:.1}%)",
        success,
        (f64::from(success) / f64::from(total)) * 100.0
    );
    println!(
        "Failures:     {} ({:.1}%)",
        errors,
        (f64::from(errors) / f64::from(total)) * 100.0
    );
    println!("\n=== TOP ERRORS ===");

    let mut err_vec: Vec<_> = error_types.iter().collect();
    err_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (msg, count) in err_vec.into_iter().take(20) {
        println!("{count:4}x {msg}");
    }
}
