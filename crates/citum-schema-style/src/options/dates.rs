/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use crate::options::localization::MonthFormat;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time display format.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TimeFormat {
    /// 12-hour clock with AM/PM (e.g., "11:30 PM")
    Hour12,
    /// 24-hour clock (e.g., "23:30")
    Hour24,
}

/// Era label profile for date rendering.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum EraLabels {
    /// Preserve current behavior: negative years use locale `before-era`, positive years unlabeled.
    #[default]
    Default,
    /// Negative years use locale `bc`, positive years use locale `ad`.
    BcAd,
    /// Negative years use locale `bce`, positive years use locale `ce`.
    BceCe,
}

/// Rendering policy for negative EDTF years with unspecified digits.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum NegativeUnspecifiedYears {
    /// Render as explicit historical ranges (e.g., `-009u` → `100–91 BC`).
    #[default]
    Range,
    /// Reserved for future prose-oriented output; falls back to `range` if selected.
    Fuzzy,
}

/// Date config: either a preset name or explicit configuration.
///
/// Allows styles to write `dates: long` as shorthand, or provide
/// full explicit configuration with field-level overrides.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum DateConfigEntry {
    /// A named preset (e.g., "long", "short", "numeric", "iso").
    Preset(crate::presets::DatePreset),
    /// Explicit date configuration.
    Explicit(DateConfig),
}

impl Default for DateConfigEntry {
    fn default() -> Self {
        DateConfigEntry::Explicit(DateConfig::default())
    }
}

impl DateConfigEntry {
    /// Resolve this entry to a concrete `DateConfig`.
    pub fn resolve(&self) -> DateConfig {
        match self {
            DateConfigEntry::Preset(preset) => preset.config(),
            DateConfigEntry::Explicit(config) => config.clone(),
        }
    }
}

/// Date formatting configuration.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DateConfig {
    pub month: MonthFormat,
    /// Marker for uncertain dates (e.g., "?" or "uncertain"). None suppresses display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncertainty_marker: Option<String>,
    /// Marker for approximate dates (e.g., "ca. " or "~"). None suppresses display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximation_marker: Option<String>,
    /// Delimiter for date ranges (default: en-dash "–").
    #[serde(default = "default_range_delimiter")]
    pub range_delimiter: String,
    /// Marker for open-ended ranges (e.g., "–present"). None uses locale default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_range_marker: Option<String>,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
    /// Time display format. None suppresses time rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_format: Option<TimeFormat>,
    /// Whether to include seconds in time display (default: false).
    #[serde(default)]
    pub show_seconds: bool,
    /// Whether to include timezone in time display (default: false).
    #[serde(default)]
    pub show_timezone: bool,
    /// Era label profile controlling which era suffixes are shown.
    #[serde(default)]
    pub era_labels: EraLabels,
    /// How negative EDTF years with unspecified digits are rendered.
    #[serde(default)]
    pub negative_unspecified_years: NegativeUnspecifiedYears,
}

fn default_range_delimiter() -> String {
    "–".to_string() // U+2013 en-dash
}

impl Default for DateConfig {
    fn default() -> Self {
        Self {
            month: MonthFormat::Long,
            uncertainty_marker: Some("?".to_string()),
            approximation_marker: Some("ca. ".to_string()),
            range_delimiter: default_range_delimiter(),
            open_range_marker: None,
            custom: None,
            time_format: None,
            show_seconds: false,
            show_timezone: false,
            era_labels: EraLabels::default(),
            negative_unspecified_years: NegativeUnspecifiedYears::default(),
        }
    }
}
