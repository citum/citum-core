/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contributor config: either a preset name or explicit configuration.
///
/// Allows styles to write `contributors: apa` as shorthand, or provide
/// full explicit configuration with field-level overrides.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum ContributorConfigEntry {
    /// A named preset (e.g., "apa", "chicago", "vancouver", "springer").
    Preset(crate::presets::ContributorPreset),
    /// Explicit contributor configuration.
    Explicit(Box<ContributorConfig>),
}

impl Default for ContributorConfigEntry {
    fn default() -> Self {
        ContributorConfigEntry::Explicit(Box::default())
    }
}

impl ContributorConfigEntry {
    /// Resolve this entry to a concrete `ContributorConfig`.
    pub fn resolve(&self) -> ContributorConfig {
        match self {
            ContributorConfigEntry::Preset(preset) => preset.config(),
            ContributorConfigEntry::Explicit(config) => *config.clone(),
        }
    }
}

/// Contributor formatting configuration.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ContributorConfig {
    /// When to display a contributor's name in sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_as_sort: Option<DisplayAsSort>,
    /// String to append after initialized given names (e.g., ". " for "J. Smith").
    /// If None, full given names are used (e.g., "John Smith").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize_with: Option<String>,
    /// Whether to include a hyphen when initializing names (e.g., "J.-P. Sartre").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize_with_hyphen: Option<bool>,
    /// Shorten the list of contributors (et al. handling).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shorten: Option<ShortenListOptions>,
    /// The delimiter between contributors. Defaults to `", "` if not specified.
    /// `None` means "not configured at this level" and will not override an inherited value.
    #[serde(
        default = "default_contributor_delimiter",
        skip_serializing_if = "is_default_contributor_delimiter"
    )]
    pub delimiter: Option<String>,
    /// Conjunction between last two contributors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<AndOptions>,
    /// When to include delimiter before the last contributor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter_precedes_last: Option<DelimiterPrecedesLast>,
    /// When to include delimiter before "et al.".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter_precedes_et_al: Option<DelimiterPrecedesLast>,
    /// When and how to display contributor roles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<RoleOptions>,
    /// Handling of non-dropping particles (e.g., "van" in "van Gogh").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demote_non_dropping_particle: Option<DemoteNonDroppingParticle>,
    /// Delimiter between family and given name when inverted (e.g., `", "` → "Smith, John").
    /// Defaults to `", "` in the engine when not specified.
    /// `None` means "not configured at this level" and will not override an inherited value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_separator: Option<String>,
    /// How to render given names. See `NameForm` for variants.
    /// Per-scope overrides (per-mode, per-position) are expressed by setting
    /// this field in the appropriate scope's contributor config block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_form: Option<NameForm>,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

impl ContributorConfig {
    /// Merge another ContributorConfig into this one.
    pub fn merge(&mut self, other: &ContributorConfig) {
        if other.display_as_sort.is_some() {
            self.display_as_sort = other.display_as_sort;
        }
        if other.initialize_with.is_some() {
            self.initialize_with = other.initialize_with.clone();
        }
        if other.initialize_with_hyphen.is_some() {
            self.initialize_with_hyphen = other.initialize_with_hyphen;
        }
        if other.shorten.is_some() {
            self.shorten = other.shorten.clone();
        }
        if other.delimiter.is_some() {
            self.delimiter = other.delimiter.clone();
        }
        if other.and.is_some() {
            self.and = other.and.clone();
        }
        if other.delimiter_precedes_last.is_some() {
            self.delimiter_precedes_last = other.delimiter_precedes_last;
        }
        if other.delimiter_precedes_et_al.is_some() {
            self.delimiter_precedes_et_al = other.delimiter_precedes_et_al;
        }
        if other.role.is_some() {
            self.role = other.role.clone();
        }
        if other.demote_non_dropping_particle.is_some() {
            self.demote_non_dropping_particle = other.demote_non_dropping_particle;
        }
        if other.sort_separator.is_some() {
            self.sort_separator = other.sort_separator.clone();
        }
        if other.name_form.is_some() {
            self.name_form = other.name_form;
        }
    }

    /// Return the configured rendering override for a specific contributor role.
    pub fn role_rendering(
        &self,
        role: &crate::template::ContributorRole,
    ) -> Option<&RoleRendering> {
        self.role.as_ref()?.role_rendering(role)
    }

    /// Resolve the effective label preset for a specific contributor role.
    pub fn effective_role_label_preset(
        &self,
        role: &crate::template::ContributorRole,
    ) -> Option<RoleLabelPreset> {
        self.role
            .as_ref()
            .and_then(|role_options| role_options.effective_label_preset(role))
    }

    /// Return the configured name-order override for a specific contributor role.
    pub fn effective_role_name_order(
        &self,
        role: &crate::template::ContributorRole,
    ) -> Option<&crate::template::NameOrder> {
        self.role_rendering(role)
            .and_then(|rendering| rendering.name_order.as_ref())
    }
}

/// Named role-label presets for secondary contributor rendering.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum RoleLabelPreset {
    /// Suppress the configured role label.
    None,
    /// Render the localized verb label before the names.
    VerbPrefix,
    /// Render the localized short verb label before the names.
    VerbShortPrefix,
    /// Render the localized short label after the names.
    ShortSuffix,
    /// Render the localized long label after the names.
    LongSuffix,
}

impl RoleLabelPreset {
    /// Resolve a legacy string form to the corresponding role-label preset.
    pub fn from_form_str(form: &str) -> Option<Self> {
        match form {
            "none" => Some(Self::None),
            "verb" => Some(Self::VerbPrefix),
            "verb-short" => Some(Self::VerbShortPrefix),
            "short" => Some(Self::ShortSuffix),
            "long" => Some(Self::LongSuffix),
            _ => None,
        }
    }
}

/// Options for demoting non-dropping particles.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum DemoteNonDroppingParticle {
    Never,
    SortOnly,
    #[default]
    DisplayAndSort,
}

/// When to display names in sort order (family-first).
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum DisplayAsSort {
    All,
    First,
    #[default]
    None,
}

/// How to render the given-name component of a contributor name.
///
/// Controls whether full given names, family name only, or initialized
/// given names are rendered. Used to express first/subsequent mention
/// differences (Chicago) and integral/non-integral differences.
///
/// Initialization formatting details (`initialize_with`, `initialize_with_hyphen`)
/// are separate fields and only take effect when `NameForm::Initials` is active.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum NameForm {
    /// Render full given names: "John D. Smith".
    #[default]
    Full,
    /// Render family name only, suppressing given names: "Smith".
    /// Used for subsequent mentions in Chicago/Turabian note styles.
    FamilyOnly,
    /// Render initialized given names using `initialize_with` separator.
    /// If `initialize_with` is None, defaults to ". " (e.g., "J. Smith").
    /// Empty string gives compact initials: "JD Smith".
    Initials,
}

/// Conjunction options between contributors.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum AndOptions {
    /// Use the localized term for "and" (e.g., "and" in English).
    Text,
    /// Use the localized symbol for "and" (e.g., "&" in English).
    Symbol,
    /// No conjunction (e.g., "Smith, Jones").
    #[default]
    None,
}

/// Role display options.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct RoleOptions {
    /// Contributor roles for which to omit the role description.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omit: Vec<String>,
    /// Global role-label preset applied before legacy compatibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<RoleLabelPreset>,
    /// Global role label form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<String>,
    /// Global prefix for role labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Global suffix for role labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Formatting for specific roles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<HashMap<String, RoleRendering>>,
}

impl RoleOptions {
    /// Return the configured rendering override for a specific contributor role.
    pub fn role_rendering(
        &self,
        role: &crate::template::ContributorRole,
    ) -> Option<&RoleRendering> {
        self.roles.as_ref()?.get(role.as_str())
    }

    /// Resolve the effective label preset for a specific contributor role.
    pub fn effective_label_preset(
        &self,
        role: &crate::template::ContributorRole,
    ) -> Option<RoleLabelPreset> {
        self.role_rendering(role)
            .and_then(|rendering| rendering.preset)
            .or(self.preset)
            .or_else(|| {
                self.form
                    .as_deref()
                    .and_then(RoleLabelPreset::from_form_str)
            })
    }
}

/// Rendering options for contributor roles.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct RoleRendering {
    /// Per-role label preset override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<RoleLabelPreset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emph: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strong: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_caps: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_order: Option<crate::template::NameOrder>,
}

impl RoleRendering {
    /// Convert the rendering override to a generic rendering config.
    pub fn to_rendering(&self) -> crate::template::Rendering {
        crate::template::Rendering {
            emph: self.emph,
            strong: self.strong,
            small_caps: self.small_caps,
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
            ..Default::default()
        }
    }
}

/// When to use delimiter before last contributor.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum DelimiterPrecedesLast {
    AfterInvertedName,
    Always,
    Never,
    #[default]
    Contextual,
}

/// Et al. / list shortening options.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct ShortenListOptions {
    /// Minimum number of names to trigger shortening.
    pub min: u8,
    /// Number of names to show when shortened.
    pub use_first: u8,
    /// Number of names to show after the ellipsis (et-al-use-last).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_last: Option<u8>,
    /// How to render "and others".
    #[serde(default)]
    pub and_others: AndOtherOptions,
    /// When to use delimiter before last name.
    #[serde(default)]
    pub delimiter_precedes_last: DelimiterPrecedesLast,
    /// Minimum number of names to trigger shortening on subsequent cites.
    /// Defaults to `min` if not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_min: Option<u8>,
    /// Number of names to show when shortened on subsequent cites.
    /// Defaults to `use_first` if not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_use_first: Option<u8>,
}

impl Default for ShortenListOptions {
    fn default() -> Self {
        Self {
            min: 4,
            use_first: 1,
            use_last: None,
            and_others: AndOtherOptions::default(),
            delimiter_precedes_last: DelimiterPrecedesLast::default(),
            subsequent_min: None,
            subsequent_use_first: None,
        }
    }
}

/// How to render "and others" / et al.
#[derive(Debug, Default, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum AndOtherOptions {
    #[default]
    EtAl,
    Text,
}

fn default_contributor_delimiter() -> Option<String> {
    Some(", ".to_string())
}

fn is_default_contributor_delimiter(v: &Option<String>) -> bool {
    v.as_deref() == Some(", ")
}
