/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Style-level preset architecture (Level 2).
//!
//! This module provides [`StylePreset`], [`StyleVariantDelta`], and
//! [`StylePresetSpec`] — the building blocks for naming well-known compiled-in
//! styles and expressing behavioral dependents (e.g. Turabian) as compact
//! variant-delta overlays rather than standalone YAML files.

use crate::embedded::get_embedded_style;
use crate::options::Config;
use crate::{BibliographySpec, CitationSpec, Style, merge_optional_serialized};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::de::Deserializer;
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::collections::{HashMap, HashSet};

const APA_7TH_BASE: &str = "preset-bases/apa-7th";
const CHICAGO_AUTHOR_DATE_18TH_BASE: &str = "preset-bases/chicago-author-date-18th";
const CHICAGO_NOTES_18TH_BASE: &str = "preset-bases/chicago-notes-18th";

/// A named, compiled-in style preset (Level 2 of Citum's preset hierarchy).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum StylePreset {
    /// Chicago Manual of Style 18th edition — notes without bibliography.
    #[serde(rename = "chicago-notes-18th")]
    ChicagoNotes18th,
    /// Chicago Manual of Style 18th edition — author-date system.
    #[serde(rename = "chicago-author-date-18th")]
    ChicagoAuthorDate18th,
    /// APA 7th edition — author-date system.
    #[serde(rename = "apa-7th")]
    Apa7th,
}

impl StylePreset {
    /// Return the embedded YAML key used to look up this preset.
    fn embedded_key(&self) -> &'static str {
        match self {
            StylePreset::ChicagoNotes18th => CHICAGO_NOTES_18TH_BASE,
            StylePreset::ChicagoAuthorDate18th => CHICAGO_AUTHOR_DATE_18TH_BASE,
            StylePreset::Apa7th => APA_7TH_BASE,
        }
    }

    /// Return the base [`Style`] for this preset.
    ///
    /// # Panics
    ///
    /// Panics if the embedded YAML is missing or malformed.
    pub fn base(&self) -> Style {
        let key = self.embedded_key();
        get_embedded_style(key)
            .unwrap_or_else(|| panic!("StylePreset: missing embedded style for key '{key}'"))
            .unwrap_or_else(|e| panic!("StylePreset: malformed embedded YAML for key '{key}': {e}"))
    }

    /// Return the canonical preset key string (kebab-case).
    pub fn key(&self) -> &'static str {
        match self {
            StylePreset::ChicagoNotes18th => "chicago-notes-18th",
            StylePreset::ChicagoAuthorDate18th => "chicago-author-date-18th",
            StylePreset::Apa7th => "apa-7th",
        }
    }

    /// Return all known preset variants.
    ///
    /// Prefer this over exhaustive `match` when iterating the registry, since
    /// [`StylePreset`] is `#[non_exhaustive]`.
    pub fn all() -> &'static [StylePreset] {
        &[
            StylePreset::ChicagoNotes18th,
            StylePreset::ChicagoAuthorDate18th,
            StylePreset::Apa7th,
        ]
    }

    /// Resolve the final [`Style`] by merging an optional [`StyleVariantDelta`]
    /// over the base preset.
    pub fn resolve(&self, delta: Option<&StyleVariantDelta>) -> Style {
        self.resolve_with_visited(&mut HashSet::new(), delta)
    }

    /// Internal recursive resolver with loop protection.
    pub(crate) fn resolve_with_visited(
        &self,
        visited: &mut HashSet<StylePreset>,
        delta: Option<&StyleVariantDelta>,
    ) -> Style {
        let mut style = self.base();

        if style.preset.is_some() {
            style = style.into_resolved_recursive(visited);
        }

        if let Some(d) = delta {
            d.apply_to(&mut style);
        }
        style
    }
}

/// A partial overlay applied over a named preset-backed base style.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct StyleVariantDelta {
    /// Overrides the base preset's global options block.
    pub options: Option<Config>,
    /// Overrides the base preset's citation block.
    pub citation: Option<CitationSpec>,
    /// Overrides the base preset's bibliography block.
    pub bibliography: Option<BibliographySpec>,
    /// Forward-compatible escape hatch for future variant fields not yet in
    /// the schema.
    pub custom: Option<HashMap<String, JsonValue>>,
    // Raw YAML values are retained alongside the typed fields so that
    // null-clear semantics (`ibid: ~`) survive serde round-trips.
    // `apply_to` uses the raw path when available; the typed path is a
    // fallback for programmatically-constructed deltas.
    #[cfg_attr(feature = "schema", schemars(skip))]
    raw_options: Option<YamlValue>,
    #[cfg_attr(feature = "schema", schemars(skip))]
    raw_citation: Option<YamlValue>,
    #[cfg_attr(feature = "schema", schemars(skip))]
    raw_bibliography: Option<YamlValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawStyleVariantDelta {
    #[serde(default)]
    options: Option<YamlValue>,
    #[serde(default)]
    citation: Option<YamlValue>,
    #[serde(default)]
    bibliography: Option<YamlValue>,
    #[serde(default)]
    custom: Option<HashMap<String, JsonValue>>,
}

impl<'de> Deserialize<'de> for StyleVariantDelta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawStyleVariantDelta::deserialize(deserializer)?;
        let options = raw
            .options
            .as_ref()
            .and_then(|value| (!matches!(value, YamlValue::Null)).then_some(value))
            .map(|value| serde_yaml::from_value(value.clone()))
            .transpose()
            .map_err(serde::de::Error::custom)?;
        let citation = raw
            .citation
            .as_ref()
            .and_then(|value| (!matches!(value, YamlValue::Null)).then_some(value))
            .map(|value| serde_yaml::from_value(value.clone()))
            .transpose()
            .map_err(serde::de::Error::custom)?;
        let bibliography = raw
            .bibliography
            .as_ref()
            .and_then(|value| (!matches!(value, YamlValue::Null)).then_some(value))
            .map(|value| serde_yaml::from_value(value.clone()))
            .transpose()
            .map_err(serde::de::Error::custom)?;

        Ok(Self {
            options,
            citation,
            bibliography,
            custom: raw.custom,
            raw_options: raw.options,
            raw_citation: raw.citation,
            raw_bibliography: raw.bibliography,
        })
    }
}

impl Serialize for StyleVariantDelta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        if let Some(value) = &self.raw_options {
            map.serialize_entry("options", value)?;
        } else if let Some(value) = &self.options {
            map.serialize_entry("options", value)?;
        }

        if let Some(value) = &self.raw_citation {
            map.serialize_entry("citation", value)?;
        } else if let Some(value) = &self.citation {
            map.serialize_entry("citation", value)?;
        }

        if let Some(value) = &self.raw_bibliography {
            map.serialize_entry("bibliography", value)?;
        } else if let Some(value) = &self.bibliography {
            map.serialize_entry("bibliography", value)?;
        }

        if let Some(value) = &self.custom {
            map.serialize_entry("custom", value)?;
        }

        map.end()
    }
}

impl StyleVariantDelta {
    /// Apply this delta onto a mutable [`Style`] in place.
    pub fn apply_to(&self, style: &mut Style) {
        if let Some(raw_options) = &self.raw_options {
            merge_optional_serialized(&mut style.options, raw_options);
        } else if let Some(options) = &self.options {
            match &mut style.options {
                Some(existing) => existing.merge(options),
                None => style.options = Some(options.clone()),
            }
        }

        if let Some(raw_citation) = &self.raw_citation {
            merge_optional_serialized(&mut style.citation, raw_citation);
        } else if let Some(citation) = &self.citation {
            style.citation = Some(match &style.citation {
                Some(existing) => crate::merge_serialized(existing.clone(), citation),
                None => citation.clone(),
            });
        }

        if let Some(raw_bibliography) = &self.raw_bibliography {
            merge_optional_serialized(&mut style.bibliography, raw_bibliography);
        } else if let Some(bibliography) = &self.bibliography {
            style.bibliography = Some(match &style.bibliography {
                Some(existing) => crate::merge_serialized(existing.clone(), bibliography),
                None => bibliography.clone(),
            });
        }

        if let Some(custom) = &self.custom {
            style.custom = Some(match &style.custom {
                Some(existing) => crate::merge_serialized(existing.clone(), custom),
                None => custom.clone(),
            });
        }
    }

    /// Return a [`StyleVariantDelta`] expressing the Turabian 9th edition
    /// deviations from Chicago Notes 18th.
    ///
    /// # Panics
    ///
    /// Panics if the embedded Turabian citation delta YAML literal becomes invalid.
    #[doc(hidden)]
    pub fn turabian() -> Self {
        use crate::NoteStartTextCase;

        let raw_citation: YamlValue =
            serde_yaml::from_str("ibid: ~\nnote-start-text-case: capitalize-first\n")
                .expect("valid Turabian citation delta");

        StyleVariantDelta {
            citation: Some(CitationSpec {
                ibid: None,
                note_start_text_case: Some(NoteStartTextCase::CapitalizeFirst),
                ..Default::default()
            }),
            raw_citation: Some(raw_citation),
            ..Default::default()
        }
    }
}

/// Top-level style preset reference with an optional variant delta overlay.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", untagged)]
pub enum StylePresetSpec {
    /// A simple named style preset without a variant delta.
    Key(StylePreset),

    /// A named style preset with an optional behavioral variant delta.
    Full {
        /// The named style preset to use as the base.
        preset: StylePreset,

        /// Optional behavioral variant delta merged over the base preset.
        #[serde(skip_serializing_if = "Option::is_none")]
        variant: Option<Box<StyleVariantDelta>>,
    },
}

impl StylePresetSpec {
    /// Return the underlying [`StylePreset`].
    #[must_use]
    pub fn preset(&self) -> &StylePreset {
        match self {
            Self::Key(p) => p,
            Self::Full { preset, .. } => preset,
        }
    }

    /// Returns the variant delta, if any.
    #[must_use]
    pub fn variant(&self) -> Option<&StyleVariantDelta> {
        match self {
            Self::Key(_) => None,
            Self::Full { variant, .. } => variant.as_deref(),
        }
    }

    /// Resolve this spec to a complete [`Style`].
    pub fn resolve(&self) -> Style {
        self.resolve_with_visited(&mut HashSet::new())
    }

    /// Resolve this spec to a complete [`Style`] using a shared visited set.
    pub(crate) fn resolve_with_visited(&self, visited: &mut HashSet<StylePreset>) -> Style {
        self.preset().resolve_with_visited(visited, self.variant())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::PageRangeFormat;

    #[test]
    fn style_preset_chicago_notes_base_is_valid() {
        let style = StylePreset::ChicagoNotes18th.base();
        let yaml = serde_yaml::to_string(&style).expect("serialization failed");
        let back: Style = serde_yaml::from_str(&yaml).expect("deserialization failed");
        assert!(back.info.title.is_some(), "title should be present");
        assert!(
            back.citation
                .as_ref()
                .and_then(|citation| citation.ibid.as_ref())
                .is_some()
        );
    }

    #[test]
    fn style_preset_chicago_author_date_base_is_valid() {
        let style = StylePreset::ChicagoAuthorDate18th.base();
        assert!(style.info.title.is_some(), "title should be present");
    }

    #[test]
    fn style_preset_apa_7th_base_is_valid() {
        let style = StylePreset::Apa7th.base();
        assert!(style.info.title.is_some(), "title should be present");
    }

    #[test]
    fn turabian_variant_delta_disables_ibid() {
        let delta = StyleVariantDelta::turabian();
        let style = StylePreset::ChicagoNotes18th.resolve(Some(&delta));
        let citation = style.citation.expect("citation should be present");
        assert!(
            citation.ibid.is_none(),
            "Turabian variant should disable ibid"
        );
        assert!(
            citation.template.is_some(),
            "variant merge should preserve the base template"
        );
    }

    #[test]
    fn style_variant_delta_custom_roundtrips() {
        let yaml = r#"
custom:
  student-paper:
    title-page: true
    institution: "University of Chicago"
"#;
        let delta: StyleVariantDelta = serde_yaml::from_str(yaml).expect("deserialization failed");
        let custom = delta.custom.as_ref().expect("custom should be present");
        assert!(custom.contains_key("student-paper"));

        let back_yaml = serde_yaml::to_string(&delta).expect("serialization failed");
        let back: StyleVariantDelta =
            serde_yaml::from_str(&back_yaml).expect("deserialization failed");
        assert!(back.custom.is_some(), "custom should survive round-trip");
    }

    #[test]
    fn style_preset_spec_resolves() {
        let spec = StylePresetSpec::Full {
            preset: StylePreset::ChicagoNotes18th,
            variant: Some(Box::new(StyleVariantDelta::turabian())),
        };
        let style = spec.resolve();
        let citation = style.citation.expect("citation should be present");
        assert!(
            citation.ibid.is_none(),
            "Turabian preset spec should disable ibid"
        );
        assert!(
            citation.template.is_some(),
            "resolution should keep the inherited citation template"
        );
    }

    #[test]
    fn style_preset_spec_yaml_roundtrip() {
        let yaml = r#"
preset: chicago-notes-18th
variant:
  citation:
    ibid: ~
"#;
        let spec: StylePresetSpec = serde_yaml::from_str(yaml).expect("deserialization failed");
        assert_eq!(*spec.preset(), StylePreset::ChicagoNotes18th);
        assert!(spec.variant().is_some());

        let simple_yaml = "chicago-notes-18th";
        let simple_spec: StylePresetSpec =
            serde_yaml::from_str(simple_yaml).expect("simple deserialization failed");
        assert_eq!(*simple_spec.preset(), StylePreset::ChicagoNotes18th);
    }

    #[test]
    fn local_style_overrides_merge_with_preset_base() {
        let style = Style {
            info: crate::StyleInfo {
                title: Some("Taylor & Francis Test".to_string()),
                id: Some("tf-test".to_string()),
                ..Default::default()
            },
            preset: Some(StylePresetSpec::Key(StylePreset::ChicagoAuthorDate18th)),
            options: Some(Config {
                page_range_format: Some(PageRangeFormat::Expanded),
                ..Default::default()
            }),
            ..Default::default()
        };

        let resolved = style.into_resolved();
        let options = resolved
            .options
            .expect("resolved options should be present");
        assert_eq!(options.page_range_format, Some(PageRangeFormat::Expanded));
        assert!(
            options.processing.is_some(),
            "local override should preserve inherited processing"
        );
        assert!(
            resolved.citation.is_some(),
            "local override should preserve inherited citation spec"
        );
    }

    #[test]
    fn style_preset_circular_dependency_is_handled() {
        let mut base = StylePreset::ChicagoNotes18th.base();
        base.preset = Some(StylePresetSpec::Key(StylePreset::ChicagoNotes18th));

        let resolved = base.into_resolved();
        assert!(resolved.preset.is_some());
    }

    #[test]
    fn all_presets_resolve_cleanly() {
        for preset in StylePreset::all() {
            let resolved = preset.base().into_resolved();
            assert!(
                resolved.citation.is_some(),
                "{} resolved citation missing",
                preset.key()
            );
            assert!(
                resolved.options.is_some(),
                "{} resolved options missing",
                preset.key()
            );
        }
    }

    #[test]
    fn explicit_null_variant_field_clears_inherited_value() {
        let yaml = r#"
citation:
  ibid: ~
"#;
        let delta: StyleVariantDelta = serde_yaml::from_str(yaml).expect("delta parses");
        let mut style = StylePreset::ChicagoNotes18th.base();
        delta.apply_to(&mut style);
        assert!(
            style
                .citation
                .as_ref()
                .expect("citation present")
                .ibid
                .is_none(),
            "explicit null should clear inherited ibid"
        );
    }
}
