/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use citum_schema::InputBibliography;
use citum_schema::reference::InputReference;
use citum_schema::reference::conversion::input_reference_from_legacy_edited_book;
use citum_schema::reference::types::{ArchiveInfo, EprintInfo};
use csl_legacy::csl_json::Reference as LegacyReference;
use indexmap::IndexMap;

use crate::render::format::OutputFormat;
use crate::render::rich_text::render_djot_inline;
use crate::{Bibliography, Citation, ProcessorError, Reference};

/// Bibliography formats supported by reusable reference conversion helpers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RefsFormat {
    /// Native Citum bibliography encoded as YAML.
    CitumYaml,
    /// Native Citum bibliography encoded as JSON.
    CitumJson,
    /// Native Citum bibliography encoded as CBOR.
    CitumCbor,
    /// Legacy CSL-JSON bibliography.
    CslJson,
    /// BibLaTeX `.bib` bibliography.
    Biblatex,
    /// RIS bibliography.
    Ris,
}

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
    /// Line break style before annotation. Default: `BlankLine`.
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
/// `OutputFormat` to handle emphasis, links, and other inline markup.
/// This is the render-time hook for processing note, abstract, and other
/// free-text reference fields that may contain djot markup.
///
/// # Arguments
/// * `src` - Input string with optional djot inline markup
/// * `fmt` - `OutputFormat` implementation for rendering
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

    if ext == "json" {
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
    } else {
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

    if ext == "json" {
        let _: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

        serde_json::from_slice::<HashMap<String, String>>(&bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))
    } else {
        let content = String::from_utf8_lossy(&bytes);
        let _: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

        serde_yaml::from_str::<HashMap<String, String>>(&content)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))
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
                        "reference '{member}' appears more than once in compound set '{set_id}'"
                    ),
                ));
            }
            if !bibliography.contains_key(member) {
                return Err(ProcessorError::ParseError(
                    "BIBLIOGRAPHY".to_string(),
                    format!("compound set '{set_id}' references unknown id '{member}'"),
                ));
            }
            if let Some(existing) = member_owner.insert(member.clone(), set_id.clone()) {
                return Err(ProcessorError::ParseError(
                    "BIBLIOGRAPHY".to_string(),
                    format!(
                        "reference '{member}' appears in both compound sets '{existing}' and '{set_id}'"
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

fn loaded_from_hybrid_json_array(
    references: &[serde_json::Value],
    sets: Option<IndexMap<String, Vec<String>>>,
) -> Result<LoadedBibliography, ProcessorError> {
    let mut bib = IndexMap::new();
    for value in references.iter().cloned() {
        let reference = parse_hybrid_json_reference(value)?;
        if let Some(id) = reference.id() {
            bib.insert(id.to_string(), reference);
        }
    }
    let sets = validate_compound_sets(sets, &bib)?;
    Ok(LoadedBibliography {
        references: bib,
        sets,
    })
}

fn apply_hybrid_json_extensions(
    mut reference: InputReference,
    value: &serde_json::Value,
) -> Result<InputReference, ProcessorError> {
    let archive_info = value
        .get("archive-info")
        .filter(|raw| !raw.is_null())
        .cloned()
        .map(serde_json::from_value::<ArchiveInfo>)
        .transpose()
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
    let eprint = value
        .get("eprint")
        .filter(|raw| !raw.is_null())
        .cloned()
        .map(serde_json::from_value::<EprintInfo>)
        .transpose()
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

    match &mut reference {
        InputReference::Monograph(record) => {
            if archive_info.is_some() {
                record.archive_info = archive_info;
            }
            if eprint.is_some() {
                record.eprint = eprint;
            }
        }
        InputReference::CollectionComponent(record) => {
            if archive_info.is_some() {
                record.archive_info = archive_info;
            }
            if eprint.is_some() {
                record.eprint = eprint;
            }
        }
        InputReference::SerialComponent(record) => {
            if archive_info.is_some() {
                record.archive_info = archive_info;
            }
            if eprint.is_some() {
                record.eprint = eprint;
            }
        }
        _ => {}
    }

    Ok(reference)
}

fn parse_hybrid_json_reference(value: serde_json::Value) -> Result<InputReference, ProcessorError> {
    if let Ok(reference) = serde_json::from_value::<InputReference>(value.clone()) {
        return Ok(reference);
    }

    let class = value
        .get("class")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let ref_type = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let legacy = serde_json::from_value::<LegacyReference>(value.clone())
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

    let reference = if class == "collection" && ref_type == "edited-book" {
        input_reference_from_legacy_edited_book(legacy)
    } else {
        InputReference::from(legacy)
    };

    apply_hybrid_json_extensions(reference, &value)
}

/// Parse JSON bibliography bytes into a `LoadedBibliography`.
///
/// Supports CSL-JSON, Citum JSON, wrapped legacy format, and `IndexMap` variants.
fn parse_json_bibliography(bytes: &[u8]) -> Result<LoadedBibliography, ProcessorError> {
    // Check for syntax errors first
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

    if let Some(references) = value.as_array() {
        return loaded_from_hybrid_json_array(references, None);
    }

    if let Some(object) = value.as_object()
        && let Some(references) = object
            .get("references")
            .and_then(serde_json::Value::as_array)
    {
        let sets = object
            .get("sets")
            .filter(|value| !value.is_null())
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
        return loaded_from_hybrid_json_array(references, sets);
    }

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
                bib.insert(id.clone(), r);
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

/// Parse YAML bibliography string into a `LoadedBibliography`.
///
/// Supports Citum YAML, wrapped legacy format, `IndexMap`, and Vec variants.
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

/// Load and merge one or more bibliography files, preserving compound set metadata.
///
/// Entries from later files replace entries with the same ID from earlier files.
/// Compound set IDs must be unique across input files, and final membership is
/// validated against the merged bibliography.
///
/// # Errors
///
/// Returns an error when no paths are supplied, any file cannot be loaded, or
/// merged compound sets are invalid.
pub fn load_merged_bibliography(paths: &[PathBuf]) -> Result<LoadedBibliography, ProcessorError> {
    if paths.is_empty() {
        return Err(ProcessorError::ParseError(
            "BIBLIOGRAPHY".to_string(),
            "At least one bibliography path is required.".to_string(),
        ));
    }

    let mut merged = Bibliography::new();
    let mut merged_sets = IndexMap::<String, Vec<String>>::new();
    for path in paths {
        let loaded = load_bibliography_with_sets(path)?;
        for (id, reference) in loaded.references {
            merged.insert(id, reference);
        }
        if let Some(sets) = loaded.sets {
            for (set_id, members) in sets {
                if merged_sets.insert(set_id.clone(), members).is_some() {
                    return Err(ProcessorError::ParseError(
                        "BIBLIOGRAPHY".to_string(),
                        format!("Duplicate compound set id while merging: {set_id}"),
                    ));
                }
            }
        }
    }

    let validated_sets = validate_compound_sets(
        if merged_sets.is_empty() {
            None
        } else {
            Some(merged_sets)
        },
        &merged,
    )?;

    Ok(LoadedBibliography {
        references: merged,
        sets: validated_sets,
    })
}

/// Load and concatenate one or more citation files.
///
/// # Errors
///
/// Returns an error when any citation file cannot be read or parsed.
pub fn load_merged_citations(paths: &[PathBuf]) -> Result<Vec<Citation>, ProcessorError> {
    let mut merged = Vec::new();
    for path in paths {
        let loaded = load_citations(path)?;
        merged.extend(loaded);
    }
    Ok(merged)
}

/// Infer a bibliography input format from a path.
///
/// JSON inputs are content-sniffed to distinguish native Citum JSON from
/// CSL-JSON.
///
/// # Errors
///
/// Returns an error when a JSON input cannot be read or parsed for detection.
pub fn infer_refs_input_format(path: &Path) -> Result<RefsFormat, ProcessorError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let fmt = match ext.to_ascii_lowercase().as_str() {
        "yaml" | "yml" => RefsFormat::CitumYaml,
        "cbor" => RefsFormat::CitumCbor,
        "bib" => RefsFormat::Biblatex,
        "ris" => RefsFormat::Ris,
        "json" => detect_json_refs_format(path)?,
        _ => RefsFormat::CitumYaml,
    };
    Ok(fmt)
}

/// Infer a bibliography output format from a path.
///
/// Unknown extensions default to native Citum YAML.
#[must_use]
pub fn infer_refs_output_format(path: &Path) -> RefsFormat {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_ascii_lowercase().as_str() {
        "yaml" | "yml" => RefsFormat::CitumYaml,
        "cbor" => RefsFormat::CitumCbor,
        "bib" => RefsFormat::Biblatex,
        "ris" => RefsFormat::Ris,
        "json" => RefsFormat::CitumJson,
        _ => RefsFormat::CitumYaml,
    }
}

fn detect_json_refs_format(path: &Path) -> Result<RefsFormat, ProcessorError> {
    let bytes = fs::read(path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
    let array = value.as_array();
    let is_citum_array = array.is_some_and(|items| items.iter().any(|v| v.get("class").is_some()));
    let is_csl_array = array.is_some_and(|items| {
        items.iter().any(|v| {
            v.get("id").is_some()
                && v.get("type").is_some()
                && (v.get("title").is_some() || v.get("author").is_some())
        })
    });
    let is_citum_object = value.get("references").is_some();
    if is_csl_array && !is_citum_array && !is_citum_object {
        Ok(RefsFormat::CslJson)
    } else {
        Ok(RefsFormat::CitumJson)
    }
}

/// Load bibliography input in a specified native or legacy reference format.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed as `format`.
pub fn load_input_bibliography(
    path: &Path,
    format: RefsFormat,
) -> Result<InputBibliography, ProcessorError> {
    match format {
        RefsFormat::CitumYaml => {
            let bytes = fs::read(path)?;
            deserialize_any(&bytes, "yaml")
        }
        RefsFormat::CitumJson => {
            let bytes = fs::read(path)?;
            load_citum_json_bibliography(&bytes)
        }
        RefsFormat::CitumCbor => {
            let bytes = fs::read(path)?;
            deserialize_any(&bytes, "cbor")
        }
        RefsFormat::CslJson => load_csl_json_bibliography(path),
        RefsFormat::Biblatex => load_biblatex_bibliography(path),
        RefsFormat::Ris => load_ris_bibliography(path),
    }
}

fn deserialize_any<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    ext: &str,
) -> Result<T, ProcessorError> {
    match ext {
        "yaml" | "yml" => serde_yaml::from_slice(bytes)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string())),
        "json" => serde_json::from_slice(bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string())),
        "cbor" => ciborium::de::from_reader(std::io::Cursor::new(bytes))
            .map_err(|e| ProcessorError::ParseError("CBOR".to_string(), e.to_string())),
        _ => serde_yaml::from_slice(bytes)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string())),
    }
}

fn serialize_any<T: serde::Serialize>(obj: &T, ext: &str) -> Result<Vec<u8>, ProcessorError> {
    match ext {
        "yaml" | "yml" => serde_yaml::to_string(obj)
            .map(String::into_bytes)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string())),
        "json" => serde_json::to_string_pretty(obj)
            .map(String::into_bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string())),
        "cbor" => {
            let mut buf = Vec::new();
            ciborium::ser::into_writer(obj, &mut buf)
                .map_err(|e| ProcessorError::ParseError("CBOR".to_string(), e.to_string()))?;
            Ok(buf)
        }
        _ => serde_yaml::to_string(obj)
            .map(String::into_bytes)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string())),
    }
}

fn load_citum_json_bibliography(bytes: &[u8]) -> Result<InputBibliography, ProcessorError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

    if let Ok(input) = serde_json::from_value::<InputBibliography>(value.clone()) {
        return Ok(input);
    }

    if let Some(references) = value.as_array() {
        return Ok(InputBibliography {
            references: load_hybrid_json_references(references)?,
            ..Default::default()
        });
    }

    if let Some(object) = value.as_object()
        && let Some(references) = object
            .get("references")
            .and_then(serde_json::Value::as_array)
    {
        return Ok(InputBibliography {
            references: load_hybrid_json_references(references)?,
            sets: object
                .get("sets")
                .filter(|value| !value.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?
                .unwrap_or_default(),
            ..Default::default()
        });
    }

    deserialize_any(bytes, "json")
}

fn load_hybrid_json_references(
    references: &[serde_json::Value],
) -> Result<Vec<InputReference>, ProcessorError> {
    references
        .iter()
        .cloned()
        .map(load_hybrid_json_reference)
        .collect()
}

fn load_hybrid_json_reference(value: serde_json::Value) -> Result<InputReference, ProcessorError> {
    if let Ok(reference) = serde_json::from_value::<InputReference>(value.clone()) {
        return Ok(reference);
    }

    let class = value
        .get("class")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let ref_type = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let legacy = serde_json::from_value::<LegacyReference>(value)
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

    if class == "collection" && ref_type == "edited-book" {
        return Ok(input_reference_from_legacy_edited_book(legacy));
    }

    Ok(InputReference::from(legacy))
}

/// Write bibliography input to a specified native or legacy reference format.
///
/// # Errors
///
/// Returns an error when serialization fails or the output file cannot be
/// written.
pub fn write_output_bibliography(
    input: &InputBibliography,
    path: &Path,
    format: RefsFormat,
) -> Result<(), ProcessorError> {
    match format {
        RefsFormat::CitumYaml => fs::write(path, serialize_any(input, "yaml")?)?,
        RefsFormat::CitumJson => fs::write(path, serialize_any(input, "json")?)?,
        RefsFormat::CitumCbor => fs::write(path, serialize_any(input, "cbor")?)?,
        RefsFormat::CslJson => {
            let refs: Vec<LegacyReference> = input
                .references
                .iter()
                .map(input_reference_to_csl_json)
                .collect();
            let json = serde_json::to_string_pretty(&refs)
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
            fs::write(path, json)?;
        }
        RefsFormat::Biblatex => {
            fs::write(path, render_biblatex(input))?;
        }
        RefsFormat::Ris => {
            fs::write(path, render_ris(input))?;
        }
    }
    Ok(())
}

fn load_csl_json_bibliography(path: &Path) -> Result<InputBibliography, ProcessorError> {
    let bytes = fs::read(path)?;
    let refs: Vec<LegacyReference> = serde_json::from_slice(&bytes)
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
    let references = refs.into_iter().map(InputReference::from).collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

fn load_biblatex_bibliography(path: &Path) -> Result<InputBibliography, ProcessorError> {
    let src = fs::read_to_string(path)?;
    let bibliography = biblatex::Bibliography::parse(&src)
        .map_err(|e| ProcessorError::ParseError("BibLaTeX".to_string(), e.to_string()))?;
    let references = bibliography
        .iter()
        .map(crate::biblatex::input_reference_from_biblatex)
        .collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

fn load_ris_bibliography(path: &Path) -> Result<InputBibliography, ProcessorError> {
    let src = fs::read_to_string(path)?;
    parse_ris(&src)
}

fn input_reference_to_csl_json(reference: &InputReference) -> LegacyReference {
    use csl_legacy::csl_json::{DateVariable, StringOrNumber};

    let id = reference.id().unwrap_or_else(|| "item".into());
    let mut r = LegacyReference {
        id: id.to_string(),
        ..Default::default()
    };

    r.title = reference.title().map(|t| t.to_string());
    r.language = reference.language().map(|lang| lang.to_string());
    r.note = reference.note().map(|rt| rt.raw().to_string());
    r.doi = reference.doi();
    r.issued = reference.csl_issued_date().and_then(|d| {
        let s = d.0;
        let year = s.get(0..4)?.parse::<i32>().ok()?;
        Some(DateVariable::year(year))
    });
    r.author = reference.author().map(contributor_to_csl_names);
    r.editor = reference.editor().map(contributor_to_csl_names);
    r.translator = reference.translator().map(contributor_to_csl_names);
    r.publisher = reference.publisher().map(|p| p.name.to_string());

    match reference {
        InputReference::Monograph(m) => {
            r.ref_type = "book".to_string();
            r.isbn.clone_from(&m.isbn);
            r.url = m.url.as_ref().map(std::string::ToString::to_string);
            r.edition = reference.edition().map(StringOrNumber::String);
        }
        InputReference::SerialComponent(s) => {
            r.ref_type = "article-journal".to_string();
            r.container_title = reference.container_title().map(|t| t.to_string());
            r.page.clone_from(&s.pages);
            r.volume = reference
                .volume()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.issue = reference
                .issue()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.url = s.url.as_ref().map(std::string::ToString::to_string);
        }
        InputReference::CollectionComponent(c) => {
            r.ref_type = "chapter".to_string();
            r.container_title = reference.container_title().map(|t| t.to_string());
            r.page = c.pages.as_ref().map(std::string::ToString::to_string);
        }
        _ => {
            r.ref_type = "book".to_string();
        }
    }

    r
}

fn contributor_to_csl_names(
    contributor: citum_schema::reference::Contributor,
) -> Vec<csl_legacy::csl_json::Name> {
    let mut names = Vec::new();
    match contributor {
        citum_schema::reference::Contributor::SimpleName(n) => {
            names.push(csl_legacy::csl_json::Name::literal(&n.name.to_string()));
        }
        citum_schema::reference::Contributor::StructuredName(n) => {
            names.push(csl_legacy::csl_json::Name {
                family: Some(n.family.to_string()),
                given: Some(n.given.to_string()),
                suffix: n.suffix,
                dropping_particle: n.dropping_particle,
                non_dropping_particle: n.non_dropping_particle,
                literal: None,
            });
        }
        citum_schema::reference::Contributor::Multilingual(n) => {
            names.push(csl_legacy::csl_json::Name {
                family: Some(n.original.family.to_string()),
                given: Some(n.original.given.to_string()),
                suffix: n.original.suffix,
                dropping_particle: n.original.dropping_particle,
                non_dropping_particle: n.original.non_dropping_particle,
                literal: None,
            });
        }
        citum_schema::reference::Contributor::ContributorList(list) => {
            for member in list.0 {
                names.extend(contributor_to_csl_names(member));
            }
        }
    }
    names
}

fn render_biblatex(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let id = reference.id().unwrap_or_else(|| "item".into());
        let entry_type = match reference {
            InputReference::SerialComponent(_) => "article",
            InputReference::CollectionComponent(_) => "incollection",
            _ => "book",
        };
        let _ = writeln!(&mut out, "@{entry_type}{{{id},");
        if let Some(title) = reference.title() {
            let _ = writeln!(&mut out, "  title = {{{title}}},");
        }
        if let Some(contributor) = reference.author() {
            let names: Vec<String> = contributor_to_biblatex_names(contributor);
            if !names.is_empty() {
                let _ = writeln!(&mut out, "  author = {{{}}},", names.join(" and "));
            }
        }
        if let Some(issued) = reference.csl_issued_date()
            && let Some(year) = issued.0.get(0..4)
        {
            let _ = writeln!(&mut out, "  year = {{{year}}},");
        }
        if let Some(doi) = reference.doi() {
            let _ = writeln!(&mut out, "  doi = {{{doi}}},");
        }
        let _ = writeln!(&mut out, "}}\n");
    }
    out
}

fn contributor_to_biblatex_names(contributor: citum_schema::reference::Contributor) -> Vec<String> {
    match contributor {
        citum_schema::reference::Contributor::SimpleName(n) => vec![n.name.to_string()],
        citum_schema::reference::Contributor::StructuredName(n) => {
            vec![format!("{}, {}", n.family, n.given)]
        }
        citum_schema::reference::Contributor::Multilingual(n) => {
            vec![format!("{}, {}", n.original.family, n.original.given)]
        }
        citum_schema::reference::Contributor::ContributorList(list) => list
            .0
            .into_iter()
            .flat_map(contributor_to_biblatex_names)
            .collect(),
    }
}

fn parse_ris(input: &str) -> Result<InputBibliography, ProcessorError> {
    let mut references = Vec::<InputReference>::new();
    let mut current = Vec::<(String, String)>::new();

    for line in input.lines() {
        let line = line.strip_prefix('\u{feff}').unwrap_or(line);
        let Some((tag, value)) = line.split_once("  - ") else {
            continue;
        };
        let tag = tag.trim();
        if tag.len() != 2 || !tag.is_ascii() {
            continue;
        }
        let tag = tag.to_string();
        let value = value.trim().to_string();
        if tag == "ER" {
            if !current.is_empty() {
                references.push(InputReference::from(ris_record_to_reference(&current)));
            }
            current.clear();
            continue;
        }
        current.push((tag, value));
    }

    if !current.is_empty() {
        references.push(InputReference::from(ris_record_to_reference(&current)));
    }

    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

fn ris_record_to_reference(fields: &[(String, String)]) -> LegacyReference {
    use csl_legacy::csl_json::{DateVariable, Name, StringOrNumber};

    let get = |tag: &str| -> Option<String> {
        fields
            .iter()
            .find_map(|(k, v)| (k == tag).then(|| v.clone()))
    };
    let get_all = |tag: &str| -> Vec<String> {
        fields
            .iter()
            .filter(|(k, _)| k == tag)
            .map(|(_, v)| v.clone())
            .collect()
    };

    let id = get("ID")
        .or_else(|| get("L1"))
        .or_else(|| get("M1"))
        .unwrap_or_else(|| "item".to_string());
    let title = get("TI").or_else(|| get("T1"));
    let ty = get("TY").unwrap_or_else(|| "BOOK".to_string());
    let author = {
        let authors = get_all("AU")
            .into_iter()
            .map(|n| {
                let parts: Vec<_> = n.split(',').map(str::trim).collect();
                if parts.len() >= 2 {
                    Name::new(parts[0], parts[1])
                } else {
                    Name::literal(parts.first().copied().unwrap_or(""))
                }
            })
            .collect::<Vec<_>>();
        (!authors.is_empty()).then_some(authors)
    };
    let issued = get("PY").or_else(|| get("Y1")).and_then(|s| {
        let year = s.chars().take(4).collect::<String>().parse::<i32>().ok()?;
        Some(DateVariable::year(year))
    });
    let doi = get("DO");
    let note = get("N1");
    let page = match (get("SP"), get("EP")) {
        (Some(sp), Some(ep)) => Some(format!("{sp}-{ep}")),
        (Some(sp), None) => Some(sp),
        _ => None,
    };
    let ref_type = if ty == "JOUR" || ty == "JFULL" {
        "article-journal".to_string()
    } else if ty == "CHAP" {
        "chapter".to_string()
    } else {
        "book".to_string()
    };

    LegacyReference {
        id,
        ref_type,
        author,
        title,
        container_title: get("JO").or_else(|| get("JF")),
        issued,
        volume: get("VL").map(StringOrNumber::String),
        issue: get("IS").map(StringOrNumber::String),
        page,
        doi,
        url: get("UR"),
        isbn: get("SN"),
        publisher: get("PB"),
        publisher_place: get("CY"),
        language: get("LA"),
        note,
        ..Default::default()
    }
}

fn render_ris(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let ty = match reference {
            InputReference::SerialComponent(_) => "JOUR",
            InputReference::CollectionComponent(_) => "CHAP",
            _ => "BOOK",
        };
        let _ = writeln!(&mut out, "TY  - {ty}");
        if let Some(id) = reference.id() {
            let _ = writeln!(&mut out, "ID  - {id}");
        }
        if let Some(title) = reference.title() {
            let _ = writeln!(&mut out, "TI  - {title}");
        }
        if let Some(contributor) = reference.author() {
            for name in contributor_to_biblatex_names(contributor) {
                let _ = writeln!(&mut out, "AU  - {name}");
            }
        }
        if let Some(issued) = reference.csl_issued_date()
            && let Some(year) = issued.0.get(0..4)
        {
            let _ = writeln!(&mut out, "PY  - {year}");
        }
        if let Some(doi) = reference.doi() {
            let _ = writeln!(&mut out, "DO  - {doi}");
        }
        let _ = writeln!(&mut out, "ER  -\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::plain::PlainText;
    use indexmap::IndexMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(stem: &str, ext: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{now}.{ext}"))
    }

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
    fn infer_refs_output_format_uses_supported_extensions() {
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.yaml")),
            RefsFormat::CitumYaml
        ));
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.bib")),
            RefsFormat::Biblatex
        ));
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.ris")),
            RefsFormat::Ris
        ));
    }

    #[test]
    fn load_merged_bibliography_rejects_cross_file_duplicate_membership() {
        let base = temp_path("citum-merged-bib", "dir");
        std::fs::create_dir_all(&base).expect("temp dir should be created");
        let bib_a = base.join("a.yaml");
        let bib_b = base.join("b.yaml");

        std::fs::write(
            &bib_a,
            r#"
references:
  - class: monograph
    id: ref-a
    type: book
    title: Book A
    issued: "2020"
sets:
  group-1: [ref-a]
"#,
        )
        .expect("first fixture should write");
        std::fs::write(
            &bib_b,
            r#"
references:
  - class: monograph
    id: ref-a
    type: book
    title: Book A
    issued: "2020"
sets:
  group-2: [ref-a]
"#,
        )
        .expect("second fixture should write");

        let err = load_merged_bibliography(&[bib_a.clone(), bib_b.clone()])
            .expect_err("must reject cross-file duplicate membership");

        assert!(
            err.to_string()
                .contains("appears in both compound sets 'group-1' and 'group-2'"),
            "unexpected error: {err}"
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn load_input_bibliography_accepts_biblatex() {
        let path = temp_path("citum-biblatex", "bib");
        std::fs::write(
            &path,
            "@article{smith2020,\n  title = {Article},\n  author = {Smith, Jane},\n  date = {2020},\n  journaltitle = {Journal}\n}",
        )
        .expect("biblatex fixture should write");

        let bibliography =
            load_input_bibliography(&path, RefsFormat::Biblatex).expect("BibLaTeX should parse");

        let reference = bibliography
            .references
            .iter()
            .find(|reference| reference.id().as_deref() == Some("smith2020"))
            .expect("reference should be loaded");
        assert_eq!(reference.ref_type(), "article-journal");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn load_and_write_ris_bibliography() {
        let input = temp_path("citum-ris-in", "ris");
        let output = temp_path("citum-ris-out", "ris");
        std::fs::write(
            &input,
            "TY  - JOUR\nID  - smith2020\nTI  - Article\nAU  - Smith, Jane\nPY  - 2020\nDO  - 10.1000/example\nER  -\n",
        )
        .expect("RIS fixture should write");

        let bibliography =
            load_input_bibliography(&input, RefsFormat::Ris).expect("RIS should parse");
        write_output_bibliography(&bibliography, &output, RefsFormat::Ris)
            .expect("RIS should serialize");
        let rendered = std::fs::read_to_string(&output).expect("RIS output should read");

        assert_eq!(
            rendered,
            "TY  - JOUR\nID  - smith2020\nTI  - Article\nAU  - Smith, Jane\nPY  - 2020\nDO  - 10.1000/example\nER  -\n\n"
        );

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn load_ris_bibliography_accepts_utf8_bom() {
        let input = temp_path("citum-ris-bom", "ris");
        std::fs::write(
            &input,
            "\u{feff}TY  - JOUR\nID  - smith2020\nTI  - Article\nER  -\n",
        )
        .expect("RIS fixture should write");

        let bibliography =
            load_input_bibliography(&input, RefsFormat::Ris).expect("RIS with BOM should parse");

        let reference = bibliography
            .references
            .iter()
            .find(|reference| reference.id().as_deref() == Some("smith2020"))
            .expect("reference should be loaded");
        assert_eq!(reference.ref_type(), "article-journal");

        let _ = std::fs::remove_file(input);
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
    /// Parse a JSON array of native Citum references before falling back to CSL-JSON.
    fn parse_json_vec_input_references() {
        let json = r#"[
  {
    "class": "collection",
    "id": "edited-book-1",
    "type": "edited-book",
    "title": "Edited Book",
    "editor": [{"family": "Miller", "given": "Ruth"}],
    "issued": "2022"
  }
]"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse native Citum JSON vec");
        assert_eq!(result.references.len(), 1);
        let reference = result
            .references
            .get("edited-book-1")
            .expect("reference should be preserved");
        assert_eq!(reference.ref_type(), "book");
        assert!(matches!(reference, Reference::Collection(_)));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse hybrid JSON fixtures that combine Citum `class` tags with CSL-JSON contributor/date shapes.
    fn parse_json_hybrid_edited_book_reference() {
        let json = r#"[
  {
    "id": "edited-book-1",
    "class": "collection",
    "type": "edited-book",
    "title": "Edited Book",
    "editor": [{"family": "Miller", "given": "Ruth"}],
    "issued": {"date-parts": [[2022]]},
    "publisher": "Example Press",
    "publisher-place": "Chicago"
  }
]"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse hybrid Citum/CSL JSON");
        let reference = result
            .references
            .get("edited-book-1")
            .expect("reference should be preserved");
        assert_eq!(reference.ref_type(), "book");
        assert!(matches!(reference, Reference::Collection(_)));
    }

    #[test]
    /// Preserve URLs when hybrid edited-book JSON falls back through the legacy loader.
    fn parse_json_hybrid_edited_book_preserves_url() {
        let json = r#"[
  {
    "id": "edited-book-1",
    "class": "collection",
    "type": "edited-book",
    "title": "Edited Book",
    "editor": [{"family": "Miller", "given": "Ruth"}],
    "issued": {"date-parts": [[2022]]},
    "publisher": "Example Press",
    "publisher-place": "Chicago",
    "URL": "https://example.com/edited-book"
  }
]"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse hybrid Citum/CSL JSON");
        let reference = result
            .references
            .get("edited-book-1")
            .expect("reference should be preserved");

        assert_eq!(
            reference.url().as_ref().map(url::Url::as_str),
            Some("https://example.com/edited-book")
        );
    }

    #[test]
    /// Parse a JSON array of CSL-JSON objects directly into `LoadedBibliography`.
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
    /// Parse a Citum `InputBibliography` from JSON with references and sets.
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
    /// Parse an `IndexMap` of CSL-JSON objects keyed by id from JSON.
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
    /// Parse a Citum YAML `InputBibliography` with references.
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
        let yaml = r"
references:
  - id: yaml-legacy-1
    type: book
    title: YAML Legacy Book
sets: null
";
        let result = parse_yaml_bibliography(yaml).expect("should parse wrapped legacy YAML");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("yaml-legacy-1"));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse an `IndexMap` of legacy references keyed by id from YAML.
    fn parse_yaml_indexmap() {
        // IndexMap format: plain object with reference-id keys mapping to legacy ref objects.
        // Structure: { id1: {type, title}, id2: {type, title} }
        // Must use legacy CSL-JSON field names (not InputReference class tags)
        // to avoid matching InputBibliography or Vec<InputReference>.
        let yaml = r"ref-yaml-1:
  id: ref-yaml-1
  type: book
  title: First Book
ref-yaml-2:
  id: ref-yaml-2
  type: journal-article
  title: Second Article
";
        let result = parse_yaml_bibliography(yaml).expect("should parse YAML IndexMap format");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("ref-yaml-1"));
        assert!(result.references.contains_key("ref-yaml-2"));
    }

    #[test]
    /// Parse a YAML sequence of `InputReference` objects.
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
