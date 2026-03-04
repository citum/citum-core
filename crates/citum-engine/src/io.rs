/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use citum_schema::InputBibliography;
use citum_schema::reference::InputReference;
use csl_legacy::csl_json::Reference as LegacyReference;

use crate::{Bibliography, Citation, ProcessorError, Reference};

/// Controls how annotation text is rendered in an annotated bibliography.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnotationStyle {
    /// Render annotation text in italics. Default: false.
    #[serde(default)]
    pub italic: bool,
    /// Indent the annotation paragraph. Default: true.
    #[serde(default = "default_true")]
    pub indent: bool,
    /// Line break style before annotation. Default: BlankLine.
    #[serde(default)]
    pub paragraph_break: ParagraphBreak,
    /// Markup format for annotation text. Default: Djot.
    #[serde(default)]
    pub format: AnnotationFormat,
}

fn default_true() -> bool {
    true
}

impl Default for AnnotationStyle {
    fn default() -> Self {
        Self {
            italic: false,
            indent: true,
            paragraph_break: ParagraphBreak::BlankLine,
            format: AnnotationFormat::Djot,
        }
    }
}

/// Line break style preceding an annotation block.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum ParagraphBreak {
    /// Single newline before annotation.
    SingleLine,
    /// Blank line before annotation (default).
    #[default]
    BlankLine,
}

/// Markup format for annotation text.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationFormat {
    /// Parse annotation as djot inline markup (default).
    #[default]
    Djot,
    /// Treat annotation as plain text with no markup interpretation.
    Plain,
    /// Parse annotation as org-mode markup.
    Org,
}

/// Load a list of citations from a file.
/// Supports Citum YAML/JSON.
pub fn load_citations(path: &Path) -> Result<Vec<Citation>, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    match ext {
        "json" => {
            // Check for syntax errors first
            let _: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

            if let Ok(citations) = serde_json::from_slice::<Vec<Citation>>(&bytes) {
                return Ok(citations);
            }
            match serde_json::from_slice::<Citation>(&bytes) {
                Ok(citation) => Ok(vec![citation]),
                Err(e) => Err(ProcessorError::ParseError(
                    "JSON".to_string(),
                    e.to_string(),
                )),
            }
        }
        _ => {
            let content = String::from_utf8_lossy(&bytes);
            // Check for syntax errors first
            let _: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

            if let Ok(citations) = serde_yaml::from_str::<Vec<Citation>>(&content) {
                return Ok(citations);
            }
            match serde_yaml::from_str::<Citation>(&content) {
                Ok(citation) => Ok(vec![citation]),
                Err(e) => Err(ProcessorError::ParseError(
                    "YAML".to_string(),
                    e.to_string(),
                )),
            }
        }
    }
}

/// Load annotations from a file (YAML or JSON).
/// Returns a mapping from reference ID to annotation text.
pub fn load_annotations(path: &Path) -> Result<HashMap<String, String>, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    match ext {
        "json" => {
            let _: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

            serde_json::from_slice::<HashMap<String, String>>(&bytes)
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))
        }
        _ => {
            let content = String::from_utf8_lossy(&bytes);
            let _: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

            serde_yaml::from_str::<HashMap<String, String>>(&content)
                .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))
        }
    }
}

/// Load a bibliography from a file given its path.
/// Supports Citum YAML/JSON/CBOR and CSL-JSON.
pub fn load_bibliography(path: &Path) -> Result<Bibliography, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    let mut bib = indexmap::IndexMap::new();

    // Try parsing as Citum formats
    match ext {
        "cbor" => {
            match ciborium::de::from_reader::<InputBibliography, _>(std::io::Cursor::new(&bytes)) {
                Ok(input_bib) => {
                    for r in input_bib.references {
                        if let Some(id) = r.id() {
                            bib.insert(id.to_string(), r);
                        }
                    }
                    Ok(bib)
                }
                Err(e) => Err(ProcessorError::ParseError(
                    "CBOR".to_string(),
                    e.to_string(),
                )),
            }
        }
        "json" => {
            // Check for syntax errors first
            let _: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

            // Try CSL-JSON (Vec<LegacyReference>)
            if let Ok(legacy_bib) = serde_json::from_slice::<Vec<LegacyReference>>(&bytes) {
                for ref_item in legacy_bib {
                    bib.insert(ref_item.id.clone(), Reference::from(ref_item));
                }
                return Ok(bib);
            }
            // Try Citum JSON (InputBibliography)
            if let Ok(input_bib) = serde_json::from_slice::<InputBibliography>(&bytes) {
                for r in input_bib.references {
                    if let Some(id) = r.id() {
                        bib.insert(id.to_string(), r);
                    }
                }
                return Ok(bib);
            }

            // Try IndexMap of LegacyReference (preserves insertion order from JSON)
            if let Ok(map) =
                serde_json::from_slice::<indexmap::IndexMap<String, serde_json::Value>>(&bytes)
            {
                let mut found = false;
                for (id, val) in map {
                    if let Ok(ref_item) = serde_json::from_value::<LegacyReference>(val) {
                        let mut r = Reference::from(ref_item);
                        if r.id().is_none() {
                            r.set_id(id.clone());
                        }
                        bib.insert(id, r);
                        found = true;
                    }
                }
                if found {
                    return Ok(bib);
                }
            }

            // If all failed, return the error from the most likely format (Citum JSON)
            match serde_json::from_slice::<InputBibliography>(&bytes) {
                Ok(_) => unreachable!(),
                Err(e) => Err(ProcessorError::ParseError(
                    "JSON".to_string(),
                    e.to_string(),
                )),
            }
        }
        _ => {
            // YAML/Fallback
            let content = String::from_utf8_lossy(&bytes);

            // Check for syntax errors first
            let _: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

            if let Ok(input_bib) = serde_yaml::from_str::<InputBibliography>(&content) {
                for r in input_bib.references {
                    if let Some(id) = r.id() {
                        bib.insert(id.to_string(), r);
                    }
                }
                return Ok(bib);
            }

            // Try parsing as IndexMap<String, serde_yaml::Value> (YAML/JSON, preserves order)
            if let Ok(map) =
                serde_yaml::from_str::<indexmap::IndexMap<String, serde_yaml::Value>>(&content)
            {
                let mut found = false;
                for (key, val) in map {
                    if let Ok(mut r) = serde_yaml::from_value::<InputReference>(val.clone()) {
                        if r.id().is_none() {
                            r.set_id(key.clone());
                        }
                        bib.insert(key, r);
                        found = true;
                    } else if let Ok(ref_item) = serde_yaml::from_value::<LegacyReference>(val) {
                        let mut r = Reference::from(ref_item);
                        if r.id().is_none() {
                            r.set_id(key.clone());
                        }
                        bib.insert(key, r);
                        found = true;
                    }
                }
                if found {
                    return Ok(bib);
                }
            }

            // Try parsing as Vec<InputReference> (YAML/JSON)
            if let Ok(refs) = serde_yaml::from_str::<Vec<InputReference>>(&content) {
                for r in refs {
                    if let Some(id) = r.id() {
                        bib.insert(id.to_string(), r);
                    }
                }
                return Ok(bib);
            }

            // If all failed, return error from Citum YAML
            match serde_yaml::from_str::<InputBibliography>(&content) {
                Ok(_) => unreachable!(),
                Err(e) => Err(ProcessorError::ParseError(
                    "YAML".to_string(),
                    e.to_string(),
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_citations_preserves_locator_labels() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/citations-expanded.json");
        let citations = load_citations(&path).expect("citations fixture should parse");
        let with_locator = citations
            .iter()
            .find(|c| c.id.as_deref() == Some("with-locator"))
            .expect("with-locator citation should exist");

        assert_eq!(with_locator.items.len(), 1);
        assert_eq!(
            with_locator.items[0].label,
            Some(citum_schema::citation::LocatorType::Page)
        );
        assert_eq!(with_locator.items[0].locator.as_deref(), Some("23"));
    }
}
