/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Integral citation name-memory policy.
///
/// The presence of this block enables full-then-short name memory for narrative
/// (integral) citations. Absence disables it — there is no on/off field.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct IntegralNameMemoryConfig {
    /// Where the first-mention memory resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<IntegralNameScope>,
    /// Which document contexts participate in name-memory tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<IntegralNameContexts>,
    /// The contributor form to use after the first mention in scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_form: Option<SubsequentNameForm>,
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

impl IntegralNameMemoryConfig {
    /// Merge another integral-name-memory config into this one.
    pub fn merge(&mut self, other: &IntegralNameMemoryConfig) {
        if other.scope.is_some() {
            self.scope = other.scope;
        }
        if other.contexts.is_some() {
            self.contexts = other.contexts;
        }
        if other.subsequent_form.is_some() {
            self.subsequent_form = other.subsequent_form;
        }
    }

    /// Resolve the effective integral-name-memory config with defaults filled in.
    pub fn resolve(&self) -> ResolvedIntegralNameMemoryConfig {
        ResolvedIntegralNameMemoryConfig {
            scope: self.scope.unwrap_or_default(),
            contexts: self.contexts.unwrap_or_default(),
            subsequent_form: self.subsequent_form.unwrap_or_default(),
        }
    }
}

/// The scope where integral citation name-memory resets.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum IntegralNameScope {
    /// Keep one name-memory scope for the whole document.
    #[default]
    Document,
    /// Reset name memory at chapter boundaries.
    Chapter,
    /// Reset name memory at section boundaries.
    Section,
}

/// Which document contexts participate in integral citation name memory.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum IntegralNameContexts {
    /// Only body-text integral citations participate.
    #[default]
    BodyOnly,
    /// Body text and note citations both participate.
    BodyAndNotes,
}

/// The contributor form used after the first integral mention in scope.
///
/// `Short` preserves non-dropping particles ("van Beethoven"); `FamilyOnly`
/// strips them ("Beethoven"). MLA-style narrative memory uses `Short`.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum SubsequentNameForm {
    /// Use the short contributor form for subsequent mentions.
    #[default]
    Short,
    /// Use family name only (without non-dropping particles) for subsequent mentions.
    FamilyOnly,
}

/// How to display a contributor's short name on the first integral mention.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ShortNameDisplay {
    /// "Full Name (Short)" on first mention (default). For narrative/integral context.
    #[default]
    FullThenParenthetical,
    /// `"Full Name [Short]"` on first mention. For parenthetical context (already inside parens).
    FullThenBracketed,
    /// "Short (Full Name)" on first mention.
    ShortThenParenthetical,
    /// "Short [Full Name]" on first mention.
    ShortThenBracketed,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "Panicking is acceptable in tests.")]
mod tests {
    use super::*;

    #[test]
    fn captures_unknown_fields_for_forward_compat() {
        let yaml = r#"
scope: chapter
future-key: true
"#;
        let cfg: IntegralNameMemoryConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(cfg.unknown_fields.contains_key("future-key"));
        assert_eq!(cfg.scope, Some(IntegralNameScope::Chapter));
    }
}

/// Integral-name-memory configuration with defaults resolved.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ResolvedIntegralNameMemoryConfig {
    /// The active scope boundary.
    pub scope: IntegralNameScope,
    /// The active context participation mode.
    pub contexts: IntegralNameContexts,
    /// The contributor form used after the first mention.
    pub subsequent_form: SubsequentNameForm,
}

/// Organizational name abbreviation expansion policy.
///
/// The presence of this block enables first-mention expansion of org names —
/// "World Health Organization (WHO)" on first mention, "WHO" thereafter.
/// Absence disables org abbreviation entirely.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct OrgAbbreviationMemoryConfig {
    /// Where the first-mention memory resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<IntegralNameScope>,
    /// Which document contexts participate in org-abbreviation tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<IntegralNameContexts>,
    /// How to display an organizational short name on the first integral mention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name_display: Option<ShortNameDisplay>,
    /// Forward-compat: captures unknown keys from newer schema versions.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub unknown_fields: std::collections::BTreeMap<String, serde_yaml::Value>,
}

impl OrgAbbreviationMemoryConfig {
    /// Merge another org-abbreviation config into this one.
    pub fn merge(&mut self, other: &OrgAbbreviationMemoryConfig) {
        if other.scope.is_some() {
            self.scope = other.scope;
        }
        if other.contexts.is_some() {
            self.contexts = other.contexts;
        }
        if other.short_name_display.is_some() {
            self.short_name_display = other.short_name_display;
        }
    }

    /// Resolve the effective org-abbreviation config with defaults filled in.
    pub fn resolve(&self) -> ResolvedOrgAbbreviationMemoryConfig {
        ResolvedOrgAbbreviationMemoryConfig {
            scope: self.scope.unwrap_or_default(),
            contexts: self.contexts.unwrap_or_default(),
            short_name_display: self.short_name_display.unwrap_or_default(),
        }
    }
}

/// Org-abbreviation-memory configuration with defaults resolved.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ResolvedOrgAbbreviationMemoryConfig {
    /// The active scope boundary.
    pub scope: IntegralNameScope,
    /// The active context participation mode.
    pub contexts: IntegralNameContexts,
    /// How to display short names on first mention.
    pub short_name_display: ShortNameDisplay,
}
