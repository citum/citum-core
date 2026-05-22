/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Store configuration from `~/.config/citum/config.yaml` or `~/.config/citum/config.toml`.

use crate::format::StoreFormat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for configuration operations.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("yaml parse error: {0}")]
    YamlLoad(#[from] serde_yaml::Error),
    #[error("yaml save error: {0}")]
    YamlSave(String),
}

/// Configuration for a single remote registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Name of the registry.
    pub name: String,
    /// URL to fetch the registry YAML from.
    pub url: String,
    /// Resolution priority; higher values are checked first. Defaults to 50.
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Cache TTL in seconds. Defaults to 3600 (1 hour).
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,
    /// Whether this registry is trusted (future use).
    #[serde(default)]
    pub trusted: bool,
}

fn default_priority() -> i32 {
    50
}

fn default_ttl_secs() -> u64 {
    3600
}

/// Configuration for the citation store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    /// Store section configuration.
    #[serde(default)]
    pub store: StoreSection,
    /// Configured remote registries.
    #[serde(default)]
    pub registries: Vec<RegistryConfig>,
    /// Optional override for the IPFS HTTP gateway used to resolve `cid:`
    /// references. When `None`, the resolver uses
    /// [`crate::resolver::DEFAULT_CID_GATEWAY`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid_gateway: Option<String>,
}

/// The `[store]` section of the config file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoreSection {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "yaml".to_string()
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            store: StoreSection {
                format: "yaml".to_string(),
            },
            registries: Vec::new(),
            cid_gateway: None,
        }
    }
}

impl StoreConfig {
    /// Load configuration from `~/.config/citum/config.yaml` or `~/.config/citum/config.toml`.
    ///
    /// Returns the default config if neither file exists.
    ///
    /// # Errors
    ///
    /// Returns an error when the config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self, ConfigError> {
        if let Some(citum_dir) = crate::platform_config_dir() {
            // Try YAML first
            let yaml_path = citum_dir.join("config.yaml");
            if yaml_path.exists() {
                let content = std::fs::read_to_string(&yaml_path)?;
                return serde_yaml::from_str(&content).map_err(ConfigError::YamlLoad);
            }

            // Fall back to TOML
            let toml_path = citum_dir.join("config.toml");
            if toml_path.exists() {
                let content = std::fs::read_to_string(&toml_path)?;
                return Ok(toml::from_str(&content)?);
            }
        }
        Ok(StoreConfig::default())
    }

    /// Save configuration to `~/.config/citum/config.yaml`.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(citum_dir) = crate::platform_config_dir() {
            std::fs::create_dir_all(&citum_dir)?;
            let config_path = citum_dir.join("config.yaml");
            let content =
                serde_yaml::to_string(self).map_err(|e| ConfigError::YamlSave(e.to_string()))?;
            std::fs::write(&config_path, content)?;
            Ok(())
        } else {
            Err(ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "config directory not found",
            )))
        }
    }

    /// Get the configured store format.
    #[must_use]
    pub fn store_format(&self) -> StoreFormat {
        self.store.format.parse().unwrap_or(StoreFormat::Yaml)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = StoreConfig::default();
        assert_eq!(cfg.store.format, "yaml");
        assert_eq!(cfg.store_format(), StoreFormat::Yaml);
        assert!(cfg.registries.is_empty());
    }

    #[test]
    fn test_parse_toml_config() {
        let toml_str = r#"
[store]
format = "json"
"#;
        let cfg: StoreConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.store.format, "json");
        assert_eq!(cfg.store_format(), StoreFormat::Json);
    }

    #[test]
    fn test_parse_yaml_config() {
        let yaml_str = r#"
store:
  format: json
registries:
  - name: example
    url: https://example.org/registry.yaml
    priority: 100
    ttl_secs: 3600
    trusted: true
"#;
        let cfg: StoreConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(cfg.store.format, "json");
        assert_eq!(cfg.registries.len(), 1);
        assert_eq!(cfg.registries[0].name, "example");
        assert_eq!(cfg.registries[0].priority, 100);
    }
}
