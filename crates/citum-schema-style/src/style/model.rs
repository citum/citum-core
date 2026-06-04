/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! The Citum style model.

use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Serialize};

#[allow(unused_imports, reason = "Referenced by intra-doc links.")]
use crate::ResolutionError;
use crate::style_base;
use crate::{BibliographySpec, CitationSpec, Config, SchemaVersion, StyleInfo, Template};

/// The new Citum Style model.
///
/// This is the target schema for Citum, featuring declarative options
/// and simple template components instead of procedural conditionals.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Style {
    /// Style schema version.
    #[serde(default)]
    pub version: SchemaVersion,
    /// Style metadata.
    #[serde(default)]
    pub info: StyleInfo,
    /// Named reusable templates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates: Option<HashMap<String, Template>>,
    /// Global style options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Config>,
    /// Citation specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<CitationSpec>,
    /// Bibliography specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography: Option<BibliographySpec>,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
    /// Extends a base style, with optional local overrides.
    ///
    /// When present, the base [`StyleReference`](style_base::StyleReference) is resolved and the local
    /// overrides are merged before any further processing. Explicit `options`,
    /// `citation`, and `bibliography` keys at the same document level take
    /// precedence over the resolved base.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<style_base::StyleReference>,
    /// Optional content-addressed integrity pin for the parent style referenced
    /// by [`extends`](Self::extends).
    ///
    /// When present, the resolver verifies that the SHA-256 of the fetched
    /// parent matches this CIDv1 string before merging. Mismatches abort
    /// resolution with [`ResolutionError::IntegrityFailure`]. Absent means
    /// "no integrity check" — appropriate for `file://` parents under user
    /// control or trusted local registries.
    #[serde(rename = "extends-pin", skip_serializing_if = "Option::is_none")]
    pub extends_pin: Option<String>,
    /// Raw YAML captured when the style was loaded via [`Style::from_yaml_str`]
    /// or [`Style::from_yaml_bytes`]. Used during style resolution for
    /// null-aware overlay merging (e.g., `ibid: ~` correctly clears an
    /// inherited preset value). Absent in programmatically-constructed styles.
    #[cfg_attr(feature = "schema", schemars(skip))]
    #[serde(skip, default)]
    pub raw_yaml: Option<serde_yaml::Value>,
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

impl Style {
    /// Parse a Citum style from a YAML string, preserving raw YAML for
    /// null-aware overlay merging during base resolution.
    ///
    /// Preferred over `serde_yaml::from_str` when the style extends a base,
    /// so that `ibid: ~` and similar null overrides correctly clear inherited values.
    ///
    /// # Errors
    ///
    /// Returns a serde error if YAML parsing or deserialization fails.
    pub fn from_yaml_str(s: &str) -> Result<Self, serde_yaml::Error> {
        let raw: serde_yaml::Value = serde_yaml::from_str(s)?;
        super::diagnostics::validate_raw_style(&raw).map_err(serde_yaml::Error::custom)?;
        let mut style: Style = serde_yaml::from_value(raw.clone())?;
        style.raw_yaml = Some(raw);
        style
            .validate_resource_limits()
            .map_err(serde_yaml::Error::custom)?;
        Ok(style)
    }

    /// Apply scoped citation and bibliography option overrides to this style.
    ///
    /// Translates typed option values (label mode, label wrap, repeated-author
    /// rendering, date position, title terminator) into concrete template mutations.
    /// Call this after mutating `bibliography.options` at runtime — e.g. after
    /// applying per-document overrides — so that template state stays consistent
    /// with the option values.
    pub fn apply_scoped_options(&mut self) {
        crate::options::scoped::apply_scoped_style_options(self);
    }

    /// Merge a partial overlay style over this style in place; overlay fields win.
    ///
    /// Overlay merging is typed and matches `extends` inheritance for the fields it supports:
    /// - `info`, `templates`, `options`, and `custom` are merged (overlay wins for `Some` fields / keys).
    /// - `citation` / `bibliography` are deep-merged; explicit YAML `~` can clear inherited fields when
    ///   `overlay.raw_yaml` is populated (e.g. via `Style::from_yaml_bytes`).
    ///
    /// The caller is responsible for calling [`apply_scoped_options`](Self::apply_scoped_options)
    /// afterwards if scoped-option side-effects (label-wrap, date-position, etc.) are needed.
    pub fn apply_overlay(&mut self, overlay: &Style) {
        super::overlay::merge_style_overlay(self, overlay);
    }

    /// Parse a Citum style from YAML bytes, preserving raw YAML for
    /// null-aware overlay merging during preset resolution.
    ///
    /// # Errors
    ///
    /// Returns a serde error if YAML parsing or deserialization fails.
    pub fn from_yaml_bytes(bytes: &[u8]) -> Result<Self, serde_yaml::Error> {
        let raw: serde_yaml::Value = serde_yaml::from_slice(bytes)?;
        super::diagnostics::validate_raw_style(&raw).map_err(serde_yaml::Error::custom)?;
        let mut style: Style = serde_yaml::from_value(raw.clone())?;
        style.raw_yaml = Some(raw);
        style
            .validate_resource_limits()
            .map_err(serde_yaml::Error::custom)?;
        Ok(style)
    }
}
