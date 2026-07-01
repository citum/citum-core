/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Substitution rules for missing author data.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum SubstituteConfig {
    /// A named preset (e.g., "standard", "editor-first", "title-first").
    Preset(crate::presets::SubstitutePreset),
    /// Explicit substitution configuration.
    Explicit(Substitute),
}

impl Default for SubstituteConfig {
    fn default() -> Self {
        SubstituteConfig::Explicit(Substitute::default())
    }
}

impl SubstituteConfig {
    /// Resolve this config to a concrete `Substitute`.
    pub fn resolve(&self) -> Substitute {
        match self {
            SubstituteConfig::Preset(preset) => preset.config(),
            SubstituteConfig::Explicit(config) => config.clone(),
        }
    }

    /// Merge an override substitute config over a base config.
    #[must_use]
    pub fn merged(base: &Self, override_config: &Self) -> Self {
        SubstituteConfig::Explicit(Substitute::merged(
            &base.resolve(),
            &override_config.resolve(),
        ))
    }
}

/// Explicit substitution configuration.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Substitute {
    /// Form to use for contributor roles when substituting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributor_role_form: Option<String>,
    /// Optional `text-case` transform applied to the substitute role label.
    ///
    /// Lets a style capitalise the locale's lowercase term, e.g. IEEE's `Eds.`
    /// from `eds.`. Only affects the substitute (editor-as-author) path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributor_role_case: Option<crate::options::titles::TextCase>,
    /// Ordered list of fields to try as substitutes.
    #[serde(default)]
    pub template: Vec<SubstituteKey>,
    /// Type-specific substitution overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub overrides: HashMap<String, Vec<SubstituteKey>>,
    /// Per-role fallback chains for non-author contributor substitution.
    ///
    /// Keys and fallback entries normalize to canonical kebab-case role names.
    /// Built-in template roles use locale-aware labels automatically. Custom
    /// role names still participate in fallback and suppression even when they
    /// do not have a dedicated template enum variant.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub role_substitute: HashMap<String, Vec<String>>,
    /// Forward-compat: captures unknown keys when an older engine reads a
    /// style produced by a newer schema. Empty by default; treated as a
    /// SoftDegrade signal. See `docs/specs/FORWARD_COMPATIBILITY.md`.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub unknown_fields: std::collections::BTreeMap<String, serde_yaml::Value>,
}

impl Default for Substitute {
    fn default() -> Self {
        Self {
            contributor_role_form: None,
            contributor_role_case: None,
            template: vec![
                SubstituteKey::Editor,
                SubstituteKey::Title,
                SubstituteKey::Translator,
            ],
            overrides: HashMap::new(),
            role_substitute: HashMap::new(),
            unknown_fields: std::collections::BTreeMap::new(),
        }
    }
}

impl Substitute {
    /// Merge an override substitute config over a base config.
    pub fn merge(&mut self, other: &Self) {
        if other.contributor_role_form.is_some() {
            self.contributor_role_form = other.contributor_role_form.clone();
        }
        if other.contributor_role_case.is_some() {
            self.contributor_role_case = other.contributor_role_case;
        }
        if !other.template.is_empty() {
            self.template = other.template.clone();
        }
        self.overrides.extend(other.overrides.clone());
        self.role_substitute.extend(other.role_substitute.clone());
    }

    /// Create a merged substitute config from base and override.
    #[must_use]
    pub fn merged(base: &Self, override_config: &Self) -> Self {
        let mut result = base.clone();
        result.merge(override_config);
        result
    }
}

/// Fields that can be used as author substitutes.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SubstituteKey {
    #[serde(rename = "collection-editor")]
    CollectionEditor,
    Editor,
    #[serde(rename = "parent-serial")]
    ParentSerial,
    Title,
    Translator,
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::{Substitute, SubstituteConfig, SubstituteKey};
    use std::collections::HashMap;

    #[test]
    fn merged_substitute_configs_preserve_role_substitute_chains() {
        let base = SubstituteConfig::Explicit(Substitute {
            role_substitute: HashMap::from([(
                "container-author".to_string(),
                vec!["editor".to_string()],
            )]),
            ..Default::default()
        });
        let override_config = SubstituteConfig::Preset(crate::presets::SubstitutePreset::Standard);

        let merged = SubstituteConfig::merged(&base, &override_config).resolve();

        assert_eq!(
            merged.role_substitute.get("container-author"),
            Some(&vec!["editor".to_string()])
        );
        assert_eq!(
            merged.template,
            vec![
                SubstituteKey::Editor,
                SubstituteKey::Title,
                SubstituteKey::Translator
            ]
        );
    }
}
