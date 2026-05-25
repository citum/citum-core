/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Refs input resolution type for interactive APIs.

use crate::reference::Bibliography;
use serde::{Deserialize, Serialize};

/// A refs input that can be resolved locally or by an external resolver.
///
/// This union type allows callers to supply reference data by local YAML file path,
/// inline YAML, or inline JSON (current API). Enables citum-server and bindings to
/// accept references from files (e.g., via pipe transport from LaTeX).
#[derive(Debug, Clone)]
pub enum RefsInput {
    /// Local filesystem path to a YAML refs file.
    Path(String),
    /// Inline YAML refs string.
    Yaml(String),
    /// Inline JSON map of reference objects.
    Json(serde_json::Value),
}

impl<'de> Deserialize<'de> for RefsInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize to a generic Value first so we can inspect the shape.
        let v = serde_json::Value::deserialize(deserializer)?;

        if let Some(object) = v.as_object() {
            let tagged_kind = object
                .get("kind")
                .and_then(|k| k.as_str())
                .filter(|k| matches!(*k, "path" | "yaml" | "json"));
            if tagged_kind.is_none() || object.get("value").is_none() {
                return Ok(RefsInput::Json(v));
            }
        } else {
            return Err(serde::de::Error::custom(
                "refs input must be a tagged object or legacy refs object",
            ));
        }

        // Tagged union: {"kind": "path"|"yaml"|"json", "value": ...}
        let kind = v
            .get("kind")
            .and_then(|k| k.as_str())
            .ok_or_else(|| serde::de::Error::custom("refs input must have a 'kind' field"))?;

        let value = v
            .get("value")
            .ok_or_else(|| serde::de::Error::missing_field("value"))?;

        match kind {
            "path" | "yaml" => {
                let s = value
                    .as_str()
                    .ok_or_else(|| {
                        serde::de::Error::custom("'value' must be a string for path/yaml refs")
                    })?
                    .to_string();
                if kind == "path" {
                    Ok(RefsInput::Path(s))
                } else {
                    Ok(RefsInput::Yaml(s))
                }
            }
            "json" => Ok(RefsInput::Json(value.clone())),
            k => Err(serde::de::Error::unknown_variant(
                k,
                &["path", "yaml", "json"],
            )),
        }
    }
}

impl Serialize for RefsInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            RefsInput::Path(s) => {
                map.serialize_entry("kind", "path")?;
                map.serialize_entry("value", s)?;
            }
            RefsInput::Yaml(s) => {
                map.serialize_entry("kind", "yaml")?;
                map.serialize_entry("value", s)?;
            }
            RefsInput::Json(v) => {
                map.serialize_entry("kind", "json")?;
                map.serialize_entry("value", v)?;
            }
        }
        map.end()
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for RefsInput {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RefsInput".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let reference_schema = generator.subschema_for::<crate::reference::Reference>();

        schemars::json_schema!({
            "oneOf": [
                {
                    "type": "object",
                    "required": ["kind", "value"],
                    "properties": {
                        "kind": {
                            "type": "string",
                            "enum": ["path", "yaml"]
                        },
                        "value": {
                            "type": "string"
                        }
                    },
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "required": ["kind", "value"],
                    "properties": {
                        "kind": {
                            "type": "string",
                            "const": "json"
                        },
                        "value": {
                            "type": "object",
                            "additionalProperties": reference_schema
                        }
                    },
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "additionalProperties": reference_schema
                }
            ]
        })
    }
}

impl RefsInput {
    /// Resolve refs input locally from Path, Yaml, or Json variants.
    ///
    /// # Errors
    ///
    /// Returns error for refs input filesystem or parse failures.
    pub fn resolve_local(&self) -> Result<Bibliography, crate::api::FormatDocumentError> {
        match self {
            RefsInput::Path(path) => {
                let yaml_bytes = std::fs::read(path).map_err(|e| {
                    crate::api::FormatDocumentError::RefsInputPath(format!(
                        "Failed to read refs input from '{}': {}",
                        path, e
                    ))
                })?;
                serde_yaml::from_slice::<Bibliography>(&yaml_bytes).map_err(|e| {
                    crate::api::FormatDocumentError::RefsInputParse(format!(
                        "Failed to parse refs input from '{}': {}",
                        path, e
                    ))
                })
            }
            RefsInput::Yaml(yaml_str) => {
                serde_yaml::from_str::<Bibliography>(yaml_str).map_err(|e| {
                    crate::api::FormatDocumentError::RefsInputParse(format!(
                        "Failed to parse inline YAML refs input: {}",
                        e
                    ))
                })
            }
            RefsInput::Json(json_val) => serde_json::from_value::<Bibliography>(json_val.clone())
                .map_err(|e| {
                    crate::api::FormatDocumentError::RefsInputParse(format!(
                        "Failed to parse JSON refs input: {}",
                        e
                    ))
                }),
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
    fn refs_input_yaml_resolves_locally() {
        let yaml_content = "test_ref:\n  id: test_ref\n  class: monograph\n  type: book\n  title: Test\n  issued: '2024'\n";
        let input = RefsInput::Yaml(yaml_content.to_string());
        let result = input.resolve_local();
        assert!(result.is_ok());
        assert!(result.unwrap().contains_key("test_ref"));
    }

    #[test]
    fn refs_input_json_resolves_locally() {
        let json_obj = serde_json::json!({
            "test_ref": {
                "id": "test_ref",
                "class": "monograph",
                "type": "book",
                "title": "Test",
                "issued": "2024"
            }
        });
        let input = RefsInput::Json(json_obj);
        let result = input.resolve_local();
        assert!(result.is_ok());
        assert!(result.unwrap().contains_key("test_ref"));
    }

    #[test]
    fn refs_input_path_reads_and_parses() {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        let yaml_content = "test_ref:\n  id: test_ref\n  class: monograph\n  type: book\n  title: Test\n  issued: '2024'\n";
        tmp.write_all(yaml_content.as_bytes())
            .expect("Failed to write temp file");
        tmp.flush().expect("Failed to flush temp file");

        let input = RefsInput::Path(tmp.path().to_string_lossy().to_string());
        let result = input.resolve_local();
        assert!(result.is_ok());
        assert!(result.unwrap().contains_key("test_ref"));
    }

    #[test]
    fn refs_input_path_missing_returns_error() {
        let input = RefsInput::Path("/nonexistent/path/refs.yaml".to_string());
        let result = input.resolve_local();
        match result {
            Err(crate::api::FormatDocumentError::RefsInputPath(msg)) => {
                assert!(msg.contains("Failed to read"));
            }
            _ => panic!("Expected RefsInputPath error"),
        }
    }

    #[test]
    fn refs_input_invalid_yaml_returns_parse_error() {
        let input = RefsInput::Yaml("{ invalid yaml: [".to_string());
        let result = input.resolve_local();
        match result {
            Err(crate::api::FormatDocumentError::RefsInputParse(msg)) => {
                assert!(msg.contains("Failed to parse"));
            }
            _ => panic!("Expected RefsInputParse error"),
        }
    }

    #[test]
    fn refs_input_deserialize_tagged_path() {
        let json_str = r#"{"kind":"path","value":"/tmp/bib.yaml"}"#;
        let input: RefsInput = serde_json::from_str(json_str).expect("deserialize");
        match input {
            RefsInput::Path(p) => assert_eq!(p, "/tmp/bib.yaml"),
            _ => panic!("Expected Path variant"),
        }
    }

    #[test]
    fn refs_input_deserialize_tagged_json() {
        let json_str = r#"{"kind":"json","value":{"key":"value"}}"#;
        let input: RefsInput = serde_json::from_str(json_str).expect("deserialize");
        match input {
            RefsInput::Json(v) => assert_eq!(v.get("key").unwrap(), "value"),
            _ => panic!("Expected Json variant"),
        }
    }

    #[test]
    fn refs_input_deserialize_bare_object_as_json() {
        let json_str = r#"{"test_ref":{"id":"test_ref","class":"monograph","type":"book","title":"Test","issued":"2024"}}"#;
        let input: RefsInput = serde_json::from_str(json_str).expect("deserialize");
        match input {
            RefsInput::Json(v) => assert!(v.get("test_ref").is_some()),
            _ => panic!("Expected Json variant"),
        }
    }

    #[test]
    fn refs_input_deserialize_legacy_kind_ref_id_as_json() {
        let json_str = r#"{"kind":{"id":"kind","class":"monograph","type":"book","title":"Kind","issued":"2024"}}"#;
        let input: RefsInput = serde_json::from_str(json_str).expect("deserialize");
        match input {
            RefsInput::Json(v) => assert!(v.get("kind").is_some()),
            _ => panic!("Expected Json variant"),
        }
    }

    #[test]
    fn refs_input_serialize_path() {
        let input = RefsInput::Path("/tmp/bib.yaml".to_string());
        let json_str = serde_json::to_string(&input).expect("serialize");
        assert!(json_str.contains("\"kind\":\"path\""));
        assert!(json_str.contains("\"/tmp/bib.yaml\""));
    }
}
