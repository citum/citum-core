/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Integral citation name-memory configuration.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
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
    }

    /// Resolve the effective integral-name config with defaults filled in.
    pub fn resolve(&self) -> ResolvedIntegralNameConfig {
        ResolvedIntegralNameConfig {
            rule: self.rule.unwrap_or_default(),
            scope: self.scope.unwrap_or_default(),
            contexts: self.contexts.unwrap_or_default(),
            subsequent_form: self.subsequent_form.unwrap_or_default(),
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
}
