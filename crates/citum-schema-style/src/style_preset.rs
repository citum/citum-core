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
//!
//! ## Design
//!
//! Two levels of preset exist in Citum:
//!
//! - **Level 1 — Options presets** ([`crate::presets`]): fine-grained
//!   configuration bundles for contributors, dates, titles, etc.
//! - **Level 2 — Style presets** (this module): named, compiled-in [`Style`]
//!   structs representing complete well-known styles.
//!
//! A YAML style file references a Level 2 preset via the top-level `preset`
//! field, optionally paired with a `variant` delta:
//!
//! ```yaml
//! preset: chicago-notes-18th
//! variant:
//!   citation:
//!     ibid: ~    # Turabian 9th: no ibid, only short subsequent form
//! ```
//!
//! See `docs/specs/STYLE_PRESET_ARCHITECTURE.md` for the full design rationale.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::{BibliographySpec, CitationSpec, Style};
use crate::embedded::get_embedded_style;
use crate::options::Config;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// StylePreset
// ---------------------------------------------------------------------------

/// A named, compiled-in style preset (Level 2 of Citum's preset hierarchy).
///
/// Each variant corresponds to a well-known citation style baked into the
/// binary. Use the `preset` key at the top of a style YAML to reference one:
///
/// ```yaml
/// preset: chicago-notes-18th
/// ```
///
/// Combine with a [`StyleVariantDelta`] to express behavioral dependents:
///
/// ```yaml
/// preset: chicago-notes-18th
/// variant:
///   citation:
///     ibid: ~
/// ```
///
/// Preset keys are stable. If the underlying style YAML changes, the embedded
/// bytes are updated automatically at compile time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum StylePreset {
    /// Chicago Manual of Style 18th edition — shortened notes and bibliography.
    ///
    /// Corresponds to `styles/chicago-shortened-notes-bibliography.yaml`.
    #[serde(rename = "chicago-notes-18th")]
    ChicagoNotes18th,
    /// Chicago Manual of Style 18th edition — author-date system.
    ///
    /// Corresponds to `styles/chicago-author-date.yaml`.
    #[serde(rename = "chicago-author-date-18th")]
    ChicagoAuthorDate18th,
    /// APA 7th edition — author-date system.
    ///
    /// Corresponds to `styles/apa-7th.yaml`.
    #[serde(rename = "apa-7th")]
    Apa7th,
}

impl StylePreset {
    /// Return the embedded YAML key used to look up this preset.
    fn embedded_key(&self) -> &'static str {
        match self {
            StylePreset::ChicagoNotes18th => "chicago-shortened-notes-bibliography",
            StylePreset::ChicagoAuthorDate18th => "chicago-author-date",
            StylePreset::Apa7th => "apa-7th",
        }
    }

    /// Return the base [`Style`] for this preset.
    ///
    /// Panics if the embedded YAML is missing or malformed — this is a
    /// compile-time invariant enforced by CI, not a runtime condition.
    pub fn base(&self) -> Style {
        let key = self.embedded_key();
        get_embedded_style(key)
            .unwrap_or_else(|| panic!("StylePreset: missing embedded style for key '{key}'"))
            .unwrap_or_else(|e| panic!("StylePreset: malformed embedded YAML for key '{key}': {e}"))
    }

    /// Return the canonical preset key string (kebab-case).
    ///
    /// This is the stable identity used in YAML and the wizard.
    pub fn key(&self) -> &'static str {
        match self {
            StylePreset::ChicagoNotes18th => "chicago-notes-18th",
            StylePreset::ChicagoAuthorDate18th => "chicago-author-date-18th",
            StylePreset::Apa7th => "apa-7th",
        }
    }

    /// Resolve the final [`Style`] by merging an optional [`StyleVariantDelta`]
    /// over the base preset.
    ///
    /// Field-level merge semantics: each `Some` field in `delta` replaces the
    /// corresponding field in the base; `None` fields inherit from the base.
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

        // If the base style itsel has a preset, resolve it recursively.
        if style.preset.is_some() {
            style = style.into_resolved_recursive(visited);
        }

        if let Some(d) = delta {
            d.apply_to(&mut style);
        }
        style
    }
}

// ---------------------------------------------------------------------------
// StyleVariantDelta
// ---------------------------------------------------------------------------

/// A partial [`Style`] overlay for expressing behavioral variant styles.
///
/// A `StyleVariantDelta` holds only the fields that differ from a named base
/// [`StylePreset`]. Fields set to `Some(…)` replace the corresponding field in
/// the base at merge time; `None` fields are inherited unchanged.
///
/// **Merge semantics are field-level, not deep.** A `Some(citation)` replaces
/// the entire citation block; it does not deep-merge individual citation
/// sub-fields.
///
/// ## Example — Turabian deviation from Chicago Notes
///
/// ```yaml
/// variant:
///   citation:
///     ibid: ~          # disables ibid — Turabian 9th uses only subsequent short form
///   options:
///     bibliography:
///       entry-suffix: "."
/// ```
///
/// ## Forward compatibility
///
/// The `custom` field is an explicit escape hatch for variant concerns not yet
/// in the schema (e.g. page-layout options for Turabian student papers, tracked
/// in a follow-up bean). It round-trips arbitrary YAML without loss and is
/// ignored by the engine until a consuming feature is implemented.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StyleVariantDelta {
    /// Overrides the base preset's global options block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Config>,

    /// Overrides the base preset's citation block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<CitationSpec>,

    /// Overrides the base preset's bibliography block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography: Option<BibliographySpec>,

    /// Forward-compatible escape hatch for future variant fields not yet in
    /// the schema (e.g. page-layout presets for student papers).
    ///
    /// Round-trips arbitrary YAML without loss. Ignored by the engine until
    /// a consuming feature is implemented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, Value>>,
}

impl StyleVariantDelta {
    /// Apply this delta onto a mutable [`Style`] in place.
    pub fn apply_to(&self, style: &mut Style) {
        if let Some(options) = &self.options {
            style.options = Some(options.clone());
        }
        if let Some(citation) = &self.citation {
            style.citation = Some(citation.clone());
        }
        if let Some(bibliography) = &self.bibliography {
            style.bibliography = Some(bibliography.clone());
        }
        // `custom` is intentionally not applied to the Style struct — it is
        // stored for round-trip fidelity and future feature use only.
    }

    /// Return a [`StyleVariantDelta`] expressing the Turabian 9th edition
    /// deviations from Chicago Notes 18th.
    ///
    /// Deviations applied:
    /// - No ibid: Turabian 9th ed. uses only the subsequent short form, not
    ///   *ibid.* Full ibid support is a separate `citation.ibid` block that we
    ///   suppress here by setting it to `None` in the resolved `CitationSpec`.
    ///
    /// Out of scope (tracked in a follow-up bean):
    /// - Student title-page layout (no schema support yet; will use `custom`).
    pub fn turabian() -> Self {
        use crate::{CitationSpec, NoteStartTextCase};
        StyleVariantDelta {
            citation: Some(CitationSpec {
                // Disable ibid — Turabian relies only on subsequent short form.
                ibid: None,
                note_start_text_case: Some(NoteStartTextCase::CapitalizeFirst),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// StylePresetSpec
// ---------------------------------------------------------------------------

/// Top-level style preset reference with an optional variant delta overlay.
///
/// Used as the `preset` field on [`Style`]. When present, the engine resolves
/// the final `Style` via [`StylePreset::resolve`] before any further
/// processing.
///
/// Any explicit `options`, `citation`, or `bibliography` keys at the same
/// document level as `preset` are applied **after** preset resolution, so
/// they take precedence over both the base preset and the variant.
///
/// Supports two YAML forms:
///
/// 1. **Simple key** (string): `preset: chicago-notes-18th`
/// 2. **Full spec** (map):
///    ```yaml
///    preset:
///      preset: chicago-notes-18th
///      variant:
///        citation:
///          ibid: ~
///    ```
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
        variant: Option<StyleVariantDelta>,
    },
}

impl StylePresetSpec {
    /// Return the underlying [`StylePreset`].
    pub fn preset(&self) -> &StylePreset {
        match self {
            StylePresetSpec::Key(p) => p,
            StylePresetSpec::Full { preset, .. } => preset,
        }
    }

    /// Return the optional [`StyleVariantDelta`].
    pub fn variant(&self) -> Option<&StyleVariantDelta> {
        match self {
            StylePresetSpec::Key(_) => None,
            StylePresetSpec::Full { variant, .. } => variant.as_ref(),
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_preset_chicago_notes_base_is_valid() {
        // Given a ChicagoNotes18th preset.
        let style = StylePreset::ChicagoNotes18th.base();
        // When base() is called, the style round-trips without panic and has
        // non-empty info.
        let yaml = serde_yaml::to_string(&style).expect("serialization failed");
        let back: Style = serde_yaml::from_str(&yaml).expect("deserialization failed");
        assert!(back.info.title.is_some(), "title should be present");
    }

    #[test]
    fn style_preset_chicago_author_date_base_is_valid() {
        // Given a ChicagoAuthorDate18th preset.
        let style = StylePreset::ChicagoAuthorDate18th.base();
        // When base() is called, the style round-trips without panic.
        assert!(style.info.title.is_some(), "title should be present");
    }

    #[test]
    fn style_preset_apa_7th_base_is_valid() {
        // Given an Apa7th preset.
        let style = StylePreset::Apa7th.base();
        // When base() is called, the style round-trips without panic.
        assert!(style.info.title.is_some(), "title should be present");
    }

    #[test]
    fn turabian_variant_delta_disables_ibid() {
        // Given the Chicago Notes 18th preset and the Turabian variant delta.
        let delta = StyleVariantDelta::turabian();
        let style = StylePreset::ChicagoNotes18th.resolve(Some(&delta));
        // When the delta is applied, citation.ibid should be None.
        let citation = style.citation.expect("citation should be present");
        assert!(
            citation.ibid.is_none(),
            "Turabian variant should disable ibid"
        );
    }

    #[test]
    fn style_variant_delta_custom_roundtrips() {
        // Given a StyleVariantDelta with a custom field containing unknown data.
        let yaml = r#"
custom:
  student-paper:
    title-page: true
    institution: "University of Chicago"
"#;
        let delta: StyleVariantDelta = serde_yaml::from_str(yaml).expect("deserialization failed");
        let custom = delta.custom.as_ref().expect("custom should be present");
        assert!(custom.contains_key("student-paper"));

        // When round-tripped through serde, the data survives without loss.
        let back_yaml = serde_yaml::to_string(&delta).expect("serialization failed");
        let back: StyleVariantDelta =
            serde_yaml::from_str(&back_yaml).expect("deserialization failed");
        assert!(back.custom.is_some(), "custom should survive round-trip");
    }

    #[test]
    fn style_preset_spec_resolves() {
        // Given a StylePresetSpec for Turabian using the new Full enum variant.
        let spec = StylePresetSpec::Full {
            preset: StylePreset::ChicagoNotes18th,
            variant: Some(StyleVariantDelta::turabian()),
        };
        // When resolve() is called, the result has ibid disabled.
        let style = spec.resolve();
        let citation = style.citation.expect("citation should be present");
        assert!(
            citation.ibid.is_none(),
            "Turabian preset spec should disable ibid"
        );
    }

    #[test]
    fn style_preset_spec_yaml_roundtrip() {
        // Given a StylePresetSpec serialized to YAML.
        let yaml = r#"
preset: chicago-notes-18th
variant:
  citation:
    ibid: ~
"#;
        let spec: StylePresetSpec =
            serde_yaml::from_str(yaml).expect("deserialization failed");
        // When deserialized: preset() key parses correctly.
        assert_eq!(*spec.preset(), StylePreset::ChicagoNotes18th);
        assert!(spec.variant().is_some());

        // Also test the simple string form.
        let simple_yaml = "chicago-notes-18th";
        let simple_spec: StylePresetSpec =
            serde_yaml::from_str(simple_yaml).expect("simple deserialization failed");
        assert_eq!(*simple_spec.preset(), StylePreset::ChicagoNotes18th);
    }

    #[test]
    fn style_preset_circular_dependency_is_handled() {
        // Given a style that points to itself in its preset field.
        // We simulate this by creating a base style that has a preset.
        let mut base = StylePreset::ChicagoNotes18th.base();
        base.preset = Some(StylePresetSpec::Key(StylePreset::ChicagoNotes18th));

        // When into_resolved is called on such a style.
        let resolved = base.into_resolved();

        // Then it should return the style as-is (with the loop detected and stopped)
        // rather than entering an infinite recursion.
        assert!(resolved.preset.is_some());
    }

    #[test]
    fn all_presets_resolve_cleanly() {
        // Verify all defined presets resolve without loops.
        for preset in [
            StylePreset::ChicagoNotes18th,
            StylePreset::ChicagoAuthorDate18th,
            StylePreset::Apa7th,
        ] {
            let style = preset.base();
            // This should not loop or panic.
            let _ = style.into_resolved();
        }
    }
}
