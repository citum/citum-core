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
use indexmap::IndexMap;

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

/// Load a list of citations from a file.
/// Supports CSLN YAML/JSON.
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

/// Validate optional compound sets against the loaded bibliography.
///
/// Validation rules:
/// - Every member ID must exist in `bibliography`.
/// - A member ID must not appear more than once in a single set.
/// - A member ID must not appear across multiple sets.
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

/// Load bibliography data from a file path, including optional compound sets.
/// Supports CSLN YAML/JSON/CBOR and CSL-JSON.
pub fn load_bibliography_with_sets(path: &Path) -> Result<LoadedBibliography, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    let mut bib = IndexMap::new();

    // Try parsing as CSLN formats
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
        "json" => {
            // Check for syntax errors first
            let _: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

            // Try CSL-JSON (Vec<LegacyReference>)
            if let Ok(legacy_bib) = serde_json::from_slice::<Vec<LegacyReference>>(&bytes) {
                for ref_item in legacy_bib {
                    bib.insert(ref_item.id.clone(), Reference::from(ref_item));
                }
                return Ok(LoadedBibliography {
                    references: bib,
                    sets: None,
                });
            }
            // Try CSLN JSON (InputBibliography)
            if let Ok(input_bib) = serde_json::from_slice::<InputBibliography>(&bytes) {
                return loaded_from_input_bibliography(input_bib);
            }

            // Try wrapped legacy JSON ({ references: [...], sets: {...} })
            if let Ok(wrapper) = serde_json::from_slice::<LegacyBibliographyWrapper>(&bytes) {
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
            if let Ok(map) = serde_json::from_slice::<IndexMap<String, serde_json::Value>>(&bytes) {
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

            // If all failed, return the error from the most likely format (CSLN JSON)
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
                return loaded_from_input_bibliography(input_bib);
            }

            // Try wrapped legacy YAML/JSON ({ references: [...], sets: {...} })
            if let Ok(wrapper) = serde_yaml::from_str::<LegacyBibliographyWrapper>(&content) {
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
            if let Ok(map) = serde_yaml::from_str::<IndexMap<String, serde_yaml::Value>>(&content) {
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
            if let Ok(refs) = serde_yaml::from_str::<Vec<InputReference>>(&content) {
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

            // If all failed, return error from CSLN YAML
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

/// Load a bibliography from a file given its path.
/// Supports CSLN YAML/JSON/CBOR and CSL-JSON.
pub fn load_bibliography(path: &Path) -> Result<Bibliography, ProcessorError> {
    Ok(load_bibliography_with_sets(path)?.references)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

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
}
