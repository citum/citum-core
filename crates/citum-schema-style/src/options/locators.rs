/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Locator rendering configuration.
//!
//! Defines how citation locators (page numbers, sections, etc.) are displayed,
//! including label forms, range formatting, and compound locator patterns.

use super::PageRangeFormat;
use citum_schema_data::citation::LocatorType;
use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// How a locator label is displayed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LabelForm {
    /// No label, bare value: "33"
    None,
    /// Short form: "p. 33"
    #[default]
    Short,
    /// Long form: "page 33"
    Long,
    /// Symbol form if available in locale
    Symbol,
}

/// Whether labels appear on every segment, only the first, or none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LabelRepeat {
    /// Label on every segment
    #[default]
    All,
    /// Label only on the first segment
    First,
    /// No labels
    None,
}

/// A coarse reference genre used as an optional gate on locator patterns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TypeClass {
    /// Legal citations (e.g., "legal-case", "statute")
    Legal,
    /// Classical works with traditional numbering
    Classical,
    /// Standard reference types
    #[default]
    Standard,
}

/// Per-locator-kind configuration overrides.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LocatorKindConfig {
    /// Override the default label form for this locator kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_form: Option<LabelForm>,
    /// Override the global range format for this locator kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_format: Option<PageRangeFormat>,
    /// Strip trailing periods from labels (e.g., "p." → "p").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_label_periods: Option<bool>,
}

/// A pattern matching a specific combination of LocatorType values.
///
/// Patterns are tested in declaration order; first match wins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LocatorPattern {
    /// The set of LocatorType values this pattern matches (order-insensitive).
    pub kinds: Vec<LocatorType>,
    /// Optional gate on reference type class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_class: Option<TypeClass>,
    /// Rendering order of segments when pattern matches.
    pub order: Vec<LocatorType>,
    /// Delimiter between segments (default: ", ").
    #[serde(default = "default_delimiter")]
    pub delimiter: String,
    /// Whether labels appear on every segment, only the first, or none.
    #[serde(default)]
    pub label_repeat: LabelRepeat,
}

/// Top-level locator rendering configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LocatorConfig {
    /// Default label form for all locator kinds (default: Short).
    #[serde(default = "default_label_form")]
    pub default_label_form: LabelForm,
    /// Range format for all locator kinds (default: Expanded).
    #[serde(default)]
    pub range_format: PageRangeFormat,
    /// Strip trailing periods from labels globally (e.g., "p." → "p").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_label_periods: Option<bool>,
    /// Per-kind configuration overrides.
    #[serde(default)]
    pub kinds: HashMap<LocatorType, LocatorKindConfig>,
    /// Patterns for compound locators and type-specific rendering.
    #[serde(default)]
    pub patterns: Vec<LocatorPattern>,
    /// Fallback delimiter for unmatched compound locators (default: ", ").
    #[serde(default = "default_delimiter")]
    pub fallback_delimiter: String,
}

impl Default for LocatorConfig {
    fn default() -> Self {
        Self {
            default_label_form: LabelForm::Short,
            range_format: PageRangeFormat::Expanded,
            strip_label_periods: None,
            kinds: HashMap::new(),
            patterns: Vec::new(),
            fallback_delimiter: ", ".to_string(),
        }
    }
}

/// Named presets for common locator configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LocatorPreset {
    /// Note style: bare page numbers, short labels for other kinds.
    Note,
    /// Author-date / numbered: short labels for all kinds.
    AuthorDate,
}

impl LocatorPreset {
    /// Resolve a preset to an explicit `LocatorConfig`.
    #[must_use]
    pub fn config(self) -> LocatorConfig {
        match self {
            LocatorPreset::Note => LocatorConfig {
                default_label_form: LabelForm::Short,
                range_format: PageRangeFormat::Expanded,
                strip_label_periods: None,
                kinds: {
                    let mut m = HashMap::new();
                    // Page locators have no label in note style
                    m.insert(
                        LocatorType::Page,
                        LocatorKindConfig {
                            label_form: Some(LabelForm::None),
                            range_format: None,
                            strip_label_periods: None,
                        },
                    );
                    m
                },
                patterns: Vec::new(),
                fallback_delimiter: ", ".to_string(),
            },
            LocatorPreset::AuthorDate => LocatorConfig {
                default_label_form: LabelForm::Short,
                range_format: PageRangeFormat::Expanded,
                strip_label_periods: None,
                kinds: HashMap::new(),
                patterns: Vec::new(),
                fallback_delimiter: ", ".to_string(),
            },
        }
    }
}

/// Preset-or-explicit entry — same pattern as DateConfigEntry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum LocatorConfigEntry {
    /// A preset name.
    Preset(LocatorPreset),
    /// Explicit configuration.
    Explicit(LocatorConfig),
}

impl LocatorConfigEntry {
    /// Resolve a LocatorConfigEntry to an explicit LocatorConfig.
    #[must_use]
    pub fn resolve(self) -> LocatorConfig {
        match self {
            LocatorConfigEntry::Preset(preset) => preset.config(),
            LocatorConfigEntry::Explicit(config) => config,
        }
    }
}

/// Default label form.
fn default_label_form() -> LabelForm {
    LabelForm::Short
}

/// Default delimiter string.
fn default_delimiter() -> String {
    ", ".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locator_preset_note() {
        let config = LocatorPreset::Note.config();
        assert_eq!(config.default_label_form, LabelForm::Short);
        assert_eq!(config.range_format, PageRangeFormat::Expanded);
    }

    #[test]
    fn test_locator_preset_author_date() {
        let config = LocatorPreset::AuthorDate.config();
        assert_eq!(config.default_label_form, LabelForm::Short);
        assert_eq!(config.range_format, PageRangeFormat::Expanded);
    }

    #[test]
    fn test_locator_config_entry_preset() {
        let entry = LocatorConfigEntry::Preset(LocatorPreset::Note);
        let config = entry.resolve();
        assert_eq!(config.default_label_form, LabelForm::Short);
    }

    #[test]
    fn test_locator_config_entry_explicit() {
        let entry = LocatorConfigEntry::Explicit(LocatorConfig {
            default_label_form: LabelForm::Long,
            ..Default::default()
        });
        let config = entry.resolve();
        assert_eq!(config.default_label_form, LabelForm::Long);
    }

    #[test]
    fn test_locator_config_default() {
        let config = LocatorConfig::default();
        assert_eq!(config.default_label_form, LabelForm::Short);
        assert_eq!(config.fallback_delimiter, ", ");
    }
}
