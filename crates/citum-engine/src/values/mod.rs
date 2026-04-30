/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Value extraction for template components.
//!
//! This module provides the logic to extract formatted values from references
//! based on template component specifications.

/// Contributor extraction and name-formatting helpers.
pub mod contributor;
/// Date extraction and date-formatting helpers.
pub mod date;
/// List-component value extraction helpers.
pub mod list;
/// Locator rendering logic.
pub mod locator;
/// Numeric variable extraction and page-range helpers.
pub mod number;
/// Shared helpers for collapsing consecutive numeric or ordinal numbering.
pub mod range;
/// Locale term resolution helpers.
pub mod term;
/// Title text-case transform functions.
pub mod text_case;
/// Title extraction and title-formatting helpers.
pub mod title;
/// Generic variable extraction helpers.
pub mod variable;

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
mod tests;

use crate::reference::Reference;
use citum_schema::locale::Locale;
use citum_schema::options::{Config, bibliography::BibliographyConfig};
use citum_schema::reference::types::Title;
use citum_schema::template::{TemplateComponent, TitleType};

pub use contributor::format_contributors_short;
pub use date::int_to_letter;

/// Resolve preferred transliteration from a map of transliterations.
///
/// Applies priority-based matching:
/// 1. Preferred transliteration list: exact match
/// 2. Preferred transliteration list: substring match
/// 3. Preferred script: exact match
/// 4. Preferred script: substring match
fn resolve_transliteration<'a>(
    transliterations: &'a std::collections::HashMap<String, String>,
    preferred_transliteration: Option<&[String]>,
    preferred_script: Option<&String>,
) -> Option<&'a str> {
    // 1. Priority list: exact match
    if let Some(tags) = preferred_transliteration {
        for tag in tags {
            if let Some(v) = transliterations.get(tag) {
                return Some(v.as_str());
            }
        }
        // 2. Priority list: substring match
        for tag in tags {
            for (k, v) in transliterations {
                if k.contains(tag.as_str()) {
                    return Some(v.as_str());
                }
            }
        }
    }
    // 3. preferred_script exact match
    if let Some(script) = preferred_script {
        if let Some(v) = transliterations.get(script) {
            return Some(v.as_str());
        }
        // 4. preferred_script substring match
        for (k, v) in transliterations {
            if k.contains(script.as_str()) {
                return Some(v.as_str());
            }
        }
    }
    None
}

/// Resolve a multilingual string based on style configuration.
///
/// Applies BCP 47 fallback logic:
/// 1. Exact tag match (e.g., "ja-Latn-hepburn")
/// 2. Script prefix match (e.g., "ja-Latn")
/// 3. Fallback to original field
///
/// # Arguments
/// * `string` - The multilingual string to resolve
/// * `mode` - The rendering mode from style config
/// * `preferred_transliteration` - Optional ordered list of BCP 47 transliteration tags
/// * `preferred_script` - Optional preferred script (e.g., "Latn")
/// * `style_locale` - The style's locale for translation matching
#[must_use]
pub fn resolve_multilingual_string(
    string: &citum_schema::reference::types::MultilingualString,
    mode: Option<&citum_schema::options::MultilingualMode>,
    preferred_transliteration: Option<&[String]>,
    preferred_script: Option<&String>,
    style_locale: &str,
) -> String {
    use citum_schema::options::MultilingualMode;
    use citum_schema::reference::types::MultilingualString;

    match string {
        MultilingualString::Simple(s) => s.clone(),
        MultilingualString::Complex(complex) => {
            let mode = mode.unwrap_or(&MultilingualMode::Primary);

            match mode {
                MultilingualMode::Primary => complex.original.clone(),

                MultilingualMode::Transliterated => {
                    if let Some(trans) = resolve_transliteration(
                        &complex.transliterations,
                        preferred_transliteration,
                        preferred_script,
                    ) {
                        return trans.to_string();
                    }

                    // Fallback: use any available transliteration, or original
                    complex
                        .transliterations
                        .values()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| complex.original.clone())
                }

                MultilingualMode::Translated => {
                    // Try to match style locale
                    complex
                        .translations
                        .get(style_locale)
                        .cloned()
                        .unwrap_or_else(|| complex.original.clone())
                }

                MultilingualMode::Combined => {
                    // Format: "transliterated [translated]" or fallback variants
                    let trans = resolve_transliteration(
                        &complex.transliterations,
                        preferred_transliteration,
                        preferred_script,
                    );

                    let translation = complex.translations.get(style_locale);

                    match (trans, translation) {
                        (Some(t), Some(tr)) => format!("{t} [{tr}]"),
                        (Some(t), None) => t.to_string(),
                        (None, Some(tr)) => format!("{} [{}]", complex.original, tr),
                        (None, None) => complex.original.clone(),
                    }
                }
            }
        }
    }
}

/// Resolve the effective language for one logical field scope on a reference.
///
/// This prefers an explicit `field_languages` entry, then a multilingual title
/// language tag for the provided title value, and finally the reference-level
/// language.
#[must_use]
pub fn effective_field_language(
    reference: &Reference,
    scope: &str,
    title: Option<&Title>,
) -> Option<String> {
    reference
        .field_languages()
        .get(scope)
        .map(ToString::to_string)
        .or_else(|| match title {
            Some(Title::Multilingual(multilingual)) => {
                multilingual.lang.as_ref().map(ToString::to_string)
            }
            _ => None,
        })
        .or_else(|| reference.language().map(|lang| lang.to_string()))
}

/// Resolve the effective language for the primary title of a reference.
#[must_use]
pub fn effective_item_language(reference: &Reference) -> Option<String> {
    effective_field_language(reference, "title", reference.title().as_ref())
}

/// Resolve the effective language for the specific template component being rendered.
#[must_use]
pub fn effective_component_language(
    reference: &Reference,
    component: &TemplateComponent,
) -> Option<String> {
    match component {
        TemplateComponent::Title(title_component) => {
            let title = match title_component.title {
                TitleType::Primary => reference.title(),
                TitleType::ParentMonograph => reference.container_title(),
                TitleType::ParentSerial => reference.container_title(),
                _ => reference.title(),
            };

            let scope = match title_component.title {
                TitleType::Primary => "title",
                TitleType::ParentMonograph => "parent-monograph.title",
                TitleType::ParentSerial => "parent-serial.title",
                _ => "title",
            };

            effective_field_language(reference, scope, title.as_ref())
        }
        _ => effective_item_language(reference),
    }
}

/// Select a structured name from transliteration maps using priority-list then script-match rules.
fn select_by_transliteration<'a>(
    m: &'a citum_schema::reference::contributor::MultilingualName,
    preferred_transliteration: Option<&[String]>,
    preferred_script: Option<&String>,
) -> &'a citum_schema::reference::contributor::StructuredName {
    // 1. Priority list: exact match
    if let Some(tags) = preferred_transliteration {
        for tag in tags {
            if let Some(name) = m.transliterations.get(tag) {
                return name;
            }
        }
        // 2. Priority list: substring match
        for tag in tags {
            if let Some((_, name)) = m
                .transliterations
                .iter()
                .find(|(k, _)| k.contains(tag.as_str()))
            {
                return name;
            }
        }
    }
    // 3. Preferred script: exact match
    if let Some(script) = preferred_script {
        if let Some(name) = m.transliterations.get(script) {
            return name;
        }
        // 4. Preferred script: substring match
        if let Some((_, name)) = m
            .transliterations
            .iter()
            .find(|(tag, _)| tag.contains(script))
        {
            return name;
        }
    }
    // Fallback: any available transliteration before falling back to original
    m.transliterations.values().next().unwrap_or(&m.original)
}

/// Resolve a multilingual contributor name based on style configuration.
///
/// Uses holistic name matching - selects the entire name variant (original/transliterated/translated)
/// as a unit rather than mixing fields from different variants.
///
/// # Arguments
/// * `contributor` - The contributor to resolve
/// * `mode` - The rendering mode from style config
/// * `preferred_transliteration` - Optional ordered list of BCP 47 transliteration tags
/// * `preferred_script` - Optional preferred script (e.g., "Latn")
/// * `style_locale` - The style's locale for translation matching
#[must_use]
pub fn resolve_multilingual_name(
    contributor: &citum_schema::reference::contributor::Contributor,
    mode: Option<&citum_schema::options::MultilingualMode>,
    preferred_transliteration: Option<&[String]>,
    preferred_script: Option<&String>,
    style_locale: &str,
) -> Vec<crate::reference::FlatName> {
    use citum_schema::options::MultilingualMode;
    use citum_schema::reference::contributor::Contributor;

    match contributor {
        // Simple and structured names have no multilingual data
        Contributor::SimpleName(_) | Contributor::StructuredName(_) => contributor.to_names_vec(),

        // Multilingual names: select variant holistically
        Contributor::Multilingual(m) => {
            let mode = mode.unwrap_or(&MultilingualMode::Primary);

            let selected_name = match mode {
                MultilingualMode::Primary => &m.original,
                MultilingualMode::Transliterated => {
                    select_by_transliteration(m, preferred_transliteration, preferred_script)
                }
                MultilingualMode::Translated => {
                    m.translations.get(style_locale).unwrap_or(&m.original)
                }
                // Combined mode for names defaults to transliterated (parenthetical combo not common for names)
                MultilingualMode::Combined => {
                    select_by_transliteration(m, preferred_transliteration, preferred_script)
                }
            };

            // Convert selected name to FlatName
            vec![crate::reference::FlatName {
                given: Some(selected_name.given.to_string()),
                family: Some(selected_name.family.to_string()),
                suffix: selected_name.suffix.clone(),
                dropping_particle: selected_name.dropping_particle.clone(),
                non_dropping_particle: selected_name.non_dropping_particle.clone(),
                literal: None,
            }]
        }

        Contributor::ContributorList(l) => {
            l.0.iter()
                .flat_map(|c| {
                    resolve_multilingual_name(
                        c,
                        mode,
                        preferred_transliteration,
                        preferred_script,
                        style_locale,
                    )
                })
                .collect()
        }
    }
}

/// Resolve the URL for a component based on its links configuration and the reference data.
#[must_use]
pub fn resolve_url(
    links: &citum_schema::options::LinksConfig,
    reference: &Reference,
) -> Option<String> {
    use citum_schema::options::LinkTarget;

    let target = links.target.as_ref().unwrap_or(&LinkTarget::UrlOrDoi);

    match target {
        LinkTarget::Url => reference.url().map(|u| u.to_string()),
        LinkTarget::Doi => reference.doi().map(|d| format!("https://doi.org/{d}")),
        LinkTarget::UrlOrDoi => reference
            .url()
            .map(|u| u.to_string())
            .or_else(|| reference.doi().map(|d| format!("https://doi.org/{d}"))),
        LinkTarget::Pubmed => reference
            .id()
            .filter(|id| id.starts_with("pmid:"))
            .map(|id| {
                #[allow(clippy::string_slice, reason = "known ASCII prefix")]
                let result = format!("https://pubmed.ncbi.nlm.nih.gov/{}/", &id[5..]);
                result
            }),
        LinkTarget::Pmcid => reference
            .id()
            .filter(|id| id.starts_with("pmc:"))
            .map(|id| {
                #[allow(clippy::string_slice, reason = "known ASCII prefix")]
                let result = format!("https://www.ncbi.nlm.nih.gov/pmc/articles/{}/", &id[4..]);
                result
            }),
    }
}

/// Resolve the effective URL for a component, checking local links then falling back to global config.
#[must_use]
pub fn resolve_effective_url(
    local_links: Option<&citum_schema::options::LinksConfig>,
    global_links: Option<&citum_schema::options::LinksConfig>,
    reference: &Reference,
    component_anchor: citum_schema::options::LinkAnchor,
) -> Option<String> {
    use citum_schema::options::LinkAnchor;

    // 1. Check local links first
    if let Some(links) = local_links {
        let anchor = links.anchor.as_ref().unwrap_or(&LinkAnchor::Component);
        if matches!(anchor, LinkAnchor::Component) || *anchor == component_anchor {
            return resolve_url(links, reference);
        }
    }

    // 2. Fall back to global links if anchor matches this component type
    if let Some(links) = global_links
        && let Some(anchor) = &links.anchor
        && *anchor == component_anchor
    {
        return resolve_url(links, reference);
    }

    None
}

/// Processed values ready for rendering.
#[derive(Debug, Clone, Default)]
pub struct ProcValues<T = String> {
    /// The primary formatted value.
    pub value: T,
    /// Optional prefix to prepend.
    pub prefix: Option<String>,
    /// Optional suffix to append.
    pub suffix: Option<String>,
    /// Optional URL for hyperlinking.
    pub url: Option<String>,
    /// Variable key that was substituted (e.g., "title:Primary" when title replaces author).
    /// Used to prevent duplicate rendering per CSL variable-once rule.
    pub substituted_key: Option<String>,
    /// Whether the value is already pre-formatted.
    pub pre_formatted: bool,
}

/// Processing hints computed before rendering a reference or citation item.
#[derive(Debug, Clone, Default)]
pub struct ProcHints {
    /// Whether disambiguation is active (triggers year-suffix).
    pub disamb_condition: bool,
    /// Index in the disambiguation group (1-based).
    pub group_index: usize,
    /// Total size of the disambiguation group.
    pub group_length: usize,
    /// The grouping key used.
    pub group_key: String,
    /// Whether to expand given names for disambiguation.
    pub expand_given_names: bool,
    /// Minimum number of names to show to resolve ambiguity (overrides et-al-use-first).
    pub min_names_to_show: Option<usize>,
    /// Citation number for numeric citation styles (1-based).
    pub citation_number: Option<usize>,
    /// Optional sub-label for compound numeric citation addressing (e.g., "a" in "1a").
    pub citation_sub_label: Option<String>,
    /// Citation position (first, subsequent, ibid, etc.).
    pub position: Option<citum_schema::citation::Position>,
    /// Explicit integral citation name-memory state for this rendered item.
    pub integral_name_state: Option<citum_schema::citation::IntegralNameState>,
}

/// Context for rendering (citation vs bibliography).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderContext {
    #[default]
    /// Render values for citation output.
    Citation,
    /// Render values for bibliography output.
    Bibliography,
}

/// Options for rendering.
#[derive(Clone)]
pub struct RenderOptions<'a> {
    /// Effective configuration after style and default resolution.
    pub config: &'a Config,
    /// Effective bibliography-only configuration when rendering bibliography behavior.
    pub bibliography_config: Option<BibliographyConfig>,
    /// Locale used for term lookup and locale-sensitive formatting.
    pub locale: &'a Locale,
    /// Whether the current render target is a citation or bibliography.
    pub context: RenderContext,
    /// Citation mode for the current render operation.
    pub mode: citum_schema::citation::CitationMode,
    /// Whether to suppress the author name for this citation.
    /// Set from the citation-level `suppress_author` flag.
    pub suppress_author: bool,
    /// Optional raw citation locator for rendering via locator config.
    pub locator_raw: Option<&'a citum_schema::citation::CitationLocator>,
    /// Reference type for optional type-class gating in locator patterns.
    pub ref_type: Option<String>,
    /// Whether to output semantic markup (HTML spans, Djot attributes).
    pub show_semantics: bool,
    /// The current top-level template index, when propagating preview annotations.
    pub current_template_index: Option<usize>,
}

/// Trait for extracting values from template components.
pub trait ComponentValues {
    /// Resolve the component into processed render values for one reference.
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>>;
}

impl ComponentValues for TemplateComponent {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        match self {
            TemplateComponent::Contributor(c) => c.values::<F>(reference, hints, options),
            TemplateComponent::Date(d) => d.values::<F>(reference, hints, options),
            TemplateComponent::Title(t) => t.values::<F>(reference, hints, options),
            TemplateComponent::Number(n) => n.values::<F>(reference, hints, options),
            TemplateComponent::Variable(v) => v.values::<F>(reference, hints, options),
            TemplateComponent::Group(l) => l.values::<F>(reference, hints, options),
            TemplateComponent::Term(t) => t.values::<F>(reference, hints, options),
            _ => None,
        }
    }
}

/// Check if periods should be stripped based on three-tier precedence.
///
/// Resolution order:
/// 1. Component-level `strip_periods`
/// 2. Global config `strip_periods`
/// 3. Defaults to false
#[must_use]
pub fn should_strip_periods(
    rendering: &citum_schema::template::Rendering,
    options: &RenderOptions<'_>,
) -> bool {
    rendering
        .strip_periods
        .or(options.config.strip_periods)
        .unwrap_or(false)
}

/// Strip trailing periods from a string.
///
/// Only removes periods at the end of the string, preserves internal periods
/// (e.g., "Ph.D." remains unchanged if there's no trailing period).
#[must_use]
pub fn strip_trailing_periods(s: &str) -> String {
    s.trim_end_matches('.').to_string()
}
