//! Store configuration from `~/.config/citum/config.toml`.

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
}

/// Configuration for the citation store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    #[serde(default)]
    pub store: StoreSection,
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
        }
    }
}

impl StoreConfig {
    /// Load configuration from `~/.config/citum/config.toml`.
    ///
    /// Returns the default config if the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error when the config file exists but cannot be read or
    /// parsed as TOML.
    pub fn load() -> Result<Self, ConfigError> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("citum").join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                return Ok(toml::from_str(&content)?);
            }
        }
        Ok(StoreConfig::default())
    }

    /// Get the configured store format.
    pub fn store_format(&self) -> StoreFormat {
        self.store.format.parse().unwrap_or(StoreFormat::Yaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = StoreConfig::default();
        assert_eq!(cfg.store.format, "yaml");
        assert_eq!(cfg.store_format(), StoreFormat::Yaml);
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[store]
format = "json"
"#;
        let cfg: StoreConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.store.format, "json");
        assert_eq!(cfg.store_format(), StoreFormat::Json);
    }
}
