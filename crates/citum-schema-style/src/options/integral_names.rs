/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Integral citation name-memory configuration.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct IntegralNameConfig {
    /// The name-memory rule to apply to integral citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<IntegralNameRule>,
    /// Where the first-mention memory resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<IntegralNameScope>,
    /// Which document contexts participate in name-memory tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<IntegralNameContexts>,
    /// The contributor form to use after the first mention in scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_form: Option<IntegralNameForm>,
    /// How to display the short name on the first mention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name_display: Option<ShortNameDisplay>,
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

impl IntegralNameConfig {
    /// Merge another integral-name config into this one.
    pub fn merge(&mut self, other: &IntegralNameConfig) {
        if other.rule.is_some() {
            self.rule = other.rule;
        }
        if other.scope.is_some() {
            self.scope = other.scope;
        }
        if other.contexts.is_some() {
            self.contexts = other.contexts;
        }
        if other.subsequent_form.is_some() {
            self.subsequent_form = other.subsequent_form;
        }
        if other.short_name_display.is_some() {
            self.short_name_display = other.short_name_display;
        }
    }

    /// Resolve the effective integral-name config with defaults filled in.
    pub fn resolve(&self) -> ResolvedIntegralNameConfig {
        ResolvedIntegralNameConfig {
            rule: self.rule.unwrap_or_default(),
            scope: self.scope.unwrap_or_default(),
            contexts: self.contexts.unwrap_or_default(),
            subsequent_form: self.subsequent_form.unwrap_or_default(),
            short_name_display: self.short_name_display.unwrap_or_default(),
        }
    }
}

/// The supported integral citation name-memory rule.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum IntegralNameRule {
    /// Render the first integral mention in scope in full, then shorten later mentions.
    #[default]
    FullThenShort,
    /// Opt out of name-memory tracking; no first/subsequent distinction is applied and the
    /// template's own contributor form is used unchanged for every integral citation.
    ShortOnly,
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum IntegralNameForm {
    /// Use the short contributor form for subsequent mentions.
    #[default]
    Short,
    /// Use family name only for subsequent mentions.
    FamilyOnly,
}

/// How to display a contributor's short name on the first integral mention.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ShortNameDisplay {
    /// Render "Full Name (Short)" on first mention (default).
    #[default]
    FullThenParenthetical,
    /// Render "Short [Full Name]" on first mention.
    ShortThenBracketed,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "Panicking is acceptable in tests.")]
mod tests {
    use super::*;

    #[test]
    fn captures_unknown_fields_for_forward_compat() {
        let yaml = r#"
rule: full-then-short
future-key: true
"#;
        let cfg: IntegralNameConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(cfg.unknown_fields.contains_key("future-key"));
        assert_eq!(cfg.rule, Some(IntegralNameRule::FullThenShort));
    }
}

/// Integral-name configuration with defaults resolved.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ResolvedIntegralNameConfig {
    /// The active integral name-memory rule.
    pub rule: IntegralNameRule,
    /// The active scope boundary.
    pub scope: IntegralNameScope,
    /// The active context participation mode.
    pub contexts: IntegralNameContexts,
    /// The contributor form used after the first mention.
    pub subsequent_form: IntegralNameForm,
    /// How to display short names on first mention.
    pub short_name_display: ShortNameDisplay,
}
