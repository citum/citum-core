/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Bibliography section specification.

use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::grouping;
use crate::options::BibliographyOptions;
use crate::template::{
    LocalizedTemplateSpec, Template, TemplateReference, TemplateVariants, locale_matches,
};

fn default_true() -> bool {
    true
}

/// Bibliography specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct BibliographySpec {
    /// Bibliography-specific option overrides merged over the style config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<BibliographyOptions>,
    /// Reference to an embedded template preset or external template.
    ///
    /// If both `template-ref` and `template` are present, `template` takes precedence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_ref: Option<TemplateReference>,
    /// The default template for bibliography entries.
    /// Default template for entries when no localized override is selected.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub template: Option<Template>,
    /// Locale-specific template overrides checked before the default template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locales: Option<Vec<LocalizedTemplateSpec>>,
    /// Type-specific template overrides. When present, replaces the default
    /// template for entries of the specified types. Keys are reference type
    /// names (e.g., "chapter", "article-journal").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_variants: Option<TemplateVariants>,
    /// Optional global bibliography sorting specification.
    ///
    /// When present, used for sorting the flat bibliography or as default
    /// for groups that don't specify their own sort.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<grouping::GroupSortEntry>,
    /// Whether to apply manual `groups:` bibliography grouping.
    ///
    /// Defaults to `true`. Set to `false` to disable the `groups:` configuration
    /// and render a flat bibliography instead. Automatic sort-partition sections
    /// are unaffected by this toggle.
    // TODO: consider defaulting to false once grouping matures for publishing workflows
    #[serde(default = "default_true")]
    pub groups_enabled: bool,
    /// Optional bibliography grouping specification.
    ///
    /// When present, divides the bibliography into labeled sections with
    /// optional per-group sorting. Items match the first group whose selector
    /// evaluates to true (first-match semantics). Omit for flat bibliography.
    ///
    /// See `BibliographyGroup` for examples.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<grouping::BibliographyGroup>>,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
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

impl Default for BibliographySpec {
    fn default() -> Self {
        Self {
            options: None,
            template_ref: None,
            template: None,
            locales: None,
            type_variants: None,
            sort: None,
            groups_enabled: true,
            groups: None,
            custom: None,
            unknown_fields: std::collections::BTreeMap::new(),
        }
    }
}

impl BibliographySpec {
    /// Resolve the effective template for this bibliography.
    ///
    /// Returns the explicit `template` if present, otherwise resolves `template-ref`.
    /// Returns `None` if neither is specified.
    pub fn resolve_template(&self) -> Option<Template> {
        self.template.clone().or_else(|| {
            self.template_ref
                .as_ref()
                .and_then(TemplateReference::bibliography_template)
        })
    }

    /// Resolve the template for a language by checking localized overrides,
    /// then the localized default, then the base template or preset.
    pub fn resolve_template_for_language(&self, language: Option<&str>) -> Option<Template> {
        if let Some(language) = language
            && let Some(locales) = &self.locales
            && let Some(matched) = locales.iter().find(|spec| {
                spec.locale
                    .as_ref()
                    .is_some_and(|targets| locale_matches(targets, language))
            })
        {
            return Some(matched.template.clone());
        }

        self.locales
            .as_ref()
            .and_then(|locales| {
                locales
                    .iter()
                    .find(|spec| spec.default.unwrap_or(false))
                    .map(|spec| spec.template.clone())
            })
            .or_else(|| self.resolve_template())
    }

    /// Resolve the bibliography template for a reference type and language.
    pub fn resolve_template_for_type(
        &self,
        ref_type: &str,
        language: Option<&str>,
    ) -> Option<Template> {
        if let Some(type_variants) = &self.type_variants {
            for (selector, variant) in type_variants {
                if selector.matches(ref_type) {
                    return variant.clone().into_template();
                }
            }
        }
        self.resolve_template_for_language(language)
    }
}
