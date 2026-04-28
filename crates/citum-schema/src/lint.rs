//! Lint helpers for raw locales and styles.

use crate::citation::LocatorType;
use crate::locale::{GeneralTerm, Locale, RawLocale, TermForm, types::MessageSyntax};
use crate::options::Config;
use crate::template::{
    ContributorForm, ContributorRole, LabelForm as TemplateLabelForm, NumberVariable,
    RoleLabelForm, TemplateComponent, TemplateContributor,
};
use crate::{CitationSpec, Style, Template};

/// A single lint finding produced by locale or style validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    /// The severity of the finding.
    pub severity: LintSeverity,
    /// The dotted path that identifies the lint target.
    pub path: String,
    /// The human-readable diagnostic message.
    pub message: String,
}

/// The severity of a lint finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    /// A non-fatal issue that should be reviewed.
    Warning,
    /// A fatal issue that makes the locale invalid.
    Error,
}

/// A lint report containing all findings emitted for a locale or style.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LintReport {
    /// All findings collected during linting.
    pub findings: Vec<LintFinding>,
}

impl LintReport {
    fn warning(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.findings.push(LintFinding {
            severity: LintSeverity::Warning,
            path: path.into(),
            message: message.into(),
        });
    }

    fn error(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.findings.push(LintFinding {
            severity: LintSeverity::Error,
            path: path.into(),
            message: message.into(),
        });
    }

    /// Return true when this report contains at least one error finding.
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| finding.severity == LintSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocaleRequirementKind {
    General {
        term: GeneralTerm,
        form: TermForm,
    },
    Role {
        role: ContributorRole,
        form: TermForm,
    },
    Locator {
        locator: LocatorType,
        form: TermForm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocaleRequirement {
    path: String,
    kind: LocaleRequirementKind,
}

/// Validate a raw locale's message syntax and alias targets.
///
/// MF2 messages are checked for placeholder syntax, selector arity, and
/// wildcard coverage. Legacy aliases must point to an existing message key.
pub fn lint_raw_locale(raw: &RawLocale) -> LintReport {
    let mut report = LintReport::default();
    let uses_mf2 = raw
        .evaluation
        .as_ref()
        .is_some_and(|config| config.message_syntax == MessageSyntax::Mf2);

    for (message_id, message) in &raw.messages {
        if uses_mf2
            && (message.contains('{') || message.contains(".match"))
            && let Err(err) = lint_mf2_message(message)
        {
            report.error(
                format!("messages.{message_id}"),
                format!("invalid MF2 message: {err}"),
            );
        }
    }

    for (legacy_key, message_id) in &raw.legacy_term_aliases {
        if !raw.messages.contains_key(message_id) {
            report.error(
                format!("legacy-term-aliases.{legacy_key}"),
                format!("target '{message_id}' does not exist in messages"),
            );
        }
    }

    report
}

/// Validate a style's locale-dependent templates against a locale.
///
/// Missing term, role, or locator resolutions are reported as warnings.
pub fn lint_style_against_locale(style: &Style, locale: &Locale) -> LintReport {
    let mut report = LintReport::default();
    let requirements = collect_style_locale_requirements(style);

    for requirement in requirements {
        match requirement.kind {
            LocaleRequirementKind::General { term, form } => {
                if locale.resolved_general_term(&term, form, None).is_none() {
                    report.warning(
                        requirement.path,
                        format!(
                            "locale does not resolve general term '{term:?}' in form '{form:?}'"
                        ),
                    );
                }
            }
            LocaleRequirementKind::Role { role, form } => {
                let singular = locale.resolved_role_term(&role, false, form, None);
                let plural = locale.resolved_role_term(&role, true, form, None);
                if singular.is_none() || plural.is_none() {
                    report.warning(
                        requirement.path,
                        format!(
                            "locale does not fully resolve role term '{role:?}' in form '{form:?}'"
                        ),
                    );
                }
            }
            LocaleRequirementKind::Locator { locator, form } => {
                let singular = lint_locator_term(locale, &locator, false, form);
                let plural = lint_locator_term(locale, &locator, true, form);
                if singular.is_none() || plural.is_none() {
                    report.warning(
                        requirement.path,
                        format!(
                            "locale does not fully resolve locator term '{locator:?}' in form '{form:?}'"
                        ),
                    );
                }
            }
        }
    }

    report
}

fn lint_locator_term(
    locale: &Locale,
    locator: &LocatorType,
    plural: bool,
    form: TermForm,
) -> Option<String> {
    match locator {
        LocatorType::Custom(_) => locale
            .locators
            .get(locator)
            .and_then(|term| match form {
                TermForm::Long => term.long.as_ref(),
                TermForm::Short => term.short.as_ref(),
                TermForm::Symbol => term.symbol.as_ref(),
                _ => term.short.as_ref(),
            })
            .or_else(|| {
                locale.locators.get(locator).and_then(|term| {
                    term.long
                        .as_ref()
                        .or(term.short.as_ref())
                        .or(term.symbol.as_ref())
                })
            })
            .map(|forms| {
                if plural {
                    forms.plural.as_str().to_string()
                } else {
                    forms.singular.as_str().to_string()
                }
            }),
        _ => locale.resolved_locator_term(locator, plural, form, None),
    }
}

fn collect_style_locale_requirements(style: &Style) -> Vec<LocaleRequirement> {
    let mut requirements = Vec::new();
    let base_config = style.options.clone().unwrap_or_default();

    if let Some(template) = &style.templates {
        for (name, components) in template {
            collect_template_requirements(
                components,
                &format!("templates.{name}"),
                &base_config,
                &mut requirements,
            );
        }
    }

    if let Some(citation) = &style.citation {
        collect_citation_spec_requirements(citation, "citation", &base_config, &mut requirements);
    }

    if let Some(bibliography) = &style.bibliography {
        collect_bibliography_spec_requirements(
            bibliography,
            "bibliography",
            &base_config,
            &mut requirements,
        );
    }

    requirements
}

fn collect_citation_spec_requirements(
    spec: &CitationSpec,
    path: &str,
    base_config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    let effective_config = spec.options.as_ref().map_or_else(
        || base_config.clone(),
        |options| options.merged_with(base_config),
    );

    if let Some(template) = spec.resolve_template() {
        collect_template_requirements(
            &template,
            &format!("{path}.template"),
            &effective_config,
            requirements,
        );
    }
    if let Some(locales) = &spec.locales {
        for (index, localized) in locales.iter().enumerate() {
            collect_template_requirements(
                &localized.template,
                &format!("{path}.locales[{index}].template"),
                &effective_config,
                requirements,
            );
        }
    }
    if let Some(spec) = &spec.integral {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.integral"),
            &effective_config,
            requirements,
        );
    }
    if let Some(spec) = &spec.non_integral {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.non-integral"),
            &effective_config,
            requirements,
        );
    }
    if let Some(spec) = &spec.subsequent {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.subsequent"),
            &effective_config,
            requirements,
        );
    }
    if let Some(spec) = &spec.ibid {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.ibid"),
            &effective_config,
            requirements,
        );
    }
}

fn collect_bibliography_spec_requirements(
    spec: &crate::BibliographySpec,
    path: &str,
    base_config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    let effective_config = spec.options.as_ref().map_or_else(
        || base_config.clone(),
        |options| options.merged_with(base_config),
    );

    if let Some(template) = spec.resolve_template() {
        collect_template_requirements(
            &template,
            &format!("{path}.template"),
            &effective_config,
            requirements,
        );
    }
    if let Some(locales) = &spec.locales {
        for (index, localized) in locales.iter().enumerate() {
            collect_template_requirements(
                &localized.template,
                &format!("{path}.locales[{index}].template"),
                &effective_config,
                requirements,
            );
        }
    }
    if let Some(type_variants) = &spec.type_variants {
        for (selector, template) in type_variants {
            collect_template_requirements(
                template,
                &format!("{path}.type-variants[{selector:?}]"),
                &effective_config,
                requirements,
            );
        }
    }
    if let Some(groups) = &spec.groups {
        for (index, group) in groups.iter().enumerate() {
            if let Some(heading) = &group.heading
                && let crate::grouping::GroupHeading::Term { term, form } = heading
            {
                requirements.push(LocaleRequirement {
                    path: format!("{path}.groups[{index}].heading"),
                    kind: LocaleRequirementKind::General {
                        term: *term,
                        form: form.unwrap_or(TermForm::Long),
                    },
                });
            }
            if let Some(template) = &group.template {
                collect_template_requirements(
                    template,
                    &format!("{path}.groups[{index}].template"),
                    &effective_config,
                    requirements,
                );
            }
        }
    }
}

fn collect_template_requirements(
    template: &Template,
    path: &str,
    config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    for (index, component) in template.iter().enumerate() {
        let component_path = format!("{path}[{index}]");
        match component {
            TemplateComponent::Term(term) => requirements.push(LocaleRequirement {
                path: component_path,
                kind: LocaleRequirementKind::General {
                    term: term.term,
                    form: term.form.unwrap_or(TermForm::Long),
                },
            }),
            TemplateComponent::Contributor(contributor) => {
                collect_contributor_requirements(
                    contributor,
                    &component_path,
                    config,
                    requirements,
                );
            }
            TemplateComponent::Number(number) => {
                if let Some(form) = number.label_form.clone()
                    && let Some(locator) = number_variable_to_locator(number.number.clone())
                {
                    let term_form = match form {
                        TemplateLabelForm::Short => TermForm::Short,
                        TemplateLabelForm::Long => TermForm::Long,
                        TemplateLabelForm::Symbol => TermForm::Symbol,
                    };
                    requirements.push(LocaleRequirement {
                        path: component_path,
                        kind: LocaleRequirementKind::Locator {
                            locator,
                            form: term_form,
                        },
                    });
                }
            }
            TemplateComponent::Date(date) => {
                if matches!(date.date, crate::template::DateVariable::Issued) {
                    requirements.push(LocaleRequirement {
                        path: component_path.clone(),
                        kind: LocaleRequirementKind::General {
                            term: GeneralTerm::NoDate,
                            form: TermForm::Short,
                        },
                    });
                }
                if let Some(fallback) = &date.fallback {
                    collect_template_requirements(
                        fallback,
                        &format!("{component_path}.fallback"),
                        config,
                        requirements,
                    );
                }
            }
            TemplateComponent::Group(list) => {
                collect_template_requirements(
                    &list.group,
                    &format!("{component_path}.items"),
                    config,
                    requirements,
                );
            }
            _ => {}
        }
    }
}

fn collect_contributor_requirements(
    contributor: &TemplateContributor,
    path: &str,
    config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    if let Some(label) = &contributor.label {
        let role =
            role_label_term_to_role(&label.term).unwrap_or_else(|| contributor.contributor.clone());
        let form = match label.form {
            RoleLabelForm::Short => TermForm::Short,
            RoleLabelForm::Long => TermForm::Long,
        };
        requirements.push(LocaleRequirement {
            path: format!("{path}.label"),
            kind: LocaleRequirementKind::Role { role, form },
        });
        return;
    }

    let configured_preset = config.contributors.as_ref().and_then(|contributors| {
        contributors.effective_role_label_preset(&contributor.contributor)
    });
    if let Some(role_label_preset) = configured_preset {
        let form = match role_label_preset {
            crate::options::RoleLabelPreset::None => return,
            crate::options::RoleLabelPreset::VerbPrefix => TermForm::Verb,
            crate::options::RoleLabelPreset::VerbShortPrefix => TermForm::VerbShort,
            crate::options::RoleLabelPreset::ShortSuffix => TermForm::Short,
            crate::options::RoleLabelPreset::LongSuffix => TermForm::Long,
        };
        requirements.push(LocaleRequirement {
            path: path.to_string(),
            kind: LocaleRequirementKind::Role {
                role: contributor.contributor.clone(),
                form,
            },
        });
        return;
    }

    match contributor.form {
        ContributorForm::Verb => requirements.push(LocaleRequirement {
            path: path.to_string(),
            kind: LocaleRequirementKind::Role {
                role: contributor.contributor.clone(),
                form: TermForm::Verb,
            },
        }),
        ContributorForm::VerbShort => requirements.push(LocaleRequirement {
            path: path.to_string(),
            kind: LocaleRequirementKind::Role {
                role: contributor.contributor.clone(),
                form: TermForm::VerbShort,
            },
        }),
        ContributorForm::Long
            if matches!(
                contributor.contributor,
                ContributorRole::Editor | ContributorRole::Translator
            ) =>
        {
            requirements.push(LocaleRequirement {
                path: path.to_string(),
                kind: LocaleRequirementKind::Role {
                    role: contributor.contributor.clone(),
                    form: TermForm::Short,
                },
            });
        }
        _ => {}
    }
}

fn role_label_term_to_role(term: &str) -> Option<ContributorRole> {
    match term {
        "editor" => Some(ContributorRole::Editor),
        "translator" => Some(ContributorRole::Translator),
        "director" => Some(ContributorRole::Director),
        "recipient" => Some(ContributorRole::Recipient),
        "interviewer" => Some(ContributorRole::Interviewer),
        _ => None,
    }
}

fn number_variable_to_locator(number: NumberVariable) -> Option<LocatorType> {
    match number {
        NumberVariable::Volume | NumberVariable::NumberOfVolumes => Some(LocatorType::Volume),
        NumberVariable::Pages | NumberVariable::NumberOfPages => Some(LocatorType::Page),
        NumberVariable::ChapterNumber => Some(LocatorType::Chapter),
        NumberVariable::Issue => Some(LocatorType::Issue),
        NumberVariable::Number
        | NumberVariable::DocketNumber
        | NumberVariable::PatentNumber
        | NumberVariable::StandardNumber
        | NumberVariable::ReportNumber
        | NumberVariable::PrintingNumber
        | NumberVariable::CitationNumber
        | NumberVariable::CitationLabel => Some(LocatorType::Number),
        NumberVariable::PartNumber => Some(LocatorType::Part),
        NumberVariable::SupplementNumber => Some(LocatorType::Supplement),
        NumberVariable::Custom(kind) => Some(LocatorType::Custom(kind)),
        _ => None,
    }
}

/// Validate an MF2 message string for structural correctness.
///
/// Checks that:
/// - `{…}` placeholders use `$variable` syntax (not bare MF1 identifiers)
/// - `.match` blocks have one or more `$`-prefixed selectors and a full-arity
///   wildcard fallback arm whose key count equals the number of selectors
///   (e.g. `when *` for a single selector, `when * *` for two selectors)
///
/// Returns `Err(description)` on the first structural violation.
fn lint_mf2_message(message: &str) -> Result<(), String> {
    let trimmed = message.trim();
    if trimmed.starts_with(".match") {
        lint_mf2_match(trimmed)
    } else {
        lint_mf2_pattern(trimmed)
    }
}

/// Validate variable placeholders in a plain MF2 pattern string.
fn lint_mf2_pattern(pattern: &str) -> Result<(), String> {
    let mut cursor = 0usize;
    while let Some(offset) = pattern[cursor..].find('{') {
        let open = cursor + offset;
        let close = find_matching_brace(pattern, open)
            .ok_or_else(|| format!("unmatched '{{' at byte offset {open}"))?;
        let inner = pattern
            .get(open + 1..close)
            .ok_or_else(|| "invalid brace range".to_string())?
            .trim();
        if inner.is_empty() {
            return Err("empty placeholder {}".to_string());
        }
        if !inner.starts_with('$') {
            return Err(format!(
                "placeholder '{{{inner}}}' must use $variable syntax \
                 (bare identifiers are MF1, not MF2)"
            ));
        }
        cursor = close + 1;
    }
    Ok(())
}

/// Validate an MF2 `.match` block.
fn lint_mf2_match(message: &str) -> Result<(), String> {
    let (selector_count, variants) = lint_mf2_selectors(message)?;
    lint_mf2_variants(variants, selector_count)?;
    Ok(())
}

fn lint_mf2_selectors(message: &str) -> Result<(usize, &str), String> {
    let mut rest = message[".match".len()..].trim_start();
    let mut selector_count = 0usize;

    while rest.starts_with('{') {
        let close = find_matching_brace(rest, 0).ok_or("unmatched '{' in .match selector")?;
        let selector = rest.get(1..close).ok_or("invalid selector range")?.trim();
        if !selector.starts_with('$') {
            return Err(format!(
                "selector '{selector}' must start with $ (e.g. {{$count :plural}})"
            ));
        }
        selector_count += 1;
        rest = rest
            .get(close + 1..)
            .ok_or("missing when blocks after .match selector")?
            .trim_start();
    }

    if selector_count == 0 {
        return Err("missing selector after .match".to_string());
    }

    Ok((selector_count, rest))
}

fn lint_mf2_variants(variants: &str, selector_count: usize) -> Result<(), String> {
    let mut rest = variants.trim();
    let mut saw_when = false;
    let mut saw_wildcard = false;

    while !rest.is_empty() {
        if !rest.starts_with("when") {
            return Err(format!("expected 'when' block, found '{rest}'"));
        }

        saw_when = true;
        let after_when = rest["when".len()..].trim_start();
        let brace_pos = after_when
            .find('{')
            .ok_or("missing pattern body in when block")?;
        let key_text = after_when[..brace_pos].trim();
        let keys: Vec<&str> = key_text.split_whitespace().collect();
        if keys.len() != selector_count {
            return Err(format!(
                "when block has {} selector keys but .match has {selector_count} selectors",
                keys.len()
            ));
        }
        if keys.iter().all(|key| *key == "*") {
            saw_wildcard = true;
        }

        let open_brace_index = rest.len() - after_when.len() + brace_pos;
        let close_brace_index = find_matching_brace(rest, open_brace_index)
            .ok_or("unmatched '{' in when block pattern")?;
        rest = rest
            .get(close_brace_index + 1..)
            .ok_or("invalid when block range")?
            .trim_start();
    }

    if !saw_when {
        return Err("no 'when' blocks found in .match".to_string());
    }
    if !saw_wildcard {
        let wildcard = vec!["*"; selector_count].join(" ");
        return Err(format!(
            "missing wildcard 'when {wildcard}' fallback in .match"
        ));
    }

    Ok(())
}

fn find_matching_brace(input: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;

    for (index, ch) in input
        .char_indices()
        .skip_while(|(index, _)| *index < open_index)
    {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::types::{EvaluationConfig, MessageSyntax};
    use crate::template::{TemplateNumber, TemplateTerm};
    use std::collections::HashMap;

    #[test]
    fn test_lint_raw_locale_reports_invalid_mf2_syntax() {
        let raw = RawLocale {
            locale: "en-US".into(),
            evaluation: Some(EvaluationConfig {
                message_syntax: MessageSyntax::Mf2,
            }),
            messages: HashMap::from([(
                "term.page-label".into(),
                // MF1-style bare identifier — invalid in MF2 (must use $count)
                "{count}".into(),
            )]),
            ..Default::default()
        };

        let report = lint_raw_locale(&raw);

        assert!(report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.path == "messages.term.page-label"
                && finding.message.contains("invalid MF2 message")
        }));
    }

    #[test]
    fn test_lint_raw_locale_accepts_multi_selector_mf2() {
        let raw = RawLocale {
            locale: "es-ES".into(),
            evaluation: Some(EvaluationConfig {
                message_syntax: MessageSyntax::Mf2,
            }),
            messages: HashMap::from([(
                "role.editor.label-long".into(),
                ".match {$gender :select} {$count :plural}\nwhen feminine one {editora}\nwhen feminine * {editoras}\nwhen * * {equipo editorial}".into(),
            )]),
            ..Default::default()
        };

        let report = lint_raw_locale(&raw);

        assert!(!report.has_errors());
    }

    #[test]
    fn test_lint_raw_locale_rejects_multi_selector_arity_mismatch() {
        let raw = RawLocale {
            locale: "es-ES".into(),
            evaluation: Some(EvaluationConfig {
                message_syntax: MessageSyntax::Mf2,
            }),
            messages: HashMap::from([(
                "role.editor.label-long".into(),
                ".match {$gender :select} {$count :plural}\nwhen feminine {editora}\nwhen * * {equipo editorial}".into(),
            )]),
            ..Default::default()
        };

        let report = lint_raw_locale(&raw);

        assert!(report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.path == "messages.role.editor.label-long"
                && finding.message.contains("selector keys")
        }));
    }

    #[test]
    fn test_lint_raw_locale_rejects_missing_multi_selector_wildcard() {
        let raw = RawLocale {
            locale: "es-ES".into(),
            evaluation: Some(EvaluationConfig {
                message_syntax: MessageSyntax::Mf2,
            }),
            messages: HashMap::from([(
                "role.editor.label-long".into(),
                ".match {$gender :select} {$count :plural}\nwhen feminine one {editora}\nwhen feminine * {editoras}".into(),
            )]),
            ..Default::default()
        };

        let report = lint_raw_locale(&raw);

        assert!(report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.path == "messages.role.editor.label-long"
                && finding.message.contains("when * *")
        }));
    }

    #[test]
    fn test_lint_raw_locale_reports_missing_alias_target() {
        let raw = RawLocale {
            locale: "en-US".into(),
            messages: HashMap::from([("term.page-label".into(), "p.".into())]),
            legacy_term_aliases: HashMap::from([("page".into(), "term.page-label-long".into())]),
            ..Default::default()
        };

        let report = lint_raw_locale(&raw);

        assert!(report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.path == "legacy-term-aliases.page" && finding.message.contains("does not exist")
        }));
    }

    #[test]
    fn test_lint_style_against_locale_warns_for_missing_general_term() {
        let style = Style {
            citation: Some(CitationSpec {
                template: Some(vec![TemplateComponent::Term(TemplateTerm {
                    term: GeneralTerm::NoDate,
                    form: Some(TermForm::Short),
                    ..Default::default()
                })]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::default();

        let report = lint_style_against_locale(&style, &locale);

        assert!(!report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.severity == LintSeverity::Warning
                && finding.path == "citation.template[0]"
                && finding.message.contains("general term")
        }));
    }

    #[test]
    fn test_lint_style_against_locale_warns_for_missing_role_term() {
        let style = Style {
            citation: Some(CitationSpec {
                template: Some(vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Editor,
                    form: ContributorForm::Verb,
                    ..Default::default()
                })]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::default();

        let report = lint_style_against_locale(&style, &locale);

        assert!(!report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.severity == LintSeverity::Warning
                && finding.path == "citation.template[0]"
                && finding.message.contains("role term")
        }));
    }

    #[test]
    fn test_lint_style_against_locale_accepts_custom_locator_when_locale_defines_it() {
        let style = Style {
            citation: Some(CitationSpec {
                template: Some(vec![TemplateComponent::Number(TemplateNumber {
                    number: NumberVariable::Custom("reel".to_string()),
                    label_form: Some(TemplateLabelForm::Short),
                    ..Default::default()
                })]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
locators:
  reel:
    short:
      singular: "reel"
      plural: "reels"
"#,
        )
        .expect("custom locale should parse");

        let report = lint_style_against_locale(&style, &locale);

        assert!(
            report.findings.is_empty(),
            "unexpected findings: {:?}",
            report.findings
        );
    }

    #[test]
    fn test_lint_style_against_locale_warns_for_missing_custom_locator_term() {
        let style = Style {
            citation: Some(CitationSpec {
                template: Some(vec![TemplateComponent::Number(TemplateNumber {
                    number: NumberVariable::Custom("reel".to_string()),
                    label_form: Some(TemplateLabelForm::Short),
                    ..Default::default()
                })]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::default();

        let report = lint_style_against_locale(&style, &locale);

        assert!(report.findings.iter().any(|finding| {
            finding.severity == LintSeverity::Warning
                && finding.path == "citation.template[0]"
                && finding.message.contains("locator term")
        }));
    }
}
