//! Processing mode and citation/bibliography rendering options.
//!
//! This module defines the processing modes (author-date, numeric, note, label, custom) that
//! determine how citations and bibliographies are sorted, grouped, and disambiguated. Each
//! mode provides default configurations for sorting and disambiguation strategies.

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::presets::SortPreset;

/// Label style preset conventions.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum LabelPreset {
    /// biblatex alphabetic / BibTeX alpha.bst: up to 4 authors, "+" marker, 2-digit year.
    #[default]
    Alpha,
    /// DIN 1505-2: up to 3 authors, no et-al marker, 2-digit year.
    Din,
    /// American Mathematical Society: same label-generation algorithm as Alpha.
    Ams,
}

/// Resolved label generation parameters after applying preset defaults.
///
/// Stores the resolved (effective) parameters for label citation mode, combining
/// preset defaults with any user-specified overrides from `LabelConfig`.
#[derive(Debug, Clone)]
pub struct LabelParams {
    /// Number of characters from a single author's family name.
    pub single_author_chars: u8,
    /// Number of characters per author when multiple authors are present.
    pub multi_author_chars: u8,
    /// Maximum number of authors before truncation (et-al).
    pub et_al_min: u8,
    /// Suffix to append when authors are truncated (e.g., "+").
    pub et_al_marker: String,
    /// Number of names to show in et-al truncation.
    pub et_al_names: u8,
    /// Number of year digits to use (typically 2 or 4).
    pub year_digits: u8,
}

/// Configuration for label citation mode.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct LabelConfig {
    /// Preset that determines default parameters.
    #[serde(default)]
    pub preset: LabelPreset,
    /// Chars taken from single author's family name. Preset default: 3 (Alpha/Ams), 4 (Din).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_author_chars: Option<u8>,
    /// Chars per author family name when 2+ authors. Preset default: 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_author_chars: Option<u8>,
    /// Max authors before truncation. Alpha/Ams default: 4, Din default: 3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_min: Option<u8>,
    /// Suffix appended when truncated. Alpha/Ams default: "+", Din default: "".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_marker: Option<String>,
    /// Names shown when truncated (et-al). Alpha default: 3, Ams default: 4.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_names: Option<u8>,
    /// Year digits: 2 or 4. Preset default: 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year_digits: Option<u8>,
}

impl LabelConfig {
    /// Resolve effective parameters by merging preset defaults with overrides.
    ///
    /// This method applies the `LabelPreset` defaults first, then applies any user-specified
    /// overrides from optional fields. For example, if the preset is `Alpha` but `single_author_chars`
    /// is specified, the specified value takes precedence over the preset default of 3.
    ///
    /// # Returns
    ///
    /// A `LabelParams` struct with all parameters resolved to concrete values.
    pub fn effective_params(&self) -> LabelParams {
        let (
            default_single_author_chars,
            default_multi_author_chars,
            default_et_al_min,
            default_marker,
            default_et_al_names,
        ) = match self.preset {
            LabelPreset::Alpha => (3u8, 1u8, 4u8, "+".to_string(), 3u8),
            LabelPreset::Ams => (3u8, 1u8, 4u8, "+".to_string(), 4u8),
            LabelPreset::Din => (4u8, 1u8, 3u8, String::new(), 3u8),
        };
        LabelParams {
            single_author_chars: self
                .single_author_chars
                .unwrap_or(default_single_author_chars),
            multi_author_chars: self
                .multi_author_chars
                .unwrap_or(default_multi_author_chars),
            et_al_min: self.et_al_min.unwrap_or(default_et_al_min),
            et_al_marker: self.et_al_marker.clone().unwrap_or(default_marker),
            et_al_names: self.et_al_names.unwrap_or(default_et_al_names),
            year_digits: self.year_digits.unwrap_or(2),
        }
    }
}

/// Processing mode for citation/bibliography generation.
///
/// Determines how citations and bibliographies are sorted, grouped, and disambiguated.
/// Can be specified as a simple string or with complex configuration maps:
/// - A string: `"author-date"`, `"numeric"`, `"note"`, or `"label"`
/// - A label config map: `{ label: { preset: din } }`
/// - A custom config map: `{ sort: ..., group: ..., disambiguate: ... }`
#[derive(Debug, Default, PartialEq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Processing {
    /// Author-date styles (e.g., APA, Chicago).
    /// Default bibliography ordering: author, year, title.
    #[default]
    AuthorDate,
    /// Numeric styles (e.g., IEEE, Nature).
    /// Do not imply a bibliography sort; citations are numbered in order of appearance.
    Numeric,
    /// Note styles (e.g., Chicago Notes-Bibliography).
    /// With a bibliography default to author, title, year ordering.
    Note,
    /// Label styles (e.g., Alpha, DIN 1505-2).
    /// Default bibliography ordering: author, year, title.
    Label(LabelConfig),
    /// Fully custom processing behavior.
    /// Explicit `sort` configuration remains authoritative.
    Custom(ProcessingCustom),
}

/// How citation-item sorting is resolved when `citation.sort` is absent.
///
/// Determines whether citation clusters can be reordered automatically or only
/// when explicitly configured.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum CitationSortPolicy {
    /// Only an explicit `citation.sort` can reorder multi-cite clusters.
    ExplicitOnly,
}

/// Custom processing configuration.
///
/// Allows explicit specification of sorting, grouping, and disambiguation rules
/// without relying on preset defaults.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct ProcessingCustom {
    /// Bibliography sorting configuration (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<SortEntry>,
    /// Bibliography grouping configuration (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<Group>,
    /// Disambiguation settings (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disambiguate: Option<Disambiguation>,
}

impl Processing {
    /// Default bibliography sort for the processing family, if any.
    ///
    /// Returns the standard bibliography sort order for the processing mode:
    /// - `AuthorDate` / `Label`: author, year, title
    /// - `Note`: author, title, year
    /// - `Numeric` / `Custom`: None (no automatic sort)
    pub fn default_bibliography_sort(&self) -> Option<SortPreset> {
        match self {
            Processing::AuthorDate => Some(SortPreset::AuthorDateTitle),
            Processing::Numeric => None,
            Processing::Note => Some(SortPreset::AuthorTitleDate),
            Processing::Label(_) => Some(SortPreset::AuthorDateTitle),
            Processing::Custom(_) => None,
        }
    }

    /// Citation sorting remains explicit-only for all processing families.
    ///
    /// All processing modes use `ExplicitOnly`, meaning citation clusters are only
    /// reordered when explicitly configured via `citation.sort`.
    pub fn default_citation_sort_policy(&self) -> CitationSortPolicy {
        CitationSortPolicy::ExplicitOnly
    }

    /// Get the effective bibliography/disambiguation configuration for this processing mode.
    ///
    /// Returns a `ProcessingCustom` struct with the resolved configuration combining
    /// preset defaults and user overrides. For `Custom` mode, returns the user-provided config as-is.
    pub fn config(&self) -> ProcessingCustom {
        match self {
            Processing::AuthorDate => ProcessingCustom {
                sort: Some(SortEntry::Preset(SortPreset::AuthorDateTitle)),
                group: Some(Group {
                    template: vec![SortKey::Author, SortKey::Year],
                }),
                disambiguate: Some(Disambiguation {
                    names: true,
                    add_givenname: true,
                    year_suffix: true,
                }),
            },
            Processing::Numeric => ProcessingCustom {
                sort: None,
                group: None,
                disambiguate: None,
            },
            Processing::Note => ProcessingCustom {
                sort: Some(SortEntry::Preset(SortPreset::AuthorTitleDate)),
                group: None,
                disambiguate: Some(Disambiguation {
                    names: true,
                    add_givenname: false,
                    year_suffix: false,
                }),
            },
            Processing::Label(_) => ProcessingCustom {
                sort: Some(SortEntry::Preset(SortPreset::AuthorDateTitle)),
                group: None,
                disambiguate: Some(Disambiguation {
                    names: false,
                    add_givenname: false,
                    year_suffix: true,
                }),
            },
            Processing::Custom(custom) => custom.clone(),
        }
    }
}

impl<'de> Deserialize<'de> for Processing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct ProcessingVisitor;

        impl<'de> Visitor<'de> for ProcessingVisitor {
            type Value = Processing;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a processing mode string or map")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Processing, E> {
                match v {
                    "author-date" => Ok(Processing::AuthorDate),
                    "numeric" => Ok(Processing::Numeric),
                    "note" => Ok(Processing::Note),
                    "label" => Ok(Processing::Label(LabelConfig::default())),
                    other => Err(E::unknown_variant(
                        other,
                        &["author-date", "numeric", "note", "label"],
                    )),
                }
            }

            fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<Processing, A::Error> {
                use serde::de::VariantAccess;
                let (variant, access) = data.variant::<String>()?;
                match variant.as_str() {
                    "custom" => {
                        let custom: ProcessingCustom = access.newtype_variant()?;
                        Ok(Processing::Custom(custom))
                    }
                    other => Err(de::Error::unknown_variant(
                        other,
                        &["author-date", "numeric", "note", "label", "custom"],
                    )),
                }
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Processing, A::Error> {
                let key: String = map
                    .next_key()?
                    .ok_or_else(|| de::Error::invalid_length(0, &"1"))?;
                match key.as_str() {
                    "label" => {
                        let config: LabelConfig = map.next_value()?;
                        Ok(Processing::Label(config))
                    }
                    "sort" | "group" | "disambiguate" => {
                        // This is a custom processing config
                        // We need to deserialize the whole map as ProcessingCustom
                        // Unfortunately we can't easily re-parse from the middle of map access.
                        // Instead, collect fields and build manually
                        let mut sort = None;
                        let mut group = None;
                        let mut disambiguate = None;

                        // Handle the first key we already read
                        match key.as_str() {
                            "sort" => sort = Some(map.next_value()?),
                            "group" => group = Some(map.next_value()?),
                            "disambiguate" => disambiguate = Some(map.next_value()?),
                            _ => {
                                return Err(de::Error::unknown_field(
                                    &key,
                                    &["sort", "group", "disambiguate"],
                                ));
                            }
                        }

                        // Read remaining keys
                        while let Some(k) = map.next_key::<String>()? {
                            match k.as_str() {
                                "sort" => sort = Some(map.next_value()?),
                                "group" => group = Some(map.next_value()?),
                                "disambiguate" => disambiguate = Some(map.next_value()?),
                                other => {
                                    return Err(de::Error::unknown_field(
                                        other,
                                        &["sort", "group", "disambiguate"],
                                    ));
                                }
                            }
                        }

                        Ok(Processing::Custom(ProcessingCustom {
                            sort,
                            group,
                            disambiguate,
                        }))
                    }
                    other => Err(de::Error::unknown_field(
                        other,
                        &["label", "sort", "group", "disambiguate"],
                    )),
                }
            }
        }

        deserializer.deserialize_any(ProcessingVisitor)
    }
}

/// Disambiguation settings.
///
/// Controls how ambiguous citations are disambiguated in the output.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Disambiguation {
    /// Whether to attempt disambiguation by expanding author names.
    pub names: bool,
    /// Whether to add given names to disambiguate similarly-named authors.
    #[serde(default)]
    pub add_givenname: bool,
    /// Whether to append year suffixes (a, b, c, ...) for multiple works from the same author-year.
    pub year_suffix: bool,
}

impl Default for Disambiguation {
    fn default() -> Self {
        Self {
            names: true,
            add_givenname: false,
            year_suffix: false,
        }
    }
}

/// Sorting configuration.
///
/// Specifies how bibliography entries are ordered.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Sort {
    /// Whether to shorten name lists for sorting the same as for display.
    #[serde(default)]
    pub shorten_names: bool,
    /// Whether to apply the same name substitutions during sorting as during rendering.
    #[serde(default)]
    pub render_substitutions: bool,
    /// Sort keys in order of application.
    pub template: Vec<SortSpec>,
}

/// Sort configuration: either a preset name or explicit configuration.
///
/// Can be a preset name like `author-date-title` or a full `Sort` struct with explicit settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum SortEntry {
    /// A named sort preset (e.g., `author-date-title`, `author-title-date`).
    Preset(crate::presets::SortPreset),
    /// Explicit sort configuration with custom keys and order.
    Explicit(Sort),
}

impl SortEntry {
    /// Resolve this entry to a concrete `Sort`.
    ///
    /// If this is a preset, returns the preset's sort definition. Otherwise returns the explicit sort as-is.
    pub fn resolve(&self) -> Sort {
        match self {
            SortEntry::Preset(preset) => preset.sort(),
            SortEntry::Explicit(sort) => sort.clone(),
        }
    }
}

/// A single sort specification.
///
/// Defines one sort dimension with its key and direction.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct SortSpec {
    /// The field to sort by.
    pub key: SortKey,
    /// Whether to sort in ascending order (default: true).
    #[serde(default = "default_ascending")]
    pub ascending: bool,
}

fn default_ascending() -> bool {
    true
}

/// Available sort keys.
///
/// Specifies what field to sort bibliography entries by.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SortKey {
    /// Sort by the work's author(s).
    #[default]
    Author,
    /// Sort by publication year.
    Year,
    /// Sort by the work's title.
    Title,
    /// Sort by citation order (typically used for numeric styles).
    CitationNumber,
}

/// Grouping configuration for bibliography.
///
/// Specifies how bibliography entries should be grouped in the output.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Group {
    /// Sort keys used to define group boundaries (e.g., [Author, Year]).
    pub template: Vec<SortKey>,
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

    /// Test that LabelConfig::effective_params() applies Alpha preset defaults.
    #[test]
    fn test_label_config_alpha_preset_defaults() {
        let config = LabelConfig {
            preset: LabelPreset::Alpha,
            single_author_chars: None,
            multi_author_chars: None,
            et_al_min: None,
            et_al_marker: None,
            et_al_names: None,
            year_digits: None,
        };

        let params = config.effective_params();
        assert_eq!(params.single_author_chars, 3);
        assert_eq!(params.multi_author_chars, 1);
        assert_eq!(params.et_al_min, 4);
        assert_eq!(params.et_al_marker, "+");
        assert_eq!(params.et_al_names, 3);
        assert_eq!(params.year_digits, 2);
    }

    /// Test that LabelConfig overrides take precedence over preset defaults.
    #[test]
    fn test_label_config_alpha_with_overrides() {
        let config = LabelConfig {
            preset: LabelPreset::Alpha,
            single_author_chars: Some(5),
            multi_author_chars: Some(2),
            et_al_min: Some(5),
            et_al_marker: Some("*".to_string()),
            et_al_names: Some(4),
            year_digits: Some(4),
        };

        let params = config.effective_params();
        assert_eq!(params.single_author_chars, 5);
        assert_eq!(params.multi_author_chars, 2);
        assert_eq!(params.et_al_min, 5);
        assert_eq!(params.et_al_marker, "*");
        assert_eq!(params.et_al_names, 4);
        assert_eq!(params.year_digits, 4);
    }

    /// Test that LabelConfig::effective_params() applies Din preset defaults.
    #[test]
    fn test_label_config_din_preset_defaults() {
        let config = LabelConfig {
            preset: LabelPreset::Din,
            single_author_chars: None,
            multi_author_chars: None,
            et_al_min: None,
            et_al_marker: None,
            et_al_names: None,
            year_digits: None,
        };

        let params = config.effective_params();
        assert_eq!(params.single_author_chars, 4);
        assert_eq!(params.multi_author_chars, 1);
        assert_eq!(params.et_al_min, 3);
        assert_eq!(params.et_al_marker, "");
        assert_eq!(params.et_al_names, 3);
        assert_eq!(params.year_digits, 2);
    }

    /// Test that Processing::AuthorDate returns correct default sort.
    #[test]
    fn test_processing_author_date_default_bibliography_sort() {
        let processing = Processing::AuthorDate;
        let sort = processing.default_bibliography_sort();
        assert_eq!(sort, Some(SortPreset::AuthorDateTitle));
    }

    /// Test that Processing::Numeric returns no default sort.
    #[test]
    fn test_processing_numeric_default_bibliography_sort() {
        let processing = Processing::Numeric;
        let sort = processing.default_bibliography_sort();
        assert_eq!(sort, None);
    }

    /// Test that Processing::Note returns correct default sort.
    #[test]
    fn test_processing_note_default_bibliography_sort() {
        let processing = Processing::Note;
        let sort = processing.default_bibliography_sort();
        assert_eq!(sort, Some(SortPreset::AuthorTitleDate));
    }

    /// Test that all Processing modes return ExplicitOnly citation sort policy.
    #[test]
    fn test_processing_citation_sort_policy() {
        let modes = vec![
            Processing::AuthorDate,
            Processing::Numeric,
            Processing::Note,
            Processing::Label(LabelConfig::default()),
            Processing::Custom(ProcessingCustom::default()),
        ];

        for mode in modes {
            assert_eq!(
                mode.default_citation_sort_policy(),
                CitationSortPolicy::ExplicitOnly
            );
        }
    }

    /// Test that Processing::config() returns correct configuration for AuthorDate.
    #[test]
    fn test_processing_author_date_config() {
        let processing = Processing::AuthorDate;
        let config = processing.config();

        assert!(config.sort.is_some());
        assert!(config.group.is_some());
        assert!(config.disambiguate.is_some());

        let disambig = config.disambiguate.unwrap();
        assert!(disambig.names);
        assert!(disambig.add_givenname);
        assert!(disambig.year_suffix);
    }

    /// Test that Disambiguation defaults have correct values.
    #[test]
    fn test_disambiguation_defaults() {
        let disambig = Disambiguation::default();
        assert!(disambig.names);
        assert!(!disambig.add_givenname);
        assert!(!disambig.year_suffix);
    }

    /// Test that SortEntry::resolve() returns preset sort for Preset variant.
    #[test]
    fn test_sort_entry_resolve_preset() {
        let entry = SortEntry::Preset(SortPreset::AuthorDateTitle);
        let sort = entry.resolve();

        // Verify it resolves to a valid Sort
        assert!(!sort.template.is_empty());
    }

    /// Test that SortEntry::resolve() returns explicit sort for Explicit variant.
    #[test]
    fn test_sort_entry_resolve_explicit() {
        let explicit = Sort {
            shorten_names: true,
            render_substitutions: false,
            template: vec![SortSpec {
                key: SortKey::Title,
                ascending: false,
            }],
        };
        let entry = SortEntry::Explicit(explicit.clone());
        let resolved = entry.resolve();

        assert!(resolved.shorten_names);
        assert!(!resolved.render_substitutions);
        assert_eq!(resolved.template.len(), 1);
        assert_eq!(resolved.template[0].key, SortKey::Title);
        assert!(!resolved.template[0].ascending);
    }
}
