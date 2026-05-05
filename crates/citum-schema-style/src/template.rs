/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Template components for Citum styles.
//!
//! This module defines the declarative template language for Citum.
//! Unlike CSL 1.0's procedural rendering elements, these components
//! are simple, typed instructions that the processor interprets.
//!
//! ## Design Philosophy
//!
//! **Explicit over magic**: All rendering behavior should be expressible in the
//! style YAML. The processor should not have hidden conditional logic based on
//! reference types. Instead, use `overrides` to declare type-specific behavior.
//!
//! ## Type-Specific Overrides
//!
//! Components support `overrides` to customize rendering per reference type:
//!
//! ```yaml
//! - variable: publisher
//!   overrides:
//!     article-journal:
//!       suppress: true  # Don't show publisher for journals
//! - number: pages
//!   overrides:
//!     chapter:
//!       wrap: parentheses
//!       prefix: "pp. "  # Show as "(pp. 1-10)" for chapters
//! ```
//!
//! This keeps all conditional logic in the style, making it testable and portable.

use crate::locale::{GeneralTerm, GrammaticalGender, TermForm};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

/// Rendering instructions applied to template components.
///
/// These fields are flattened into parent structs, so in YAML you write:
/// ```yaml
/// - title: primary
///   emph: true
///   prefix: "In "
/// ```
/// Rather than nesting under a `rendering:` key.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", default)]
pub struct Rendering {
    /// Text-case transform to apply to the rendered value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<crate::options::titles::TextCase>,
    /// Render in italics/emphasis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emph: Option<bool>,
    /// Render in quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<bool>,
    /// Render in bold/strong.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strong: Option<bool>,
    /// Render in small caps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_caps: Option<bool>,
    /// Vertical alignment to apply to rendered output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<crate::VerticalAlign>,
    /// Text to prepend to the rendered value (outside any wrap).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Text to append to the rendered value (outside any wrap).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Wrapping punctuation and optional inner affixes (text inside the wrap).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<WrapConfig>,
    /// If true, suppress this component entirely (render as empty string).
    /// Useful for type-specific overrides like suppressing publisher for journals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress: Option<bool>,
    /// Override name initialization (e.g., ". " or "").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize_with: Option<String>,
    /// Override name form (e.g., initials, full, family-only).
    #[serde(skip_serializing_if = "Option::is_none", rename = "name-form")]
    pub name_form: Option<crate::options::contributors::NameForm>,
    /// Strip trailing periods from rendered value.
    #[serde(skip_serializing_if = "Option::is_none", rename = "strip-periods")]
    pub strip_periods: Option<bool>,
}

impl Rendering {
    /// Merge another rendering configuration into this one.
    ///
    /// The other rendering takes precedence, overwriting any fields that are present.
    pub fn merge(&mut self, other: &Rendering) {
        crate::merge_options!(
            self,
            other,
            text_case,
            emph,
            quote,
            strong,
            small_caps,
            vertical_align,
            prefix,
            suffix,
            wrap,
            suppress,
            initialize_with,
            name_form,
            strip_periods,
        );
    }
}

/// Punctuation to wrap a component in.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum WrapPunctuation {
    #[default]
    Parentheses,
    Brackets,
    Quotes,
}

/// Wrapping punctuation and optional inner affixes applied around a rendered value.
///
/// Combines the wrap punctuation with optional text that appears inside the wrap
/// (between the wrap character and the rendered content).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct WrapConfig {
    /// The wrapping punctuation style.
    pub punctuation: WrapPunctuation,
    /// Text inserted after the opening wrap character but before the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner_prefix: Option<String>,
    /// Text inserted after the content but before the closing wrap character.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner_suffix: Option<String>,
}

impl<'de> serde::Deserialize<'de> for WrapConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WrapConfigVisitor;

        impl<'de> serde::de::Visitor<'de> for WrapConfigVisitor {
            type Value = WrapConfig;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "a wrap punctuation string or a mapping with a 'punctuation' key"
                )
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<WrapConfig, E> {
                let punctuation = match v {
                    "parentheses" => WrapPunctuation::Parentheses,
                    "brackets" => WrapPunctuation::Brackets,
                    "quotes" => WrapPunctuation::Quotes,
                    other => {
                        return Err(E::unknown_variant(
                            other,
                            &["parentheses", "brackets", "quotes"],
                        ));
                    }
                };
                Ok(WrapConfig {
                    punctuation,
                    inner_prefix: None,
                    inner_suffix: None,
                })
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<WrapConfig, A::Error> {
                let mut punctuation: Option<WrapPunctuation> = None;
                let mut inner_prefix: Option<String> = None;
                let mut inner_suffix: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "punctuation" => {
                            punctuation = Some(map.next_value()?);
                        }
                        "inner-prefix" => {
                            inner_prefix = Some(map.next_value()?);
                        }
                        "inner-suffix" => {
                            inner_suffix = Some(map.next_value()?);
                        }
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &["punctuation", "inner-prefix", "inner-suffix"],
                            ));
                        }
                    }
                }

                let punctuation =
                    punctuation.ok_or_else(|| serde::de::Error::missing_field("punctuation"))?;
                Ok(WrapConfig {
                    punctuation,
                    inner_prefix,
                    inner_suffix,
                })
            }
        }

        deserializer.deserialize_any(WrapConfigVisitor)
    }
}

impl From<WrapPunctuation> for WrapConfig {
    fn from(punctuation: WrapPunctuation) -> Self {
        WrapConfig {
            punctuation,
            inner_prefix: None,
            inner_suffix: None,
        }
    }
}

/// Canonical reference type names recognized by the Citum engine.
///
/// Used by [`validate_type_name`] to detect likely typos.
pub const VALID_TYPE_NAMES: &[&str] = &[
    "book",
    "manual",
    "report",
    "thesis",
    "webpage",
    "post",
    "interview",
    "manuscript",
    "personal-communication",
    "document",
    "chapter",
    "paper-conference",
    "article-journal",
    "article-magazine",
    "article-newspaper",
    "broadcast",
    "motion-picture",
    "collection",
    "legal-case",
    "statute",
    "treaty",
    "hearing",
    "regulation",
    "brief",
    "classic",
    "patent",
    "dataset",
    "standard",
    "software",
    // Special keywords
    "all",
    "default",
];

/// Returns `true` if `s` is a recognized reference type name.
///
/// Normalizes underscores to hyphens before comparing, so both
/// `"article_journal"` and `"article-journal"` are accepted.
/// Returns `false` for unrecognized names (likely typos).
pub fn validate_type_name(s: &str) -> bool {
    let normalized = s.replace('_', "-");
    VALID_TYPE_NAMES.iter().any(|&known| known == normalized)
}

/// Selector for reference types in overrides.
/// Can be a single type string or a list of types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum TypeSelector {
    Single(String),
    Multiple(Vec<String>),
}

impl Serialize for TypeSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TypeSelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = TypeSelector;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a sequence of strings")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(E::custom)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut types = Vec::new();
                while let Some(t) = seq.next_element::<String>()? {
                    types.push(t);
                }
                if types.len() == 1 {
                    Ok(TypeSelector::Single(types.remove(0)))
                } else {
                    Ok(TypeSelector::Multiple(types))
                }
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

impl std::fmt::Display for TypeSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSelector::Single(s) => write!(f, "{s}"),
            TypeSelector::Multiple(types) => write!(f, "{}", types.join(",")),
        }
    }
}

impl std::str::FromStr for TypeSelector {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(',') {
            Ok(TypeSelector::Multiple(
                s.split(',').map(|t| t.trim().to_string()).collect(),
            ))
        } else {
            Ok(TypeSelector::Single(s.to_string()))
        }
    }
}

impl TypeSelector {
    /// Check whether this selector matches a reference type.
    ///
    /// Type names are compared after normalizing underscores to hyphens, so
    /// "legal_case" and "legal-case" are treated as equivalent (matching both
    /// CSL 1.0 underscore convention and Citum hyphen convention).
    ///
    /// The special keyword "all" always matches any reference type.
    pub fn matches(&self, ref_type: &str) -> bool {
        let normalized_ref = ref_type.replace('_', "-");
        let base_ref = normalized_ref
            .split_once('+')
            .map(|(base, _)| base)
            .unwrap_or(&normalized_ref);
        let eq = |s: &str| -> bool {
            s == ref_type
                || s.replace('_', "-") == normalized_ref
                || s.replace('_', "-") == base_ref
                || s == "all"
                || (s == "default" && ref_type == "default")
        };
        match self {
            TypeSelector::Single(s) => eq(s),
            TypeSelector::Multiple(types) => types.iter().any(|t| eq(t)),
        }
    }

    /// Returns any type names in this selector that are not in [`VALID_TYPE_NAMES`].
    ///
    /// An empty vec means all names are valid. Callers should emit a
    /// [`crate::SchemaWarning`] for each returned name.
    pub fn unknown_type_names(&self) -> Vec<&str> {
        match self {
            TypeSelector::Single(s) => {
                if validate_type_name(s) {
                    vec![]
                } else {
                    vec![s.as_str()]
                }
            }
            TypeSelector::Multiple(types) => types
                .iter()
                .filter(|s| !validate_type_name(s))
                .map(|s| s.as_str())
                .collect(),
        }
    }
}

/// A template component - the building blocks of citation/bibliography templates.
///
/// Each variant handles a specific data type with appropriate formatting options.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum TemplateComponent {
    Contributor(TemplateContributor),
    Date(TemplateDate),
    Title(TemplateTitle),
    Number(TemplateNumber),
    Variable(TemplateVariable),
    Group(TemplateGroup),
    Term(TemplateTerm),
}

impl Default for TemplateComponent {
    fn default() -> Self {
        TemplateComponent::Variable(TemplateVariable::default())
    }
}

impl TemplateComponent {
    /// Return the rendering options for this component.
    ///
    /// Every template component has rendering options like emphasis, wrapping, and prefixes.
    pub fn rendering(&self) -> &Rendering {
        crate::dispatch_component!(self, |inner| &inner.rendering)
    }

    /// Return the mutable rendering options for this component.
    ///
    /// Provides mutable access to rendering fields (prefix, suffix, etc.)
    /// that are present on all template component variants.
    pub fn rendering_mut(&mut self) -> &mut Rendering {
        crate::dispatch_component!(self, |inner| &mut inner.rendering)
    }
}

/// Type-specific template override, either as a complete legacy template or a V3 diff.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum TemplateVariant {
    /// Complete replacement template used by Template V1/V2 styles.
    Full(Vec<TemplateComponent>),
    /// Structural diff applied to a parent template during style resolution.
    Diff(TemplateVariantDiff),
}

impl TemplateVariant {
    /// Return this variant as a concrete template if it has already been resolved.
    #[must_use]
    pub fn as_template(&self) -> Option<&[TemplateComponent]> {
        match self {
            Self::Full(template) => Some(template.as_slice()),
            Self::Diff(_) => None,
        }
    }

    /// Return this variant as a mutable concrete template if it has already been resolved.
    pub fn as_template_mut(&mut self) -> Option<&mut Vec<TemplateComponent>> {
        match self {
            Self::Full(template) => Some(template),
            Self::Diff(_) => None,
        }
    }

    /// Convert this variant into its concrete template if it has already been resolved.
    #[must_use]
    pub fn into_template(self) -> Option<Vec<TemplateComponent>> {
        match self {
            Self::Full(template) => Some(template),
            Self::Diff(_) => None,
        }
    }
}

impl From<Vec<TemplateComponent>> for TemplateVariant {
    fn from(template: Vec<TemplateComponent>) -> Self {
        Self::Full(template)
    }
}

/// Structural diff that derives a type-specific template from a parent template.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateVariantDiff {
    /// Optional parent type variant selector within the same section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<TypeSelector>,
    /// Rendering-only modifications applied in authored order.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub modify: Vec<TemplateModifyOperation>,
    /// Component removals applied in authored order.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub remove: Vec<TemplateRemoveOperation>,
    /// Component additions applied in authored order.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub add: Vec<TemplateAddOperation>,
}

/// Partial component selector used to locate anchors in a template.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(transparent)]
pub struct TemplateComponentSelector {
    /// Component fields that must be present with equal values on the target component.
    pub fields: BTreeMap<String, serde_json::Value>,
}

impl TemplateComponentSelector {
    /// Returns `true` when this selector has no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns `true` when every selector field is present with the same value.
    #[must_use]
    pub fn matches(&self, component: &TemplateComponent) -> bool {
        let Ok(serde_json::Value::Object(component_fields)) = serde_json::to_value(component)
        else {
            return false;
        };

        self.fields.iter().all(|(key, expected)| {
            component_fields
                .get(key)
                .is_some_and(|actual| selector_value_matches(expected, actual))
        })
    }
}

fn selector_value_matches(expected: &serde_json::Value, actual: &serde_json::Value) -> bool {
    match (expected, actual) {
        (serde_json::Value::Object(expected_fields), serde_json::Value::Object(actual_fields)) => {
            expected_fields.iter().all(|(key, expected_value)| {
                actual_fields.get(key).is_some_and(|actual_value| {
                    selector_value_matches(expected_value, actual_value)
                })
            })
        }
        (serde_json::Value::Array(expected_items), serde_json::Value::Array(actual_items)) => {
            expected_items.len() == actual_items.len()
                && expected_items.iter().zip(actual_items.iter()).all(
                    |(expected_item, actual_item)| {
                        selector_value_matches(expected_item, actual_item)
                    },
                )
        }
        _ => expected == actual,
    }
}

/// Rendering-only modification for the component matched by `match`.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateModifyOperation {
    /// Selector identifying exactly one component to modify.
    #[serde(rename = "match")]
    pub match_selector: TemplateComponentSelector,
    /// Rendering fields to merge onto the matched component.
    #[serde(flatten, default)]
    pub rendering: Rendering,
}

/// Removal operation for the component matched by `match`.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateRemoveOperation {
    /// Selector identifying exactly one component to remove.
    #[serde(rename = "match")]
    pub match_selector: TemplateComponentSelector,
}

/// Addition operation that inserts a component before or after an anchor.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateAddOperation {
    /// Anchor selector before which the component should be inserted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<TemplateComponentSelector>,
    /// Anchor selector after which the component should be inserted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<TemplateComponentSelector>,
    /// Component to insert.
    pub component: TemplateComponent,
}

/// Configuration for role labels (e.g., "eds.", "trans.").
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct RoleLabel {
    /// Locale term key for the role (e.g., "editor", "translator").
    pub term: String,
    /// Term form: short ("eds.") or long ("editors").
    #[serde(default)]
    pub form: RoleLabelForm,
    /// Where to place the label relative to names.
    #[serde(default)]
    pub placement: LabelPlacement,
}

/// Term form for role labels.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum RoleLabelForm {
    #[default]
    Short,
    Long,
}

/// Label placement relative to contributor names.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LabelPlacement {
    Prefix,
    #[default]
    Suffix,
}

/// A contributor component for rendering names.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateContributor {
    /// Which contributor role to render (author, editor, etc.).
    pub contributor: ContributorRole,
    /// How to display the contributor (long names, short, with label, etc.).
    pub form: ContributorForm,
    /// Optional role label configuration (e.g., "eds." for editors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<RoleLabel>,
    /// Override the global name order for this specific component.
    /// Use to show editors as "Given Family" even when global setting is "Family, Given".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_order: Option<NameOrder>,
    /// Override the name form (e.g., initials, full, family-only) for this specific component.
    #[serde(skip_serializing_if = "Option::is_none", rename = "name-form")]
    pub name_form: Option<crate::options::contributors::NameForm>,
    /// Custom delimiter between names (overrides global setting).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
    /// Delimiter between family and given name when inverted (overrides global setting).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_separator: Option<String>,
    /// Shorten the list of names (et al. configuration).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shorten: Option<crate::options::ShortenListOptions>,
    /// Override the conjunction between the last two names.
    /// Use `none` for bibliography when citation uses `text` or `symbol`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<crate::options::AndOptions>,
    #[serde(flatten, default)]
    pub rendering: Rendering,
    /// Structured link options (DOI, URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<crate::options::LinksConfig>,
    /// Explicit grammatical gender override for role-label agreement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<GrammaticalGender>,

    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Name display order.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum NameOrder {
    /// Display as "Given Family" (e.g., "John Smith").
    GivenFirst,
    /// Display as "Family, Given" (e.g., "Smith, John").
    #[default]
    FamilyFirst,
    /// First contributor inverted ("Family, Given"); subsequent contributors given-first.
    FamilyFirstOnly,
}

/// How to render contributor names.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ContributorForm {
    #[default]
    Long,
    Short,
    FamilyOnly,
    Verb,
    VerbShort,
}

crate::str_enum! {
    /// Contributor roles.
    #[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    #[serde(rename_all = "kebab-case")]
    pub enum ContributorRole {
        #[default] Author = "author",
        Chair = "chair",
        Editor = "editor",
        Translator = "translator",
        Director = "director",
        Publisher = "publisher",
        Recipient = "recipient",
        Interviewer = "interviewer",
        Interviewee = "interviewee",
        Guest = "guest",
        Inventor = "inventor",
        Counsel = "counsel",
        Composer = "composer",
        CollectionEditor = "collection-editor",
        ContainerAuthor = "container-author",
        EditorialDirector = "editorial-director",
        TextualEditor = "textual-editor",
        Illustrator = "illustrator",
        OriginalAuthor = "original-author",
        ReviewedAuthor = "reviewed-author"
    }
}

/// A date component for rendering dates.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateDate {
    pub date: DateVariable,
    pub form: DateForm,
    /// Fallback components if the primary date is missing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Vec<TemplateComponent>>,
    #[serde(flatten, default)]
    pub rendering: Rendering,
    /// Structured link options (DOI, URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<crate::options::LinksConfig>,

    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Date variables.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum DateVariable {
    #[default]
    Issued,
    Accessed,
    OriginalPublished,
    Submitted,
    EventDate,
}

/// Date rendering forms.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum DateForm {
    #[default]
    Year,
    YearMonth,
    Full,
    MonthDay,
    YearMonthDay,
    DayMonthAbbrYear,
    /// Abbreviated month + day + year in US order: "Jan 15, 2024".
    MonthAbbrDayYear,
}

/// A title component.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateTitle {
    pub title: TitleType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<TitleForm>,
    /// When true, suppress this title component unless the reference needs
    /// disambiguation (i.e. multiple works by the same author appear in the
    /// document). Used by author-class styles (e.g. MLA) where the title
    /// appears in citations only to resolve same-author ambiguity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disambiguate_only: Option<bool>,
    #[serde(flatten, default)]
    pub rendering: Rendering,
    /// Structured link options (DOI, URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<crate::options::LinksConfig>,

    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Types of titles.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum TitleType {
    /// The primary title of the cited work.
    #[default]
    Primary,
    /// Title of a book/monograph containing the cited work.
    ParentMonograph,
    /// Title of a periodical/serial containing the cited work.
    ParentSerial,
}

/// Title rendering forms.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TitleForm {
    Short,
    #[default]
    Long,
}

/// A number component (volume, issue, pages, etc.).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateNumber {
    pub number: NumberVariable,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<NumberForm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_form: Option<LabelForm>,
    /// When `true`, show this pages component even when a locator is present in a note-style citation.
    /// By default, pages are suppressed in note-style citations when a locator is present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_with_locator: Option<bool>,
    #[serde(flatten)]
    pub rendering: Rendering,
    /// Structured link options (DOI, URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<crate::options::LinksConfig>,
    /// Explicit grammatical gender override for number/ordinal agreement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<GrammaticalGender>,

    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Number variables.
///
/// Use `number:` when the value is treated as a number by the style:
/// numeric labels, numeric-specific formatting, ordinals, roman numerals, or
/// locator-aware punctuation. Use `variable:` instead when the field should be
/// passed through as plain text without number formatting semantics.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub enum NumberVariable {
    #[default]
    Volume,
    Issue,
    Pages,
    Edition,
    ChapterNumber,
    CollectionNumber,
    NumberOfPages,
    NumberOfVolumes,
    CitationNumber,
    CitationLabel,
    Number,
    DocketNumber,
    PatentNumber,
    StandardNumber,
    ReportNumber,
    PartNumber,
    SupplementNumber,
    PrintingNumber,
    /// A custom numbering variable rendered from an arbitrary numbering kind.
    Custom(String),
}

impl NumberVariable {
    /// Return the canonical kebab-case key for this numeric variable.
    #[must_use]
    pub fn as_key(&self) -> Cow<'_, str> {
        match self {
            Self::Volume => Cow::Borrowed("volume"),
            Self::Issue => Cow::Borrowed("issue"),
            Self::Pages => Cow::Borrowed("pages"),
            Self::Edition => Cow::Borrowed("edition"),
            Self::ChapterNumber => Cow::Borrowed("chapter-number"),
            Self::CollectionNumber => Cow::Borrowed("collection-number"),
            Self::NumberOfPages => Cow::Borrowed("number-of-pages"),
            Self::NumberOfVolumes => Cow::Borrowed("number-of-volumes"),
            Self::CitationNumber => Cow::Borrowed("citation-number"),
            Self::CitationLabel => Cow::Borrowed("citation-label"),
            Self::Number => Cow::Borrowed("number"),
            Self::DocketNumber => Cow::Borrowed("docket-number"),
            Self::PatentNumber => Cow::Borrowed("patent-number"),
            Self::StandardNumber => Cow::Borrowed("standard-number"),
            Self::ReportNumber => Cow::Borrowed("report-number"),
            Self::PartNumber => Cow::Borrowed("part-number"),
            Self::SupplementNumber => Cow::Borrowed("supplement-number"),
            Self::PrintingNumber => Cow::Borrowed("printing-number"),
            Self::Custom(value) => normalize_kind_key(value)
                .map(Cow::Owned)
                .unwrap_or_else(|| Cow::Borrowed(value.as_str())),
        }
    }

    fn from_key(value: &str) -> Result<Self, String> {
        let canonical = normalize_kind_key(value)
            .ok_or_else(|| "number variable must not be empty".to_string())?;
        Ok(match canonical.as_str() {
            "volume" => Self::Volume,
            "issue" => Self::Issue,
            "pages" => Self::Pages,
            "edition" => Self::Edition,
            "chapter-number" => Self::ChapterNumber,
            "collection-number" => Self::CollectionNumber,
            "number-of-pages" => Self::NumberOfPages,
            "number-of-volumes" => Self::NumberOfVolumes,
            "citation-number" => Self::CitationNumber,
            "citation-label" => Self::CitationLabel,
            "number" => Self::Number,
            "docket-number" => Self::DocketNumber,
            "patent-number" => Self::PatentNumber,
            "standard-number" => Self::StandardNumber,
            "report-number" => Self::ReportNumber,
            "part-number" => Self::PartNumber,
            "supplement-number" => Self::SupplementNumber,
            "printing-number" => Self::PrintingNumber,
            _ => Self::Custom(canonical),
        })
    }
}

impl PartialEq for NumberVariable {
    fn eq(&self, other: &Self) -> bool {
        self.as_key().as_ref() == other.as_key().as_ref()
    }
}

impl Eq for NumberVariable {}

impl Hash for NumberVariable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_key().as_ref().hash(state);
    }
}

impl Serialize for NumberVariable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_key().as_ref())
    }
}

impl<'de> Deserialize<'de> for NumberVariable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_key(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl JsonSchema for NumberVariable {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "NumberVariable".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "description": "Known number variable keyword or custom kebab-case identifier."
        })
    }
}

/// Number rendering forms.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum NumberForm {
    #[default]
    Numeric,
    Ordinal,
    Roman,
}

fn normalize_kind_key(value: &str) -> Option<String> {
    let mut normalized = String::new();
    let mut pending_dash = false;

    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !normalized.is_empty() {
                normalized.push('-');
            }
            normalized.push(ch.to_ascii_lowercase());
            pending_dash = false;
        } else if !normalized.is_empty() {
            pending_dash = true;
        }
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// Label rendering forms.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LabelForm {
    Long,
    #[default]
    Short,
    Symbol,
}

/// A simple variable component (DOI, ISBN, URL, etc.).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateVariable {
    pub variable: SimpleVariable,
    #[serde(flatten)]
    pub rendering: Rendering,
    /// Structured link options (DOI, URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<crate::options::LinksConfig>,

    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Simple string variables.
///
/// Use `variable:` for string passthrough fields, even when the field name is
/// also present in [`NumberVariable`]. For example, `variable: volume` keeps the
/// source value as plain text, while `number: volume` opts into numeric
/// formatting behavior.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SimpleVariable {
    #[default]
    Doi,
    Isbn,
    Issn,
    Url,
    Pmid,
    Pmcid,
    Abstract,
    Note,
    Annote,
    Keyword,
    Genre,
    Medium,
    Source,
    Status,
    Archive,
    ArchiveLocation,
    ArchiveName,
    ArchivePlace,
    ArchiveCollection,
    ArchiveCollectionId,
    ArchiveSeries,
    ArchiveBox,
    ArchiveFolder,
    ArchiveItem,
    ArchiveUrl,
    EprintId,
    EprintServer,
    EprintClass,
    Publisher,
    PublisherPlace,
    OriginalPublisher,
    OriginalPublisherPlace,
    EventPlace,
    Dimensions,
    Scale,
    Version,
    Locator,
    ContainerTitleShort,
    Authority,
    Code,
    Reporter,
    Page,
    Section,
    Volume,
    Number,
    DocketNumber,
    PatentNumber,
    StandardNumber,
    ReportNumber,
    AdsBibcode,
}

/// A term component for rendering locale-specific text.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateTerm {
    /// Which term to render.
    pub term: GeneralTerm,
    /// Form: long (default), short, or symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<TermForm>,
    /// Explicit grammatical gender override for term selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<GrammaticalGender>,
    #[serde(flatten, default)]
    pub rendering: Rendering,

    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// A group component for grouping multiple components with a delimiter,
/// matching CSL 1.0 `<group>` semantics.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TemplateGroup {
    pub group: Vec<TemplateComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<DelimiterPunctuation>,
    #[serde(flatten, default)]
    pub rendering: Rendering,

    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Delimiter punctuation options.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DelimiterPunctuation {
    #[default]
    Comma,
    Semicolon,
    Period,
    Colon,
    Ampersand,
    VerticalLine,
    Slash,
    Hyphen,
    Space,
    None,
    /// Custom delimiter string (e.g., ": ").
    #[serde(untagged)]
    Custom(String),
}

#[cfg(feature = "schema")]
impl JsonSchema for DelimiterPunctuation {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DelimiterPunctuation".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({"type": "string"})
    }
}

impl DelimiterPunctuation {
    /// Convert this delimiter to a string with trailing space.
    ///
    /// Returns the punctuation followed by a space, except for Space (single space) and None (empty string).
    pub fn to_string_with_space(&self) -> String {
        match self {
            Self::Comma => ", ".to_string(),
            Self::Semicolon => "; ".to_string(),
            Self::Period => ". ".to_string(),
            Self::Colon => ": ".to_string(),
            Self::Ampersand => " & ".to_string(),
            Self::VerticalLine => " | ".to_string(),
            Self::Slash => "/".to_string(),
            Self::Hyphen => "-".to_string(),
            Self::Space => " ".to_string(),
            Self::None => "".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }

    /// Parse a delimiter from a CSL 1.0 delimiter string.
    ///
    /// Handles common patterns like ", ", ": ", etc.
    /// Returns the Custom variant for unrecognized delimiters.
    pub fn from_csl_string(s: &str) -> Self {
        if s == " " {
            return Self::Space;
        }

        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            return Self::None;
        }

        match trimmed {
            "," => Self::Comma,
            ";" => Self::Semicolon,
            "." => Self::Period,
            ":" => Self::Colon,
            "&" => Self::Ampersand,
            "|" => Self::VerticalLine,
            "/" => Self::Slash,
            "-" => Self::Hyphen,
            _ => Self::Custom(s.to_string()),
        }
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
    fn test_contributor_deserialization() {
        let yaml = r#"
contributor: author
form: long
"#;
        let comp: TemplateContributor = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(comp.contributor, ContributorRole::Author);
        assert_eq!(comp.form, ContributorForm::Long);
    }

    #[test]
    fn test_template_component_untagged() {
        let yaml = r#"
- contributor: author
  form: short
- date: issued
  form: year
- title: primary
"#;
        let components: Vec<TemplateComponent> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(components.len(), 3);

        match &components[0] {
            TemplateComponent::Contributor(c) => {
                assert_eq!(c.contributor, ContributorRole::Author);
            }
            _ => panic!("Expected Contributor"),
        }

        match &components[1] {
            TemplateComponent::Date(d) => {
                assert_eq!(d.date, DateVariable::Issued);
            }
            _ => panic!("Expected Date"),
        }
    }

    #[test]
    fn test_flattened_rendering() {
        // Test that rendering options can be specified directly on the component
        let yaml = r#"
- title: parent-monograph
  prefix: "In "
  emph: true
- date: issued
  form: year
  wrap: parentheses
"#;
        let components: Vec<TemplateComponent> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(components.len(), 2);

        match &components[0] {
            TemplateComponent::Title(t) => {
                assert_eq!(t.rendering.prefix, Some("In ".to_string()));
                assert_eq!(t.rendering.emph, Some(true));
            }
            _ => panic!("Expected Title"),
        }

        match &components[1] {
            TemplateComponent::Date(d) => {
                assert_eq!(
                    d.rendering.wrap,
                    Some(WrapConfig {
                        punctuation: WrapPunctuation::Parentheses,
                        inner_prefix: None,
                        inner_suffix: None,
                    })
                );
            }
            _ => panic!("Expected Date"),
        }
    }

    #[test]
    fn test_number_variable_custom_normalizes_manual_construction() {
        let number = NumberVariable::Custom("Reel Label".to_string());

        assert_eq!(number.as_key(), "reel-label");
        assert_eq!(
            number,
            serde_yaml::from_str::<NumberVariable>("reel-label")
                .expect("custom number variable should parse")
        );
        assert_eq!(
            serde_json::to_string(&number).expect("custom number variable should serialize"),
            "\"reel-label\""
        );
    }

    #[test]
    fn test_contributor_with_wrap() {
        let yaml = r#"
contributor: publisher
form: short
wrap: parentheses
"#;
        let comp: TemplateContributor = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(comp.contributor, ContributorRole::Publisher);
        assert_eq!(
            comp.rendering.wrap,
            Some(WrapConfig {
                punctuation: WrapPunctuation::Parentheses,
                inner_prefix: None,
                inner_suffix: None,
            })
        );
    }

    #[test]
    fn test_variable_deserialization() {
        // Test that `variable: publisher` parses as Variable, not Number
        let yaml = "variable: publisher\n";
        let comp: TemplateComponent = serde_yaml::from_str(yaml).unwrap();
        match comp {
            TemplateComponent::Variable(v) => {
                assert_eq!(v.variable, SimpleVariable::Publisher);
            }
            _ => panic!("Expected Variable(Publisher), got {:?}", comp),
        }
    }

    #[test]
    fn test_variable_array_parsing() {
        let yaml = r#"
- variable: doi
  prefix: "https://doi.org/"
- variable: publisher
"#;
        let comps: Vec<TemplateComponent> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(comps.len(), 2);
        match &comps[0] {
            TemplateComponent::Variable(v) => assert_eq!(v.variable, SimpleVariable::Doi),
            _ => panic!("Expected Variable for doi, got {:?}", comps[0]),
        }
        match &comps[1] {
            TemplateComponent::Variable(v) => assert_eq!(v.variable, SimpleVariable::Publisher),
            _ => panic!("Expected Variable for publisher, got {:?}", comps[1]),
        }
    }

    #[test]
    fn test_type_selector_default_only_matches_default_context() {
        let selector = TypeSelector::Single("default".to_string());
        assert!(selector.matches("default"));
        assert!(!selector.matches("article-journal"));

        let mixed = TypeSelector::Multiple(vec!["default".to_string(), "chapter".to_string()]);
        assert!(mixed.matches("default"));
        assert!(mixed.matches("chapter"));
        assert!(!mixed.matches("book"));
    }

    #[test]
    fn test_template_component_selector_matches_nested_partial_group() {
        let component: TemplateComponent = serde_yaml::from_str(
            r#"
delimiter: ""
group:
- number: citation-number
  wrap:
    punctuation: brackets
- contributor: author
  form: long
"#,
        )
        .unwrap();
        let selector = TemplateComponentSelector {
            fields: BTreeMap::from([(
                "group".to_string(),
                serde_json::json!([
                    { "number": "citation-number" },
                    { "contributor": "author" }
                ]),
            )]),
        };

        assert!(selector.matches(&component));
    }

    #[test]
    fn test_delimiter_from_csl_string_normalizes_none_and_trimmed_values() {
        assert_eq!(
            DelimiterPunctuation::from_csl_string("none"),
            DelimiterPunctuation::None
        );
        assert_eq!(
            DelimiterPunctuation::from_csl_string(" none "),
            DelimiterPunctuation::None
        );
        assert_eq!(
            DelimiterPunctuation::from_csl_string(" "),
            DelimiterPunctuation::Space
        );
        assert_eq!(
            DelimiterPunctuation::from_csl_string(" : "),
            DelimiterPunctuation::Colon
        );
    }
}
