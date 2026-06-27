/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Install and remove user-installed styles, and the style-browser action shim.

use super::CliResult;
use super::catalog::{CatalogSourceFilter, style_catalog_entries, style_row_matches_query};
use super::util::{confirm, validate_resource_name};
use crate::style_browser::StyleBrowserActions;
use crate::style_catalog::StyleCatalogRow;
use crate::style_resolver::load_any_style;
use citum_schema::Style;
use citum_store::{StoreConfig, StoreResolver, platform_data_dir};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

fn style_install_name_from_url(input: &str) -> Result<String, Box<dyn Error>> {
    let url = url::Url::parse(input)?;
    url.path_segments()
        .and_then(Iterator::last)
        .and_then(|segment| segment.split('.').next())
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("cannot infer style name from {input}").into())
}

fn write_installed_style(name: &str, style: &Style) -> CliResult {
    validate_resource_name(name)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let format = config.store_format();
    let styles_dir = data_dir.join("styles");
    fs::create_dir_all(&styles_dir)?;
    let path = styles_dir.join(format!("{name}.{}", format.extension()));
    let bytes = match format {
        citum_store::StoreFormat::Yaml => serde_yaml::to_string(style)?.into_bytes(),
        citum_store::StoreFormat::Json => serde_json::to_string(style)?.into_bytes(),
        citum_store::StoreFormat::Cbor => {
            let mut bytes = Vec::new();
            ciborium::ser::into_writer(style, &mut bytes)?;
            bytes
        }
    };
    fs::write(path, bytes)?;
    Ok(())
}

fn remove_installed_style(name: &str) -> CliResult {
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    let styles = resolver.list_styles()?;
    if !styles.iter().any(|style| style == name) {
        return Err(format!("installed style not found: {name}").into());
    }
    resolver.remove_style(name)?;
    Ok(())
}

pub(super) struct CliStyleBrowserActions;

impl StyleBrowserActions for CliStyleBrowserActions {
    fn install_style(&mut self, row: &StyleCatalogRow) -> CliResult {
        let style = load_any_style(&row.id, false)?;
        write_installed_style(&row.id, &style)
    }

    fn remove_style(&mut self, row: &StyleCatalogRow) -> CliResult {
        remove_installed_style(&row.id)
    }
}

fn install_style_from_file(path: &Path) -> Result<String, Box<dyn Error>> {
    if !path.exists() || !path.is_file() {
        return Err(format!("file not found: {}", path.display()).into());
    }
    let _ = load_any_style(&path.display().to_string(), false)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    Ok(resolver.install_style(path)?)
}

fn select_style_match(query: &str, yes: bool) -> Result<StyleCatalogRow, Box<dyn Error>> {
    let rows = style_catalog_entries(CatalogSourceFilter::All)?;
    let exact: Vec<_> = rows
        .iter()
        .filter(|row| row.id == query || row.aliases.iter().any(|alias| alias == query))
        .cloned()
        .collect();
    if let [row] = exact.as_slice() {
        return Ok(row.clone());
    }

    let mut matches: Vec<_> = rows
        .into_iter()
        .filter(|row| style_row_matches_query(row, query))
        .collect();
    matches.sort_by(|a, b| a.id.len().cmp(&b.id.len()).then_with(|| a.id.cmp(&b.id)));

    match matches.as_slice() {
        [] => Err(format!(
            "style not found: {query}\n\nSearch styles with: citum style search {query}"
        )
        .into()),
        [row] => Ok(row.clone()),
        _ if yes || !io::stdin().is_terminal() => {
            let mut msg = format!("style query is ambiguous: {query}\n\nMatches:");
            for row in matches.iter().take(10) {
                let _ = write!(
                    msg,
                    "\n  - {} ({})",
                    row.id,
                    row.title.as_deref().unwrap_or(&row.source)
                );
            }
            msg.push_str("\n\nRerun with an exact ID or alias.");
            Err(msg.into())
        }
        _ => {
            println!("Multiple styles match '{query}':");
            for (idx, row) in matches.iter().take(10).enumerate() {
                println!(
                    "  {}. {} - {}",
                    idx + 1,
                    row.id,
                    row.title.as_deref().unwrap_or("-")
                );
            }
            print!("Choose a style to install [1-{}]: ", matches.len().min(10));
            io::stdout().flush()?;
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            let choice = response.trim().parse::<usize>()?;
            if choice == 0 || choice > matches.len().min(10) {
                return Err("selection out of range".into());
            }
            matches
                .get(choice - 1)
                .cloned()
                .ok_or_else(|| "selection out of range".into())
        }
    }
}

pub(super) fn run_style_add(query: &str, yes: bool) -> CliResult {
    let path = Path::new(query);
    let name = if path.exists() || query.starts_with("file://") {
        let raw_path = query.strip_prefix("file://").unwrap_or(query);
        install_style_from_file(Path::new(raw_path))?
    } else if query.starts_with("http://") || query.starts_with("https://") {
        let style = load_any_style(query, false)?;
        let name = style_install_name_from_url(query)?;
        write_installed_style(&name, &style)?;
        name
    } else {
        let row = select_style_match(query, yes)?;
        let style = load_any_style(&row.id, false)?;
        write_installed_style(&row.id, &style)?;
        row.id
    };

    println!("Installed style: {name}");
    Ok(())
}

pub(super) fn run_style_remove(name: &str, yes: bool) -> CliResult {
    validate_resource_name(name)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    let styles = resolver.list_styles()?;
    if !styles.contains(&name.to_string()) {
        return Err(format!("installed style not found: {name}").into());
    }
    if !yes && !confirm(&format!("Remove installed style '{name}'?"))? {
        return Ok(());
    }
    resolver.remove_style(name)?;
    println!("Removed style: {name}");
    Ok(())
}
