/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Citum-native (YAML / JSON / CBOR) bibliography parsing and serialization helpers.

use std::io::Cursor;

use citum_engine::processor::validate_compound_sets;
use citum_engine::{ProcessorError, Reference};
use citum_schema::InputBibliography;
use citum_schema::reference::conversion::input_reference_from_legacy_edited_book;
use citum_schema::reference::types::{ArchiveInfo, EprintInfo};
use citum_schema::reference::{ClassExtension, InputReference};
use csl_legacy::csl_json::Reference as LegacyReference;
use indexmap::IndexMap;

use crate::LoadedBibliography;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct LegacyBibliographyWrapper {
    pub(crate) references: Vec<LegacyReference>,
    #[serde(default)]
    pub(crate) sets: Option<IndexMap<String, Vec<String>>>,
}

pub(crate) fn loaded_from_input_bibliography(
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

pub(crate) fn parse_cbor_bibliography(bytes: &[u8]) -> Result<LoadedBibliography, ProcessorError> {
    let input_bib = ciborium::de::from_reader::<InputBibliography, _>(Cursor::new(bytes))
        .map_err(|e| ProcessorError::ParseError("CBOR".to_string(), e.to_string()))?;
    loaded_from_input_bibliography(input_bib)
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

/// Parse a JSON value into an [`InputReference`] with class/type routing and
/// optional archive-info / eprint extension fields.
///
/// Unified from the former `parse_hybrid_json_reference` and
/// `load_hybrid_json_reference` — the extension pass is a no-op when neither
/// `archive-info` nor `eprint` keys are present.
fn parse_hybrid_json_reference(value: serde_json::Value) -> Result<InputReference, ProcessorError> {
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
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

    let reference = if class == "collection" && ref_type == "edited-book" {
        input_reference_from_legacy_edited_book(legacy)
    } else {
        InputReference::from(legacy)
    };

    apply_hybrid_json_extensions(reference, &value)
}

fn load_hybrid_json_references(
    references: &[serde_json::Value],
) -> Result<Vec<InputReference>, ProcessorError> {
    references
        .iter()
        .cloned()
        .map(parse_hybrid_json_reference)
        .collect()
}

/// Parse JSON bytes into a [`LoadedBibliography`].
///
/// Accepts Citum JSON, CSL-JSON arrays, wrapped legacy objects, and `IndexMap`
/// keyed-by-id objects.
pub(crate) fn parse_json_bibliography(bytes: &[u8]) -> Result<LoadedBibliography, ProcessorError> {
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
            .filter(|v| !v.is_null())
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
        return loaded_from_hybrid_json_array(references, sets);
    }

    let mut bib = IndexMap::new();

    if let Ok(legacy_bib) = serde_json::from_slice::<Vec<LegacyReference>>(bytes) {
        for ref_item in legacy_bib {
            bib.insert(ref_item.id.clone(), Reference::from(ref_item));
        }
        return Ok(LoadedBibliography {
            references: bib,
            sets: None,
        });
    }
    if let Ok(input_bib) = serde_json::from_slice::<InputBibliography>(bytes) {
        return loaded_from_input_bibliography(input_bib);
    }

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

    match serde_json::from_slice::<InputBibliography>(bytes) {
        #[allow(clippy::unreachable, reason = "Primary format must have failed")]
        Ok(_) => unreachable!(),
        Err(e) => Err(ProcessorError::ParseError(
            "JSON".to_string(),
            e.to_string(),
        )),
    }
}

/// Parse a YAML string into a [`LoadedBibliography`].
///
/// Accepts Citum YAML, wrapped legacy format, `IndexMap`, and sequence variants.
pub(crate) fn parse_yaml_bibliography(content: &str) -> Result<LoadedBibliography, ProcessorError> {
    let _: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

    let mut bib = IndexMap::new();

    if let Ok(input_bib) = serde_yaml::from_str::<InputBibliography>(content) {
        return loaded_from_input_bibliography(input_bib);
    }

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

    match serde_yaml::from_str::<InputBibliography>(content) {
        #[allow(clippy::unreachable, reason = "Primary format must have failed")]
        Ok(_) => unreachable!(),
        Err(e) => Err(ProcessorError::ParseError(
            "YAML".to_string(),
            e.to_string(),
        )),
    }
}

/// Deserialize bytes into any serde-compatible type based on the file extension.
pub(crate) fn deserialize_any<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    ext: &str,
) -> Result<T, ProcessorError> {
    match ext {
        "yaml" | "yml" => serde_yaml::from_slice(bytes)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string())),
        "json" => serde_json::from_slice(bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string())),
        "cbor" => ciborium::de::from_reader(Cursor::new(bytes))
            .map_err(|e| ProcessorError::ParseError("CBOR".to_string(), e.to_string())),
        _ => serde_yaml::from_slice(bytes)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string())),
    }
}

/// Serialize a value to bytes in the format identified by the file extension.
pub(crate) fn serialize_any<T: serde::Serialize>(
    obj: &T,
    ext: &str,
) -> Result<Vec<u8>, ProcessorError> {
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

/// Load a Citum JSON bibliography from raw bytes.
///
/// Tries native `InputBibliography` first, then falls back to hybrid
/// JSON array / wrapped-object formats.
pub(crate) fn load_citum_json_bibliography(
    bytes: &[u8],
) -> Result<InputBibliography, ProcessorError> {
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
                .filter(|v| !v.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?,
            ..Default::default()
        });
    }

    deserialize_any(bytes, "json")
}
