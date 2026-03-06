/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bibliography-specific configuration.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct BibliographyConfig {
    /// String to substitute for repeating authors (e.g., "———").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_author_substitute: Option<String>,
    /// Rule for when to apply the substitute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_author_substitute_rule: Option<SubsequentAuthorSubstituteRule>,
    /// Whether to use a hanging indent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hanging_indent: Option<bool>,
    /// Suffix appended to each bibliography entry (e.g., ".").
    /// Extracted from CSL 1.0 `<layout suffix=".">` attribute.
    /// If None, a trailing period is added by default unless entry ends with DOI/URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_suffix: Option<String>,
    /// Separator between bibliography components (e.g., ". " for Chicago/APA, ", " for Elsevier).
    /// Extracted from CSL 1.0 group delimiter attribute.
    /// Defaults to ". " if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    /// Whether to suppress the trailing period after URLs/DOIs.
    /// Default behavior is to add a period (Chicago, MLA style).
    /// Set to true to suppress the period (APA 7th, Bluebook style).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub suppress_period_after_url: bool,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
    /// Configuration for compound numeric bibliography entries.
    /// When present, enables grouping of references by input bibliography `sets`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compound_numeric: Option<CompoundNumericConfig>,
}

/// Rules for subsequent author substitution.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum SubsequentAuthorSubstituteRule {
    /// Substitute only if ALL authors match.
    #[default]
    CompleteAll,
    /// Substitute each matching name individually.
    CompleteEach,
    /// Substitute each matching name until the first mismatch.
    PartialEach,
    /// Substitute only the first name if it matches.
    PartialFirst,
}

/// Sub-label style for compound numeric bibliography entries.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum SubLabelStyle {
    /// Alphabetic sub-labels: a, b, c, ...
    #[default]
    Alphabetic,
    /// Numeric sub-labels: 1, 2, 3, ...
    Numeric,
}

/// Default sub-label suffix.
fn default_sub_label_suffix() -> String {
    ")".to_string()
}

/// Default sub-item delimiter.
fn default_sub_delimiter() -> String {
    ", ".to_string()
}

/// Default subentry citation behavior.
fn default_subentry() -> bool {
    true
}

/// Default compound subentry collapse behavior.
fn default_collapse_subentries() -> bool {
    false
}

/// Configuration for compound numeric bibliography entries.
///
/// Groups multiple references under a single citation number with sub-labels.
/// Used in chemistry journals (e.g., Angewandte Chemie).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CompoundNumericConfig {
    /// Whether grouped item citations render sub-entry labels (`1a`, `1b`).
    ///
    /// When false, grouped item citations render the whole-group number (`1`).
    #[serde(default = "default_subentry")]
    pub subentry: bool,
    /// Whether adjacent grouped sub-entries collapse in citations.
    ///
    /// When true, adjacent members from the same group may render as
    /// `1a,b` or `1a-c` instead of `1a,1b` or `1a,1b,1c`.
    #[serde(default = "default_collapse_subentries")]
    pub collapse_subentries: bool,
    /// Sub-label style: alphabetic (a, b, c) or numeric (1, 2, 3).
    #[serde(default)]
    pub sub_label: SubLabelStyle,
    /// Suffix after sub-label (e.g., ")" → "a)", "." → "a.").
    #[serde(default = "default_sub_label_suffix")]
    pub sub_label_suffix: String,
    /// Delimiter between sub-items (default: ", ").
    #[serde(default = "default_sub_delimiter")]
    pub sub_delimiter: String,
}

impl Default for CompoundNumericConfig {
    fn default() -> Self {
        Self {
            subentry: default_subentry(),
            collapse_subentries: default_collapse_subentries(),
            sub_label: SubLabelStyle::default(),
            sub_label_suffix: default_sub_label_suffix(),
            sub_delimiter: default_sub_delimiter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_numeric_config_defaults() {
        let config: CompoundNumericConfig = serde_json::from_str("{}").unwrap();
        assert!(config.subentry);
        assert!(!config.collapse_subentries);
        assert_eq!(config.sub_label, SubLabelStyle::Alphabetic);
        assert_eq!(config.sub_label_suffix, ")");
        assert_eq!(config.sub_delimiter, ", ");
    }

    #[test]
    fn test_compound_numeric_config_custom() {
        let json = r#"{"subentry": false, "collapse-subentries": true, "sub-label": "numeric", "sub-label-suffix": ".", "sub-delimiter": "; "}"#;
        let config: CompoundNumericConfig = serde_json::from_str(json).unwrap();
        assert!(!config.subentry);
        assert!(config.collapse_subentries);
        assert_eq!(config.sub_label, SubLabelStyle::Numeric);
        assert_eq!(config.sub_label_suffix, ".");
        assert_eq!(config.sub_delimiter, "; ");
    }

    #[test]
    fn test_compound_numeric_roundtrip() {
        let config = CompoundNumericConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CompoundNumericConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_bibliography_config_with_compound() {
        let json = r#"{"compound-numeric": {"sub-label": "alphabetic"}}"#;
        let config: BibliographyConfig = serde_json::from_str(json).unwrap();
        assert!(config.compound_numeric.is_some());
    }
}
