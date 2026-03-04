//! Store format detection and serialization.

use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

/// Supported serialization formats for stored styles and locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreFormat {
    /// YAML format
    Yaml,
    /// JSON format
    Json,
    /// CBOR (binary) format
    Cbor,
}

#[derive(Error, Debug)]
#[error("invalid store format: {0}")]
pub struct FormatParseError(pub String);

impl FromStr for StoreFormat {
    type Err = FormatParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yaml" | "yml" => Ok(StoreFormat::Yaml),
            "json" => Ok(StoreFormat::Json),
            "cbor" => Ok(StoreFormat::Cbor),
            _ => Err(FormatParseError(s.to_string())),
        }
    }
}

impl std::fmt::Display for StoreFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreFormat::Yaml => write!(f, "yaml"),
            StoreFormat::Json => write!(f, "json"),
            StoreFormat::Cbor => write!(f, "cbor"),
        }
    }
}

impl StoreFormat {
    /// Detect format from file extension.
    pub fn detect_from_extension(path: &Path) -> Option<StoreFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "yaml" | "yml" => Some(StoreFormat::Yaml),
                "json" => Some(StoreFormat::Json),
                "cbor" => Some(StoreFormat::Cbor),
                _ => None,
            })
    }

    /// File extension without leading dot.
    pub fn extension(&self) -> &'static str {
        match self {
            StoreFormat::Yaml => "yaml",
            StoreFormat::Json => "json",
            StoreFormat::Cbor => "cbor",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert_eq!("yaml".parse::<StoreFormat>().unwrap(), StoreFormat::Yaml);
        assert_eq!("json".parse::<StoreFormat>().unwrap(), StoreFormat::Json);
        assert_eq!("cbor".parse::<StoreFormat>().unwrap(), StoreFormat::Cbor);
        assert_eq!("YAML".parse::<StoreFormat>().unwrap(), StoreFormat::Yaml);
    }

    #[test]
    fn test_format_display() {
        assert_eq!(StoreFormat::Yaml.to_string(), "yaml");
        assert_eq!(StoreFormat::Json.to_string(), "json");
        assert_eq!(StoreFormat::Cbor.to_string(), "cbor");
    }

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(
            StoreFormat::detect_from_extension(Path::new("style.yaml")),
            Some(StoreFormat::Yaml)
        );
        assert_eq!(
            StoreFormat::detect_from_extension(Path::new("style.json")),
            Some(StoreFormat::Json)
        );
        assert_eq!(
            StoreFormat::detect_from_extension(Path::new("style.cbor")),
            Some(StoreFormat::Cbor)
        );
    }

    #[test]
    fn test_extension() {
        assert_eq!(StoreFormat::Yaml.extension(), "yaml");
        assert_eq!(StoreFormat::Json.extension(), "json");
        assert_eq!(StoreFormat::Cbor.extension(), "cbor");
    }
}
