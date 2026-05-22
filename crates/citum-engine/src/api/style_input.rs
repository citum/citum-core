/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Style resolution input type for interactive APIs.

use citum_schema::Style;
use serde::{Deserialize, Serialize};

/// A style reference that can be resolved locally or by an external resolver.
///
/// This union type allows callers to supply a style by local path, inline YAML,
/// or as an identifier that requires a remote resolver. Only `Path` and `Yaml`
/// can be resolved locally; `Id` and `Uri` require the citum-server resolver chain.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum StyleInput {
    /// A style identifier to be resolved from registries (builtin, store, remote).
    /// Requires external resolver chain — local resolution returns UnresolvedInput error.
    Id(String),
    /// A remote URI to be resolved (requires HTTP access).
    /// Requires external resolver chain — local resolution returns UnresolvedInput error.
    Uri(String),
    /// A local filesystem path to a YAML style file.
    Path(String),
    /// Inline YAML style definition.
    Yaml(String),
}

impl StyleInput {
    /// Resolve the style locally from Path or Yaml variants.
    ///
    /// This method handles local filesystem paths and inline YAML content.
    /// For `Id` and `Uri` variants, which require a resolver chain, this returns
    /// an UnresolvedInput error.
    ///
    /// # Errors
    ///
    /// Returns `FormatDocumentError::UnresolvedInput` for `Id` and `Uri` variants.
    /// Returns `FormatDocumentError::StylePath` for filesystem errors.
    /// Returns `FormatDocumentError::StyleParse` for YAML parsing errors.
    pub fn resolve_local(&self) -> Result<Style, crate::api::FormatDocumentError> {
        match self {
            StyleInput::Path(path) => {
                let yaml_bytes = std::fs::read(path).map_err(|e| {
                    crate::api::FormatDocumentError::StylePath(format!(
                        "Failed to read style from '{}': {}",
                        path, e
                    ))
                })?;
                Style::from_yaml_bytes(&yaml_bytes).map_err(|e| {
                    crate::api::FormatDocumentError::StyleParse(format!(
                        "Failed to parse style from '{}': {}",
                        path, e
                    ))
                })
            }
            StyleInput::Yaml(yaml_str) => {
                Style::from_yaml_bytes(yaml_str.as_bytes()).map_err(|e| {
                    crate::api::FormatDocumentError::StyleParse(format!(
                        "Failed to parse inline YAML style: {}",
                        e
                    ))
                })
            }
            StyleInput::Id(id) => Err(crate::api::FormatDocumentError::UnresolvedInput(format!(
                "Style ID '{}' requires resolver chain (not available in engine)",
                id
            ))),
            StyleInput::Uri(uri) => Err(crate::api::FormatDocumentError::UnresolvedInput(format!(
                "Style URI '{}' requires resolver chain (not available in engine)",
                uri
            ))),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test code uses assertions and panic"
)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn style_input_yaml_resolves_locally() {
        let yaml_content = r#"---
info:
  title: Test Style
  default-locale: en-us
"#;
        let input = StyleInput::Yaml(yaml_content.to_string());
        let result = input.resolve_local();
        assert!(result.is_ok());
    }

    #[test]
    fn style_input_id_returns_unresolved_error() {
        let input = StyleInput::Id("apa-7th".to_string());
        let result = input.resolve_local();
        match result {
            Err(crate::api::FormatDocumentError::UnresolvedInput(msg)) => {
                assert!(msg.contains("apa-7th"));
            }
            _ => panic!("Expected UnresolvedInput error"),
        }
    }

    #[test]
    fn style_input_uri_returns_unresolved_error() {
        let input = StyleInput::Uri("https://example.com/style.yaml".to_string());
        let result = input.resolve_local();
        match result {
            Err(crate::api::FormatDocumentError::UnresolvedInput(msg)) => {
                assert!(msg.contains("https://example.com/style.yaml"));
            }
            _ => panic!("Expected UnresolvedInput error"),
        }
    }

    #[test]
    fn style_input_path_reads_and_parses() {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        let yaml_content = r#"---
info:
  title: Test Style
  default-locale: en-us
"#;
        tmp.write_all(yaml_content.as_bytes())
            .expect("Failed to write temp file");
        tmp.flush().expect("Failed to flush temp file");

        let input = StyleInput::Path(tmp.path().to_string_lossy().to_string());
        let result = input.resolve_local();
        assert!(result.is_ok());
    }

    #[test]
    fn style_input_path_missing_returns_error() {
        let input = StyleInput::Path("/nonexistent/path/style.yaml".to_string());
        let result = input.resolve_local();
        match result {
            Err(crate::api::FormatDocumentError::StylePath(msg)) => {
                assert!(msg.contains("Failed to read"));
            }
            _ => panic!("Expected StylePath error"),
        }
    }

    #[test]
    fn style_input_invalid_yaml_returns_parse_error() {
        let input = StyleInput::Yaml("{ invalid yaml: [".to_string());
        let result = input.resolve_local();
        match result {
            Err(crate::api::FormatDocumentError::StyleParse(msg)) => {
                assert!(msg.contains("Failed to parse"));
            }
            _ => panic!("Expected StyleParse error"),
        }
    }
}
