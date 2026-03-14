/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use citum_schema::InputBibliography;
use citum_schema::reference::InputReference;
use csl_legacy::csl_json::Reference as LegacyReference;
use indexmap::IndexMap;

use crate::render::format::OutputFormat;
use crate::render::rich_text::render_djot_inline;
use crate::{Bibliography, Citation, ProcessorError, Reference};

/// Bibliography data loaded from input, including optional compound sets.
#[derive(Debug, Clone, Default)]
pub struct LoadedBibliography {
    /// Parsed bibliography references keyed by ID.
    pub references: Bibliography,
    /// Optional compound sets keyed by set ID.
    pub sets: Option<IndexMap<String, Vec<String>>>,
}

#[derive(Debug, serde::Deserialize)]
struct LegacyBibliographyWrapper {
    references: Vec<LegacyReference>,
    #[serde(default)]
    sets: Option<IndexMap<String, Vec<String>>>,
}

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

/// Render a free-text reference field with djot inline markup.
///
/// Applies `render_djot_inline` to the source text, using the provided
/// OutputFormat to handle emphasis, links, and other inline markup.
/// This is the render-time hook for processing note, abstract, and other
/// free-text reference fields that may contain djot markup.
///
/// # Arguments
/// * `src` - Input string with optional djot inline markup
/// * `fmt` - OutputFormat implementation for rendering
///
/// # Returns
/// Rendered string with markup applied
pub fn render_rich_text_field<F: OutputFormat<Output = String>>(src: &str, fmt: &F) -> String {
    render_djot_inline(src, fmt)
}

/// Load a list of citations from a file.
/// Supports Citum YAML/JSON.
///
/// # Errors
///
/// Returns an error when the file cannot be read or when its contents are not
/// valid citation data in a supported format.
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
///
/// # Errors
///
/// Returns an error when the file cannot be read or when its contents are not
/// valid annotation data in a supported format.
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

/// Validate optional compound sets against the loaded bibliography.
///
/// Validation rules:
/// - Every member ID must exist in `bibliography`.
/// - A member ID must not appear more than once in a single set.
/// - A member ID must not appear across multiple sets.
///
/// # Errors
///
/// Returns an error when a compound set references an unknown ID or reuses the
/// same member within or across sets.
pub fn validate_compound_sets(
    sets: Option<IndexMap<String, Vec<String>>>,
    bibliography: &Bibliography,
) -> Result<Option<IndexMap<String, Vec<String>>>, ProcessorError> {
    let Some(sets) = sets else {
        return Ok(None);
    };

    let mut member_owner: HashMap<String, String> = HashMap::new();
    for (set_id, members) in &sets {
        let mut seen_in_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for member in members {
            if !seen_in_set.insert(member.clone()) {
                return Err(ProcessorError::ParseError(
                    "BIBLIOGRAPHY".to_string(),
                    format!(
                        "reference '{}' appears more than once in compound set '{}'",
                        member, set_id
                    ),
                ));
            }
            if !bibliography.contains_key(member) {
                return Err(ProcessorError::ParseError(
                    "BIBLIOGRAPHY".to_string(),
                    format!(
                        "compound set '{}' references unknown id '{}'",
                        set_id, member
                    ),
                ));
            }
            if let Some(existing) = member_owner.insert(member.clone(), set_id.clone()) {
                return Err(ProcessorError::ParseError(
                    "BIBLIOGRAPHY".to_string(),
                    format!(
                        "reference '{}' appears in both compound sets '{}' and '{}'",
                        member, existing, set_id
                    ),
                ));
            }
        }
    }

    Ok(Some(sets))
}

fn loaded_from_input_bibliography(
    input_bib: InputBibliography,
) -> Result<LoadedBibliography, ProcessorError> {
    let mut references = IndexMap::new();
    for r in input_bib.references {
        if let Some(id) = r.id() {
            references.insert(id.to_string(), r);
        }
    }
    let sets = validate_compound_sets(input_bib.sets, &references)?;
    Ok(LoadedBibliography { references, sets })
}

/// Parse JSON bibliography bytes into a LoadedBibliography.
///
/// Supports CSL-JSON, Citum JSON, wrapped legacy format, and IndexMap variants.
fn parse_json_bibliography(bytes: &[u8]) -> Result<LoadedBibliography, ProcessorError> {
    // Check for syntax errors first
    let _: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

    let mut bib = IndexMap::new();

    // Try CSL-JSON (Vec<LegacyReference>)
    if let Ok(legacy_bib) = serde_json::from_slice::<Vec<LegacyReference>>(bytes) {
        for ref_item in legacy_bib {
            bib.insert(ref_item.id.clone(), Reference::from(ref_item));
        }
        return Ok(LoadedBibliography {
            references: bib,
            sets: None,
        });
    }
    // Try Citum JSON (InputBibliography)
    if let Ok(input_bib) = serde_json::from_slice::<InputBibliography>(bytes) {
        return loaded_from_input_bibliography(input_bib);
    }

    // Try wrapped legacy JSON ({ references: [...], sets: {...} })
    if let Ok(wrapper) = serde_json::from_slice::<LegacyBibliographyWrapper>(bytes) {
        for ref_item in wrapper.references {
            bib.insert(ref_item.id.clone(), Reference::from(ref_item));
        }
        let sets = validate_compound_sets(wrapper.sets, &bib)?;
        return Ok(LoadedBibliography {
            references: bib,
            sets,
        });
    }

    // Try IndexMap of LegacyReference (preserves insertion order from JSON)
    if let Ok(map) = serde_json::from_slice::<IndexMap<String, serde_json::Value>>(bytes) {
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
            return Ok(LoadedBibliography {
                references: bib,
                sets: None,
            });
        }
    }

    // If all failed, return the error from the most likely format (Citum JSON)
    match serde_json::from_slice::<InputBibliography>(bytes) {
        Ok(_) => unreachable!(),
        Err(e) => Err(ProcessorError::ParseError(
            "JSON".to_string(),
            e.to_string(),
        )),
    }
}

/// Parse YAML bibliography string into a LoadedBibliography.
///
/// Supports Citum YAML, wrapped legacy format, IndexMap, and Vec variants.
fn parse_yaml_bibliography(content: &str) -> Result<LoadedBibliography, ProcessorError> {
    // Check for syntax errors first
    let _: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

    let mut bib = IndexMap::new();

    if let Ok(input_bib) = serde_yaml::from_str::<InputBibliography>(content) {
        return loaded_from_input_bibliography(input_bib);
    }

    // Try wrapped legacy YAML/JSON ({ references: [...], sets: {...} })
    if let Ok(wrapper) = serde_yaml::from_str::<LegacyBibliographyWrapper>(content) {
        for ref_item in wrapper.references {
            bib.insert(ref_item.id.clone(), Reference::from(ref_item));
        }
        let sets = validate_compound_sets(wrapper.sets, &bib)?;
        return Ok(LoadedBibliography {
            references: bib,
            sets,
        });
    }

    // Try parsing as IndexMap<String, serde_yaml::Value> (YAML/JSON, preserves order)
    if let Ok(map) = serde_yaml::from_str::<IndexMap<String, serde_yaml::Value>>(content) {
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
            return Ok(LoadedBibliography {
                references: bib,
                sets: None,
            });
        }
    }

    // Try parsing as Vec<InputReference> (YAML/JSON)
    if let Ok(refs) = serde_yaml::from_str::<Vec<InputReference>>(content) {
        for r in refs {
            if let Some(id) = r.id() {
                bib.insert(id.to_string(), r);
            }
        }
        return Ok(LoadedBibliography {
            references: bib,
            sets: None,
        });
    }

    // If all failed, return error from Citum YAML
    match serde_yaml::from_str::<InputBibliography>(content) {
        Ok(_) => unreachable!(),
        Err(e) => Err(ProcessorError::ParseError(
            "YAML".to_string(),
            e.to_string(),
        )),
    }
}

/// Load bibliography data from a file path, including optional compound sets.
/// Supports Citum YAML/JSON/CBOR and CSL-JSON.
///
/// # Errors
///
/// Returns an error when the file cannot be read, when its contents do not
/// match a supported bibliography format, or when compound sets are invalid.
pub fn load_bibliography_with_sets(path: &Path) -> Result<LoadedBibliography, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    // Try parsing as Citum formats
    match ext {
        "cbor" => {
            match ciborium::de::from_reader::<InputBibliography, _>(std::io::Cursor::new(&bytes)) {
                Ok(input_bib) => loaded_from_input_bibliography(input_bib),
                Err(e) => Err(ProcessorError::ParseError(
                    "CBOR".to_string(),
                    e.to_string(),
                )),
            }
        }
        "json" => parse_json_bibliography(&bytes),
        _ => {
            // YAML/Fallback
            let content = String::from_utf8_lossy(&bytes);
            parse_yaml_bibliography(&content)
        }
    }
}

/// Load a bibliography from a file given its path.
/// Supports Citum YAML/JSON/CBOR and CSL-JSON.
///
/// # Errors
///
/// Returns an error when the file cannot be read, its contents cannot be
/// parsed, or embedded compound-set metadata is invalid.
pub fn load_bibliography(path: &Path) -> Result<Bibliography, ProcessorError> {
    Ok(load_bibliography_with_sets(path)?.references)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::plain::PlainText;
    use indexmap::IndexMap;

    #[test]
    fn render_rich_text_field_with_bold() {
        let fmt = PlainText;
        let result = render_rich_text_field("This is *bold* text", &fmt);
        assert_eq!(result, "This is **bold** text");
    }

    #[test]
    fn render_rich_text_field_with_link() {
        let fmt = PlainText;
        let result = render_rich_text_field("See [this](https://example.com) for details", &fmt);
        assert_eq!(result, "See this for details");
    }

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
            with_locator.items[0].locator,
            Some(citum_schema::citation::CitationLocator::single(
                citum_schema::citation::LocatorType::Page,
                "23",
            ))
        );
    }

    #[test]
    fn loaded_bibliography_rejects_unknown_set_member() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([(
                "group-1".to_string(),
                vec!["missing-ref".to_string()],
            )])),
            ..Default::default()
        };

        let err = loaded_from_input_bibliography(input).expect_err("must reject unknown member");
        let msg = err.to_string();
        assert!(
            msg.contains("unknown id 'missing-ref'"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn loaded_bibliography_rejects_duplicate_set_membership() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([
                ("group-1".to_string(), vec!["ref-a".to_string()]),
                ("group-2".to_string(), vec!["ref-a".to_string()]),
            ])),
            ..Default::default()
        };

        let err =
            loaded_from_input_bibliography(input).expect_err("must reject duplicate membership");
        let msg = err.to_string();
        assert!(
            msg.contains("appears in both compound sets 'group-1' and 'group-2'"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn loaded_bibliography_rejects_duplicate_within_same_set() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([(
                "group-1".to_string(),
                vec!["ref-a".to_string(), "ref-a".to_string()],
            )])),
            ..Default::default()
        };

        let err = loaded_from_input_bibliography(input)
            .expect_err("must reject duplicate member in the same set");
        let msg = err.to_string();
        assert!(
            msg.contains("appears more than once in compound set 'group-1'"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn loaded_bibliography_accepts_empty_and_singleton_sets() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([
                ("empty".to_string(), Vec::new()),
                ("single".to_string(), vec!["ref-a".to_string()]),
            ])),
            ..Default::default()
        };

        let loaded = loaded_from_input_bibliography(input).expect("sets should be accepted");
        let sets = loaded.sets.expect("sets should be present");
        assert!(sets.contains_key("empty"));
        assert_eq!(sets.get("single"), Some(&vec!["ref-a".to_string()]));
    }

    #[test]
    /// Parse a JSON array of CSL-JSON objects directly into LoadedBibliography.
    fn parse_json_csl_vec() {
        let json = r#"[
  {"id": "smith-2020", "type": "book", "title": "Test Book"},
  {"id": "doe-2021", "type": "journal-article", "title": "Test Article"}
]"#;
        let result = parse_json_bibliography(json.as_bytes()).expect("should parse CSL-JSON vec");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("smith-2020"));
        assert!(result.references.contains_key("doe-2021"));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse a Citum InputBibliography from JSON with references and sets.
    fn parse_json_citum_input_bibliography() {
        let json = r#"{
  "references": [
    {
      "class": "monograph",
      "id": "ref-x",
      "type": "book",
      "title": "Citum Book",
      "issued": "2020"
    }
  ],
  "sets": {
    "group-a": ["ref-x"]
  }
}"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse Citum InputBibliography");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("ref-x"));
        let sets = result.sets.expect("sets should be present");
        assert_eq!(sets.get("group-a"), Some(&vec!["ref-x".to_string()]));
    }

    #[test]
    /// Parse a wrapped legacy JSON object with references and optional sets.
    fn parse_json_wrapped_legacy() {
        let json = r#"{
  "references": [
    {"id": "legacy-1", "type": "book", "title": "Legacy Book"}
  ],
  "sets": null
}"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse wrapped legacy format");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("legacy-1"));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse an IndexMap of CSL-JSON objects keyed by id from JSON.
    fn parse_json_indexmap() {
        // IndexMap format: object with entries having no "references" key,
        // where each value can be parsed as a LegacyReference.
        let json = r#"{
  "book-1": {"type": "book", "title": "First Book", "id": "book-1"},
  "article-2": {"type": "journal-article", "title": "First Article", "id": "article-2"}
}"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse IndexMap format");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("book-1"));
        assert!(result.references.contains_key("article-2"));
    }

    #[test]
    /// Parse a Citum YAML InputBibliography with references.
    fn parse_yaml_citum_input_bibliography() {
        let yaml = r#"
references:
  - class: monograph
    id: yaml-ref-1
    type: book
    title: YAML Book
    issued: "2021"
"#;
        let result = parse_yaml_bibliography(yaml).expect("should parse Citum YAML bibliography");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("yaml-ref-1"));
    }

    #[test]
    /// Parse a wrapped legacy YAML object with references and optional sets.
    fn parse_yaml_wrapped_legacy() {
        let yaml = r#"
references:
  - id: yaml-legacy-1
    type: book
    title: YAML Legacy Book
sets: null
"#;
        let result = parse_yaml_bibliography(yaml).expect("should parse wrapped legacy YAML");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("yaml-legacy-1"));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse an IndexMap of legacy references keyed by id from YAML.
    fn parse_yaml_indexmap() {
        // IndexMap format: plain object with reference-id keys mapping to legacy ref objects.
        // Structure: { id1: {type, title}, id2: {type, title} }
        // Must use legacy CSL-JSON field names (not InputReference class tags)
        // to avoid matching InputBibliography or Vec<InputReference>.
        let yaml = r#"ref-yaml-1:
  id: ref-yaml-1
  type: book
  title: First Book
ref-yaml-2:
  id: ref-yaml-2
  type: journal-article
  title: Second Article
"#;
        let result = parse_yaml_bibliography(yaml).expect("should parse YAML IndexMap format");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("ref-yaml-1"));
        assert!(result.references.contains_key("ref-yaml-2"));
    }

    #[test]
    /// Parse a YAML sequence of InputReference objects.
    fn parse_yaml_vec_input_references() {
        let yaml = r#"
- class: monograph
  id: seq-ref-1
  type: book
  title: Sequential Book
  issued: "2024"
- class: monograph
  id: seq-ref-2
  type: book
  title: Another Sequential Book
  issued: "2025"
"#;
        let result =
            parse_yaml_bibliography(yaml).expect("should parse YAML sequence of references");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("seq-ref-1"));
        assert!(result.references.contains_key("seq-ref-2"));
    }
}
