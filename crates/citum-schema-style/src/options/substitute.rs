/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

/// Substitution rules for missing author data.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum SubstituteConfig {
    /// A named preset (e.g., "standard", "editor-first", "title-first").
    Preset(crate::presets::SubstitutePreset),
    /// Explicit substitution configuration.
    Explicit(Substitute),
}

impl Default for SubstituteConfig {
    fn default() -> Self {
        SubstituteConfig::Explicit(Substitute::default())
    }
}

impl SubstituteConfig {
    /// Resolve this config to a concrete `Substitute`.
    pub fn resolve(&self) -> Substitute {
        match self {
            SubstituteConfig::Preset(preset) => preset.config(),
            SubstituteConfig::Explicit(config) => config.clone(),
        }
    }

    /// Resolve this config without cloning an explicit configuration.
    pub fn resolve_ref(&self) -> Cow<'_, Substitute> {
        match self {
            SubstituteConfig::Preset(preset) => Cow::Owned(preset.config()),
            SubstituteConfig::Explicit(config) => Cow::Borrowed(config),
        }
    }

    /// Resolve an optional config, allocating only when a default or preset is required.
    pub fn resolve_or_default(config: Option<&Self>) -> Cow<'_, Substitute> {
        config.map_or_else(
            || Cow::Owned(Substitute::default()),
            SubstituteConfig::resolve_ref,
        )
    }

    /// Merge an override substitute config over a base config.
    #[must_use]
    pub fn merged(base: &Self, override_config: &Self) -> Self {
        SubstituteConfig::Explicit(Substitute::merged(
            &base.resolve(),
            &override_config.resolve(),
        ))
    }
}

/// Explicit substitution configuration.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Substitute {
    /// Form to use for contributor roles when substituting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributor_role_form: Option<String>,
    /// Optional `text-case` transform applied to the substitute role label.
    ///
    /// Lets a style capitalise the locale's lowercase term, e.g. IEEE's `Eds.`
    /// from `eds.`. Only affects the substitute (editor-as-author) path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributor_role_case: Option<crate::options::titles::TextCase>,
    /// Ordered list of fields to try as substitutes.
    #[serde(default)]
    pub template: Vec<SubstituteKey>,
    /// Type-specific substitution overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub overrides: HashMap<String, Vec<SubstituteKey>>,
    /// Per-role fallback chains for non-author contributor substitution.
    ///
    /// Keys and fallback entries normalize to canonical kebab-case role names.
    /// Built-in template roles use locale-aware labels automatically. Custom
    /// role names still participate in fallback and suppression even when they
    /// do not have a dedicated template enum variant.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub role_substitute: HashMap<String, Vec<String>>,
    /// Quoting policy for a title substituted into the author position in
    /// citation context (the `title` substitute key). Unset (or `always`)
    /// preserves the historical unconditional-quote behavior; `by-category`
    /// defers to the style's `titles:` category rendering, so e.g. a book
    /// title italicizes instead of quoting. See divergence register div-011.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_quote: Option<SubstituteTitleQuoteMode>,
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

impl Default for Substitute {
    fn default() -> Self {
        Self {
            contributor_role_form: None,
            contributor_role_case: None,
            template: vec![
                SubstituteKey::Field(SubstituteField::Editor),
                SubstituteKey::Field(SubstituteField::Title),
                SubstituteKey::Field(SubstituteField::Translator),
            ],
            overrides: HashMap::new(),
            role_substitute: HashMap::new(),
            title_quote: None,
            unknown_fields: std::collections::BTreeMap::new(),
        }
    }
}

/// How a title used as an author substitute is quoted in citation context.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum SubstituteTitleQuoteMode {
    /// Always quote the substituted title in citation context (historical
    /// Citum default; matches prior behavior regardless of reference type).
    Always,
    /// Resolve quoting via the normal title-category rendering machinery
    /// (`titles:` config), the same way a non-substitute title would be
    /// quoted/italicized for that reference type.
    ByCategory,
}

impl Substitute {
    /// Merge an override substitute config over a base config.
    pub fn merge(&mut self, other: &Self) {
        if other.contributor_role_form.is_some() {
            self.contributor_role_form = other.contributor_role_form.clone();
        }
        if other.contributor_role_case.is_some() {
            self.contributor_role_case = other.contributor_role_case;
        }
        if !other.template.is_empty() {
            self.template = other.template.clone();
        }
        self.overrides.extend(other.overrides.clone());
        self.role_substitute.extend(other.role_substitute.clone());
        if other.title_quote.is_some() {
            self.title_quote = other.title_quote;
        }
    }

    /// Create a merged substitute config from base and override.
    #[must_use]
    pub fn merged(base: &Self, override_config: &Self) -> Self {
        let mut result = base.clone();
        result.merge(override_config);
        result
    }
}

/// One candidate in an effective-primary substitution chain.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum SubstituteKey {
    /// A legacy scalar field candidate such as `editor` or `title`.
    Field(SubstituteField),
    /// A scalar or merged contributor-role candidate.
    Contributor(SubstituteContributor),
}

#[allow(
    non_upper_case_globals,
    reason = "preserve the legacy SubstituteKey constant API"
)]
impl SubstituteKey {
    /// Legacy `collection-editor` scalar candidate.
    pub const CollectionEditor: Self = Self::Field(SubstituteField::CollectionEditor);
    /// Legacy `editor` scalar candidate.
    pub const Editor: Self = Self::Field(SubstituteField::Editor);
    /// Legacy `parent-serial` scalar candidate.
    pub const ParentSerial: Self = Self::Field(SubstituteField::ParentSerial);
    /// Legacy `title` scalar candidate.
    pub const Title: Self = Self::Field(SubstituteField::Title);
    /// Legacy `translator` scalar candidate.
    pub const Translator: Self = Self::Field(SubstituteField::Translator);
}

/// A contributor-role candidate in an effective-primary substitution chain.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SubstituteContributor {
    /// One contributor role or an ordered list of roles to promote.
    pub contributor: crate::template::ContributorRoles,
}

/// Legacy scalar fields accepted in substitution chains.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SubstituteField {
    /// The collection editor contributor role.
    #[serde(rename = "collection-editor")]
    CollectionEditor,
    /// The editor contributor role.
    Editor,
    /// The parent serial title.
    #[serde(rename = "parent-serial")]
    ParentSerial,
    /// The primary title.
    Title,
    /// The translator contributor role.
    Translator,
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
    use super::{Substitute, SubstituteConfig, SubstituteField, SubstituteKey};
    use std::borrow::Cow;
    use std::collections::HashMap;

    #[test]
    fn explicit_substitute_resolution_borrows_while_presets_are_owned() {
        let explicit = SubstituteConfig::Explicit(Substitute::default());
        let preset = SubstituteConfig::Preset(crate::presets::SubstitutePreset::Standard);

        assert!(matches!(explicit.resolve_ref(), Cow::Borrowed(_)));
        assert!(matches!(preset.resolve_ref(), Cow::Owned(_)));
        assert!(matches!(
            SubstituteConfig::resolve_or_default(Some(&explicit)),
            Cow::Borrowed(_)
        ));
        assert!(matches!(
            SubstituteConfig::resolve_or_default(None),
            Cow::Owned(_)
        ));
    }

    #[test]
    fn merged_substitute_configs_preserve_role_substitute_chains() {
        let base = SubstituteConfig::Explicit(Substitute {
            role_substitute: HashMap::from([(
                "container-author".to_string(),
                vec!["editor".to_string()],
            )]),
            ..Default::default()
        });
        let override_config = SubstituteConfig::Preset(crate::presets::SubstitutePreset::Standard);

        let merged = SubstituteConfig::merged(&base, &override_config).resolve();

        assert_eq!(
            merged.role_substitute.get("container-author"),
            Some(&vec!["editor".to_string()])
        );
        assert_eq!(
            merged.template,
            vec![
                SubstituteKey::Field(SubstituteField::Editor),
                SubstituteKey::Field(SubstituteField::Title),
                SubstituteKey::Field(SubstituteField::Translator)
            ]
        );
    }

    #[test]
    fn substitution_candidates_round_trip_scalar_and_merged_roles() {
        let yaml = r#"template:
  - editor
  - contributor: director
overrides:
  episode:
    - contributor: [writer, director]
"#;

        let parsed: Substitute = serde_yaml::from_str(yaml).expect("valid substitution yaml");

        assert_eq!(
            parsed.template[0],
            SubstituteKey::Field(SubstituteField::Editor)
        );
        assert_eq!(
            parsed.template[1],
            SubstituteKey::Contributor(super::SubstituteContributor {
                contributor: crate::template::ContributorRole::Director.into(),
            })
        );
        let serialized = serde_yaml::to_string(&parsed).expect("serializable");
        assert_eq!(
            serde_yaml::from_str::<Substitute>(&serialized).expect("round-trippable"),
            parsed
        );
    }
}
