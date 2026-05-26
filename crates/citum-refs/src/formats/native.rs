/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Citum-native (YAML / JSON / CBOR) reference data parsing helpers.

use std::io::Cursor;

use citum_schema::InputBibliography;
use citum_schema::reference::conversion::input_reference_from_legacy_edited_book;
use citum_schema::reference::types::{ArchiveInfo, EprintInfo};
use citum_schema::reference::{ClassExtension, InputReference};
use csl_legacy::csl_json::Reference as LegacyReference;
use indexmap::IndexMap;

use crate::{LoadedRefs, RefsError};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct LegacyRefsWrapper {
    pub(crate) references: Vec<LegacyReference>,
    #[serde(default)]
    pub(crate) sets: Option<IndexMap<String, Vec<String>>>,
}

/// Convert an `InputBibliography` into a `LoadedRefs`, validating compound sets.
///
/// # Errors
///
/// Returns an error when compound sets reference unknown IDs or contain duplicates.
pub fn loaded_from_input_refs(input_bib: InputBibliography) -> Result<LoadedRefs, RefsError> {
    let mut references = IndexMap::new();
    for r in input_bib.references {
        if let Some(id) = r.id() {
            references.insert(id.to_string(), r);
        }
    }
    let sets = crate::validate_compound_sets(input_bib.sets, &references)?;
    Ok(LoadedRefs { references, sets })
}

/// Parse CBOR bytes into a `LoadedRefs`.
///
/// # Errors
///
/// Returns an error when the bytes cannot be parsed as a Citum bibliography.
pub fn parse_cbor_refs(bytes: &[u8]) -> Result<LoadedRefs, RefsError> {
    let input_bib = ciborium::de::from_reader::<InputBibliography, _>(Cursor::new(bytes))
        .map_err(|e| RefsError::ParseError("CBOR".to_string(), e.to_string()))?;
    loaded_from_input_refs(input_bib)
}

fn loaded_from_hybrid_json_array(
    references: &[serde_json::Value],
    sets: Option<IndexMap<String, Vec<String>>>,
) -> Result<LoadedRefs, RefsError> {
    let mut bib = IndexMap::new();
    for value in references.iter().cloned() {
        let reference = parse_hybrid_json_reference(value)?;
        if let Some(id) = reference.id() {
            bib.insert(id.to_string(), reference);
        }
    }
    let sets = crate::validate_compound_sets(sets, &bib)?;
    Ok(LoadedRefs {
        references: bib,
        sets,
    })
}

fn apply_hybrid_json_extensions(
    mut reference: InputReference,
    value: &serde_json::Value,
) -> Result<InputReference, RefsError> {
    let archive_info = value
        .get("archive-info")
        .filter(|raw| !raw.is_null())
        .cloned()
        .map(serde_json::from_value::<ArchiveInfo>)
        .transpose()
        .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?;
    let eprint = value
        .get("eprint")
        .filter(|raw| !raw.is_null())
        .cloned()
        .map(serde_json::from_value::<EprintInfo>)
        .transpose()
        .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?;

    match reference.extension_mut() {
        ClassExtension::Monograph(record) => {
            if let Some(info) = archive_info.clone() {
                record.archive_info = Some(info);
            }
            if let Some(info) = eprint.clone() {
                record.eprint = Some(info);
            }
        }
        ClassExtension::CollectionComponent(record) => {
            if let Some(info) = archive_info.clone() {
                record.archive_info = Some(info);
            }
            if let Some(info) = eprint.clone() {
                record.eprint = Some(info);
            }
        }
        ClassExtension::SerialComponent(record) => {
            if let Some(info) = archive_info {
                record.archive_info = Some(info);
            }
            if let Some(info) = eprint {
                record.eprint = Some(info);
            }
        }
        _ => {}
    }
    Ok(reference)
}

fn parse_hybrid_json_reference(value: serde_json::Value) -> Result<InputReference, RefsError> {
    if let Ok(reference) = serde_json::from_value::<InputReference>(value.clone()) {
        return Ok(reference);
    }

    let class = value
        .get("class")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let ref_type = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let legacy = serde_json::from_value::<LegacyReference>(value.clone())
        .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?;

    let reference = if class == "collection" && ref_type == "edited-book" {
        input_reference_from_legacy_edited_book(legacy)
    } else {
        InputReference::from(legacy)
    };

    apply_hybrid_json_extensions(reference, &value)
}

fn load_hybrid_json_references(
    references: &[serde_json::Value],
) -> Result<Vec<InputReference>, RefsError> {
    references
        .iter()
        .cloned()
        .map(parse_hybrid_json_reference)
        .collect()
}

/// Parse JSON bytes into a `LoadedRefs`.
///
/// Accepts Citum JSON, CSL-JSON arrays, wrapped legacy objects, and
/// `IndexMap` keyed-by-id objects.
///
/// # Errors
///
/// Returns an error when the bytes cannot be parsed in any supported format.
pub fn parse_json_refs(bytes: &[u8]) -> Result<LoadedRefs, RefsError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?;

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
            .filter(|v| !v.is_null())
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?;
        return loaded_from_hybrid_json_array(references, sets);
    }

    let mut bib = IndexMap::new();

    if let Ok(legacy_bib) = serde_json::from_slice::<Vec<LegacyReference>>(bytes) {
        for ref_item in legacy_bib {
            bib.insert(ref_item.id.clone(), InputReference::from(ref_item));
        }
        return Ok(LoadedRefs {
            references: bib,
            sets: None,
        });
    }
    if let Ok(input_bib) = serde_json::from_slice::<InputBibliography>(bytes) {
        return loaded_from_input_refs(input_bib);
    }

    if let Ok(wrapper) = serde_json::from_slice::<LegacyRefsWrapper>(bytes) {
        for ref_item in wrapper.references {
            bib.insert(ref_item.id.clone(), InputReference::from(ref_item));
        }
        let sets = crate::validate_compound_sets(wrapper.sets, &bib)?;
        return Ok(LoadedRefs {
            references: bib,
            sets,
        });
    }

    if let Ok(map) = serde_json::from_slice::<IndexMap<String, serde_json::Value>>(bytes) {
        let mut found = false;
        for (id, val) in map {
            if let Ok(ref_item) = serde_json::from_value::<LegacyReference>(val) {
                let mut r = InputReference::from(ref_item);
                if r.id().is_none() {
                    r.set_id(id.clone());
                }
                bib.insert(id.clone(), r);
                found = true;
            }
        }
        if found {
            return Ok(LoadedRefs {
                references: bib,
                sets: None,
            });
        }
    }

    match serde_json::from_slice::<InputBibliography>(bytes) {
        #[allow(clippy::unreachable, reason = "Primary format must have failed")]
        Ok(_) => unreachable!(),
        Err(e) => Err(RefsError::ParseError("JSON".to_string(), e.to_string())),
    }
}

/// Parse a YAML string into a `LoadedRefs`.
///
/// Accepts Citum YAML, wrapped legacy format, `IndexMap`, and sequence variants.
///
/// # Errors
///
/// Returns an error when the string cannot be parsed in any supported format.
pub fn parse_yaml_refs(content: &str) -> Result<LoadedRefs, RefsError> {
    let _: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| RefsError::ParseError("YAML".to_string(), e.to_string()))?;

    let mut bib = IndexMap::new();

    if let Ok(input_bib) = serde_yaml::from_str::<InputBibliography>(content) {
        return loaded_from_input_refs(input_bib);
    }

    if let Ok(wrapper) = serde_yaml::from_str::<LegacyRefsWrapper>(content) {
        for ref_item in wrapper.references {
            bib.insert(ref_item.id.clone(), InputReference::from(ref_item));
        }
        let sets = crate::validate_compound_sets(wrapper.sets, &bib)?;
        return Ok(LoadedRefs {
            references: bib,
            sets,
        });
    }

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
                let mut r = InputReference::from(ref_item);
                if r.id().is_none() {
                    r.set_id(key.clone());
                }
                bib.insert(key, r);
                found = true;
            }
        }
        if found {
            return Ok(LoadedRefs {
                references: bib,
                sets: None,
            });
        }
    }

    if let Ok(refs) = serde_yaml::from_str::<Vec<InputReference>>(content) {
        for r in refs {
            if let Some(id) = r.id() {
                bib.insert(id.to_string(), r);
            }
        }
        return Ok(LoadedRefs {
            references: bib,
            sets: None,
        });
    }

    match serde_yaml::from_str::<InputBibliography>(content) {
        #[allow(clippy::unreachable, reason = "Primary format must have failed")]
        Ok(_) => unreachable!(),
        Err(e) => Err(RefsError::ParseError("YAML".to_string(), e.to_string())),
    }
}

/// Deserialize bytes into any serde-compatible type based on file extension.
///
/// # Errors
///
/// Returns an error when parsing fails.
pub fn deserialize_any<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    ext: &str,
) -> Result<T, RefsError> {
    match ext {
        "yaml" | "yml" => serde_yaml::from_slice(bytes)
            .map_err(|e| RefsError::ParseError("YAML".to_string(), e.to_string())),
        "json" => serde_json::from_slice(bytes)
            .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string())),
        "cbor" => ciborium::de::from_reader(Cursor::new(bytes))
            .map_err(|e| RefsError::ParseError("CBOR".to_string(), e.to_string())),
        _ => serde_yaml::from_slice(bytes)
            .map_err(|e| RefsError::ParseError("YAML".to_string(), e.to_string())),
    }
}

/// Load a Citum JSON bibliography from raw bytes into `InputBibliography`.
///
/// Tries native `InputBibliography` first, then falls back to hybrid JSON
/// array and wrapped-object formats.
///
/// # Errors
///
/// Returns an error when the bytes cannot be parsed in any supported format.
pub fn load_citum_json(bytes: &[u8]) -> Result<InputBibliography, RefsError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?;

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
                .filter(|v| !v.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| RefsError::ParseError("JSON".to_string(), e.to_string()))?,
            ..Default::default()
        });
    }

    deserialize_any(bytes, "json")
}
