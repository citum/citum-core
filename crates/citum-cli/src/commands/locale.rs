/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Locale subcommands: list, add, remove.

use super::CliResult;
use super::lint::{load_raw_locale, run_lint_locale};
use super::util::{confirm, validate_resource_name};
use crate::args::{LocaleCommands, StyleCatalogFormat};
use crate::table::build_table;
use citum_store::{StoreConfig, StoreResolver, platform_data_dir};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct LocaleRow {
    source: String,
    id: String,
}

fn embedded_locale_ids() -> Vec<String> {
    citum_schema::embedded::EMBEDDED_LOCALE_IDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn locale_rows(source: &str) -> Result<Vec<LocaleRow>, Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    if matches!(source, "all" | "embedded") {
        rows.extend(embedded_locale_ids().into_iter().map(|id| LocaleRow {
            source: "embedded".to_string(),
            id,
        }));
    }
    if matches!(source, "all" | "installed")
        && let Some(data_dir) = platform_data_dir()
    {
        let config = StoreConfig::load().unwrap_or_default();
        let resolver = StoreResolver::new(data_dir, config.store_format());
        rows.extend(
            resolver
                .list_locales()
                .unwrap_or_default()
                .into_iter()
                .map(|id| LocaleRow {
                    source: "installed".to_string(),
                    id,
                }),
        );
    }
    if !matches!(source, "all" | "embedded" | "installed") {
        return Err(
            format!("unknown source '{source}' (expected all, embedded, or installed)").into(),
        );
    }
    Ok(rows)
}

pub(super) fn dispatch(command: LocaleCommands) -> CliResult {
    match command {
        LocaleCommands::List { source, format } => run_locale_list(&source, format),
        LocaleCommands::Add { path } => run_locale_add(&path),
        LocaleCommands::Remove { name, yes } => run_locale_remove(&name, yes),
        LocaleCommands::Lint(args) => run_lint_locale(args),
    }
}

fn run_locale_list(source: &str, format: StyleCatalogFormat) -> CliResult {
    let rows = locale_rows(source)?;
    if format == StyleCatalogFormat::Json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    let table_rows = rows
        .iter()
        .map(|row| vec![row.source.clone(), row.id.clone()])
        .collect();
    println!("{}", build_table(&["Source", "ID"], table_rows));
    Ok(())
}

fn run_locale_add(path: &Path) -> CliResult {
    let _ = load_raw_locale(path)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    let name = resolver.install_locale(path)?;
    println!("Installed locale: {name}");
    Ok(())
}

fn run_locale_remove(name: &str, yes: bool) -> CliResult {
    validate_resource_name(name)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    let locales = resolver.list_locales()?;
    if !locales.contains(&name.to_string()) {
        return Err(format!("installed locale not found: {name}").into());
    }
    if !yes && !confirm(&format!("Remove installed locale '{name}'?"))? {
        return Ok(());
    }
    resolver.remove_locale(name)?;
    println!("Removed locale: {name}");
    Ok(())
}
