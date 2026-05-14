/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Public schema types for Citum styles, citations, references, and locales.

use indexmap::IndexMap;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::de::{DeserializeOwned, Error as _};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Compatibility facade merging data input types with style-specific logic.
#[allow(missing_docs, reason = "internal derives")]
pub mod citation {
    pub use crate::locale::locator::normalize_locator_text;
    pub use citum_schema_data::citation::*;
}

/// Bibliographic reference data types.
pub use citum_schema_data::reference;

/// Renderer for converting processor output to different formats.
pub mod renderer;
pub use renderer::Renderer;

/// Bibliography grouping and sorting specifications.
pub mod grouping;
/// Legacy CSL 1.0 compatibility types.
#[allow(missing_docs, reason = "internal derives")]
pub mod legacy;
/// Locale-specific terms and translations.
pub mod locale;
/// Style configuration options.
#[allow(missing_docs, reason = "internal derives")]
pub mod options;
/// Configuration presets for common styles.
pub mod presets;
/// Style base-inheritance mechanism (named compiled-in Style structs).
pub mod style_base;
/// Citation and bibliography template components.
#[allow(missing_docs, reason = "internal derives")]
pub mod template;

/// Embedded templates for priority styles (APA, Chicago, Vancouver, IEEE, Harvard).
pub mod embedded;

/// Style registry — discovery and alias resolution.
pub mod registry;

/// Declarative macros for AST and configurations.
pub mod macros;

/// Lint helpers for raw locales and styles.
pub mod lint;

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
mod reference_multilingual_tests;

pub use citation::{
    Citation, CitationItem, CitationMode, Citations, IntegralNameState, LocatorType, Position,
};
pub use citum_schema_data::{InputBibliography, InputBibliographyInfo};
pub use grouping::{
    BibliographyGroup, CitedStatus, FieldMatcher, GroupHeading, GroupSelector, GroupSort,
    GroupSortEntry, GroupSortKey, NameSortOrder, SortKey,
};
pub use legacy::{
    AndTerm, ConditionBlock, CslnInfo, CslnLocale, CslnNode, CslnStyle, DateBlock, DateForm,
    DateOptions, DatePartForm, DateParts, DelimiterPrecedes, ElseIfBranch, EtAlOptions,
    EtAlSubsequent, FontStyle, FontVariant, FontWeight, FormattingOptions, GroupBlock, ItemType,
    LabelForm, LabelOptions, NameAsSortOrder, NameMode, NamesBlock, NamesOptions, TermBlock,
    TextDecoration, Variable, VariableBlock, VerticalAlign,
};
pub use locale::Locale;
pub use options::TextCase;
pub use options::{BibliographyOptions, CitationOptions, Config};
pub use presets::{ContributorPreset, DatePreset, SortPreset, SubstitutePreset, TitlePreset};
pub use registry::{RegistryEntry, StyleRegistry};
pub use style_base::StyleBase;
pub use template::{
    Rendering, TemplateAddOperation, TemplateComponent, TemplateComponentSelector,
    TemplateContributor, TemplateDate, TemplateGroup, TemplateModifyOperation, TemplateNumber,
    TemplateRemoveOperation, TemplateTerm, TemplateTitle, TemplateVariable, TemplateVariant,
    TemplateVariantDiff, TypeSelector, WrapConfig, WrapPunctuation,
};

/// A named template (reusable sequence of components).
pub type Template = Vec<TemplateComponent>;

/// Type-specific template variants keyed by reference-type selector.
pub type TemplateVariants = IndexMap<TypeSelector, TemplateVariant>;

/// Canonical style resolution interfaces and error types.
pub use citum_resolver_api::{ResolutionError, ResolverError};

/// Resolver interface used by schema-layer style inheritance.
pub type StyleResolver = dyn citum_resolver_api::StyleResolver<Style = Style, Locale = Locale>;

/// Canonical Citum style schema version used when `Style.version` is omitted.
pub const STYLE_SCHEMA_VERSION: &str = "0.49.0";

/// Maximum accepted nesting depth for authored template groups and fallbacks.
pub const MAX_TEMPLATE_NESTING_DEPTH: usize = 64;

/// Maximum accepted authored template components in one style.
pub const MAX_TEMPLATE_COMPONENTS: usize = 16_384;

/// A schema version (major.minor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema), schemars(with = "String"))]
pub struct SchemaVersion {
    /// Major version number.
    pub major: u32,
    /// Minor version number (None if not provided in string).
    pub minor: Option<u32>,
    /// Patch version number (None if not provided in string).
    pub patch: Option<u32>,
}

impl SchemaVersion {
    /// Parse a version string into a `SchemaVersion`.
    ///
    /// Requires at least "X.Y". Supports "X.Y.Z".
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid version format
    /// or lacks the required minor version.
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(format!(
                "invalid version format (expected X.Y or X.Y.Z): \"{}\"",
                s
            ));
        }

        let major_str = parts
            .first()
            .ok_or_else(|| "missing major version".to_string())?;
        let major = major_str
            .parse::<u32>()
            .map_err(|_| format!("invalid major version: \"{}\"", major_str))?;

        let minor_str = parts
            .get(1)
            .ok_or_else(|| "missing minor version".to_string())?;
        let minor = Some(
            minor_str
                .parse::<u32>()
                .map_err(|_| format!("invalid minor version: \"{}\"", minor_str))?,
        );

        let patch = if let Some(patch_str) = parts.get(2) {
            Some(
                patch_str
                    .parse::<u32>()
                    .map_err(|_| format!("invalid patch version: \"{}\"", patch_str))?,
            )
        } else {
            None
        };

        Ok(SchemaVersion {
            major,
            minor,
            patch,
        })
    }
}

impl PartialOrd for SchemaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchemaVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {
                let self_minor = self.minor.unwrap_or(0);
                let other_minor = other.minor.unwrap_or(0);
                match self_minor.cmp(&other_minor) {
                    std::cmp::Ordering::Equal => {
                        let self_patch = self.patch.unwrap_or(0);
                        let other_patch = other.patch.unwrap_or(0);
                        self_patch.cmp(&other_patch)
                    }
                    ord => ord,
                }
            }
            ord => ord,
        }
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{}", minor)?;
            if let Some(patch) = self.patch {
                write!(f, ".{}", patch)?;
            }
        }
        Ok(())
    }
}

impl Default for SchemaVersion {
    #[allow(
        clippy::expect_used,
        reason = "STYLE_SCHEMA_VERSION is a canonical constant"
    )]
    fn default() -> Self {
        SchemaVersion::parse(STYLE_SCHEMA_VERSION).expect("STYLE_SCHEMA_VERSION is valid")
    }
}

impl<'de> serde::Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SchemaVersion::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for SchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// A non-fatal validation warning emitted by [`Style::validate`].
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaWarning {
    /// A `TypeSelector` references an unrecognized reference type name.
    ///
    /// This usually indicates a typo (e.g., `article_journal` instead of
    /// `article-journal`). The selector will silently match nothing at
    /// render time.
    UnknownTypeName {
        /// The unrecognized type name string.
        name: String,
        /// Human-readable location hint (e.g., `"bibliography.type-variants"`).
        location: String,
    },
}

impl std::fmt::Display for SchemaWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaWarning::UnknownTypeName { name, location } => {
                write!(
                    f,
                    "unknown reference type \"{name}\" in {location} \
                     (will silently match nothing; check for typos)"
                )
            }
        }
    }
}

/// The new Citum Style model.
///
/// This is the target schema for Citum, featuring declarative options
/// and simple template components instead of procedural conditionals.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Style {
    /// Style schema version.
    #[serde(default = "default_version")]
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
}

impl Style {
    /// Resolve this style into its final effective form by applying base inheritance.
    ///
    /// If the `extends` field is present, the base [`StyleBase`] is loaded
    /// and any explicit `options`, `citation`, or `bibliography` keys in the current
    /// style document are merged on top (taking ultimate precedence).
    ///
    /// Styles without a base still resolve Template V3 variants and scoped
    /// options, but do not merge any inherited style data.
    ///
    /// # Panics
    ///
    /// Panics when style resolution fails. Use [`Style::try_into_resolved`]
    /// to handle profile-contract and inheritance errors explicitly.
    #[must_use]
    #[allow(
        clippy::panic,
        reason = "Convenience API for infallible resolution contexts"
    )]
    pub fn into_resolved(self) -> Self {
        self.try_into_resolved()
            .unwrap_or_else(|err| panic!("style resolution failed: {err}"))
    }

    /// Resolve this style into its final effective form, returning validation errors.
    ///
    /// Unlike [`Style::into_resolved`], this preserves resolution failures as
    /// structured [`ResolutionError`] values.
    ///
    /// # Errors
    ///
    /// Returns an error when profile wrappers try to override template-bearing
    /// structure, when profile capability validation fails, or when inheritance
    /// loops are detected.
    pub fn try_into_resolved(self) -> Result<Self, ResolutionError> {
        self.try_into_resolved_with(None)
    }

    /// Resolve this style into its final effective form using an optional style resolver.
    ///
    /// The resolver is used for URI, registry, and remote `extends` references
    /// that cannot be satisfied by embedded bases alone.
    ///
    /// # Errors
    ///
    /// Returns an error when style inheritance or Template V3 variant
    /// resolution fails.
    pub fn try_into_resolved_with(
        self,
        resolver: Option<&StyleResolver>,
    ) -> Result<Self, ResolutionError> {
        self.try_into_resolved_recursive_with_depth(resolver, &mut HashSet::new(), 0)
    }

    /// Internal recursive resolver with loop protection.
    ///
    /// # Panics
    ///
    /// Panics when style resolution fails. Use
    /// [`Style::try_into_resolved_recursive`] to preserve errors.
    #[must_use]
    #[allow(
        clippy::panic,
        reason = "Convenience API for infallible resolution contexts"
    )]
    pub fn into_resolved_recursive(self, visited: &mut HashSet<String>) -> Self {
        self.try_into_resolved_recursive(visited)
            .unwrap_or_else(|err| panic!("style resolution failed: {err}"))
    }

    /// Internal recursive resolver with loop protection.
    ///
    /// # Errors
    ///
    /// Returns an error when profile wrappers violate the config-only
    /// contract, when profile capability validation fails, or when
    /// inheritance loops are detected.
    pub fn try_into_resolved_recursive(
        self,
        visited: &mut HashSet<String>,
    ) -> Result<Self, ResolutionError> {
        self.try_into_resolved_recursive_with(None, visited)
    }

    /// Internal recursive resolver with loop protection and optional external resolver.
    ///
    /// # Errors
    ///
    /// Returns an error when style inheritance or Template V3 variant
    /// resolution fails.
    pub fn try_into_resolved_recursive_with(
        self,
        resolver: Option<&StyleResolver>,
        visited: &mut HashSet<String>,
    ) -> Result<Self, ResolutionError> {
        self.try_into_resolved_recursive_with_depth(resolver, visited, 0)
    }

    /// Internal recursive resolver with depth limit.
    fn try_into_resolved_recursive_with_depth(
        self,
        resolver: Option<&StyleResolver>,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> Result<Self, ResolutionError> {
        const MAX_DEPTH: usize = 5;

        // Reject root styles whose declared engine compat range we don't satisfy
        // before we waste any work on inheritance or template variants.
        let root_label = self
            .info
            .id
            .as_deref()
            .or(self.info.title.as_deref())
            .unwrap_or("<root>");
        check_citum_version(root_label, &self.info)?;

        let Some(base_ref) = self.extends.clone() else {
            let mut style = self;
            resolve_style_template_variants(&mut style, None)?;
            options::scoped::apply_scoped_style_options(&mut style);
            return Ok(style);
        };

        if depth >= MAX_DEPTH {
            let uri = base_ref.key();
            return Err(ResolutionError::UriResolutionFailed {
                uri: uri.to_string(),
                reason: format!("inheritance chain exceeds maximum depth of {MAX_DEPTH}"),
            });
        }

        let key = base_ref.key().to_string();
        if visited.contains(&key) {
            return Err(ResolutionError::InheritanceLoop { base: key });
        }
        visited.insert(key);

        let is_profile = self.resolves_as_profile();
        let pin = self.extends_pin.clone();
        let mut effective = match base_ref {
            style_base::StyleReference::Base(base) => {
                if pin.is_some() {
                    return Err(ResolutionError::UriResolutionFailed {
                        uri: base.key().to_string(),
                        reason:
                            "extends-pin is only supported for URI-based parents (https://, cid:); \
                             builtin StyleBase parents are content-fixed already"
                                .to_string(),
                    });
                }
                base.try_resolve_with_visited(resolver, visited)?
            }
            style_base::StyleReference::Uri(ref uri) => {
                let base_style = resolve_style_reference_uri(uri, resolver)?;
                if let Some(ref expected) = pin {
                    verify_parent_pin(uri, &base_style, expected)?;
                }
                base_style.try_into_resolved_recursive_with_depth(resolver, visited, depth + 1)?
            }
        };
        if is_profile {
            self.validate_profile_shape()?;
        }

        let inherited_variants = inherited_variant_context(&effective);
        merge_style_overlay(&mut effective, &self);
        effective.version = self.version;
        effective.extends = self.extends;
        effective.extends_pin = self.extends_pin;
        effective.raw_yaml = self.raw_yaml;
        resolve_style_template_variants(&mut effective, inherited_variants.as_ref())?;
        options::scoped::apply_scoped_style_options(&mut effective);
        if is_profile {
            effective.extends = None;
        }

        Ok(effective)
    }

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
        let mut style: Style = serde_yaml::from_value(raw.clone())?;
        style.raw_yaml = Some(raw);
        style
            .validate_resource_limits()
            .map_err(serde_yaml::Error::custom)?;
        Ok(style)
    }

    /// Parse a Citum style from YAML bytes, preserving raw YAML for
    /// null-aware overlay merging during preset resolution.
    ///
    /// # Errors
    ///
    /// Returns a serde error if YAML parsing or deserialization fails.
    pub fn from_yaml_bytes(bytes: &[u8]) -> Result<Self, serde_yaml::Error> {
        let raw: serde_yaml::Value = serde_yaml::from_slice(bytes)?;
        let mut style: Style = serde_yaml::from_value(raw.clone())?;
        style.raw_yaml = Some(raw);
        style
            .validate_resource_limits()
            .map_err(serde_yaml::Error::custom)?;
        Ok(style)
    }

    /// Validate hard resource limits for style templates.
    ///
    /// # Errors
    ///
    /// Returns an error when authored template structure exceeds the maximum
    /// depth or component count accepted by the engine.
    pub fn validate_resource_limits(&self) -> Result<(), String> {
        let mut budget = TemplateResourceBudget::default();

        if let Some(templates) = &self.templates {
            for (name, template) in templates {
                budget.check_template(template, &format!("templates.{name}"), 0)?;
            }
        }
        if let Some(citation) = &self.citation {
            budget.check_citation_spec(citation, "citation", 0)?;
        }
        if let Some(bibliography) = &self.bibliography {
            budget.check_bibliography_spec(bibliography, "bibliography", 0)?;
        }

        Ok(())
    }

    /// Validate the style and return any non-fatal warnings.
    ///
    /// This method checks for issues that are syntactically valid but
    /// semantically suspect, such as unrecognized reference type names in
    /// `TypeSelector` values.
    ///
    /// Warnings do not prevent rendering; they are informational only.
    pub fn validate(&self) -> Vec<SchemaWarning> {
        let mut warnings = Vec::new();
        self.collect_type_selector_warnings(&mut warnings);
        warnings
    }

    /// Collect warnings for all `TypeSelector` values in the style.
    fn collect_type_selector_warnings(&self, warnings: &mut Vec<SchemaWarning>) {
        if let Some(bib) = &self.bibliography
            && let Some(type_variants) = &bib.type_variants
        {
            for selector in type_variants.keys() {
                for name in selector.unknown_type_names() {
                    warnings.push(SchemaWarning::UnknownTypeName {
                        name: name.to_string(),
                        location: "bibliography.type-variants".to_string(),
                    });
                }
            }
        }
        if let Some(cit) = &self.citation {
            collect_citation_spec_warnings(cit, "citation", warnings);
        }
    }

    fn style_kind(&self) -> Option<registry::StyleKind> {
        let id = self.info.id.as_deref()?;
        registry::StyleRegistry::load_default()
            .resolve(id)
            .and_then(|entry| entry.kind.clone())
    }

    fn resolves_as_profile(&self) -> bool {
        self.style_kind() == Some(registry::StyleKind::Profile)
    }

    pub(crate) fn validate_profile_shape(&self) -> Result<(), ResolutionError> {
        if self.templates.is_some() || yaml_path_present(self.raw_yaml.as_ref(), &["templates"]) {
            return Err(ResolutionError::InvalidProfileOverride {
                location: "templates".to_string(),
            });
        }

        for path in [
            ["citation", "template"].as_slice(),
            ["citation", "type-variants"].as_slice(),
            ["citation", "integral", "template"].as_slice(),
            ["citation", "integral", "type-variants"].as_slice(),
            ["citation", "non-integral", "template"].as_slice(),
            ["citation", "non-integral", "type-variants"].as_slice(),
            ["bibliography", "template"].as_slice(),
            ["bibliography", "type-variants"].as_slice(),
        ] {
            if yaml_path_present(self.raw_yaml.as_ref(), path) {
                return Err(ResolutionError::InvalidProfileOverride {
                    location: path.join("."),
                });
            }
        }

        Ok(())
    }
}

/// Collect warnings from a `CitationSpec` and its sub-specs.
fn collect_citation_spec_warnings(
    spec: &CitationSpec,
    location: &str,
    warnings: &mut Vec<SchemaWarning>,
) {
    if let Some(type_variants) = &spec.type_variants {
        for selector in type_variants.keys() {
            for name in selector.unknown_type_names() {
                warnings.push(SchemaWarning::UnknownTypeName {
                    name: name.to_string(),
                    location: format!("{location}.type-variants"),
                });
            }
        }
    }
    // Recurse into sub-specs
    for (sub_name, sub_spec) in [
        ("integral", spec.integral.as_deref()),
        ("non-integral", spec.non_integral.as_deref()),
        ("subsequent", spec.subsequent.as_deref()),
        ("ibid", spec.ibid.as_deref()),
    ]
    .into_iter()
    .filter_map(|(n, s)| s.map(|s| (n, s)))
    {
        collect_citation_spec_warnings(sub_spec, &format!("{location}.{sub_name}"), warnings);
    }
}

#[derive(Default)]
struct TemplateResourceBudget {
    component_count: usize,
}

impl TemplateResourceBudget {
    fn check_template(
        &mut self,
        template: &[TemplateComponent],
        location: &str,
        depth: usize,
    ) -> Result<(), String> {
        if depth > MAX_TEMPLATE_NESTING_DEPTH {
            return Err(format!(
                "{location} exceeds maximum template nesting depth of {MAX_TEMPLATE_NESTING_DEPTH}"
            ));
        }
        for component in template {
            self.check_component(component, location, depth)?;
        }
        Ok(())
    }

    fn check_component(
        &mut self,
        component: &TemplateComponent,
        location: &str,
        depth: usize,
    ) -> Result<(), String> {
        self.component_count = self.component_count.saturating_add(1);
        if self.component_count > MAX_TEMPLATE_COMPONENTS {
            return Err(format!(
                "style exceeds maximum template component count of {MAX_TEMPLATE_COMPONENTS}"
            ));
        }

        match component {
            TemplateComponent::Date(date) => {
                if let Some(fallback) = &date.fallback {
                    self.check_template(fallback, &format!("{location}.date.fallback"), depth + 1)?;
                }
            }
            TemplateComponent::Group(group) => {
                self.check_template(&group.group, &format!("{location}.group"), depth + 1)?;
            }
            TemplateComponent::Contributor(_)
            | TemplateComponent::Title(_)
            | TemplateComponent::Number(_)
            | TemplateComponent::Variable(_)
            | TemplateComponent::Term(_) => {}
        }

        Ok(())
    }

    fn check_variant(
        &mut self,
        variant: &TemplateVariant,
        location: &str,
        depth: usize,
    ) -> Result<(), String> {
        match variant {
            TemplateVariant::Full(template) => self.check_template(template, location, depth),
            TemplateVariant::Diff(diff) => {
                for (index, add) in diff.add.iter().enumerate() {
                    self.check_component(
                        &add.component,
                        &format!("{location}.add[{index}].component"),
                        depth,
                    )?;
                }
                Ok(())
            }
        }
    }

    fn check_variants(
        &mut self,
        variants: &TemplateVariants,
        location: &str,
        depth: usize,
    ) -> Result<(), String> {
        for (selector, variant) in variants {
            self.check_variant(variant, &format!("{location}.{selector:?}"), depth)?;
        }
        Ok(())
    }

    fn check_locales(
        &mut self,
        locales: &[LocalizedTemplateSpec],
        location: &str,
        depth: usize,
    ) -> Result<(), String> {
        for (index, locale) in locales.iter().enumerate() {
            self.check_template(
                &locale.template,
                &format!("{location}[{index}].template"),
                depth,
            )?;
        }
        Ok(())
    }

    fn check_citation_spec(
        &mut self,
        spec: &CitationSpec,
        location: &str,
        depth: usize,
    ) -> Result<(), String> {
        if let Some(template) = &spec.template {
            self.check_template(template, &format!("{location}.template"), depth)?;
        }
        if let Some(locales) = &spec.locales {
            self.check_locales(locales, &format!("{location}.locales"), depth)?;
        }
        if let Some(variants) = &spec.type_variants {
            self.check_variants(variants, &format!("{location}.type-variants"), depth)?;
        }
        for (sub_name, sub_spec) in [
            ("integral", spec.integral.as_deref()),
            ("non-integral", spec.non_integral.as_deref()),
            ("subsequent", spec.subsequent.as_deref()),
            ("ibid", spec.ibid.as_deref()),
        ]
        .into_iter()
        .filter_map(|(n, s)| s.map(|s| (n, s)))
        {
            self.check_citation_spec(sub_spec, &format!("{location}.{sub_name}"), depth + 1)?;
        }
        Ok(())
    }

    fn check_bibliography_spec(
        &mut self,
        spec: &BibliographySpec,
        location: &str,
        depth: usize,
    ) -> Result<(), String> {
        if let Some(template) = &spec.template {
            self.check_template(template, &format!("{location}.template"), depth)?;
        }
        if let Some(locales) = &spec.locales {
            self.check_locales(locales, &format!("{location}.locales"), depth)?;
        }
        if let Some(variants) = &spec.type_variants {
            self.check_variants(variants, &format!("{location}.type-variants"), depth)?;
        }
        Ok(())
    }
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
mod security_resource_tests {
    use super::*;

    fn nested_group(depth: usize) -> TemplateComponent {
        if depth == 0 {
            TemplateComponent::default()
        } else {
            TemplateComponent::Group(TemplateGroup {
                group: vec![nested_group(depth - 1)],
                ..TemplateGroup::default()
            })
        }
    }

    #[test]
    fn validate_resource_limits_rejects_deeply_nested_templates() {
        let style = Style {
            bibliography: Some(BibliographySpec {
                template: Some(vec![nested_group(MAX_TEMPLATE_NESTING_DEPTH + 1)]),
                ..BibliographySpec::default()
            }),
            ..Style::default()
        };

        let err = style
            .validate_resource_limits()
            .expect_err("deep template must be rejected");

        assert!(err.contains("maximum template nesting depth"));
    }

    #[test]
    fn validate_resource_limits_rejects_too_many_components() {
        let style = Style {
            bibliography: Some(BibliographySpec {
                template: Some(vec![
                    TemplateComponent::default();
                    MAX_TEMPLATE_COMPONENTS + 1
                ]),
                ..BibliographySpec::default()
            }),
            ..Style::default()
        };

        let err = style
            .validate_resource_limits()
            .expect_err("oversized template must be rejected");

        assert!(err.contains("maximum template component count"));
    }
}

fn yaml_path_present(value: Option<&serde_yaml::Value>, path: &[&str]) -> bool {
    let Some(mut current) = value else {
        return false;
    };
    for segment in path {
        let serde_yaml::Value::Mapping(map) = current else {
            return false;
        };
        let key = serde_yaml::Value::String((*segment).to_string());
        let Some(next) = map.get(&key) else {
            return false;
        };
        current = next;
    }
    true
}

/// Compute the canonical CIDv1 (raw codec, sha-256, base32 lower) for the
/// bytes that represent a parent style.
///
/// Used to back `extends-pin` enforcement at the schema layer. The encoding
/// matches `citum_store::cid::compute_style_cid`; the function lives here
/// (rather than depending on `citum_store`) so the schema layer remains free
/// of network-feature crates.
#[allow(
    clippy::panic,
    reason = "Multihash::wrap on a 32-byte SHA-256 digest is infallible by construction"
)]
fn schema_compute_style_cid(bytes: &[u8]) -> String {
    use cid::Cid;
    use multihash::Multihash;
    use sha2::{Digest, Sha256};

    const RAW_CODEC: u64 = 0x55;
    const SHA256_CODE: u64 = 0x12;

    let digest: [u8; 32] = Sha256::digest(bytes).into();
    let mh = Multihash::<64>::wrap(SHA256_CODE, &digest)
        .unwrap_or_else(|_| panic!("32-byte SHA-256 digest fits in Multihash<64>"));
    Cid::new_v1(RAW_CODEC, mh).to_string()
}

/// Normalize a CID-or-`cid:`-URI to its canonical lowercase string form.
fn schema_canonicalize_cid(s: &str) -> Result<String, ResolutionError> {
    use cid::Cid;
    let trimmed = s.strip_prefix("cid:").unwrap_or(s);
    let cid: Cid =
        trimmed
            .parse()
            .map_err(|err: cid::Error| ResolutionError::UriResolutionFailed {
                uri: s.to_string(),
                reason: format!("invalid CID '{s}': {err}"),
            })?;
    Ok(cid.to_string())
}

/// Verify that the parent style's serialized form matches `expected_pin`.
///
/// Re-serializes the parsed `Style` to its canonical YAML form and computes
/// its CIDv1. Mismatch produces [`ResolutionError::IntegrityFailure`]. This
/// is a best-effort check at the schema layer; for byte-exact verification
/// of the originally-fetched bytes, route through
/// `citum_store::fetch_and_verify_bytes` before parsing.
fn verify_parent_pin(uri: &str, parent: &Style, expected_pin: &str) -> Result<(), ResolutionError> {
    let expected = schema_canonicalize_cid(expected_pin)?;
    let bytes =
        serde_yaml::to_string(parent).map_err(|err| ResolutionError::UriResolutionFailed {
            uri: uri.to_string(),
            reason: format!("re-serialize for extends-pin verification: {err}"),
        })?;
    let actual = schema_compute_style_cid(bytes.as_bytes());
    if actual == expected {
        Ok(())
    } else {
        Err(ResolutionError::IntegrityFailure {
            uri: uri.to_string(),
            expected,
            actual,
        })
    }
}

/// Apply an `info.citum-version` requirement check against the running
/// engine version (CARGO_PKG_VERSION).
///
/// Returns `Ok(())` when the field is absent or the requirement is satisfied;
/// returns [`ResolutionError::VersionMismatch`] otherwise.
///
/// # Errors
///
/// Returns [`ResolutionError::VersionMismatch`] when the running engine
/// version does not satisfy the style's declared `citum-version` requirement.
/// Returns [`ResolutionError::UriResolutionFailed`] when the requirement
/// string itself fails to parse as a semver `VersionReq`, or when the
/// running engine's `CARGO_PKG_VERSION` cannot be parsed (this latter case
/// is structurally impossible at runtime but is preserved as a typed error
/// rather than a panic).
pub fn check_citum_version(uri: &str, info: &StyleInfo) -> Result<(), ResolutionError> {
    let Some(req_str) = info.citum_version.as_ref() else {
        return Ok(());
    };
    let req =
        semver::VersionReq::parse(req_str).map_err(|err| ResolutionError::UriResolutionFailed {
            uri: uri.to_string(),
            reason: format!("invalid `info.citum-version` requirement '{req_str}': {err}"),
        })?;
    let engine_str = env!("CARGO_PKG_VERSION");
    let engine =
        semver::Version::parse(engine_str).map_err(|err| ResolutionError::UriResolutionFailed {
            uri: uri.to_string(),
            reason: format!("unparseable engine version `{engine_str}`: {err}"),
        })?;
    if req.matches(&engine) {
        Ok(())
    } else {
        Err(ResolutionError::VersionMismatch {
            uri: uri.to_string(),
            required: req_str.clone(),
            declared: engine_str.to_string(),
        })
    }
}

fn resolve_style_reference_uri(
    uri: &str,
    resolver: Option<&StyleResolver>,
) -> Result<Style, ResolutionError> {
    if let Some(resolver) = resolver {
        let style = resolver
            .resolve_style(uri)
            .map_err(|e| ResolutionError::from_resolver_error(uri, e))?;
        check_citum_version(uri, &style.info)?;
        return Ok(style);
    }

    let Some(raw_path) = uri.strip_prefix("file://") else {
        return Err(ResolutionError::UriResolutionFailed {
            uri: uri.to_string(),
            reason: "unsupported scheme; an external style resolver is required".to_string(),
        });
    };
    let path = std::path::Path::new(raw_path);
    let bytes = std::fs::read(path).map_err(|e| ResolutionError::UriResolutionFailed {
        uri: uri.to_string(),
        reason: e.to_string(),
    })?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");
    let style: Style = match ext {
        "cbor" => ciborium::de::from_reader(std::io::Cursor::new(&bytes)).map_err(|e| {
            ResolutionError::UriResolutionFailed {
                uri: uri.to_string(),
                reason: e.to_string(),
            }
        })?,
        "json" => {
            serde_json::from_slice(&bytes).map_err(|e| ResolutionError::UriResolutionFailed {
                uri: uri.to_string(),
                reason: e.to_string(),
            })?
        }
        _ => Style::from_yaml_bytes(&bytes).map_err(|e| ResolutionError::UriResolutionFailed {
            uri: uri.to_string(),
            reason: e.to_string(),
        })?,
    };
    check_citum_version(uri, &style.info)?;
    Ok(style)
}

#[derive(Clone, Default)]
struct StyleVariantContext {
    citation: Option<CitationVariantContext>,
    bibliography: Option<TemplateVariants>,
}

#[derive(Clone, Default)]
struct CitationVariantContext {
    type_variants: Option<TemplateVariants>,
    integral: Option<Box<CitationVariantContext>>,
    non_integral: Option<Box<CitationVariantContext>>,
    subsequent: Option<Box<CitationVariantContext>>,
    ibid: Option<Box<CitationVariantContext>>,
}

fn inherited_variant_context(style: &Style) -> Option<StyleVariantContext> {
    let context = StyleVariantContext {
        citation: style.citation.as_ref().map(citation_variant_context),
        bibliography: style
            .bibliography
            .as_ref()
            .and_then(|bib| bib.type_variants.clone()),
    };
    (context.citation.is_some() || context.bibliography.is_some()).then_some(context)
}

fn citation_variant_context(spec: &CitationSpec) -> CitationVariantContext {
    CitationVariantContext {
        type_variants: spec.type_variants.clone(),
        integral: spec
            .integral
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
        non_integral: spec
            .non_integral
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
        subsequent: spec
            .subsequent
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
        ibid: spec
            .ibid
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
    }
}

fn resolve_style_template_variants(
    style: &mut Style,
    inherited: Option<&StyleVariantContext>,
) -> Result<(), ResolutionError> {
    if let Some(citation) = style.citation.as_mut() {
        resolve_citation_template_variants(
            citation,
            inherited.and_then(|context| context.citation.as_ref()),
            "citation",
            None,
        )?;
    }
    if let Some(bibliography) = style.bibliography.as_mut() {
        let section_template = bibliography.resolve_template();
        resolve_template_variant_map(
            bibliography.type_variants.as_mut(),
            section_template.as_deref(),
            inherited.and_then(|context| context.bibliography.as_ref()),
            "bibliography.type-variants",
        )?;
    }
    Ok(())
}

fn resolve_citation_template_variants(
    spec: &mut CitationSpec,
    inherited: Option<&CitationVariantContext>,
    location: &str,
    fallback_template: Option<&[TemplateComponent]>,
) -> Result<(), ResolutionError> {
    let section_template = spec.resolve_template();
    let effective_section_template = section_template.as_deref().or(fallback_template);
    resolve_template_variant_map(
        spec.type_variants.as_mut(),
        effective_section_template,
        inherited.and_then(|context| context.type_variants.as_ref()),
        &format!("{location}.type-variants"),
    )?;

    for (name, child, inherited_child) in [
        (
            "integral",
            spec.integral.as_deref_mut(),
            inherited.and_then(|context| context.integral.as_deref()),
        ),
        (
            "non-integral",
            spec.non_integral.as_deref_mut(),
            inherited.and_then(|context| context.non_integral.as_deref()),
        ),
        (
            "subsequent",
            spec.subsequent.as_deref_mut(),
            inherited.and_then(|context| context.subsequent.as_deref()),
        ),
        (
            "ibid",
            spec.ibid.as_deref_mut(),
            inherited.and_then(|context| context.ibid.as_deref()),
        ),
    ] {
        if let Some(child) = child {
            resolve_citation_template_variants(
                child,
                inherited_child,
                &format!("{location}.{name}"),
                effective_section_template,
            )?;
        }
    }
    Ok(())
}

fn resolve_template_variant_map(
    variants: Option<&mut TemplateVariants>,
    section_template: Option<&[TemplateComponent]>,
    inherited: Option<&TemplateVariants>,
    location: &str,
) -> Result<(), ResolutionError> {
    let Some(variants) = variants else {
        return Ok(());
    };
    let original = variants.clone();
    let mut resolved = TemplateVariants::new();
    let mut visiting = HashSet::new();

    for selector in original.keys() {
        let template = resolve_template_variant(
            selector,
            &original,
            &mut resolved,
            inherited,
            section_template,
            location,
            &mut visiting,
        )?;
        resolved.insert(selector.clone(), TemplateVariant::Full(template));
    }

    *variants = resolved;
    Ok(())
}

fn resolve_template_variant(
    selector: &TypeSelector,
    original: &TemplateVariants,
    resolved: &mut TemplateVariants,
    inherited: Option<&TemplateVariants>,
    section_template: Option<&[TemplateComponent]>,
    location: &str,
    visiting: &mut HashSet<TypeSelector>,
) -> Result<Template, ResolutionError> {
    let variant_location = format!("{location}[{selector}]");
    if let Some(template) = resolved
        .get(selector)
        .and_then(TemplateVariant::as_template)
        .map(<[TemplateComponent]>::to_vec)
    {
        return Ok(template);
    }

    if !visiting.insert(selector.clone()) {
        return Err(ResolutionError::TemplateVariantCycle {
            location: variant_location,
            selector: selector.to_string(),
        });
    }

    let variant =
        original
            .get(selector)
            .ok_or_else(|| ResolutionError::MissingTemplateVariantParent {
                location: variant_location.clone(),
                selector: selector.to_string(),
            })?;

    let template = match variant {
        TemplateVariant::Full(template) => template.clone(),
        TemplateVariant::Diff(diff) => {
            let mut parent = resolve_variant_parent_template(
                selector,
                diff,
                original,
                resolved,
                inherited,
                section_template,
                &variant_location,
                visiting,
            )?;
            apply_template_variant_diff(&mut parent, diff, &variant_location)?;
            parent
        }
    };

    visiting.remove(selector);
    Ok(template)
}

#[allow(
    clippy::too_many_arguments,
    reason = "Template variant resolution needs explicit inherited and local context."
)]
fn resolve_variant_parent_template(
    selector: &TypeSelector,
    diff: &TemplateVariantDiff,
    original: &TemplateVariants,
    resolved: &mut TemplateVariants,
    inherited: Option<&TemplateVariants>,
    section_template: Option<&[TemplateComponent]>,
    location: &str,
    visiting: &mut HashSet<TypeSelector>,
) -> Result<Template, ResolutionError> {
    if let Some(parent_selector) = &diff.extends {
        if parent_selector != selector && original.contains_key(parent_selector) {
            return resolve_template_variant(
                parent_selector,
                original,
                resolved,
                inherited,
                section_template,
                location,
                visiting,
            );
        }
        return inherited
            .and_then(|variants| variants.get(parent_selector))
            .and_then(TemplateVariant::as_template)
            .map(<[TemplateComponent]>::to_vec)
            .ok_or_else(|| ResolutionError::MissingTemplateVariantParent {
                location: location.to_string(),
                selector: parent_selector.to_string(),
            });
    }

    inherited
        .and_then(|variants| variants.get(selector))
        .and_then(TemplateVariant::as_template)
        .map(<[TemplateComponent]>::to_vec)
        .or_else(|| section_template.map(<[TemplateComponent]>::to_vec))
        .ok_or_else(|| ResolutionError::MissingTemplateVariantParent {
            location: location.to_string(),
            selector: selector.to_string(),
        })
}

fn apply_template_variant_diff(
    template: &mut Template,
    diff: &TemplateVariantDiff,
    location: &str,
) -> Result<(), ResolutionError> {
    for op in &diff.modify {
        let index = find_required_anchor(template, &op.match_selector, location)?;
        if let Some(component) = template.get_mut(index) {
            if let Some(label_form) = op.label_form.clone()
                && let TemplateComponent::Number(number) = component
            {
                number.label_form = Some(label_form);
            }
            component.rendering_mut().merge(&op.rendering);
        }
    }
    for op in &diff.remove {
        let index = find_required_anchor(template, &op.match_selector, location)?;
        template.remove(index);
    }
    for op in &diff.add {
        let anchor = match (&op.before, &op.after) {
            (Some(selector), None) => Some((selector, false)),
            (None, Some(selector)) => Some((selector, true)),
            _ => {
                return Err(ResolutionError::InvalidTemplateVariantAdd {
                    location: location.to_string(),
                });
            }
        };
        let Some((selector, insert_after)) = anchor else {
            return Err(ResolutionError::InvalidTemplateVariantAdd {
                location: location.to_string(),
            });
        };
        let anchor_index = find_required_anchor(template, selector, location)?;
        let insert_at = if insert_after {
            anchor_index.saturating_add(1)
        } else {
            anchor_index
        };
        template.insert(insert_at, op.component.clone());
    }
    Ok(())
}

fn find_required_anchor(
    template: &[TemplateComponent],
    selector: &TemplateComponentSelector,
    location: &str,
) -> Result<usize, ResolutionError> {
    find_optional_anchor(template, selector, location)?.ok_or_else(|| {
        ResolutionError::TemplateVariantAnchorNotFound {
            location: location.to_string(),
        }
    })
}

fn find_optional_anchor(
    template: &[TemplateComponent],
    selector: &TemplateComponentSelector,
    location: &str,
) -> Result<Option<usize>, ResolutionError> {
    if selector.is_empty() {
        return Err(ResolutionError::TemplateVariantAmbiguousAnchor {
            location: location.to_string(),
        });
    }
    let mut matches = template
        .iter()
        .enumerate()
        .filter_map(|(index, component)| selector.matches(component).then_some(index));
    let first = matches.next();
    if matches.next().is_some() {
        return Err(ResolutionError::TemplateVariantAmbiguousAnchor {
            location: location.to_string(),
        });
    }
    Ok(first)
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Internal merging logic ensures presence of re-parsed values"
)]
pub(crate) fn merge_style_overlay(base: &mut Style, overlay: &Style) {
    if !overlay.info.is_empty() {
        base.info = merge_serialized(base.info.clone(), &overlay.info);
    }

    if let Some(templates) = &overlay.templates {
        base.templates = Some(match &base.templates {
            Some(existing) => merge_serialized(existing.clone(), templates),
            None => templates.clone(),
        });
    }

    if let Some(options) = &overlay.options {
        match &mut base.options {
            Some(existing) => existing.merge(options),
            None => base.options = Some(options.clone()),
        }
    }

    let raw_citation = overlay.raw_yaml.as_ref().and_then(|v| v.get("citation"));
    if raw_citation.is_some() || overlay.citation.is_some() {
        base.citation = Some(match (&base.citation, raw_citation) {
            (Some(existing), Some(raw)) => merge_serialized_value(existing.clone(), raw),
            (Some(existing), None) => {
                merge_serialized(existing.clone(), overlay.citation.as_ref().unwrap())
            }
            (None, Some(raw)) => serde_yaml::from_value(raw.clone()).expect("citation parses"),
            (None, None) => overlay.citation.clone().unwrap(),
        });
    }

    let raw_bibliography = overlay
        .raw_yaml
        .as_ref()
        .and_then(|v| v.get("bibliography"));
    if raw_bibliography.is_some() || overlay.bibliography.is_some() {
        base.bibliography = Some(match (&base.bibliography, raw_bibliography) {
            (Some(existing), Some(raw)) => merge_serialized_value(existing.clone(), raw),
            (Some(existing), None) => {
                merge_serialized(existing.clone(), overlay.bibliography.as_ref().unwrap())
            }
            (None, Some(raw)) => serde_yaml::from_value(raw.clone()).expect("bibliography parses"),
            (None, None) => overlay.bibliography.clone().unwrap(),
        });
    }

    if let Some(custom) = &overlay.custom {
        base.custom = Some(match &base.custom {
            Some(existing) => merge_serialized(existing.clone(), custom),
            None => custom.clone(),
        });
    }
}

#[allow(clippy::expect_used, reason = "T must be serializable to YAML")]
pub(crate) fn merge_serialized<T>(base: T, overlay: &T) -> T
where
    T: Clone + DeserializeOwned + Serialize,
{
    let overlay_value = serde_yaml::to_value(overlay).expect("serializable overlay");
    merge_serialized_value(base, &overlay_value)
}

#[allow(
    clippy::expect_used,
    reason = "T must be serializable and merged values must match schema"
)]
pub(crate) fn merge_serialized_value<T>(base: T, overlay: &serde_yaml::Value) -> T
where
    T: Clone + DeserializeOwned + Serialize,
{
    let mut base_value = serde_yaml::to_value(base).expect("serializable base");
    merge_yaml_value(&mut base_value, overlay);
    serde_yaml::from_value(base_value).expect("merged value matches schema")
}

pub(crate) fn merge_yaml_value(base: &mut serde_yaml::Value, overlay: &serde_yaml::Value) {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_yaml_value(base_value, overlay_value);
                } else {
                    base_map.insert(key.clone(), overlay_value.clone());
                }
            }
        }
        (base_value, overlay_value) => {
            *base_value = overlay_value.clone();
        }
    }
}

fn default_version() -> SchemaVersion {
    SchemaVersion::default()
}

/// Available embedded template presets.
///
/// These reference battle-tested templates for common citation styles.
/// See `citum_schema::embedded` for the actual template implementations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TemplatePreset {
    /// APA 7th edition (author-date)
    Apa,
    /// Chicago Manual of Style (author-date)
    ChicagoAuthorDate,
    /// Vancouver (numeric)
    Vancouver,
    /// IEEE (numeric)
    Ieee,
    /// Harvard/Elsevier (author-date)
    Harvard,
    /// Numeric citation number only (citation-focused preset)
    NumericCitation,
}

/// A reference to a template, which can be either a named builtin preset
/// or a URI (e.g., `file://...`, `@hub/...`, `https://...`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum TemplateReference {
    /// A named builtin template preset.
    Preset(TemplatePreset),
    /// A URI reference to a remote or local template.
    Uri(String),
}

impl From<TemplatePreset> for TemplateReference {
    fn from(preset: TemplatePreset) -> Self {
        TemplateReference::Preset(preset)
    }
}

impl TemplatePreset {
    /// Resolve this preset to a citation template.
    pub fn citation_template(&self) -> Template {
        match self {
            TemplatePreset::Apa => embedded::apa_citation(),
            TemplatePreset::ChicagoAuthorDate => embedded::chicago_author_date_citation(),
            TemplatePreset::Vancouver => embedded::vancouver_citation(),
            TemplatePreset::Ieee => embedded::ieee_citation(),
            TemplatePreset::Harvard => embedded::harvard_citation(),
            TemplatePreset::NumericCitation => embedded::numeric_citation(),
        }
    }

    /// Resolve this preset to a bibliography template.
    pub fn bibliography_template(&self) -> Template {
        match self {
            TemplatePreset::Apa => embedded::apa_bibliography(),
            TemplatePreset::ChicagoAuthorDate => embedded::chicago_author_date_bibliography(),
            TemplatePreset::Vancouver => embedded::vancouver_bibliography(),
            TemplatePreset::Ieee => embedded::ieee_bibliography(),
            TemplatePreset::Harvard => embedded::harvard_bibliography(),
            // Citation-focused preset; Vancouver bibliography is the closest numeric fallback.
            TemplatePreset::NumericCitation => embedded::vancouver_bibliography(),
        }
    }
}

/// Locale-scoped template override with optional fallback behavior.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LocalizedTemplateSpec {
    /// Language tags that should select this template override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<Vec<String>>,
    /// Whether this override is the fallback when no locale matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// Template used when this localized override is selected.
    pub template: Template,
}

fn locale_matches(targets: &[String], language: &str) -> bool {
    let primary = language.split('-').next().unwrap_or(language);
    targets.iter().any(|candidate| {
        candidate == language || candidate.split('-').next().unwrap_or(candidate) == primary
    })
}

/// Citation collapse behavior for multi-item citations.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum CitationCollapse {
    /// Collapse adjacent citation numbers into a numeric range such as `1–3`.
    CitationNumber,
}

/// Text-case transform applied when a citation renders at note start.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum NoteStartTextCase {
    /// Uppercase the first character of the rendered citation.
    CapitalizeFirst,
    /// Lowercase the rendered citation text.
    Lowercase,
}

/// Citation specification.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CitationSpec {
    /// Citation-specific option overrides merged over the style config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<CitationOptions>,
    /// Reference to an embedded template preset or external template.
    ///
    /// If both `template-ref` and `template` are present, `template` takes precedence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_ref: Option<TemplateReference>,
    /// Default template when no localized override is selected.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub template: Option<Template>,
    /// Locale-specific template overrides checked before the default template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locales: Option<Vec<LocalizedTemplateSpec>>,
    /// Type-specific template overrides for citations. When present, replaces
    /// the default citation template for references of the specified types.
    /// Type-variant lookup happens after mode (integral/non-integral) resolution.
    /// If both the main spec and the active mode sub-spec have a `type-variants`
    /// entry for the same type, the mode-specific one wins.
    #[serde(skip_serializing_if = "Option::is_none", rename = "type-variants")]
    pub type_variants: Option<TemplateVariants>,
    /// Wrap the entire citation in punctuation. Preferred over prefix/suffix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<template::WrapConfig>,
    /// Prefix for the citation (use only when `wrap` doesn't suffice, e.g., " (" or "[Ref ").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Suffix for the citation (use only when `wrap` doesn't suffice).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Delimiter between components within a single citation item (e.g., ", " or " ").
    /// Defaults to ", ".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
    /// Delimiter between multiple citation items (e.g., "; ").
    /// Defaults to "; ".
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "multi-cite-delimiter")]
    pub multi_cite_delimiter: Option<String>,
    /// Optional collapse behavior for adjacent multi-item citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse: Option<CitationCollapse>,
    /// Optional citation sorting specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<grouping::GroupSortEntry>,
    /// Configuration for integral (narrative) citations (e.g., "Smith (2020)").
    /// Overrides fields from the main citation spec when mode is Integral.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integral: Option<Box<CitationSpec>>,
    /// Configuration for non-integral (parenthetical) citations (e.g., "(Smith, 2020)").
    /// Overrides fields from the main citation spec when mode is NonIntegral.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_integral: Option<Box<CitationSpec>>,
    /// Configuration for subsequent citations.
    /// Overrides fields from the main citation spec when position is Subsequent.
    /// Useful for short-form citations in note-based styles or author-date styles
    /// that show abbreviated citations after the first mention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent: Option<Box<CitationSpec>>,
    /// Configuration for ibid citations (ibid or ibid with locator).
    /// Overrides fields from the main citation spec when position is Ibid or IbidWithLocator.
    /// If present, takes precedence over `subsequent` for these positions.
    /// Allows compact rendering like "ibid." or "ibid., p. 45".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ibid: Option<Box<CitationSpec>>,
    /// Optional text-case transform for standalone note-start citation output.
    ///
    /// This is a style-owned rendering dimension layered on top of the
    /// existing repeated-note state, not a new citation `Position`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_start_text_case: Option<NoteStartTextCase>,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

impl CitationSpec {
    /// Resolve the effective template for this citation.
    ///
    /// Returns the explicit `template` if present, otherwise resolves `template-ref`.
    /// Returns `None` if neither is specified.
    pub fn resolve_template(&self) -> Option<Template> {
        self.template.clone().or_else(|| {
            self.template_ref.as_ref().and_then(|r| match r {
                TemplateReference::Preset(p) => Some(p.citation_template()),
                TemplateReference::Uri(_) => {
                    // TODO (Phase 2): Emit validation warning for unresolved URIs
                    None
                }
            })
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

    /// Resolve the template for a given reference type and language.
    ///
    /// First checks `type_variants` for an entry matching `ref_type`.
    /// Falls back to `resolve_template_for_language` if no type-specific
    /// template is found.
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

    /// Resolve the effective spec for a given citation mode.
    ///
    /// If a mode-specific spec exists (e.g., `integral`), it merges with and overrides
    /// the base spec.
    pub fn resolve_for_mode(
        &self,
        mode: &crate::citation::CitationMode,
    ) -> std::borrow::Cow<'_, CitationSpec> {
        use crate::citation::CitationMode;
        let mode_spec = match mode {
            CitationMode::Integral => self.integral.as_ref(),
            CitationMode::NonIntegral => self.non_integral.as_ref(),
        };

        match mode_spec {
            Some(spec) => {
                // Merge logic: mode specific > base
                let mut merged = self.clone();
                // We don't want to recurse infinitely or keep the mode specs in the merged result
                merged.integral = None;
                merged.non_integral = None;

                if spec.options.is_some() {
                    merged.options = spec.options.clone();
                }
                if spec.template_ref.is_some() {
                    merged.template_ref = spec.template_ref.clone();
                }
                if spec.template.is_some() {
                    merged.template = spec.template.clone();
                }
                if spec.locales.is_some() {
                    merged.locales = spec.locales.clone();
                }
                if spec.type_variants.is_some() {
                    merged.type_variants = spec.type_variants.clone();
                }
                if spec.wrap.is_some() {
                    merged.wrap = spec.wrap.clone();
                }
                if spec.prefix.is_some() {
                    merged.prefix = spec.prefix.clone();
                }
                if spec.suffix.is_some() {
                    merged.suffix = spec.suffix.clone();
                }
                if spec.delimiter.is_some() {
                    merged.delimiter = spec.delimiter.clone();
                }
                if spec.multi_cite_delimiter.is_some() {
                    merged.multi_cite_delimiter = spec.multi_cite_delimiter.clone();
                }
                if spec.collapse.is_some() {
                    merged.collapse = spec.collapse.clone();
                }
                if spec.sort.is_some() {
                    merged.sort = spec.sort.clone();
                }
                if spec.note_start_text_case.is_some() {
                    merged.note_start_text_case = spec.note_start_text_case;
                }

                std::borrow::Cow::Owned(merged)
            }
            None => std::borrow::Cow::Borrowed(self),
        }
    }

    /// Resolve the effective spec for a given citation position.
    ///
    /// If a position-specific spec exists (e.g., `ibid` for Ibid position),
    /// it merges with and overrides the base spec. Position resolution should
    /// be applied before mode resolution to allow position-specific modes.
    ///
    /// Priority: ibid > subsequent > base
    pub fn resolve_for_position(
        &self,
        position: Option<&crate::citation::Position>,
    ) -> std::borrow::Cow<'_, CitationSpec> {
        use crate::citation::Position;

        let position_spec = match position {
            Some(Position::Ibid | Position::IbidWithLocator) => {
                self.ibid.as_ref().or(self.subsequent.as_ref())
            }
            Some(Position::Subsequent) => self.subsequent.as_ref(),
            Some(Position::First) | None => None,
        };

        match position_spec {
            Some(spec) => {
                // Merge logic: position specific > base
                let mut merged = self.clone();
                // Don't recurse infinitely or keep position specs in merged result
                merged.subsequent = None;
                merged.ibid = None;

                if spec.options.is_some() {
                    merged.options = spec.options.clone();
                }
                if spec.template_ref.is_some() {
                    merged.template_ref = spec.template_ref.clone();
                }
                if spec.template.is_some() {
                    merged.template = spec.template.clone();
                    // A position spec with its own template is a complete override —
                    // clear inherited type_variants so the engine uses this template
                    // directly rather than branching by ref type. If the position spec
                    // wants type-specific rendering it must declare type_variants itself.
                    if spec.type_variants.is_none() {
                        merged.type_variants = None;
                    }
                }
                if spec.locales.is_some() {
                    merged.locales = spec.locales.clone();
                }
                if spec.type_variants.is_some() {
                    merged.type_variants = spec.type_variants.clone();
                }
                if spec.wrap.is_some() {
                    merged.wrap = spec.wrap.clone();
                }
                if spec.prefix.is_some() {
                    merged.prefix = spec.prefix.clone();
                }
                if spec.suffix.is_some() {
                    merged.suffix = spec.suffix.clone();
                }
                if spec.delimiter.is_some() {
                    merged.delimiter = spec.delimiter.clone();
                }
                if spec.multi_cite_delimiter.is_some() {
                    merged.multi_cite_delimiter = spec.multi_cite_delimiter.clone();
                }
                if spec.collapse.is_some() {
                    merged.collapse = spec.collapse.clone();
                }
                if spec.sort.is_some() {
                    merged.sort = spec.sort.clone();
                }
                if spec.note_start_text_case.is_some() {
                    merged.note_start_text_case = spec.note_start_text_case;
                }

                std::borrow::Cow::Owned(merged)
            }
            None => std::borrow::Cow::Borrowed(self),
        }
    }
}

fn default_true() -> bool {
    true
}

/// Bibliography specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
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
            self.template_ref.as_ref().and_then(|r| match r {
                TemplateReference::Preset(p) => Some(p.bibliography_template()),
                TemplateReference::Uri(_) => {
                    // TODO (Phase 2): Emit validation warning for unresolved URIs
                    None
                }
            })
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
}

/// Discipline/field classification for a citation style.
///
/// Values correspond to the CSL 1.0 `<category field="..."/>` attribute,
/// `generic-base` is silently ignored during migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum CitationField {
    /// Anthropology styles.
    Anthropology,
    /// Biology styles.
    Biology,
    /// Botany styles.
    Botany,
    /// Chemistry styles.
    Chemistry,
    /// Communications studies styles.
    Communications,
    /// Engineering styles.
    Engineering,
    /// Geography styles.
    Geography,
    /// Geology styles.
    Geology,
    /// History styles.
    History,
    /// Humanities styles.
    Humanities,
    /// Law styles.
    Law,
    /// Linguistics styles.
    Linguistics,
    /// Literature styles.
    Literature,
    /// Mathematics styles.
    Math,
    /// Medicine styles.
    Medicine,
    /// Philosophy styles.
    Philosophy,
    /// Physics styles.
    Physics,
    #[serde(rename = "political-science")]
    /// Political science styles.
    PoliticalScience,
    /// Psychology styles.
    Psychology,
    /// General science styles.
    Science,
    #[serde(rename = "social-science")]
    /// Social science styles.
    SocialScience,
    /// Sociology styles.
    Sociology,
    /// Theology styles.
    Theology,
    /// Zoology styles.
    Zoology,
}

/// A hyperlink associated with a style (documentation, self-link, etc.).
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StyleLink {
    /// Link target for related style metadata.
    pub href: String,
    /// Relationship type for the link, such as `self` or `documentation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
}

/// A person credit (author or contributor) for a style.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StylePerson {
    /// Display name for the credited person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Contact email for the credited person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// URI identifying the credited person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Provenance block for styles adapted from a CSL 1.0 source.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StyleSource {
    /// The original CSL style ID (URI).
    pub csl_id: String,
    /// Who performed the adaptation (e.g., "citum-migrate" or a person's name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapted_by: Option<String>,
    /// License URI (e.g., CC BY-SA).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Original CSL style authors.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub original_authors: Vec<StylePerson>,
    /// Links from the original CSL style (documentation, self, etc.).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<StyleLink>,
}

/// Style metadata.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct StyleInfo {
    /// Human-readable title of the style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Stable identifier for the style, usually a URI or slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Short summary of the style's intended use or provenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Default locale for the style (e.g., "en-US", "de-DE").
    /// Used for locale-aware term resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_locale: Option<String>,
    /// Discipline classifications for this style.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<CitationField>,
    /// Provenance: set when this style was adapted from a CSL 1.0 source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<StyleSource>,
    /// Concise display name for the style family, used by UIs to label
    /// search results and match banners (e.g. `"APA"`, `"Chicago Notes"`,
    /// `"MLA"`). Omit for journal-specific styles whose full title is their
    /// identity. Combine with `edition` to produce labels like `"APA 7th"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Edition or version qualifier used alongside `short_name` to
    /// disambiguate multiple editions of the same style family
    /// (e.g. `"7th"`, `"18th edition"`). Omit when only one edition exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Minimum Citum engine version required to load and render this style.
    ///
    /// Accepts a [`semver::VersionReq`]-compatible string (e.g. `">=0.38.0"`,
    /// `"^0.40"`). When the running engine does not satisfy the requirement,
    /// the resolver returns
    /// [`ResolutionError::VersionMismatch`]
    /// instead of attempting to deserialize fields the engine may not
    /// understand. Omit for styles that use only stable, long-lived features.
    #[serde(rename = "citum-version", skip_serializing_if = "Option::is_none")]
    pub citum_version: Option<String>,
}

impl StyleInfo {
    /// Returns `true` when all fields are absent (no content to merge).
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.id.is_none()
            && self.description.is_none()
            && self.default_locale.is_none()
            && self.fields.is_empty()
            && self.source.is_none()
            && self.short_name.is_none()
            && self.edition.is_none()
            && self.citum_version.is_none()
    }
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
    use super::*;

    #[test]
    fn test_style_minimal_deserialization() {
        let yaml = r#"
info:
  title: Test Style
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(style.info.title.as_ref().unwrap(), "Test Style");
    }

    #[test]
    fn test_style_with_citation() {
        let yaml = r#"
info:
  title: Test
citation:
  template:
    - contributor: author
      form: short
    - date: issued
      form: year
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        let citation = style.citation.unwrap();
        assert_eq!(citation.resolve_template().unwrap().len(), 2);
    }

    #[test]
    fn test_style_with_options() {
        let yaml = r#"
info:
  title: APA
options:
  processing: author-date
  contributors:
    display-as-sort: first
    and: symbol
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        let options = style.options.unwrap();
        assert_eq!(options.processing, Some(options::Processing::AuthorDate));
    }

    #[test]
    fn test_resolve_for_position_ibid_falls_back_to_subsequent() {
        let citation = CitationSpec {
            suffix: Some("base".to_string()),
            subsequent: Some(Box::new(CitationSpec {
                suffix: Some("subseq".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let resolved = citation
            .resolve_for_position(Some(&crate::citation::Position::Ibid))
            .into_owned();
        assert_eq!(resolved.suffix, Some("subseq".to_string()));

        let resolved_with_locator = citation
            .resolve_for_position(Some(&crate::citation::Position::IbidWithLocator))
            .into_owned();
        assert_eq!(resolved_with_locator.suffix, Some("subseq".to_string()));
    }

    #[test]
    fn test_resolve_for_position_ibid_precedes_subsequent() {
        let citation = CitationSpec {
            suffix: Some("base".to_string()),
            subsequent: Some(Box::new(CitationSpec {
                suffix: Some("subseq".to_string()),
                ..Default::default()
            })),
            ibid: Some(Box::new(CitationSpec {
                suffix: Some("ibid".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let resolved = citation
            .resolve_for_position(Some(&crate::citation::Position::Ibid))
            .into_owned();
        assert_eq!(resolved.suffix, Some("ibid".to_string()));

        let resolved_subsequent = citation
            .resolve_for_position(Some(&crate::citation::Position::Subsequent))
            .into_owned();
        assert_eq!(resolved_subsequent.suffix, Some("subseq".to_string()));
    }

    #[test]
    fn test_resolve_for_position_merges_note_start_text_case() {
        let citation = CitationSpec {
            note_start_text_case: Some(NoteStartTextCase::Lowercase),
            ibid: Some(Box::new(CitationSpec {
                note_start_text_case: Some(NoteStartTextCase::CapitalizeFirst),
                ..Default::default()
            })),
            ..Default::default()
        };

        let resolved = citation
            .resolve_for_position(Some(&crate::citation::Position::Ibid))
            .into_owned();
        assert_eq!(
            resolved.note_start_text_case,
            Some(NoteStartTextCase::CapitalizeFirst)
        );

        let unresolved = citation.resolve_for_position(None).into_owned();
        assert_eq!(
            unresolved.note_start_text_case,
            Some(NoteStartTextCase::Lowercase)
        );
    }

    #[test]
    fn test_citum_first_yaml() {
        // Test parsing the actual citum-first.yaml file structure
        let yaml = r#"
info:
  title: APA
options:
  substitute:
    contributor-role-form: short
    template:
      - editor
      - title
  processing: author-date
  contributors:
    display-as-sort: first
    and: symbol
citation:
  template:
    - contributor: author
      form: short
    - date: issued
      form: year
bibliography:
  template:
    - contributor: author
      form: long
    - date: issued
      form: year
      wrap: parentheses
    - title: primary
    - title: parent-monograph
      prefix: "In "
      emph: true
    - number: volume
    - variable: doi
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();

        // Verify info
        assert_eq!(style.info.title.as_ref().unwrap(), "APA");

        // Verify options
        let options = style.options.unwrap();
        assert_eq!(options.processing, Some(options::Processing::AuthorDate));
        assert!(options.substitute.is_some());

        // Verify citation
        let citation = style.citation.unwrap();
        let citation_template = citation.resolve_template().unwrap();
        assert_eq!(citation_template.len(), 2);

        // Verify bibliography
        let bib = style.bibliography.unwrap();
        let bib_template = bib.resolve_template().unwrap();
        assert_eq!(bib_template.len(), 6);

        // Verify flattened rendering worked
        match &bib_template[1] {
            template::TemplateComponent::Date(d) => {
                assert_eq!(
                    d.rendering.wrap,
                    Some(template::WrapConfig {
                        punctuation: template::WrapPunctuation::Parentheses,
                        inner_prefix: None,
                        inner_suffix: None,
                    })
                );
            }
            _ => panic!("Expected Date"),
        }

        match &bib_template[3] {
            template::TemplateComponent::Title(t) => {
                assert_eq!(t.rendering.prefix, Some("In ".to_string()));
                assert_eq!(t.rendering.emph, Some(true));
            }
            _ => panic!("Expected Title"),
        }
    }

    #[test]
    fn test_style_custom_fields() {
        let yaml = r#"
version: "1.1"
info:
  title: Custom Fields Test
custom:
  my-extension: true
  author-tool: "StyleAuthor v2.0"
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(style.version, SchemaVersion::parse("1.1").unwrap());
        let custom = style.custom.as_ref().unwrap();
        assert_eq!(
            custom.get("my-extension").unwrap(),
            &serde_json::Value::Bool(true)
        );
        assert_eq!(
            custom.get("author-tool").unwrap(),
            &serde_json::Value::String("StyleAuthor v2.0".to_string())
        );

        // Round-trip test
        let round_tripped = serde_yaml::to_string(&style).unwrap();
        assert!(
            round_tripped.contains("version: 1.1")
                || round_tripped.contains("version: \"1.1\"")
                || round_tripped.contains("version: '1.1'")
        );
        assert!(round_tripped.contains("my-extension: true"));
        assert!(round_tripped.contains("author-tool:"));
    }

    #[test]
    fn test_style_with_template_ref() {
        let yaml = r#"
info:
  title: Preset Test
citation:
  template-ref: apa
bibliography:
  template-ref: vancouver
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();

        // Test Citation template reference (APA)
        let citation = style.citation.unwrap();
        assert!(citation.template_ref.is_some());
        assert!(citation.template.is_none());

        let citation_template = citation.resolve_template().unwrap();
        assert_eq!(citation_template.len(), 3); // APA citation is (Author, Year, Locator)

        // precise check for APA structure
        match &citation_template[0] {
            template::TemplateComponent::Contributor(c) => {
                assert_eq!(c.contributor, template::ContributorRole::Author);
            }
            _ => panic!("Expected Contributor"),
        }

        // Test Bibliography template extension (Vancouver)
        let bib = style.bibliography.unwrap();
        let bib_template = bib.resolve_template().unwrap();
        // Vancouver bib has roughly 8 components
        assert!(bib_template.len() >= 5);

        // Verify first component is citation number
        match &bib_template[0] {
            template::TemplateComponent::Number(n) => {
                assert_eq!(n.number, template::NumberVariable::CitationNumber);
            }
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_template_overrides_template_ref_precedence() {
        let yaml = r#"
info:
  title: Override Test
citation:
  template-ref: apa
  template:
    - variable: doi
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        let citation = style.citation.unwrap();

        // Should have both
        assert!(citation.template_ref.is_some());
        assert!(citation.template.is_some());

        // Template should win
        let resolved = citation.resolve_template().unwrap();
        assert_eq!(resolved.len(), 1);
        match &resolved[0] {
            template::TemplateComponent::Variable(v) => {
                assert_eq!(v.variable, template::SimpleVariable::Doi);
            }
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn cid_and_integrity_fields_round_trip() {
        let yaml = r#"
extends: https://hub.citum.org/styles/apa-7th.yaml
extends-pin: bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
info:
  title: Pinned APA Variant
  citum-version: ">=0.38.0"
"#;
        let style: Style = Style::from_yaml_str(yaml).expect("yaml parses");
        assert!(style.extends.is_some(), "extends preserved");
        assert_eq!(
            style.extends_pin.as_deref(),
            Some("bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa")
        );
        assert_eq!(style.info.citum_version.as_deref(), Some(">=0.38.0"));

        let serialized = serde_yaml::to_string(&style).expect("serializes");
        assert!(
            serialized.contains("extends-pin: bafkrei"),
            "extends-pin field name preserved on serialization: {serialized}"
        );
        assert!(
            serialized.contains("citum-version:"),
            "citum-version field name preserved on serialization: {serialized}"
        );
    }

    #[test]
    fn citum_version_too_high_rejects_resolution() {
        let yaml = r#"
info:
  title: From the Future
  citum-version: ">=999.0.0"
"#;
        let style = Style::from_yaml_str(yaml).unwrap();
        let err = style.try_into_resolved().expect_err("must reject");
        assert!(
            matches!(err, ResolutionError::VersionMismatch { .. }),
            "expected VersionMismatch, got {err:?}"
        );
    }

    #[test]
    fn citum_version_satisfied_resolves_normally() {
        let yaml = r#"
info:
  title: Satisfied
  citum-version: ">=0.0.1"
"#;
        let style = Style::from_yaml_str(yaml).unwrap();
        let resolved = style.try_into_resolved().expect("must resolve");
        assert_eq!(resolved.info.title.as_deref(), Some("Satisfied"));
    }

    #[test]
    fn extends_pin_on_builtin_base_is_rejected() {
        let yaml = r#"
extends: apa-7th
extends-pin: bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
info:
  title: Bad Pin Target
"#;
        let style = Style::from_yaml_str(yaml).unwrap();
        let err = style.try_into_resolved().expect_err("must reject");
        assert!(
            matches!(err, ResolutionError::UriResolutionFailed { .. }),
            "expected UriResolutionFailed for builtin pin, got {err:?}"
        );
    }

    #[test]
    fn cid_extends_uri_round_trips_as_uri_variant() {
        let yaml = r#"
extends: cid:bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
info:
  title: CID-extended Style
"#;
        let style: Style = Style::from_yaml_str(yaml).expect("yaml parses");
        let extends = style.extends.expect("extends present");
        assert!(extends.is_cid(), "cid: prefix detected as CID URI");
        assert_eq!(
            extends.key(),
            "cid:bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa"
        );
    }

    #[test]
    fn old_section_extends_key_is_rejected() {
        let yaml = r#"
info:
  title: Old Section Template Reuse Key
citation:
  extends: apa
"#;
        let err = serde_yaml::from_str::<Style>(yaml)
            .expect_err("section extends should no longer be accepted");
        assert!(err.to_string().contains("extends"));
    }

    #[test]
    fn test_citation_localized_templates() {
        let yaml = r#"
info:
  title: Localized Citation
citation:
  template:
    - variable: note
  locales:
    - locale: [de]
      template:
        - variable: publisher
    - default: true
      template:
        - variable: doi
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        let citation = style.citation.unwrap();

        assert_eq!(
            citation
                .resolve_template_for_language(Some("de-AT"))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            citation
                .resolve_template_for_language(Some("fr"))
                .unwrap()
                .len(),
            1
        );
        match &citation.resolve_template_for_language(Some("de")).unwrap()[0] {
            template::TemplateComponent::Variable(v) => {
                assert_eq!(v.variable, template::SimpleVariable::Publisher);
            }
            _ => panic!("Expected Variable"),
        }
        match &citation.resolve_template_for_language(Some("fr")).unwrap()[0] {
            template::TemplateComponent::Variable(v) => {
                assert_eq!(v.variable, template::SimpleVariable::Doi);
            }
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_bibliography_localized_templates() {
        let yaml = r#"
info:
  title: Localized Bibliography
bibliography:
  template:
    - variable: note
  locales:
    - locale: [ja, zh]
      template:
        - title: primary
    - default: true
      template:
        - contributor: author
          form: long
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        let bibliography = style.bibliography.unwrap();

        match &bibliography
            .resolve_template_for_language(Some("ja-JP"))
            .unwrap()[0]
        {
            template::TemplateComponent::Title(_) => {}
            _ => panic!("Expected Title"),
        }
        match &bibliography
            .resolve_template_for_language(Some("en-US"))
            .unwrap()[0]
        {
            template::TemplateComponent::Contributor(_) => {}
            _ => panic!("Expected Contributor"),
        }
    }

    #[test]
    fn test_bibliography_with_groups() {
        let yaml = r#"
info:
  title: Grouped Bibliography Test
bibliography:
  template:
    - contributor: author
      form: long
  groups:
    - id: vietnamese
      heading:
        localized:
          vi: "Tài liệu tiếng Việt"
          en-US: "Vietnamese Sources"
      selector:
        field:
          language: vi
      sort:
        template:
          - key: author
            sort-order: given-family
    - id: other
      selector:
        not:
          field:
            language: vi
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        let bib = style.bibliography.unwrap();

        assert!(bib.groups.is_some());
        let groups = bib.groups.unwrap();
        assert_eq!(groups.len(), 2);

        // First group
        assert_eq!(groups[0].id, "vietnamese");
        match groups[0].heading.as_ref().unwrap() {
            grouping::GroupHeading::Localized { localized } => {
                assert_eq!(localized.get("vi").unwrap(), "Tài liệu tiếng Việt");
            }
            _ => panic!("expected localized heading"),
        }
        assert!(groups[0].sort.is_some());

        // Second group (fallback with negation)
        assert_eq!(groups[1].id, "other");
        assert!(groups[1].heading.is_none());
        assert!(groups[1].selector.not.is_some());
    }

    #[test]
    fn validate_type_name_accepts_known_types() {
        assert!(template::validate_type_name("article-journal"));
        assert!(template::validate_type_name("legal-case"));
        assert!(template::validate_type_name("all"));
        assert!(template::validate_type_name("default"));
    }

    #[test]
    fn validate_type_name_normalizes_underscores() {
        assert!(template::validate_type_name("article_journal"));
        assert!(template::validate_type_name("legal_case"));
    }

    #[test]
    fn validate_type_name_rejects_unknown() {
        assert!(!template::validate_type_name("article-journall"));
        assert!(!template::validate_type_name("unknown-type"));
        assert!(!template::validate_type_name(""));
    }

    #[test]
    fn style_validate_emits_warning_for_unknown_type_in_bib_type_variants() {
        let mut type_variants = IndexMap::new();
        type_variants.insert(
            template::TypeSelector::Single("typo-type".to_string()),
            TemplateVariant::Full(vec![]),
        );

        let style = Style {
            bibliography: Some(BibliographySpec {
                type_variants: Some(type_variants),
                ..Default::default()
            }),
            ..Default::default()
        };

        let warnings = style.validate();
        assert_eq!(warnings.len(), 1);
        match &warnings[0] {
            SchemaWarning::UnknownTypeName { name, location } => {
                assert_eq!(name, "typo-type");
                assert_eq!(location, "bibliography.type-variants");
            }
        }
    }

    #[test]
    fn style_validate_no_warnings_for_valid_style() {
        let mut type_variants = IndexMap::new();
        type_variants.insert(
            template::TypeSelector::Single("legal-case".to_string()),
            TemplateVariant::Full(vec![]),
        );

        let style = Style {
            bibliography: Some(BibliographySpec {
                type_variants: Some(type_variants),
                ..Default::default()
            }),
            ..Default::default()
        };

        let warnings = style.validate();
        assert!(warnings.is_empty());
    }

    #[test]
    fn null_type_variants_override_clears_preset_type_variants() {
        let child_yaml = r#"
extends: chicago-notes-18th
citation:
  type-variants: ~
  template:
  - contributor: author
    form: short
"#;
        let style = Style::from_yaml_str(child_yaml).expect("parses");
        let resolved = style.into_resolved();
        let citation = resolved.citation.unwrap();
        assert!(
            citation.type_variants.is_none(),
            "type_variants should be None after null override, got: {:?}",
            citation.type_variants.as_ref().map(|tv| tv.keys().count())
        );
    }

    #[test]
    fn template_v3_diff_resolves_to_full_type_variant() {
        let yaml = r#"
bibliography:
  template:
  - contributor: author
    form: long
  - title: primary
  - variable: publisher
  - variable: url
  type-variants:
    article-journal:
      modify:
      - match: { variable: publisher }
        suppress: true
      remove:
      - match: { variable: url }
      add:
      - after: { title: primary }
        component: { title: parent-serial, emph: true }
"#;
        let resolved = Style::from_yaml_str(yaml)
            .expect("style should parse")
            .try_into_resolved()
            .expect("diff should resolve");
        let variants = resolved
            .bibliography
            .expect("bibliography should exist")
            .type_variants
            .expect("variants should exist");
        let template = variants
            .get(&TypeSelector::Single("article-journal".to_string()))
            .and_then(TemplateVariant::as_template)
            .expect("variant should resolve to a full template");

        assert_eq!(template.len(), 4);
        assert!(matches!(
            &template[2],
            TemplateComponent::Title(title)
                if title.title == template::TitleType::ParentSerial
                    && title.rendering.emph == Some(true)
        ));
        assert!(template.iter().any(|component| matches!(
            component,
            TemplateComponent::Variable(variable)
                if variable.variable == template::SimpleVariable::Publisher
                    && variable.rendering.suppress == Some(true)
        )));
    }

    #[test]
    fn template_v3_modify_can_set_number_label_form() {
        let yaml = r#"
bibliography:
  template:
  - number: pages
  type-variants:
    chapter:
      modify:
      - match: { number: pages }
        label-form: short
"#;
        let resolved = Style::from_yaml_str(yaml)
            .expect("style should parse")
            .try_into_resolved()
            .expect("diff should resolve");
        let variants = resolved
            .bibliography
            .expect("bibliography should exist")
            .type_variants
            .expect("variants should exist");
        let template = variants
            .get(&TypeSelector::Single("chapter".to_string()))
            .and_then(TemplateVariant::as_template)
            .expect("variant should resolve to a full template");

        assert!(matches!(
            &template[0],
            TemplateComponent::Number(number)
                if number.number == template::NumberVariable::Pages
                    && number.label_form == Some(template::LabelForm::Short)
        ));
    }

    #[test]
    fn template_v3_add_with_missing_anchor_returns_resolution_error() {
        let yaml = r#"
bibliography:
  template:
  - title: primary
  type-variants:
    book:
      add:
      - after: { variable: publisher }
        component: { date: issued, form: year }
"#;
        let err = Style::from_yaml_str(yaml)
            .expect("style should parse")
            .try_into_resolved()
            .expect_err("missing add anchor should reject the diff");

        assert!(matches!(
            err,
            ResolutionError::TemplateVariantAnchorNotFound { location }
                if location == "bibliography.type-variants[book]"
        ));
    }

    #[test]
    fn template_v3_nested_citation_diff_can_use_outer_template() {
        let style = Style {
            citation: Some(CitationSpec {
                template: Some(vec![
                    TemplateComponent::Contributor(template::TemplateContributor {
                        contributor: template::ContributorRole::Author,
                        ..Default::default()
                    }),
                    TemplateComponent::Title(template::TemplateTitle {
                        title: template::TitleType::Primary,
                        ..Default::default()
                    }),
                ]),
                integral: Some(Box::new(CitationSpec {
                    type_variants: Some(indexmap::IndexMap::from([(
                        TypeSelector::Single("personal_communication".to_string()),
                        TemplateVariant::Diff(TemplateVariantDiff {
                            remove: vec![TemplateRemoveOperation {
                                match_selector: TemplateComponentSelector {
                                    fields: std::collections::BTreeMap::from([(
                                        "title".to_string(),
                                        serde_json::json!("primary"),
                                    )]),
                                },
                            }],
                            ..Default::default()
                        }),
                    )])),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let resolved = style
            .try_into_resolved()
            .expect("nested diff should resolve against outer citation template");
        let variants = resolved
            .citation
            .expect("citation should exist")
            .integral
            .expect("integral citation should exist")
            .type_variants
            .expect("variants should exist");
        let template = variants
            .get(&TypeSelector::Single("personal_communication".to_string()))
            .and_then(TemplateVariant::as_template)
            .expect("variant should resolve");

        assert_eq!(template.len(), 1);
        assert!(matches!(template[0], TemplateComponent::Contributor(_)));
    }

    #[test]
    fn template_v3_overlay_variant_defaults_to_inherited_variant() {
        #[derive(Clone)]
        struct FakeResolver {
            style: Style,
        }

        impl citum_resolver_api::StyleResolver for FakeResolver {
            type Style = Style;
            type Locale = Locale;

            fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
                if uri == "parent-style" {
                    Ok(self.style.clone())
                } else {
                    Err(ResolverError::StyleNotFound(std::borrow::Cow::Owned(
                        uri.to_string(),
                    )))
                }
            }

            fn resolve_locale(&self, id: &str) -> Result<Self::Locale, ResolverError> {
                Err(ResolverError::LocaleNotFound(std::borrow::Cow::Owned(
                    id.to_string(),
                )))
            }
        }

        let parent = Style::from_yaml_str(
            r#"
bibliography:
  template:
  - title: primary
  type-variants:
    book:
    - title: primary
      emph: true
"#,
        )
        .expect("parent should parse");
        let child = Style::from_yaml_str(
            r#"
extends: parent-style
bibliography:
  type-variants:
    book:
      modify:
      - match: { title: primary }
        quote: true
"#,
        )
        .expect("child should parse");

        let resolved = child
            .try_into_resolved_with(Some(&FakeResolver { style: parent }))
            .expect("child diff should resolve");
        let template = resolved
            .bibliography
            .expect("bibliography should exist")
            .type_variants
            .expect("variants should exist")
            .get(&TypeSelector::Single("book".to_string()))
            .and_then(TemplateVariant::as_template)
            .expect("book variant should resolve")
            .to_vec();

        assert!(matches!(
            &template[0],
            TemplateComponent::Title(title)
                if title.rendering.emph == Some(true)
                    && title.rendering.quote == Some(true)
        ));
    }

    #[test]
    fn citation_options_parse_valid_citation_fields() {
        let yaml = r#"
citation:
  options:
    contributors:
      shorten: {min: 3, use-first: 1}
    links:
      doi: true
"#;

        let style = Style::from_yaml_str(yaml).expect("citation options should parse");
        let options = style
            .citation
            .and_then(|citation| citation.options)
            .expect("citation options should exist");
        assert!(options.contributors.is_some());
        assert_eq!(options.links.and_then(|links| links.doi), Some(true));
    }

    #[test]
    fn citation_options_reject_bibliography_only_fields() {
        let yaml = r#"
citation:
  options:
    entry-suffix: "."
"#;

        let err = Style::from_yaml_str(yaml).expect_err("citation entry-suffix must fail");
        assert!(err.to_string().contains("entry-suffix"));
    }

    #[test]
    fn bibliography_options_parse_valid_bibliography_fields() {
        let yaml = r#"
bibliography:
  options:
    entry-suffix: "."
    separator: ", "
"#;

        let style = Style::from_yaml_str(yaml).expect("bibliography options should parse");
        let options = style
            .bibliography
            .and_then(|bibliography| bibliography.options)
            .expect("bibliography options should exist");
        assert_eq!(options.entry_suffix.as_deref(), Some("."));
        assert_eq!(options.separator.as_deref(), Some(", "));
    }

    #[test]
    fn bibliography_options_reject_citation_only_fields() {
        let yaml = r#"
bibliography:
  options:
    locators:
      form: short
"#;

        let err = Style::from_yaml_str(yaml).expect_err("bibliography locators must fail");
        assert!(err.to_string().contains("locators"));
    }

    #[test]
    fn top_level_options_reject_bibliography_only_fields() {
        let yaml = r#"
options:
  bibliography:
    entry-suffix: "."
"#;

        let err = Style::from_yaml_str(yaml).expect_err("top-level bibliography config must fail");
        assert!(err.to_string().contains("bibliography"));
    }

    #[test]
    fn profile_rejects_top_level_templates() {
        let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
templates:
  foo:
    - title: primary
"#;
        let err = Style::from_yaml_str(yaml)
            .unwrap()
            .try_into_resolved()
            .expect_err("profile template override must fail");
        assert!(matches!(
            err,
            ResolutionError::InvalidProfileOverride { location } if location == "templates"
        ));
    }

    #[test]
    fn profile_rejects_citation_template_override() {
        let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
citation:
  template:
    - title: primary
"#;
        let err = Style::from_yaml_str(yaml)
            .unwrap()
            .try_into_resolved()
            .expect_err("profile citation template override must fail");
        assert!(matches!(
            err,
            ResolutionError::InvalidProfileOverride { location } if location == "citation.template"
        ));
    }

    #[test]
    fn profile_rejects_bibliography_type_variants_override() {
        let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
bibliography:
  type-variants:
    default:
      - title: primary
"#;
        let err = Style::from_yaml_str(yaml)
            .unwrap()
            .try_into_resolved()
            .expect_err("profile bibliography type variants must fail");
        assert!(matches!(
            err,
            ResolutionError::InvalidProfileOverride { location } if location == "bibliography.type-variants"
        ));
    }

    #[test]
    fn profile_rejects_null_template_clear() {
        let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
bibliography:
  template: ~
"#;
        let err = Style::from_yaml_str(yaml)
            .unwrap()
            .try_into_resolved()
            .expect_err("profile null template clear must fail");
        assert!(matches!(
            err,
            ResolutionError::InvalidProfileOverride { location } if location == "bibliography.template"
        ));
    }

    #[test]
    fn profile_allows_normal_options() {
        let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
options:
  page-range-format: expanded
"#;
        let resolved = Style::from_yaml_str(yaml)
            .unwrap()
            .try_into_resolved()
            .expect("profile wrappers should accept normal options");
        assert_eq!(
            resolved
                .options
                .as_ref()
                .and_then(|options| options.page_range_format.clone()),
            Some(options::PageRangeFormat::Expanded)
        );
    }

    #[test]
    fn profile_rejects_removed_options_profile_surface() {
        let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
options:
  profile:
    citation-label-wrap: brackets
"#;
        let err = Style::from_yaml_str(yaml).expect_err("legacy profile surface must fail");
        assert!(err.to_string().contains("`options.profile` was removed"));
    }

    #[test]
    fn profile_resolution_leaves_hidden_core_templates_intact() {
        let base = StyleBase::ElsevierHarvardCore.base();
        let wrapper = Style::from_yaml_str(
            r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
"#,
        )
        .unwrap()
        .try_into_resolved()
        .expect("wrapper should resolve");

        assert_eq!(
            wrapper
                .citation
                .as_ref()
                .and_then(|citation| citation.resolve_template()),
            base.citation
                .as_ref()
                .and_then(|citation| citation.resolve_template())
        );
        assert_eq!(
            wrapper
                .bibliography
                .as_ref()
                .and_then(|bib| bib.resolve_template()),
            base.bibliography
                .as_ref()
                .and_then(|bib| bib.resolve_template())
        );
    }

    #[test]
    fn scoped_options_apply_to_profile_wrappers() {
        let resolved = Style::from_yaml_str(
            r#"
info:
  id: elsevier-vancouver
extends: elsevier-vancouver-core
citation:
  options:
    label-wrap: brackets
    group-delimiter: comma
bibliography:
  options:
    title-terminator: comma
    repeated-author-rendering: dash
"#,
        )
        .unwrap()
        .try_into_resolved()
        .expect("scoped wrapper config should resolve");

        assert_eq!(
            resolved
                .citation
                .as_ref()
                .and_then(|citation| citation.wrap.clone()),
            Some(template::WrapConfig::from(
                template::WrapPunctuation::Brackets
            ))
        );
        assert_eq!(
            resolved
                .citation
                .as_ref()
                .and_then(|citation| citation.multi_cite_delimiter.clone())
                .as_deref(),
            Some(", ")
        );
        assert_eq!(
            resolved
                .bibliography
                .as_ref()
                .and_then(|bib| bib.options.as_ref())
                .and_then(|options| options.subsequent_author_substitute.clone())
                .as_deref(),
            Some("———")
        );
    }

    #[test]
    fn options_contributors_replaces_profile_contributor_slot() {
        let resolved = Style::from_yaml_str(
            r#"
info:
  id: springer-basic-author-date
extends: springer-basic-author-date-core
options:
  contributors: springer
"#,
        )
        .unwrap()
        .try_into_resolved()
        .expect("top-level contributor preset should resolve");

        let contributors = resolved
            .options
            .as_ref()
            .and_then(|options| options.contributors.as_ref())
            .expect("resolved style should include contributor config");
        assert_eq!(contributors.name_form, Some(options::NameForm::Initials));
        assert_eq!(
            contributors.demote_non_dropping_particle,
            Some(options::DemoteNonDroppingParticle::Never)
        );
    }

    #[test]
    fn citation_superscript_wrap_applies_vertical_align() {
        let resolved = Style::from_yaml_str(
            r#"
info:
  id: elsevier-vancouver
extends: elsevier-vancouver-core
citation:
  options:
    label-wrap: superscript
"#,
        )
        .unwrap()
        .try_into_resolved()
        .expect("superscript citation wrap should resolve");

        let citation_number_rendering = resolved
            .citation
            .as_ref()
            .and_then(|citation| citation.resolve_template())
            .and_then(|template| {
                template.iter().find_map(|component| match component {
                    template::TemplateComponent::Number(number)
                        if matches!(
                            number.number,
                            template::NumberVariable::CitationNumber
                                | template::NumberVariable::CitationLabel
                        ) =>
                    {
                        Some(number.rendering.clone())
                    }
                    _ => None,
                })
            })
            .expect("numeric citation template should include a citation label");

        assert_eq!(
            citation_number_rendering.vertical_align,
            Some(VerticalAlign::Superscript)
        );
        assert_eq!(citation_number_rendering.wrap, None);
    }

    #[test]
    fn bibliography_rejects_superscript_label_wrap_at_parse_time() {
        let yaml = r#"
bibliography:
  options:
    label-wrap: superscript
"#;
        let err = Style::from_yaml_str(yaml).expect_err("bibliography superscript wrap must fail");
        assert!(err.to_string().contains("unknown variant `superscript`"));
    }

    #[test]
    fn standalone_styles_can_use_scoped_options() {
        let resolved = Style::from_yaml_str(
            r#"
citation:
  template-ref: numeric-citation
  options:
    label-wrap: superscript
    group-delimiter: comma
bibliography:
  template-ref: vancouver
  options:
    label-mode: numeric
    title-terminator: comma
    repeated-author-rendering: dash-with-space
"#,
        )
        .unwrap()
        .try_into_resolved()
        .expect("standalone scoped options should resolve");

        assert_eq!(
            resolved
                .citation
                .as_ref()
                .and_then(|citation| citation.multi_cite_delimiter.as_deref()),
            Some(", ")
        );
        assert_eq!(
            resolved
                .citation
                .as_ref()
                .and_then(|citation| citation.resolve_template())
                .and_then(|template| {
                    template.iter().find_map(|component| match component {
                        template::TemplateComponent::Number(number)
                            if matches!(
                                number.number,
                                template::NumberVariable::CitationNumber
                                    | template::NumberVariable::CitationLabel
                            ) =>
                        {
                            Some(number.rendering.vertical_align.clone())
                        }
                        _ => None,
                    })
                }),
            Some(Some(VerticalAlign::Superscript))
        );
        assert_eq!(
            resolved
                .bibliography
                .as_ref()
                .and_then(|bib| bib.options.as_ref())
                .and_then(|options| options.subsequent_author_substitute.as_deref()),
            Some("——— ")
        );
    }

    #[test]
    fn non_registry_extends_styles_do_not_use_profile_contract() {
        let yaml = r#"
info:
  id: local-custom-profile
extends: elsevier-vancouver-core
citation:
  template:
    - number: citation-number
"#;
        let resolved = Style::from_yaml_str(yaml)
            .unwrap()
            .try_into_resolved()
            .expect("non-registry extends styles should retain merge semantics");
        assert!(resolved.citation.is_some());
    }

    #[test]
    fn uri_extends_file_resolves_yaml() {
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let dir = std::env::temp_dir().join(format!("citum_test_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        let parent_path = dir.join("parent.yaml");
        let mut f = std::fs::File::create(&parent_path).unwrap();
        f.write_all(b"info:\n  title: Parent\ncitation:\n  template: []\n")
            .unwrap();
        let child_yaml = format!(
            "info:\n  title: Child\nextends: \"file://{}\"\n",
            parent_path.display()
        );
        let child = Style::from_yaml_str(&child_yaml).unwrap();
        let resolved = child.try_into_resolved().unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            resolved.citation.is_some(),
            "should inherit citation from file-backed parent"
        );
    }

    #[test]
    fn uri_extends_missing_file_returns_error() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let dir = std::env::temp_dir().join(format!("citum_test_{nanos}_missing"));
        std::fs::create_dir_all(&dir).unwrap();
        let missing = dir.join("does_not_exist.yaml");
        let yaml = format!(
            "info:\n  title: Child\nextends: \"file://{}\"\n",
            missing.display()
        );
        let child = Style::from_yaml_str(&yaml).unwrap();
        let err = child.try_into_resolved().unwrap_err();
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(err, ResolutionError::UriResolutionFailed { .. }),
            "expected UriResolutionFailed, got {err:?}"
        );
    }

    #[test]
    fn uri_extends_unsupported_scheme_returns_error() {
        let yaml = "info:\n  title: Child\nextends: \"https://example.com/style.yaml\"\n";
        let child = Style::from_yaml_str(yaml).unwrap();
        let err = child.try_into_resolved().unwrap_err();
        assert!(
            matches!(err, ResolutionError::UriResolutionFailed { .. }),
            "expected UriResolutionFailed for unsupported scheme, got {err:?}"
        );
    }
}
