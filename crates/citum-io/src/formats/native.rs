/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Citum-native (YAML / JSON / CBOR) bibliography serialization helpers.
//!
//! Parsing is delegated to `citum-refs`. Only serialization (output) lives here.

use citum_engine::ProcessorError;

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

// Test-only aliases exposing citum-refs parsers under the legacy names used by
// existing lib.rs tests (via `use super::*`).
#[cfg(test)]
pub(crate) use citum_refs::formats::native::loaded_from_input_refs as loaded_from_input_bibliography;
#[cfg(test)]
pub(crate) use citum_refs::formats::native::parse_json_refs as parse_json_bibliography;
#[cfg(test)]
pub(crate) use citum_refs::formats::native::parse_yaml_refs as parse_yaml_bibliography;
