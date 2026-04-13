/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Style-level preset architecture (Level 2).
//!
//! This module provides [`StylePreset`] — the mechanism for naming
//! well-known compiled-in styles so that a YAML file can declare
//! `preset: chicago-notes-18th` and inherit the full style, then override
//! any fields it needs at the top level of the style document.

use crate::Style;
use crate::embedded::get_embedded_style;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const APA_7TH_BASE: &str = "preset-bases/apa-7th";
const CHICAGO_AUTHOR_DATE_18TH_BASE: &str = "preset-bases/chicago-author-date-18th";
const CHICAGO_NOTES_18TH_BASE: &str = "preset-bases/chicago-notes-18th";

/// A named, compiled-in style preset (Level 2 of Citum's preset hierarchy).
///
/// A style file declares `preset: <key>` to inherit a complete base style.
/// Any top-level fields in the file (`options`, `citation`, `bibliography`,
/// etc.) are merged over the preset base, with local fields taking
/// ultimate precedence.
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

    /// Internal resolver with loop protection.
    pub(crate) fn resolve_with_visited(&self, visited: &mut HashSet<StylePreset>) -> Style {
        let mut style = self.base();
        if style.preset.is_some() {
            style = style.into_resolved_recursive(visited);
        }
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{Config, PageRangeFormat};
    use crate::{Style, StyleInfo};

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
        assert_eq!(style.preset, Some(StylePreset::Apa7th));
        let citation = style.citation.as_ref().expect("citation should be present");
        assert!(
            citation.use_preset.is_none(),
            "APA preset base should carry authored citation templates"
        );
        assert!(
            citation.template.is_none(),
            "APA preset base should not define a top-level citation template"
        );
        assert!(
            citation
                .integral
                .as_ref()
                .is_some_and(|i| i.template.is_some()),
            "APA preset base should define an authored integral citation template"
        );
        assert!(
            citation
                .non_integral
                .as_ref()
                .is_some_and(|ni| ni.template.is_some()),
            "APA preset base should define an authored non-integral citation template"
        );

        let bibliography = style
            .bibliography
            .as_ref()
            .expect("bibliography should be present");
        assert!(
            bibliography.use_preset.is_none(),
            "APA preset base should carry authored bibliography templates"
        );
        assert!(
            bibliography.template.is_some(),
            "APA preset base should define an authored bibliography template"
        );
        assert!(
            bibliography
                .type_variants
                .as_ref()
                .is_some_and(|variants| !variants.is_empty()),
            "APA preset base should define authored bibliography type variants"
        );
    }

    #[test]
    fn style_preset_yaml_roundtrip() {
        let yaml = "chicago-notes-18th";
        let preset: StylePreset = serde_yaml::from_str(yaml).expect("deserialization failed");
        assert_eq!(preset, StylePreset::ChicagoNotes18th);

        let back = serde_yaml::to_string(&preset).expect("serialization failed");
        assert!(back.trim() == "chicago-notes-18th");
    }

    #[test]
    fn top_level_null_field_clears_inherited_preset_value() {
        // A style that inherits Chicago Notes but disables ibid via a
        // top-level citation block — the canonical authoring pattern
        // since there is no separate variant layer.
        let yaml = r#"
preset: chicago-notes-18th
citation:
  ibid: ~
"#;
        let style: Style = Style::from_yaml_str(yaml).expect("style parses");
        let resolved = style.into_resolved();
        assert!(
            resolved
                .citation
                .as_ref()
                .expect("citation present")
                .ibid
                .is_none(),
            "top-level null should clear inherited ibid"
        );
        assert!(
            resolved.citation.as_ref().unwrap().template.is_some(),
            "top-level override should preserve the inherited template"
        );
    }

    #[test]
    fn local_style_overrides_merge_with_preset_base() {
        let style = Style {
            info: StyleInfo {
                title: Some("Taylor & Francis Test".to_string()),
                id: Some("tf-test".into()),
                ..Default::default()
            },
            preset: Some(StylePreset::ChicagoAuthorDate18th),
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
        base.preset = Some(StylePreset::ChicagoNotes18th);

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
    fn turabian_pattern_disables_ibid_via_top_level_citation() {
        // Turabian 9th ed. = Chicago Notes + ibid disabled.
        // With no variant layer, this is expressed as a top-level citation override.
        let yaml = r#"
info:
  title: "Turabian 9th"
preset: chicago-notes-18th
citation:
  ibid: ~
"#;
        let style = Style::from_yaml_str(yaml).expect("style parses");
        let resolved = style.into_resolved();
        let citation = resolved.citation.expect("citation should be present");
        assert!(citation.ibid.is_none(), "ibid should be disabled");
        assert!(
            citation.template.is_some(),
            "inherited template should be preserved"
        );
    }
}
