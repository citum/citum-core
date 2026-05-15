/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Document-level abbreviation map type.

use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Document-level map from full rendered strings to their abbreviations.
///
/// Keys are the full rendered string (exact, case-sensitive). Values are the
/// replacement abbreviation. Applied after value extraction, before output.
///
/// Accepts both `abbreviation-map` (YAML frontmatter) and `abbreviation_map` forms.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(transparent)]
pub struct AbbreviationMap(pub HashMap<String, String>);

#[cfg(test)]
mod tests {
    use super::AbbreviationMap;

    #[test]
    fn given_flat_map_when_deserialized_then_entries_are_preserved() {
        let json = r#"{
            "Estates Gazette": "EG",
            "Lloyd's Law Reports": "Lloyd's Rep"
        }"#;

        let map: AbbreviationMap = serde_json::from_str(json).unwrap_or_default();

        assert_eq!(map.0.get("Estates Gazette"), Some(&"EG".to_string()));
        assert_eq!(
            map.0.get("Lloyd's Law Reports"),
            Some(&"Lloyd's Rep".to_string())
        );
    }

    #[test]
    fn given_reserved_looking_keys_when_deserialized_then_keys_are_entries() {
        let json = r#"{
            "title": "ttl.",
            "description": "desc.",
            "metadata": "meta.",
            "entries": "ent."
        }"#;

        let map: AbbreviationMap = serde_json::from_str(json).unwrap_or_default();

        assert_eq!(map.0.get("title"), Some(&"ttl.".to_string()));
        assert_eq!(map.0.get("description"), Some(&"desc.".to_string()));
        assert_eq!(map.0.get("metadata"), Some(&"meta.".to_string()));
        assert_eq!(map.0.get("entries"), Some(&"ent.".to_string()));
    }

    #[test]
    fn given_flat_map_when_serialized_then_output_has_only_entries() {
        let json = r#"{
            "title": "ttl.",
            "World Health Organization": "WHO"
        }"#;
        let map: AbbreviationMap = serde_json::from_str(json).unwrap_or_default();

        let serialized = serde_json::to_value(&map).unwrap_or(serde_json::Value::Null);

        assert_eq!(serialized.get("title"), Some(&serde_json::json!("ttl.")));
        assert_eq!(
            serialized.get("World Health Organization"),
            Some(&serde_json::json!("WHO"))
        );
        assert!(serialized.get("metadata").is_none());
    }
}
