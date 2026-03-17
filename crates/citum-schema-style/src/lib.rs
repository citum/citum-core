//! Public schema types for Citum styles, citations, references, and locales.

use indexmap::IndexMap;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Renderer for converting processor output to different formats.
pub mod renderer;
pub use renderer::Renderer;

/// Citation input model.
pub mod citation;
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
/// Bibliographic reference data types.
#[allow(missing_docs, reason = "internal derives")]
pub mod reference;
/// Citation and bibliography template components.
#[allow(missing_docs, reason = "internal derives")]
pub mod template;

/// Embedded templates for priority styles (APA, Chicago, Vancouver, IEEE, Harvard).
pub mod embedded;

/// Declarative macros for AST and configurations.
pub mod macros;

pub use citation::{
    Citation, CitationItem, CitationMode, Citations, IntegralNameState, LocatorType, Position,
};
pub use grouping::{
    BibliographyGroup, CitedStatus, FieldMatcher, GroupHeading, GroupSelector, GroupSort,
    GroupSortEntry, GroupSortKey, NameSortOrder, SortKey, TypeSelector,
};
pub use legacy::{
    AndTerm, ConditionBlock, CslnInfo, CslnLocale, CslnNode, CslnStyle, DateBlock, DateForm,
    DateOptions, DatePartForm, DateParts, DelimiterPrecedes, ElseIfBranch, EtAlOptions,
    EtAlSubsequent, FontStyle, FontVariant, FontWeight, FormattingOptions, GroupBlock, ItemType,
    LabelForm, LabelOptions, NameAsSortOrder, NameMode, NamesBlock, NamesOptions, TermBlock,
    TextDecoration, Variable, VariableBlock, VerticalAlign,
};
pub use locale::Locale;
pub use options::Config;
pub use options::TextCase;
pub use presets::{ContributorPreset, DatePreset, SortPreset, SubstitutePreset, TitlePreset};
pub use template::{
    Rendering, TemplateComponent, TemplateContributor, TemplateDate, TemplateList, TemplateNumber,
    TemplateTerm, TemplateTitle, TemplateVariable, WrapPunctuation,
};

/// A collection of bibliographic references with optional metadata.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct InputBibliography {
    /// Bibliography metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<InputBibliographyInfo>,
    /// The list of references.
    pub references: Vec<reference::InputReference>,
    /// Optional compound entry sets keyed by set id.
    ///
    /// Each set id maps to an ordered list of reference ids that should be treated
    /// as one compound numeric group when `compound-numeric` is enabled by style.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<std::collections::BTreeMap<String, Vec<String>>>")
    )]
    pub sets: Option<IndexMap<String, Vec<String>>>,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Metadata for an input bibliography.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct InputBibliographyInfo {
    /// Human-readable title for the bibliography dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Creator or maintainer of the bibliography dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// A named template (reusable sequence of components).
pub type Template = Vec<TemplateComponent>;

/// Canonical Citum style schema version used when `Style.version` is omitted.
pub const STYLE_SCHEMA_VERSION: &str = "0.8.0";

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
    pub version: String,
    /// Style metadata.
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
}

fn default_version() -> String {
    STYLE_SCHEMA_VERSION.to_string()
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
    pub options: Option<Config>,
    /// Reference to an embedded template preset.
    /// If both `use_preset` and `template` are present, `template` takes precedence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_preset: Option<TemplatePreset>,
    /// Default template when no localized override is selected.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub template: Option<Template>,
    /// Locale-specific template overrides checked before the default template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locales: Option<Vec<LocalizedTemplateSpec>>,
    /// Wrap the entire citation in punctuation. Preferred over prefix/suffix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<template::WrapPunctuation>,
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
    /// Returns the explicit `template` if present, otherwise resolves `use_preset`.
    /// Returns `None` if neither is specified.
    pub fn resolve_template(&self) -> Option<Template> {
        self.template
            .clone()
            .or_else(|| self.use_preset.as_ref().map(|p| p.citation_template()))
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
                if spec.use_preset.is_some() {
                    merged.use_preset = spec.use_preset.clone();
                }
                if spec.template.is_some() {
                    merged.template = spec.template.clone();
                }
                if spec.locales.is_some() {
                    merged.locales = spec.locales.clone();
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
                if spec.use_preset.is_some() {
                    merged.use_preset = spec.use_preset.clone();
                }
                if spec.template.is_some() {
                    merged.template = spec.template.clone();
                }
                if spec.locales.is_some() {
                    merged.locales = spec.locales.clone();
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

/// Bibliography specification.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct BibliographySpec {
    /// Bibliography-specific option overrides merged over the style config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Config>,
    /// Reference to an embedded template preset.
    /// If both `use_preset` and `template` are present, `template` takes precedence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_preset: Option<TemplatePreset>,
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
    pub type_templates: Option<HashMap<template::TypeSelector, Template>>,
    /// Optional global bibliography sorting specification.
    ///
    /// When present, used for sorting the flat bibliography or as default
    /// for groups that don't specify their own sort.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<grouping::GroupSortEntry>,
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

impl BibliographySpec {
    /// Resolve the effective template for this bibliography.
    ///
    /// Returns the explicit `template` if present, otherwise resolves `use_preset`.
    /// Returns `None` if neither is specified.
    pub fn resolve_template(&self) -> Option<Template> {
        self.template
            .clone()
            .or_else(|| self.use_preset.as_ref().map(|p| p.bibliography_template()))
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
}

#[cfg(test)]
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
    fn test_csln_first_yaml() {
        // Test parsing the actual csln-first.yaml file structure
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
                    Some(template::WrapPunctuation::Parentheses)
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
        assert_eq!(style.version, "1.1");
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
    fn test_style_with_preset() {
        let yaml = r#"
info:
  title: Preset Test
citation:
  use-preset: apa
bibliography:
  use-preset: vancouver
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();

        // Test Citation Preset (APA)
        let citation = style.citation.unwrap();
        assert!(citation.use_preset.is_some());
        assert!(citation.template.is_none());

        let citation_template = citation.resolve_template().unwrap();
        assert_eq!(citation_template.len(), 2); // APA citation is (Author, Year)

        // precise check for APA structure
        match &citation_template[0] {
            template::TemplateComponent::Contributor(c) => {
                assert_eq!(c.contributor, template::ContributorRole::Author);
            }
            _ => panic!("Expected Contributor"),
        }

        // Test Bibliography Preset (Vancouver)
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
    fn test_preset_override_precedence() {
        let yaml = r#"
info:
  title: Override Test
citation:
  use-preset: apa
  template:
    - variable: doi
"#;
        let style: Style = serde_yaml::from_str(yaml).unwrap();
        let citation = style.citation.unwrap();

        // Should have both
        assert!(citation.use_preset.is_some());
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
}
