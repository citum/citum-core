/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Style catalog: row type, source filter, pagination, formatting, and
//! source-aggregating lookup.

use super::CliResult;
use super::registry::load_registry_chain;
use crate::args::StyleCatalogFormat;
use crate::table::build_table;
use citum_schema::RegistryEntry;
use citum_store::{StoreConfig, StoreResolver, platform_data_dir};
use serde::Serialize;
use std::error::Error;
use std::fmt::Write as _;

/// A single row in a style listing. `pub(crate)` so `style_browser` can take
/// rows in its config.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct StyleCatalogRow {
    pub(crate) source: String,
    pub(crate) id: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) fields: Vec<String>,
    pub(crate) url: Option<String>,
}

impl StyleCatalogRow {
    /// Build a row from a registry entry, resolving the title from embedded
    /// style metadata when the entry omits one.
    pub(super) fn from_entry(source: &str, entry: &RegistryEntry) -> Self {
        let title = entry.title.clone().or_else(|| {
            entry.builtin.as_ref().and_then(|builtin| {
                citum_schema::embedded::get_embedded_style(builtin)
                    .and_then(Result::ok)
                    .and_then(|style| style.info.title)
            })
        });

        Self {
            source: source.to_string(),
            id: entry.id.clone(),
            aliases: entry.aliases.clone(),
            title,
            description: entry.description.clone(),
            fields: entry.fields.clone(),
            url: entry.url.clone(),
        }
    }

    /// Build a row for a user-installed style, where only the id is known.
    pub(super) fn installed(id: String) -> Self {
        Self {
            source: "installed".to_string(),
            id,
            aliases: Vec::new(),
            title: None,
            description: None,
            fields: Vec::new(),
            url: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CatalogSourceFilter<'a> {
    All,
    Embedded,
    Installed,
    Registry(&'a str),
}

impl<'a> CatalogSourceFilter<'a> {
    pub(super) fn parse(source: &'a str) -> Result<Self, Box<dyn Error>> {
        match source {
            "all" => Ok(Self::All),
            "embedded" => Ok(Self::Embedded),
            "installed" => Ok(Self::Installed),
            s if s.starts_with("registry:") => {
                let name = s.trim_start_matches("registry:");
                if name.is_empty() {
                    Err("registry source filter requires a name: registry:<name>".into())
                } else {
                    Ok(Self::Registry(name))
                }
            }
            _ => Err(format!(
                "unknown source '{source}' (expected all, embedded, installed, or registry:<name>)"
            )
            .into()),
        }
    }

    pub(super) fn label(self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Embedded => "embedded".to_string(),
            Self::Installed => "installed".to_string(),
            Self::Registry(name) => format!("registry:{name}"),
        }
    }
}

pub(super) fn style_entry_kind(entry: &RegistryEntry) -> &'static str {
    if entry.builtin.is_some() {
        "embedded"
    } else if entry.url.is_some() {
        "url"
    } else if entry.path.is_some() {
        "path"
    } else {
        "unknown"
    }
}

fn style_entry_matches_source(source_name: &str, source: CatalogSourceFilter<'_>) -> bool {
    match source {
        CatalogSourceFilter::All => true,
        CatalogSourceFilter::Embedded => source_name == "embedded",
        CatalogSourceFilter::Installed => source_name == "installed",
        CatalogSourceFilter::Registry(name) => source_name == format!("registry:{name}"),
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct StyleCatalogPage {
    pub(super) limit: Option<usize>,
    pub(super) offset: usize,
}

pub(super) fn paginate_style_catalog_rows(
    mut rows: Vec<StyleCatalogRow>,
    page: StyleCatalogPage,
) -> (usize, Vec<StyleCatalogRow>) {
    let total = rows.len();
    if page.offset >= total {
        return (total, Vec::new());
    }
    rows.drain(..page.offset);
    if let Some(limit) = page.limit {
        rows.truncate(limit);
    }
    (total, rows)
}

pub(super) fn print_style_catalog_rows(
    rows: &[StyleCatalogRow],
    total: usize,
    source: &str,
    format: StyleCatalogFormat,
) -> CliResult {
    if format == StyleCatalogFormat::Json {
        println!("{}", serde_json::to_string_pretty(rows)?);
        return Ok(());
    }

    print!("{}", format_style_catalog_text(rows, total, source));
    Ok(())
}

pub(super) fn format_style_catalog_text(
    rows: &[StyleCatalogRow],
    total: usize,
    source: &str,
) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "{total} {source} styles");
    if rows.len() != total {
        let _ = writeln!(output, "showing {}", rows.len());
    }
    output.push('\n');

    let table_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            vec![
                row.source.clone(),
                row.id.clone(),
                row.title.as_deref().unwrap_or("-").to_string(),
            ]
        })
        .collect();

    let table = build_table(&["Source", "ID", "Title"], table_rows);
    output.push_str(&table);
    output
}

pub(super) fn style_row_matches_query(row: &StyleCatalogRow, query: &str) -> bool {
    let query = query.to_lowercase();
    row.id.to_lowercase().contains(&query)
        || row
            .aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(&query))
        || row
            .title
            .as_ref()
            .is_some_and(|title| title.to_lowercase().contains(&query))
        || row
            .description
            .as_ref()
            .is_some_and(|description| description.to_lowercase().contains(&query))
        || row
            .fields
            .iter()
            .any(|field| field.to_lowercase().contains(&query))
}

/// Aggregate style rows from all configured sources (registries + installed
/// styles), filtered by `source`.
pub(super) fn style_catalog_entries(
    source: CatalogSourceFilter<'_>,
) -> Result<Vec<StyleCatalogRow>, Box<dyn Error>> {
    let mut rows = Vec::new();
    for loaded in load_registry_chain()? {
        for entry in &loaded.registry.styles {
            let actual_kind = style_entry_kind(entry);
            let row_source = if loaded.name == "embedded" {
                if actual_kind == "embedded" {
                    "embedded".to_string()
                } else {
                    "registry:default".to_string()
                }
            } else {
                format!("registry:{}", loaded.name)
            };

            if style_entry_matches_source(&row_source, source) {
                // Special case: if filtering for 'embedded', only show truly embedded entries
                if matches!(source, CatalogSourceFilter::Embedded) && actual_kind != "embedded" {
                    continue;
                }
                rows.push(StyleCatalogRow::from_entry(&row_source, entry));
            }
        }
    }

    if style_entry_matches_source("installed", source)
        && let Some(data_dir) = platform_data_dir()
    {
        let config = StoreConfig::load().unwrap_or_default();
        let resolver = StoreResolver::new(data_dir, config.store_format());
        rows.extend(
            resolver
                .list_styles()?
                .into_iter()
                .map(StyleCatalogRow::installed),
        );
    }

    Ok(rows)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn test_style_catalog_embedded_title_falls_back_to_style_metadata() {
        let registry = citum_schema::embedded::default_registry();
        let entry = registry.resolve("apa").expect("APA alias should resolve");

        let row = StyleCatalogRow::from_entry("embedded", entry);

        assert_eq!(
            row.title.as_deref(),
            Some("American Psychological Association 7th edition")
        );
    }

    #[test]
    fn test_style_catalog_source_filter_and_pagination() {
        let rows = style_catalog_entries(CatalogSourceFilter::Embedded)
            .expect("embedded catalog should load");
        let (total, page) = paginate_style_catalog_rows(
            rows,
            StyleCatalogPage {
                limit: Some(2),
                offset: 1,
            },
        );

        assert!(total > 2);
        assert_eq!(page.len(), 2);
        assert!(page.iter().all(|row| row.source == "embedded"));
    }

    #[test]
    fn test_style_catalog_search_matches_embedded_title() {
        let registry = citum_schema::embedded::default_registry();
        let rows: Vec<_> = registry
            .styles
            .iter()
            .map(|entry| StyleCatalogRow::from_entry("embedded", entry))
            .filter(|row| style_row_matches_query(row, "Psychological Association"))
            .collect();

        assert!(rows.iter().any(|row| row.id == "apa-7th"));
    }

    #[test]
    fn test_style_catalog_text_output_contains_table() {
        let rows = vec![StyleCatalogRow {
            source: "embedded".to_string(),
            id: "alpha".to_string(),
            aliases: Vec::new(),
            title: Some("Alpha (biblatex-alpha)".to_string()),
            description: None,
            fields: Vec::new(),
            url: None,
        }];

        let output = format_style_catalog_text(&rows, 3, "embedded");

        assert!(output.contains("3 embedded styles"));
        assert!(output.contains("showing 1"));
        assert!(output.contains("Source"));
        assert!(output.contains("ID"));
        assert!(output.contains("Title"));
        assert!(output.contains("embedded"));
        assert!(output.contains("alpha"));
        assert!(output.contains("Alpha (biblatex-alpha)"));
    }
}
