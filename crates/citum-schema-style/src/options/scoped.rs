/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Typed scoped options that normalize resolved citation and bibliography specs.

use crate::options::bibliography::SubsequentAuthorSubstituteRule;
use crate::template::{TemplateComponent, WrapConfig, WrapPunctuation};
use crate::{BibliographySpec, CitationSpec, Style, Template};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Wrapper punctuation supported by citation and bibliography label options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LabelWrap {
    /// No outer punctuation.
    None,
    /// Parentheses.
    Parentheses,
    /// Brackets.
    Brackets,
    /// Superscript-style numeric labels.
    Superscript,
}

impl LabelWrap {
    /// Convert a supported punctuation style into a concrete wrap config.
    #[must_use]
    pub fn as_wrap_config(self) -> Option<WrapConfig> {
        match self {
            LabelWrap::None => None,
            LabelWrap::Parentheses => Some(WrapConfig::from(WrapPunctuation::Parentheses)),
            LabelWrap::Brackets => Some(WrapConfig::from(WrapPunctuation::Brackets)),
            LabelWrap::Superscript => None,
        }
    }
}

/// Wrapper punctuation supported by bibliography label options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum BibliographyLabelWrap {
    /// No outer punctuation.
    None,
    /// Parentheses.
    Parentheses,
    /// Brackets.
    Brackets,
}

impl BibliographyLabelWrap {
    /// Convert a supported punctuation style into a concrete wrap config.
    #[must_use]
    pub fn as_wrap_config(self) -> Option<WrapConfig> {
        match self {
            BibliographyLabelWrap::None => None,
            BibliographyLabelWrap::Parentheses => {
                Some(WrapConfig::from(WrapPunctuation::Parentheses))
            }
            BibliographyLabelWrap::Brackets => Some(WrapConfig::from(WrapPunctuation::Brackets)),
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

/// Bibliography label modes supported by scoped bibliography options.
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

/// Apply scoped citation and bibliography options to a resolved style.
pub(crate) fn apply_scoped_style_options(style: &mut Style) {
    if let Some(citation) = style.citation.as_mut() {
        apply_citation_options_recursive(citation);
    }
    if let Some(bibliography) = style.bibliography.as_mut() {
        apply_bibliography_options(bibliography);
    }
}

fn apply_citation_options_recursive(citation: &mut CitationSpec) {
    let options = citation.options.clone();

    if let Some(options) = options {
        if let Some(delimiter) = options.group_delimiter {
            citation.multi_cite_delimiter = Some(delimiter.as_str().to_string());
        }
        if let Some(wrap) = options.label_wrap {
            if citation.template.is_none() && citation.use_preset.is_some() {
                citation.template = citation.resolve_template();
            }
            set_citation_wrap(citation, wrap);
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
        apply_citation_options_recursive(child);
    }
}

fn apply_bibliography_options(bibliography: &mut BibliographySpec) {
    let options = bibliography.options.clone();
    let Some(options) = options else {
        return;
    };

    let needs_template = options.label_mode.is_some()
        || options.label_wrap.is_some()
        || options.date_position.is_some()
        || options.title_terminator.is_some();
    if needs_template && bibliography.template.is_none() && bibliography.use_preset.is_some() {
        bibliography.template = bibliography.resolve_template();
    }

    if let Some(mode) = options.label_mode {
        apply_bibliography_label_mode(bibliography, mode);
    }
    if let Some(wrap) = options.label_wrap {
        apply_bibliography_label_wrap(bibliography, wrap);
    }
    if let Some(position) = options.date_position {
        apply_date_position(bibliography, position);
    }
    if let Some(terminator) = options.title_terminator {
        apply_title_terminator(bibliography, terminator);
    }
    if let Some(repeated) = options.repeated_author_rendering {
        apply_repeated_author_rendering(bibliography, repeated);
    }
}

fn set_citation_wrap(citation: &mut CitationSpec, wrap: LabelWrap) {
    apply_citation_wrap_recursive(citation, wrap, false);
}

fn apply_bibliography_label_mode(bibliography: &mut BibliographySpec, mode: BibliographyLabelMode) {
    update_label_mode(bibliography.template.as_mut(), mode);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            update_label_mode(Some(template), mode);
        }
    }
}

fn apply_bibliography_label_wrap(bibliography: &mut BibliographySpec, wrap: BibliographyLabelWrap) {
    update_label_wrap(bibliography.template.as_mut(), wrap);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            update_label_wrap(Some(template), wrap);
        }
    }
}

fn apply_date_position(bibliography: &mut BibliographySpec, position: DatePosition) {
    reposition_date(bibliography.template.as_mut(), position);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            reposition_date(Some(template), position);
        }
    }
}

fn apply_title_terminator(bibliography: &mut BibliographySpec, terminator: TitleTerminator) {
    update_title_terminator(bibliography.template.as_mut(), terminator);
    if let Some(variants) = bibliography.type_variants.as_mut() {
        for template in variants.values_mut() {
            update_title_terminator(Some(template), terminator);
        }
    }
}

fn apply_repeated_author_rendering(
    bibliography: &mut BibliographySpec,
    repeated: RepeatedAuthorRendering,
) {
    let options = bibliography.options.get_or_insert_with(Default::default);
    match repeated {
        RepeatedAuthorRendering::Full => {
            options.subsequent_author_substitute = None;
            options.subsequent_author_substitute_rule = None;
        }
        RepeatedAuthorRendering::Dash => {
            options.subsequent_author_substitute = Some("———".to_string());
            options.subsequent_author_substitute_rule =
                Some(SubsequentAuthorSubstituteRule::CompleteAll);
        }
        RepeatedAuthorRendering::DashWithSpace => {
            options.subsequent_author_substitute = Some("——— ".to_string());
            options.subsequent_author_substitute_rule =
                Some(SubsequentAuthorSubstituteRule::CompleteAll);
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

trait LabelWrapConfig {
    fn wrap_config(self) -> Option<WrapConfig>;
}

impl LabelWrapConfig for LabelWrap {
    fn wrap_config(self) -> Option<WrapConfig> {
        self.as_wrap_config()
    }
}

impl LabelWrapConfig for BibliographyLabelWrap {
    fn wrap_config(self) -> Option<WrapConfig> {
        self.as_wrap_config()
    }
}

fn update_label_wrap<W>(template: Option<&mut Template>, wrap: W)
where
    W: Copy + LabelWrapConfig,
{
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
            number.rendering.wrap = wrap.wrap_config();
        }
    }
}

fn update_citation_label_rendering(
    template: Option<&mut Template>,
    wrap: Option<WrapConfig>,
    vertical_align: Option<crate::VerticalAlign>,
) {
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
            number.rendering.vertical_align = vertical_align.clone();
            number.rendering.wrap = wrap.clone();
        }
    }
}

fn apply_citation_wrap_recursive(
    citation: &mut CitationSpec,
    wrap: LabelWrap,
    component_wrap_mode: bool,
) {
    if citation.template.is_none() && citation.use_preset.is_some() {
        citation.template = citation.resolve_template();
    }

    if wrap == LabelWrap::Superscript {
        citation.wrap = None;
        update_citation_label_rendering(
            citation.template.as_mut(),
            None,
            Some(crate::VerticalAlign::Superscript),
        );
        if let Some(variants) = citation.type_variants.as_mut() {
            for template in variants.values_mut() {
                update_citation_label_rendering(
                    Some(template),
                    None,
                    Some(crate::VerticalAlign::Superscript),
                );
            }
        }
    } else if component_wrap_mode {
        citation.wrap = None;
        update_citation_label_rendering(citation.template.as_mut(), wrap.as_wrap_config(), None);
        if let Some(variants) = citation.type_variants.as_mut() {
            for template in variants.values_mut() {
                update_citation_label_rendering(Some(template), wrap.as_wrap_config(), None);
            }
        }
    } else {
        citation.wrap = wrap.as_wrap_config();
        update_citation_label_rendering(citation.template.as_mut(), None, None);
        if let Some(variants) = citation.type_variants.as_mut() {
            for template in variants.values_mut() {
                update_citation_label_rendering(Some(template), None, None);
            }
        }
    }

    if let Some(child) = citation.integral.as_deref_mut() {
        apply_citation_wrap_recursive(child, wrap, true);
    }
    for child in [
        citation.non_integral.as_deref_mut(),
        citation.subsequent.as_deref_mut(),
        citation.ibid.as_deref_mut(),
    ]
    .into_iter()
    .flatten()
    {
        apply_citation_wrap_recursive(child, wrap, component_wrap_mode);
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
