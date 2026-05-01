/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::locale::{GeneralTerm, TermForm};

/// Bibliography-specific configuration.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct BibliographyConfig {
    /// Article-journal-specific bibliography policies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_journal: Option<ArticleJournalBibliographyConfig>,
    /// String to substitute for repeating authors (e.g., "———").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_author_substitute: Option<String>,
    /// Rule for when to apply the substitute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_author_substitute_rule: Option<SubsequentAuthorSubstituteRule>,
    /// Whether to use a hanging indent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hanging_indent: Option<bool>,
    /// Suffix appended to each bibliography entry (e.g., `"."`).
    /// Extracted from CSL 1.0 `<layout suffix=".">` attribute.
    /// If `None`, no suffix is appended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_suffix: Option<String>,
    /// Separator between bibliography components (e.g., `". "` for Chicago/APA, `", "` for Elsevier).
    /// Extracted from CSL 1.0 group delimiter attribute.
    /// Defaults to `". "`.
    #[serde(
        default = "default_separator",
        skip_serializing_if = "is_default_separator"
    )]
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
    /// Partitioning policy for multilingual bibliography sorting and sections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_partitioning: Option<BibliographySortPartitioning>,
}

/// Article-journal-specific bibliography configuration.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ArticleJournalBibliographyConfig {
    /// Fallback policy used when page data is absent from an article-journal reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_page_fallback: Option<ArticleJournalNoPageFallback>,
}

/// Named fallback policies for page-less article-journal bibliography entries.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ArticleJournalNoPageFallback {
    /// Replace the standard article detail block with the DOI component.
    Doi,
}

/// Bibliography partitioning policy for multilingual sort order and sections.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct BibliographySortPartitioning {
    /// Source used to derive each reference's partition key.
    pub by: BibliographyPartitionKind,
    /// Whether partitioning affects flat sorting, visible sections, or both.
    #[serde(default)]
    pub mode: BibliographyPartitionMode,
    /// Preferred partition order. Unlisted partitions sort after listed ones.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<String>,
    /// Optional headings for visible partition sections.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headings: HashMap<String, BibliographyPartitionHeading>,
}

/// Localizable heading source for automatic bibliography partition sections.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", untagged)]
pub enum BibliographyPartitionHeading {
    /// Fixed literal heading text.
    Literal {
        /// Literal heading value.
        literal: String,
    },
    /// Locale general term key resolved at render time.
    Term {
        /// Locale general term key.
        term: GeneralTerm,
        /// Optional term form (defaults to long).
        #[serde(skip_serializing_if = "Option::is_none")]
        form: Option<TermForm>,
    },
    /// Locale-indexed heading map.
    Localized {
        /// Map keyed by BCP 47 locale identifiers or language tags.
        localized: HashMap<String, String>,
    },
}

/// Partition key source for bibliography partitioning.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum BibliographyPartitionKind {
    /// Partition by Unicode script detected from author/editor/title sort text.
    Script,
    /// Partition by the reference's effective item language.
    Language,
}

/// Rendering mode for bibliography partitioning.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum BibliographyPartitionMode {
    /// Sort a flat bibliography by partition before normal sort keys.
    #[default]
    SortOnly,
    /// Render visible sections for grouped bibliography output only.
    Sections,
    /// Sort flat output by partition and render visible sections for grouped output.
    SortAndSections,
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

/// Default bibliography component separator.
pub(crate) fn default_separator() -> Option<String> {
    Some(". ".to_string())
}

/// Skip serializing separator when it is the default value.
pub(crate) fn is_default_separator(v: &Option<String>) -> bool {
    v.as_deref() == Some(". ")
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

impl Default for BibliographyConfig {
    fn default() -> Self {
        Self {
            article_journal: None,
            subsequent_author_substitute: None,
            subsequent_author_substitute_rule: None,
            hanging_indent: None,
            entry_suffix: None,
            separator: default_separator(),
            suppress_period_after_url: false,
            custom: None,
            compound_numeric: None,
            sort_partitioning: None,
        }
    }
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

    #[test]
    fn test_article_journal_no_page_fallback_deserializes() {
        let json = r#"{"article-journal":{"no-page-fallback":"doi"}}"#;
        let config: BibliographyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.article_journal.and_then(|cfg| cfg.no_page_fallback),
            Some(ArticleJournalNoPageFallback::Doi)
        );
    }

    #[test]
    fn test_article_journal_no_page_fallback_roundtrip() {
        let config = BibliographyConfig {
            article_journal: Some(ArticleJournalBibliographyConfig {
                no_page_fallback: Some(ArticleJournalNoPageFallback::Doi),
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BibliographyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_sort_partitioning_deserializes_script_sort_only() {
        let json = r#"{
            "sort-partitioning": {
                "by": "script",
                "mode": "sort-only",
                "order": ["Cyrl", "Latn"],
                "headings": {
                    "Cyrl": {"literal": "Cyrillic"}
                }
            }
        }"#;
        let config: BibliographyConfig = serde_json::from_str(json).unwrap();
        let partitioning = config
            .sort_partitioning
            .expect("partitioning should deserialize");

        assert_eq!(partitioning.by, BibliographyPartitionKind::Script);
        assert_eq!(partitioning.mode, BibliographyPartitionMode::SortOnly);
        assert_eq!(
            partitioning.order,
            vec!["Cyrl".to_string(), "Latn".to_string()]
        );
        assert_eq!(
            partitioning.headings.get("Cyrl"),
            Some(&BibliographyPartitionHeading::Literal {
                literal: "Cyrillic".to_string()
            })
        );
    }

    #[test]
    fn test_sort_partitioning_defaults_to_sort_only() {
        let json = r#"{"sort-partitioning": {"by": "language"}}"#;
        let config: BibliographyConfig = serde_json::from_str(json).unwrap();
        let partitioning = config
            .sort_partitioning
            .expect("partitioning should deserialize");

        assert_eq!(partitioning.by, BibliographyPartitionKind::Language);
        assert_eq!(partitioning.mode, BibliographyPartitionMode::SortOnly);
        assert!(partitioning.order.is_empty());
        assert!(partitioning.headings.is_empty());
    }

    #[test]
    fn test_sort_partitioning_roundtrip() {
        let mut headings = HashMap::new();
        headings.insert(
            "Latn".to_string(),
            BibliographyPartitionHeading::Literal {
                literal: "Latin".to_string(),
            },
        );
        let config = BibliographyConfig {
            sort_partitioning: Some(BibliographySortPartitioning {
                by: BibliographyPartitionKind::Script,
                mode: BibliographyPartitionMode::SortAndSections,
                order: vec!["Latn".to_string()],
                headings,
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BibliographyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_sort_partitioning_rejects_unknown_fields() {
        let json = r#"{"sort-partitioning": {"by": "script", "unknown": true}}"#;
        let error = serde_json::from_str::<BibliographyConfig>(json)
            .expect_err("unknown partitioning fields should be rejected");

        assert!(
            error.to_string().contains("unknown field"),
            "unexpected error: {error}"
        );
    }
}
