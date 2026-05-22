/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Lint subcommands for style and locale files.

use super::CliResult;
use crate::args::{LintLocaleArgs, LintStyleArgs};
use crate::output::print_lint_report;
use crate::style_resolver::{load_any_style, load_locale_file};
use citum_schema::lint::{lint_raw_locale, lint_style_against_locale};
use citum_schema::locale::RawLocale;
use std::error::Error;
use std::fs;
use std::path::Path;

pub(super) fn run_lint_locale(args: LintLocaleArgs) -> CliResult {
    let raw = load_raw_locale(&args.path)?;
    let report = lint_raw_locale(&raw);
    print_lint_report("locale lint", &report);
    if report.has_errors() {
        return Err(format!("Locale lint failed: {}", args.path.display()).into());
    }
    Ok(())
}

pub(super) fn run_lint_style(args: LintStyleArgs) -> CliResult {
    let style = load_any_style(&args.style, false)?;
    let locale = load_locale_file(&args.locale)?;
    let report = lint_style_against_locale(&style, &locale);
    print_lint_report("style lint", &report);
    Ok(())
}

/// Read a `RawLocale` from a YAML, JSON, or CBOR file. Default to YAML.
pub(super) fn load_raw_locale(path: &Path) -> Result<RawLocale, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    let raw = match ext {
        "cbor" => ciborium::de::from_reader::<RawLocale, _>(std::io::Cursor::new(&bytes))?,
        "json" => serde_json::from_slice::<RawLocale>(&bytes)?,
        _ => serde_yaml::from_slice::<RawLocale>(&bytes)?,
    };

    Ok(raw)
}
