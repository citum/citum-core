/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Refs input resolution type for interactive APIs.

use crate::reference::{Bibliography, Reference};
use serde::{Deserialize, Serialize};

/// A refs input that can be resolved locally or by an external resolver.
///
/// This union type allows callers to supply reference data by local file path,
/// inline YAML, inline JSON, or inline BibLaTeX. Enables citum-server and
/// bindings to accept references from files (e.g., via pipe transport from
/// LaTeX or Emacs).
///
/// Supported tagged-object shapes over JSON/RPC:
///
/// ```json
/// {"kind": "path",     "value": "/abs/path/refs.yaml"}
/// {"kind": "path",     "value": "/abs/path/refs.bib"}  // .bib detected by extension
/// {"kind": "yaml",     "value": "references:\n  - id: …"}
/// {"kind": "json",     "value": {"id": { … }}}
/// {"kind": "biblatex", "value": "@book{key, title={…}, …}"}
/// ```
#[derive(Debug, Clone)]
pub enum RefsInput {
    /// Local filesystem path to a refs file.
    ///
    /// `.bib` extensions are parsed as BibLaTeX; all other extensions are
    /// parsed as native Citum YAML (JSON parses as a YAML subset).
    Path(String),
    /// Inline YAML refs string.
    Yaml(String),
    /// Inline JSON map of reference objects.
    Json(serde_json::Value),
    /// Inline BibLaTeX (`.bib`) content.
    Biblatex(String),
}

impl<'de> Deserialize<'de> for RefsInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize to a generic Value first so we can inspect the shape.
        let v = serde_json::Value::deserialize(deserializer)?;

        if let Some(object) = v.as_object() {
            // If the object has a string "kind" and a "value" field it is a
            // tagged-union wrapper — validate the kind rather than silently
            // treating an unrecognised kind as a legacy bare-map, which would
            // produce a confusing downstream parse error.
            let kind_str = object.get("kind").and_then(|k| k.as_str());
            let has_value = object.contains_key("value");
            match (kind_str, has_value) {
                (Some(k), true) => {
                    if !matches!(k, "path" | "yaml" | "json" | "biblatex") {
                        return Err(serde::de::Error::unknown_variant(
                            k,
                            &["path", "yaml", "json", "biblatex"],
                        ));
                    }
                    // Recognised kind — fall through to dispatch below.
                }
                // No string kind, or no value field: legacy bare refs map.
                _ => return Ok(RefsInput::Json(v)),
            }
        } else {
            return Err(serde::de::Error::custom(
                "refs input must be a tagged object or legacy refs object",
            ));
        }

        // Tagged union: {"kind": "path"|"yaml"|"json"|"biblatex", "value": ...}
        let kind = v
            .get("kind")
            .and_then(|k| k.as_str())
            .ok_or_else(|| serde::de::Error::custom("refs input must have a 'kind' field"))?;

        let value = v
            .get("value")
            .ok_or_else(|| serde::de::Error::missing_field("value"))?;

        match kind {
            "path" | "yaml" | "biblatex" => {
                let s = value
                    .as_str()
                    .ok_or_else(|| {
                        serde::de::Error::custom(
                            "'value' must be a string for path/yaml/biblatex refs",
                        )
                    })?
                    .to_string();
                match kind {
                    "path" => Ok(RefsInput::Path(s)),
                    "yaml" => Ok(RefsInput::Yaml(s)),
                    _ => Ok(RefsInput::Biblatex(s)),
                }
            }
            "json" => Ok(RefsInput::Json(value.clone())),
            k => Err(serde::de::Error::unknown_variant(
                k,
                &["path", "yaml", "json", "biblatex"],
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
            RefsInput::Biblatex(s) => {
                map.serialize_entry("kind", "biblatex")?;
                map.serialize_entry("value", s)?;
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
                            "enum": ["path", "yaml", "biblatex"]
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
    /// Resolve refs input locally from Path, Yaml, Json, or Biblatex variants.
    ///
    /// For `Path` inputs, `.bib` files are parsed as BibLaTeX; all other
    /// extensions are parsed as native Citum YAML (JSON parses as a YAML
    /// subset).
    ///
    /// # Errors
    ///
    /// Returns error for refs input filesystem or parse failures.
    pub fn resolve_local(&self) -> Result<Bibliography, crate::api::FormatDocumentError> {
        match self {
            RefsInput::Path(path) => {
                let p = std::path::Path::new(path);
                if p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("bib"))
                {
                    let input = citum_refs::formats::biblatex::load_biblatex(p).map_err(|e| {
                        crate::api::FormatDocumentError::RefsInputParse(format!(
                            "Failed to parse BibLaTeX refs from '{}': {}",
                            path, e
                        ))
                    })?;
                    return Ok(bibliography_from_references(input.references));
                }
                let bytes = std::fs::read(path).map_err(|e| {
                    crate::api::FormatDocumentError::RefsInputPath(format!(
                        "Failed to read refs input from '{}': {}",
                        path, e
                    ))
                })?;
                let yaml_str = String::from_utf8(bytes).map_err(|_| {
                    crate::api::FormatDocumentError::RefsInputParse(format!(
                        "Refs input file '{}' is not valid UTF-8",
                        path
                    ))
                })?;
                parse_yaml_bibliography(&yaml_str).map_err(|e| {
                    crate::api::FormatDocumentError::RefsInputParse(format!(
                        "Failed to parse refs input from '{}': {}",
                        path, e
                    ))
                })
            }
            RefsInput::Yaml(yaml_str) => parse_yaml_bibliography(yaml_str).map_err(|e| {
                crate::api::FormatDocumentError::RefsInputParse(format!(
                    "Failed to parse inline YAML refs input: {}",
                    e
                ))
            }),
            RefsInput::Json(json_val) => serde_json::from_value::<Bibliography>(json_val.clone())
                .map_err(|e| {
                    crate::api::FormatDocumentError::RefsInputParse(format!(
                        "Failed to parse JSON refs input: {}",
                        e
                    ))
                }),
            RefsInput::Biblatex(src) => {
                let input =
                    citum_refs::formats::biblatex::parse_biblatex_str(src).map_err(|e| {
                        crate::api::FormatDocumentError::RefsInputParse(format!(
                            "Failed to parse inline BibLaTeX refs input: {}",
                            e
                        ))
                    })?;
                Ok(bibliography_from_references(input.references))
            }
        }
    }
}

fn parse_yaml_bibliography(yaml_str: &str) -> Result<Bibliography, String> {
    let native_err = match serde_yaml::from_str::<citum_schema::InputBibliography>(yaml_str) {
        Ok(input) => return Ok(bibliography_from_references(input.references)),
        Err(e) => e,
    };

    if let Ok(bibliography) = serde_yaml::from_str::<Bibliography>(yaml_str) {
        return Ok(bibliography);
    }

    if let Ok(references) = serde_yaml::from_str::<Vec<Reference>>(yaml_str) {
        return Ok(bibliography_from_references(references));
    }

    Err(format!(
        "tried native `references:` bibliography, flat id-to-reference map, and reference sequence: {native_err}"
    ))
}

fn bibliography_from_references(references: Vec<Reference>) -> Bibliography {
    references
        .into_iter()
        .filter_map(|reference| {
            let id = reference.id()?.to_string();
            Some((id, reference))
        })
        .collect()
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
    fn refs_input_path_reads_native_input_bibliography() {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        let yaml_content = "info:\n  title: Test Bibliography\nreferences:\n  - id: test_ref\n    class: monograph\n    type: book\n    title: Test\n    issued: '2024'\n";
        tmp.write_all(yaml_content.as_bytes())
            .expect("Failed to write temp file");
        tmp.flush().expect("Failed to flush temp file");

        let input = RefsInput::Path(tmp.path().to_string_lossy().to_string());
        let result = input
            .resolve_local()
            .expect("native bibliography should parse");
        assert!(result.contains_key("test_ref"));
    }

    #[test]
    fn refs_input_yaml_reads_native_input_bibliography() {
        let yaml_content = "info:\n  title: Test Bibliography\nreferences:\n  - id: test_ref\n    class: monograph\n    type: book\n    title: Test\n    issued: '2024'\n";
        let input = RefsInput::Yaml(yaml_content.to_string());
        let result = input
            .resolve_local()
            .expect("native bibliography should parse");
        assert!(result.contains_key("test_ref"));
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
    fn refs_input_path_invalid_utf8_returns_parse_error() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".yaml")
            .tempfile()
            .expect("Failed to create temp .yaml file");
        tmp.write_all(&[0xff, 0xfe, 0x00, 0x01])
            .expect("Failed to write temp file");
        tmp.flush().expect("Failed to flush temp file");

        let input = RefsInput::Path(tmp.path().to_string_lossy().to_string());
        let result = input.resolve_local();
        match result {
            Err(crate::api::FormatDocumentError::RefsInputParse(msg)) => {
                assert!(msg.contains("not valid UTF-8"));
            }
            _ => panic!("Expected RefsInputParse error"),
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

    #[test]
    fn refs_input_deserialize_tagged_biblatex() {
        let bib_src = "@book{hawking1988, title = {A Brief History of Time}, author = {Hawking, Stephen}, date = {1988}}";
        let json_str = format!(
            r#"{{"kind":"biblatex","value":{}}}"#,
            serde_json::to_string(bib_src).unwrap()
        );
        let input: RefsInput = serde_json::from_str(&json_str).expect("deserialize biblatex");
        match input {
            RefsInput::Biblatex(s) => assert!(s.contains("hawking1988")),
            _ => panic!("Expected Biblatex variant"),
        }
    }

    #[test]
    fn refs_input_biblatex_resolves_locally() {
        let bib_src = "@book{hawking1988, title = {A Brief History of Time}, author = {Hawking, Stephen}, date = {1988}}";
        let input = RefsInput::Biblatex(bib_src.to_string());
        let result = input.resolve_local().expect("biblatex should parse");
        assert!(result.contains_key("hawking1988"));
    }

    #[test]
    fn refs_input_path_bib_extension_parses_biblatex() {
        let bib_content = "@article{doe2024, title = {Test Article}, author = {Doe, Jane}, journaltitle = {Journal of Tests}, date = {2024}}";
        let mut tmp = tempfile::Builder::new()
            .suffix(".bib")
            .tempfile()
            .expect("Failed to create temp .bib file");
        tmp.write_all(bib_content.as_bytes())
            .expect("Failed to write temp file");
        tmp.flush().expect("Failed to flush temp file");

        let input = RefsInput::Path(tmp.path().to_string_lossy().to_string());
        let result = input.resolve_local().expect(".bib path should parse");
        assert!(result.contains_key("doe2024"));
    }

    #[test]
    fn refs_input_serialize_biblatex() {
        let input = RefsInput::Biblatex("@book{key, title = {T}}".to_string());
        let json_str = serde_json::to_string(&input).expect("serialize");
        assert!(json_str.contains("\"kind\":\"biblatex\""));
        assert!(json_str.contains("@book{key"));
    }

    #[test]
    fn refs_input_deserialize_unknown_kind_returns_error() {
        let json_str = r#"{"kind":"csl-json","value":"..."}"#;
        let result = serde_json::from_str::<RefsInput>(json_str);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("csl-json"),
            "error should name the unknown variant: {msg}"
        );
    }
}
