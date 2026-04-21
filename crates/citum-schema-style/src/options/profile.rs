/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Typed profile-only override axes for config-backed style wrappers.

use crate::options::bibliography::SubsequentAuthorSubstituteRule;
use crate::presets::ContributorPreset;
use crate::template::{TemplateComponent, WrapConfig, WrapPunctuation};
use crate::{BibliographySpec, CitationSpec, Style, Template};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Profile-local configuration that may vary without overriding templates.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ProfileConfig {
    /// Citation wrapper punctuation for grouped citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_label_wrap: Option<ProfileWrap>,
    /// Delimiter between multiple citations in a grouped citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_group_delimiter: Option<CitationGroupDelimiter>,
    /// Bibliography label mode for label-bearing styles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography_label_mode: Option<BibliographyLabelMode>,
    /// Bibliography label wrapper punctuation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography_label_wrap: Option<ProfileWrap>,
    /// Placement of issued dates within bibliography entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_position: Option<DatePosition>,
    /// Delimiter between volume and pages in bibliography entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_pages_delimiter: Option<VolumePagesDelimiter>,
    /// Terminator applied to primary-title bibliography components.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_terminator: Option<TitleTerminator>,
    /// Profile-scoped contributor preset slot for this family of wrappers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributor_preset: Option<ContributorPreset>,
    /// Repeated-author rendering mode for bibliographies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeated_author_rendering: Option<RepeatedAuthorRendering>,
}

impl ProfileConfig {
    /// Merge another profile config over this one, field by field.
    pub fn merge(&mut self, other: &ProfileConfig) {
        crate::merge_options!(
            self,
            other,
            citation_label_wrap,
            citation_group_delimiter,
            bibliography_label_mode,
            bibliography_label_wrap,
            date_position,
            volume_pages_delimiter,
            title_terminator,
            contributor_preset,
            repeated_author_rendering,
        );
    }

    /// Apply this profile config to a resolved effective style.
    pub fn apply_to_style(&self, style: &mut Style) {
        if let Some(wrap) = self.citation_label_wrap {
            set_citation_wrap(style.citation.as_mut(), wrap);
        }
        if let Some(delimiter) = self.citation_group_delimiter
            && let Some(citation) = style.citation.as_mut()
        {
            citation.multi_cite_delimiter = Some(delimiter.as_str().to_string());
        }
        if let Some(mode) = self.bibliography_label_mode {
            apply_bibliography_label_mode(style.bibliography.as_mut(), mode);
        }
        if let Some(wrap) = self.bibliography_label_wrap {
            apply_bibliography_label_wrap(style.bibliography.as_mut(), wrap);
        }
        if let Some(position) = self.date_position {
            apply_date_position(style.bibliography.as_mut(), position);
        }
        if let Some(delimiter) = self.volume_pages_delimiter
            && let Some(options) = style
                .options
                .get_or_insert_default()
                .volume_pages_delimiter
                .as_mut()
        {
            *options = delimiter.into();
        } else if let Some(delimiter) = self.volume_pages_delimiter {
            style.options.get_or_insert_default().volume_pages_delimiter = Some(delimiter.into());
        }
        if let Some(terminator) = self.title_terminator {
            apply_title_terminator(style.bibliography.as_mut(), terminator);
        }
        if let Some(preset) = self.contributor_preset {
            style.options.get_or_insert_default().contributors = Some(preset.config());
        }
        if let Some(repeated) = self.repeated_author_rendering {
            let bib_opts = style
                .bibliography
                .get_or_insert_with(Default::default)
                .options
                .get_or_insert_with(Default::default);
            match repeated {
                RepeatedAuthorRendering::Full => {
                    bib_opts.subsequent_author_substitute = None;
                    bib_opts.subsequent_author_substitute_rule = None;
                }
                RepeatedAuthorRendering::Dash => {
                    bib_opts.subsequent_author_substitute = Some("———".to_string());
                    bib_opts.subsequent_author_substitute_rule =
                        Some(SubsequentAuthorSubstituteRule::CompleteAll);
                }
                RepeatedAuthorRendering::DashWithSpace => {
                    bib_opts.subsequent_author_substitute = Some("——— ".to_string());
                    bib_opts.subsequent_author_substitute_rule =
                        Some(SubsequentAuthorSubstituteRule::CompleteAll);
                }
            }
        }
    }
}

/// Wrapper punctuation supported by profile axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ProfileWrap {
    /// No outer punctuation.
    None,
    /// Parentheses.
    Parentheses,
    /// Brackets.
    Brackets,
    /// Superscript-style numeric labels.
    Superscript,
}

impl ProfileWrap {
    /// Convert a supported punctuation style into a concrete wrap config.
    #[must_use]
    pub fn as_wrap_config(self) -> Option<WrapConfig> {
        match self {
            ProfileWrap::None => None,
            ProfileWrap::Parentheses => Some(WrapConfig::from(WrapPunctuation::Parentheses)),
            ProfileWrap::Brackets => Some(WrapConfig::from(WrapPunctuation::Brackets)),
            ProfileWrap::Superscript => None,
        }
    }
}

/// Delimiters between grouped citations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum CitationGroupDelimiter {
    /// `, `
    Comma,
    /// `; `
    Semicolon,
    /// ` `
    Space,
}

impl CitationGroupDelimiter {
    fn as_str(self) -> &'static str {
        match self {
            CitationGroupDelimiter::Comma => ", ",
            CitationGroupDelimiter::Semicolon => "; ",
            CitationGroupDelimiter::Space => " ",
        }
    }
}

/// Bibliography label modes supported by profile wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum BibliographyLabelMode {
    /// No explicit label component.
    None,
    /// Numeric bibliography labels.
    Numeric,
    /// Author-date bibliography labels.
    AuthorDate,
}

/// Placement of issued dates inside bibliography entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum DatePosition {
    /// Immediately after the contributor component.
    AfterAuthor,
    /// Immediately after the title component.
    AfterTitle,
    /// At the end of the entry.
    Terminal,
}

/// Volume/pages delimiters used by profile wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum VolumePagesDelimiter {
    /// Comma.
    Comma,
    /// Colon.
    Colon,
    /// Space.
    Space,
}

impl From<VolumePagesDelimiter> for crate::template::DelimiterPunctuation {
    fn from(value: VolumePagesDelimiter) -> Self {
        match value {
            VolumePagesDelimiter::Comma => crate::template::DelimiterPunctuation::Comma,
            VolumePagesDelimiter::Colon => crate::template::DelimiterPunctuation::Colon,
            VolumePagesDelimiter::Space => crate::template::DelimiterPunctuation::Space,
        }
    }
}

/// Terminator punctuation for bibliography titles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TitleTerminator {
    /// Period.
    Period,
    /// Comma.
    Comma,
    /// No terminator.
    None,
}

impl TitleTerminator {
    fn as_suffix(self) -> Option<&'static str> {
        match self {
            TitleTerminator::Period => Some("."),
            TitleTerminator::Comma => Some(","),
            TitleTerminator::None => None,
        }
    }
}

/// Repeated-author rendering policies for bibliographies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum RepeatedAuthorRendering {
    /// Always render full contributor names.
    Full,
    /// Replace repeated authors with an em dash.
    Dash,
    /// Replace repeated authors with an em dash followed by a space.
    DashWithSpace,
}

fn set_citation_wrap(citation: Option<&mut CitationSpec>, wrap: ProfileWrap) {
    if let Some(citation) = citation {
        if wrap == ProfileWrap::Superscript {
            citation.wrap = None;
            apply_citation_wrap_recursive(citation, wrap);
            return;
        }
        citation.wrap = wrap.as_wrap_config();
        apply_citation_wrap_recursive(citation, wrap);
    }
}

fn apply_bibliography_label_mode(
    bibliography: Option<&mut BibliographySpec>,
    mode: BibliographyLabelMode,
) {
    let Some(bibliography) = bibliography else {
        return;
    };
    update_label_mode(bibliography.template.as_mut(), mode);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            update_label_mode(Some(template), mode);
        }
    }
}

fn apply_bibliography_label_wrap(bibliography: Option<&mut BibliographySpec>, wrap: ProfileWrap) {
    let Some(bibliography) = bibliography else {
        return;
    };
    update_label_wrap(bibliography.template.as_mut(), wrap);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            update_label_wrap(Some(template), wrap);
        }
    }
}

fn apply_date_position(bibliography: Option<&mut BibliographySpec>, position: DatePosition) {
    let Some(bibliography) = bibliography else {
        return;
    };
    reposition_date(bibliography.template.as_mut(), position);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            reposition_date(Some(template), position);
        }
    }
}

fn apply_title_terminator(
    bibliography: Option<&mut BibliographySpec>,
    terminator: TitleTerminator,
) {
    let Some(bibliography) = bibliography else {
        return;
    };
    update_title_terminator(bibliography.template.as_mut(), terminator);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            update_title_terminator(Some(template), terminator);
        }
    }
}

fn update_label_mode(template: Option<&mut Template>, mode: BibliographyLabelMode) {
    let Some(template) = template else {
        return;
    };
    match mode {
        BibliographyLabelMode::None | BibliographyLabelMode::AuthorDate => {
            template.retain(|component| {
                !matches!(
                    component,
                    TemplateComponent::Number(number)
                        if matches!(
                            number.number,
                            crate::template::NumberVariable::CitationNumber
                                | crate::template::NumberVariable::CitationLabel
                        )
                )
            });
        }
        BibliographyLabelMode::Numeric => {
            let has_label = template.iter().any(|component| {
                matches!(
                    component,
                    TemplateComponent::Number(number)
                        if matches!(number.number, crate::template::NumberVariable::CitationNumber)
                )
            });
            if !has_label {
                template.insert(
                    0,
                    TemplateComponent::Number(crate::TemplateNumber {
                        number: crate::template::NumberVariable::CitationNumber,
                        ..Default::default()
                    }),
                );
            }
        }
    }
}

fn update_label_wrap(template: Option<&mut Template>, wrap: ProfileWrap) {
    let Some(template) = template else {
        return;
    };
    for component in template.iter_mut() {
        if let TemplateComponent::Number(number) = component
            && matches!(
                number.number,
                crate::template::NumberVariable::CitationNumber
                    | crate::template::NumberVariable::CitationLabel
            )
        {
            number.rendering.wrap = wrap.as_wrap_config();
        }
    }
}

fn apply_citation_superscript(template: Option<&mut Template>) {
    let Some(template) = template else {
        return;
    };
    for component in template.iter_mut() {
        if let TemplateComponent::Number(number) = component
            && matches!(
                number.number,
                crate::template::NumberVariable::CitationNumber
                    | crate::template::NumberVariable::CitationLabel
            )
        {
            number.rendering.vertical_align = Some(crate::VerticalAlign::Superscript);
            number.rendering.wrap = None;
        }
    }
}

fn apply_citation_wrap_recursive(citation: &mut CitationSpec, wrap: ProfileWrap) {
    if wrap == ProfileWrap::Superscript && citation.template.is_none() {
        citation.template = citation.resolve_template();
    }

    if wrap == ProfileWrap::Superscript {
        apply_citation_superscript(citation.template.as_mut());
        if let Some(variants) = citation.type_variants.as_mut() {
            for template in variants.values_mut() {
                apply_citation_superscript(Some(template));
            }
        }
    } else {
        update_label_wrap(citation.template.as_mut(), wrap);
        if let Some(variants) = citation.type_variants.as_mut() {
            for template in variants.values_mut() {
                update_label_wrap(Some(template), wrap);
            }
        }
    }

    for child in [
        citation.integral.as_deref_mut(),
        citation.non_integral.as_deref_mut(),
        citation.subsequent.as_deref_mut(),
        citation.ibid.as_deref_mut(),
    ]
    .into_iter()
    .flatten()
    {
        child.wrap = if wrap == ProfileWrap::Superscript {
            None
        } else {
            wrap.as_wrap_config()
        };
        apply_citation_wrap_recursive(child, wrap);
    }
}

fn reposition_date(template: Option<&mut Template>, position: DatePosition) {
    let Some(template) = template else {
        return;
    };
    let Some(index) = template.iter().position(|component| {
        matches!(
            component,
            TemplateComponent::Date(date) if date.date == crate::template::DateVariable::Issued
        )
    }) else {
        return;
    };
    let date = template.remove(index);
    let target = match position {
        DatePosition::AfterAuthor => template
            .iter()
            .position(|component| matches!(component, TemplateComponent::Contributor(_)))
            .map(|idx| idx + 1)
            .unwrap_or(0),
        DatePosition::AfterTitle => template
            .iter()
            .position(|component| matches!(component, TemplateComponent::Title(_)))
            .map(|idx| idx + 1)
            .unwrap_or(template.len()),
        DatePosition::Terminal => template.len(),
    };
    template.insert(target, date);
}

fn update_title_terminator(template: Option<&mut Template>, terminator: TitleTerminator) {
    let Some(template) = template else {
        return;
    };
    for component in template.iter_mut() {
        if let TemplateComponent::Title(title) = component
            && title.title == crate::template::TitleType::Primary
        {
            title.rendering.suffix = terminator.as_suffix().map(ToString::to_string);
        }
    }
}

/// Capability metadata for a profile-capable base style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileAxisCapabilities {
    /// Whether the base supports citation-label-wrap.
    pub citation_label_wrap: &'static [ProfileWrap],
    /// Whether the base supports citation-group-delimiter.
    pub citation_group_delimiter: &'static [CitationGroupDelimiter],
    /// Whether the base supports bibliography-label-mode.
    pub bibliography_label_mode: &'static [BibliographyLabelMode],
    /// Whether the base supports bibliography-label-wrap.
    pub bibliography_label_wrap: &'static [ProfileWrap],
    /// Whether the base supports date-position.
    pub date_position: &'static [DatePosition],
    /// Whether the base supports volume-pages-delimiter.
    pub volume_pages_delimiter: &'static [VolumePagesDelimiter],
    /// Whether the base supports title-terminator.
    pub title_terminator: &'static [TitleTerminator],
    /// Which contributor presets the base exposes through a profile slot.
    pub contributor_preset: &'static [ContributorPreset],
    /// Whether the base supports repeated-author-rendering.
    pub repeated_author_rendering: &'static [RepeatedAuthorRendering],
}

impl ProfileAxisCapabilities {
    /// Capability block for bases that do not expose any profile axes.
    pub const NONE: Self = Self {
        citation_label_wrap: &[],
        citation_group_delimiter: &[],
        bibliography_label_mode: &[],
        bibliography_label_wrap: &[],
        date_position: &[],
        volume_pages_delimiter: &[],
        title_terminator: &[],
        contributor_preset: &[],
        repeated_author_rendering: &[],
    };
}
