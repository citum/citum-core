/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Shared style catalog row data used by CLI commands and the style browser.

use citum_schema::RegistryEntry;
use serde::Serialize;

/// A single row in a style listing or browser view.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct StyleCatalogRow {
    /// Source label for the row, such as `embedded`, `installed`, or `registry:<name>`.
    pub(crate) source: String,
    /// Canonical style identifier.
    pub(crate) id: String,
    /// Alternate names that resolve to the same style.
    pub(crate) aliases: Vec<String>,
    /// Human-readable style title, when known.
    pub(crate) title: Option<String>,
    /// Optional catalog summary.
    pub(crate) description: Option<String>,
    /// Citation fields associated with the style catalog entry.
    pub(crate) fields: Vec<String>,
    /// Remote or registry URL for the style, when known.
    pub(crate) url: Option<String>,
}

impl StyleCatalogRow {
    /// Build a row from a registry entry, resolving the title from embedded
    /// style metadata when the entry omits one.
    pub(crate) fn from_entry(source: &str, entry: &RegistryEntry) -> Self {
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
    pub(crate) fn installed(id: String) -> Self {
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
