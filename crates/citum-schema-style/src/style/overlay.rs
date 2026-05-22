/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Null-aware style overlay merging.

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::Style;

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Internal merging logic ensures presence of re-parsed values"
)]
pub(crate) fn merge_style_overlay(base: &mut Style, overlay: &Style) {
    if !overlay.info.is_empty() {
        base.info = merge_serialized(base.info.clone(), &overlay.info);
    }

    if let Some(templates) = &overlay.templates {
        base.templates = Some(match &base.templates {
            Some(existing) => merge_serialized(existing.clone(), templates),
            None => templates.clone(),
        });
    }

    if let Some(options) = &overlay.options {
        match &mut base.options {
            Some(existing) => existing.merge(options),
            None => base.options = Some(options.clone()),
        }
    }

    let raw_citation = overlay.raw_yaml.as_ref().and_then(|v| v.get("citation"));
    if raw_citation.is_some() || overlay.citation.is_some() {
        base.citation = Some(match (&base.citation, raw_citation) {
            (Some(existing), Some(raw)) => merge_serialized_value(existing.clone(), raw),
            (Some(existing), None) => {
                merge_serialized(existing.clone(), overlay.citation.as_ref().unwrap())
            }
            (None, Some(raw)) => serde_yaml::from_value(raw.clone()).expect("citation parses"),
            (None, None) => overlay.citation.clone().unwrap(),
        });
    }

    let raw_bibliography = overlay
        .raw_yaml
        .as_ref()
        .and_then(|v| v.get("bibliography"));
    if raw_bibliography.is_some() || overlay.bibliography.is_some() {
        base.bibliography = Some(match (&base.bibliography, raw_bibliography) {
            (Some(existing), Some(raw)) => merge_serialized_value(existing.clone(), raw),
            (Some(existing), None) => {
                merge_serialized(existing.clone(), overlay.bibliography.as_ref().unwrap())
            }
            (None, Some(raw)) => serde_yaml::from_value(raw.clone()).expect("bibliography parses"),
            (None, None) => overlay.bibliography.clone().unwrap(),
        });
    }

    if let Some(custom) = &overlay.custom {
        base.custom = Some(match &base.custom {
            Some(existing) => merge_serialized(existing.clone(), custom),
            None => custom.clone(),
        });
    }
}

#[allow(clippy::expect_used, reason = "T must be serializable to YAML")]
pub(crate) fn merge_serialized<T>(base: T, overlay: &T) -> T
where
    T: Clone + DeserializeOwned + Serialize,
{
    let overlay_value = serde_yaml::to_value(overlay).expect("serializable overlay");
    merge_serialized_value(base, &overlay_value)
}

#[allow(
    clippy::expect_used,
    reason = "T must be serializable and merged values must match schema"
)]
pub(crate) fn merge_serialized_value<T>(base: T, overlay: &serde_yaml::Value) -> T
where
    T: Clone + DeserializeOwned + Serialize,
{
    let mut base_value = serde_yaml::to_value(base).expect("serializable base");
    merge_yaml_value(&mut base_value, overlay);
    serde_yaml::from_value(base_value).expect("merged value matches schema")
}

pub(crate) fn merge_yaml_value(base: &mut serde_yaml::Value, overlay: &serde_yaml::Value) {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_yaml_value(base_value, overlay_value);
                } else {
                    base_map.insert(key.clone(), overlay_value.clone());
                }
            }
        }
        (base_value, overlay_value) => {
            *base_value = overlay_value.clone();
        }
    }
}
