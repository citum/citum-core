/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Style registry — discovery and alias resolution for citation styles.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Tier classification for a style in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum StyleKind {
    /// Complete style that serves as an inheritance root.
    Base,
    /// Organizational adaptation of a base style (publisher, society, standards body).
    Profile,
    /// Pure alias pointing to a profile or base style.
    Journal,
    /// Standalone style with no aliases and no inheritance role.
    Independent,
}

/// A single entry in a style registry.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RegistryEntry {
    /// Canonical style ID, must match the key used in `get_embedded_style`.
    pub id: String,
    /// Short aliases that resolve to this entry (default empty).
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Name of an embedded style (present for the default registry).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtin: Option<String>,
    /// Relative path to a YAML file (used in local registries).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Subject/domain classification tags (default empty).
    #[serde(default)]
    pub fields: Vec<String>,
    /// Tier classification (base, profile, journal, independent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<StyleKind>,
}

/// A registry of citation styles with alias resolution.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct StyleRegistry {
    /// Version identifier for the registry format.
    pub version: String,
    /// List of style entries in the registry.
    pub styles: Vec<RegistryEntry>,
}

impl StyleRegistry {
    /// Resolve a name or alias to the matching registry entry.
    ///
    /// Checks `id` first, then searches aliases.
    pub fn resolve(&self, name: &str) -> Option<&RegistryEntry> {
        // Check exact ID match
        if let Some(entry) = self.styles.iter().find(|e| e.id == name) {
            return Some(entry);
        }
        // Check aliases
        self.styles
            .iter()
            .find(|e| e.aliases.iter().any(|a| a == name))
    }

    /// All canonical style IDs in the registry.
    pub fn all_ids(&self) -> impl Iterator<Item = &str> {
        self.styles.iter().map(|e| e.id.as_str())
    }

    /// Merge another registry over self (self wins on ID conflict).
    ///
    /// Entries from `base` are included first. If an entry in `self`
    /// has the same ID as one in `base`, the entry from `self` replaces it.
    /// New entries from `self` are appended.
    #[must_use]
    pub fn merge_over(&self, base: &StyleRegistry) -> StyleRegistry {
        let mut result = base.clone();
        for entry in &self.styles {
            if let Some(pos) = result.styles.iter().position(|e| e.id == entry.id) {
                result.styles[pos] = entry.clone();
            } else {
                result.styles.push(entry.clone());
            }
        }
        result
    }

    /// Build a registry from embedded style name and alias slices.
    ///
    /// Used to construct the default registry from hardcoded embedded data.
    pub fn from_slices(names: &[&str], aliases: &[(&str, &str)]) -> Self {
        let mut styles = Vec::new();

        // Create entries for each embedded style name.
        for name in names {
            let style_aliases: Vec<String> = aliases
                .iter()
                .filter(|(_, full)| full == name)
                .map(|(alias, _)| (*alias).to_string())
                .collect();

            styles.push(RegistryEntry {
                id: (*name).to_string(),
                aliases: style_aliases,
                builtin: Some((*name).to_string()),
                path: None,
                description: None,
                fields: Vec::new(),
                kind: None,
            });
        }

        StyleRegistry {
            version: "1".to_string(),
            styles,
        }
    }

    /// Load the embedded default registry from the compiled-in YAML data.
    ///
    /// # Panics
    /// Panics only if the embedded YAML is malformed (should never happen in
    /// a correctly built binary).
    pub fn load_default() -> Self {
        let bytes = include_bytes!("../../../registry/default.yaml");
        serde_yaml::from_slice(bytes).expect("embedded registry/default.yaml is valid YAML")
    }

    /// Load a registry from a YAML file on disk.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or if the YAML cannot be parsed.
    /// Also returns an error if any entry does not have exactly one of `builtin` or `path`.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read(path)?;
        let registry: Self = serde_yaml::from_slice(&content)?;
        // Validate: each entry must have exactly one of builtin or path.
        for entry in &registry.styles {
            match (&entry.builtin, &entry.path) {
                (None, None) => {
                    return Err(format!(
                        "Registry entry '{}' must have either 'builtin' or 'path'",
                        entry.id
                    )
                    .into());
                }
                (Some(_), Some(_)) => {
                    return Err(format!(
                        "Registry entry '{}' cannot have both 'builtin' and 'path'",
                        entry.id
                    )
                    .into());
                }
                _ => {}
            }
        }
        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_exact_id() {
        let registry = StyleRegistry {
            version: "1".to_string(),
            styles: vec![RegistryEntry {
                id: "apa-7th".to_string(),
                aliases: vec!["apa".to_string()],
                builtin: Some("apa-7th".to_string()),
                path: None,
                description: Some("APA 7th edition".to_string()),
                fields: vec!["psychology".to_string()],
                kind: None,
            }],
        };

        assert!(registry.resolve("apa-7th").is_some());
        assert_eq!(registry.resolve("apa-7th").unwrap().id, "apa-7th");
    }

    #[test]
    fn test_resolve_alias() {
        let registry = StyleRegistry {
            version: "1".to_string(),
            styles: vec![RegistryEntry {
                id: "apa-7th".to_string(),
                aliases: vec!["apa".to_string()],
                builtin: Some("apa-7th".to_string()),
                path: None,
                description: Some("APA 7th edition".to_string()),
                fields: vec!["psychology".to_string()],
                kind: None,
            }],
        };

        assert!(registry.resolve("apa").is_some());
        assert_eq!(registry.resolve("apa").unwrap().id, "apa-7th");
    }

    #[test]
    fn test_all_ids() {
        let registry = StyleRegistry {
            version: "1".to_string(),
            styles: vec![
                RegistryEntry {
                    id: "apa-7th".to_string(),
                    aliases: vec!["apa".to_string()],
                    builtin: Some("apa-7th".to_string()),
                    path: None,
                    description: None,
                    fields: vec![],
                    kind: None,
                },
                RegistryEntry {
                    id: "mla".to_string(),
                    aliases: vec![],
                    builtin: Some("mla".to_string()),
                    path: None,
                    description: None,
                    fields: vec![],
                    kind: None,
                },
            ],
        };

        let ids: Vec<_> = registry.all_ids().collect();
        assert_eq!(ids, vec!["apa-7th", "mla"]);
    }

    #[test]
    fn test_merge_over() {
        let base = StyleRegistry {
            version: "1".to_string(),
            styles: vec![RegistryEntry {
                id: "apa-7th".to_string(),
                aliases: vec!["apa".to_string()],
                builtin: Some("apa-7th".to_string()),
                path: None,
                description: Some("APA 7th edition".to_string()),
                fields: vec!["psychology".to_string()],
                kind: None,
            }],
        };

        let custom = StyleRegistry {
            version: "1".to_string(),
            styles: vec![
                RegistryEntry {
                    id: "custom-style".to_string(),
                    aliases: vec!["custom".to_string()],
                    path: Some(PathBuf::from("custom.yaml")),
                    builtin: None,
                    description: Some("Custom style".to_string()),
                    fields: vec![],
                    kind: None,
                },
                RegistryEntry {
                    id: "apa-7th".to_string(),
                    aliases: vec!["apa".to_string()],
                    builtin: Some("apa-7th".to_string()),
                    path: None,
                    description: Some("APA 7th edition (modified)".to_string()),
                    fields: vec!["psychology".to_string(), "custom".to_string()],
                    kind: None,
                },
            ],
        };

        let merged = custom.merge_over(&base);

        assert_eq!(merged.styles.len(), 2);
        assert!(merged.resolve("custom").is_some());
        assert_eq!(
            merged.resolve("apa-7th").unwrap().description,
            Some("APA 7th edition (modified)".to_string())
        );
    }

    #[test]
    fn test_from_slices() {
        let names = &["apa-7th", "mla"];
        let aliases = &[("apa", "apa-7th"), ("mla", "mla")];

        let registry = StyleRegistry::from_slices(names, aliases);

        assert_eq!(registry.styles.len(), 2);
        assert_eq!(registry.resolve("apa").unwrap().id, "apa-7th");
        assert_eq!(registry.resolve("mla").unwrap().id, "mla");
    }
}
