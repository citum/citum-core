/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Role-label resolution for contributor rendering.

use super::contributor_role_to_reference_role;
use crate::reference::Reference;
use crate::render::format::OutputFormat;
use crate::values::{RenderContext, RenderOptions};
use citum_schema::locale::{GrammaticalGender, TermForm};
use citum_schema::options::RoleLabelPreset;
use citum_schema::reference::ContributorGender;
use citum_schema::template::{ContributorForm, ContributorRole, Rendering, TemplateContributor};

fn map_contributor_gender(gender: ContributorGender) -> GrammaticalGender {
    match gender {
        ContributorGender::Masculine => GrammaticalGender::Masculine,
        ContributorGender::Feminine => GrammaticalGender::Feminine,
        ContributorGender::Neuter => GrammaticalGender::Neuter,
        ContributorGender::Common => GrammaticalGender::Common,
    }
}

#[derive(Debug, Clone)]
pub(super) enum RoleGenderRequest {
    Specific(GrammaticalGender),
    Neutral,
}

/// How to resolve and transform a role-label term: the requested grammatical
/// gender and an optional `text-case` transform applied to the resolved term.
#[derive(Debug, Clone, Default)]
pub(super) struct RoleLabelTermOptions {
    pub gender: Option<RoleGenderRequest>,
    pub text_case: Option<citum_schema::options::titles::TextCase>,
}

fn requested_role_gender(
    component: &TemplateContributor,
    reference: &Reference,
) -> Option<RoleGenderRequest> {
    if let Some(gender) = &component.gender {
        return Some(RoleGenderRequest::Specific(gender.clone()));
    }

    let data_role = contributor_role_to_reference_role(&component.contributor)?;

    let entries = reference.contributor_entries(&data_role);
    let mut genders = entries
        .iter()
        .filter_map(|entry| entry.gender.map(map_contributor_gender));
    let first = genders.next()?;

    if genders.all(|gender| gender == first) {
        Some(RoleGenderRequest::Specific(first))
    } else {
        Some(RoleGenderRequest::Neutral)
    }
}

fn resolve_role_term_by_request(
    locale: &citum_schema::locale::Locale,
    role: &ContributorRole,
    plural: bool,
    term_form: TermForm,
    requested_gender: Option<RoleGenderRequest>,
) -> Option<String> {
    match requested_gender {
        Some(RoleGenderRequest::Specific(gender)) => {
            locale.resolved_role_term(role, plural, &term_form, Some(gender))
        }
        Some(RoleGenderRequest::Neutral) => {
            locale.resolved_role_term_neutral(role, plural, &term_form)
        }
        None => locale.resolved_role_term(role, plural, &term_form, None),
    }
}

/// Resolve a configured role-label preset to `(prefix, suffix)`.
pub(super) fn resolve_role_label_preset<F: OutputFormat<Output = String>>(
    role: &ContributorRole,
    preset: RoleLabelPreset,
    names_count: usize,
    term_opts: RoleLabelTermOptions,
    effective_rendering: &Rendering,
    options: &RenderOptions<'_>,
    fmt: &F,
) -> (Option<String>, Option<String>) {
    let plural = names_count > 1;
    let language = options.locale.locale.as_str();
    let RoleLabelTermOptions {
        gender: requested_gender,
        text_case,
    } = term_opts;
    match preset {
        RoleLabelPreset::None => (None, None),
        RoleLabelPreset::VerbPrefix => {
            let term = options
                .locale
                .resolved_role_term(role, plural, &TermForm::Verb, None);
            (
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        RoleLabelPreset::VerbShortPrefix => {
            let term = options
                .locale
                .resolved_role_term(role, plural, &TermForm::VerbShort, None);
            (
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        RoleLabelPreset::ShortSuffix => {
            let term = resolve_role_term_by_request(
                options.locale,
                role,
                plural,
                TermForm::Short,
                requested_gender,
            )
            .map(|t| apply_label_case(t, text_case, language));
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, " (", ")")
                }),
            )
        }
        RoleLabelPreset::ShortSuffixComma => {
            let term = resolve_role_term_by_request(
                options.locale,
                role,
                plural,
                TermForm::Short,
                requested_gender,
            )
            .map(|t| apply_label_case(t, text_case, language));
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, ", ", "")
                }),
            )
        }
        RoleLabelPreset::LongSuffix => {
            let term = resolve_role_term_by_request(
                options.locale,
                role,
                plural,
                TermForm::Long,
                requested_gender,
            )
            .map(|t| apply_label_case(t, text_case, language));
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, ", ", "")
                }),
            )
        }
    }
}

/// Apply an optional `text-case` transform to a resolved role-label term.
///
/// Used so a style can render, e.g., "Eds." from the locale's "eds." (IEEE).
/// `language` gates English-only transforms via [`resolve_text_case`].
fn apply_label_case(
    term: String,
    text_case: Option<citum_schema::options::titles::TextCase>,
    language: &str,
) -> String {
    match text_case {
        Some(case) => {
            let resolved = crate::values::text_case::resolve_text_case(case, Some(language));
            crate::values::text_case::apply_text_case(&term, resolved)
        }
        None => term,
    }
}

/// Resolve a contributor's explicit `label` config to `(prefix, suffix)`.
///
/// Honours the term key, short/long form, optional `text-case`, and placement.
/// `label.term` keys recognized by [`resolve_explicit_label`]. Shared with
/// the style-load-time warning scan (`api::warnings`) so the two can't drift.
pub(crate) const RECOGNIZED_LABEL_TERMS: &[&str] = &["chair", "editor", "translator"];

fn resolve_explicit_label<F: OutputFormat<Output = String>>(
    label_config: &citum_schema::template::RoleLabel,
    component: &TemplateContributor,
    reference: &Reference,
    names_count: usize,
    effective_rendering: &Rendering,
    options: &RenderOptions<'_>,
    fmt: &F,
) -> (Option<String>, Option<String>) {
    use citum_schema::template::{LabelPlacement, RoleLabelForm};

    let plural = names_count > 1;
    let term_form = match label_config.form {
        RoleLabelForm::Short => TermForm::Short,
        RoleLabelForm::Long => TermForm::Long,
    };

    let role = match label_config.term.as_str() {
        "chair" => Some(ContributorRole::Chair),
        "editor" => Some(ContributorRole::Editor),
        "translator" => Some(ContributorRole::Translator),
        _ => Some(component.contributor.clone()),
    };

    let requested_gender = requested_role_gender(component, reference);
    let term_text = role
        .and_then(|r| {
            resolve_role_term_by_request(options.locale, &r, plural, term_form, requested_gender)
        })
        .map(|t| apply_label_case(t, label_config.text_case, options.locale.locale.as_str()));

    // Explicit label affixes override the placement-derived defaults,
    // mirroring CSL 1.0 `cs:label` prefix/suffix (e.g. `" ("`/`")"`).
    let (default_before, default_after) = match label_config.placement {
        LabelPlacement::Prefix => ("", " "),
        LabelPlacement::Suffix => (", ", ""),
    };
    let before = label_config.prefix.as_deref().unwrap_or(default_before);
    let after = label_config.suffix.as_deref().unwrap_or(default_after);

    match label_config.placement {
        LabelPlacement::Prefix => (
            term_text.map(|t| {
                super::format_role_term::<F>(&t, fmt, effective_rendering, options, before, after)
            }),
            None,
        ),
        LabelPlacement::Suffix => (
            None,
            term_text.map(|t| {
                super::format_role_term::<F>(&t, fmt, effective_rendering, options, before, after)
            }),
        ),
    }
}

/// Resolve the role-label prefix and suffix for a formatted contributor.
///
/// Returns `(prefix, suffix)` strings to wrap the formatted name list.
/// Precedence: explicit `label` config > configured role presets > form-based defaults.
pub(super) fn resolve_role_labels<F: OutputFormat<Output = String>>(
    component: &TemplateContributor,
    reference: &Reference,
    names_count: usize,
    effective_rendering: &Rendering,
    options: &RenderOptions<'_>,
    fmt: &F,
    role_omitted: bool,
) -> (Option<String>, Option<String>) {
    if let Some(label_config) = &component.label {
        return resolve_explicit_label(
            label_config,
            component,
            reference,
            names_count,
            effective_rendering,
            options,
            fmt,
        );
    }

    // `role.omit` suppresses the *decorative* default/preset label (e.g. a
    // trailing "(Trans.)"), not a `form: verb`/`form: verb-short` label: the
    // verb phrase ("Translated by X") is structural to that form, not an
    // optional suffix, so a style that both requests verb form and omits the
    // role's default label (to avoid double-labeling under a different form)
    // must still get its verb label.
    if role_omitted
        && !matches!(
            component.form,
            ContributorForm::Verb | ContributorForm::VerbShort
        )
    {
        return (None, None);
    }

    if let Some(preset) = options
        .config
        .contributors
        .as_ref()
        .and_then(|contributors| contributors.effective_role_label_preset(&component.contributor))
    {
        let requested_gender = requested_role_gender(component, reference);
        return resolve_role_label_preset(
            &component.contributor,
            preset,
            names_count,
            RoleLabelTermOptions {
                gender: requested_gender,
                text_case: None,
            },
            effective_rendering,
            options,
            fmt,
        );
    }

    // Style-declared default bundle (`contributors.role.defaults`). Role
    // labels are a bibliography-only convention in every examined style
    // guide (div-012), so the bundle never fires in citation context.
    if options.context == RenderContext::Bibliography
        && !matches!(
            component.form,
            ContributorForm::Verb | ContributorForm::VerbShort
        )
        && let Some(preset) = options
            .config
            .contributors
            .as_ref()
            .and_then(|contributors| contributors.default_role_label_preset(&component.contributor))
    {
        let requested_gender = requested_role_gender(component, reference);
        return resolve_role_label_preset(
            &component.contributor,
            preset,
            names_count,
            RoleLabelTermOptions {
                gender: requested_gender,
                text_case: None,
            },
            effective_rendering,
            options,
            fmt,
        );
    }

    // Form-based defaults: verb forms carry their structural verb phrase;
    // no other form receives an automatic label.
    match (&component.form, &component.contributor) {
        (ContributorForm::Verb | ContributorForm::VerbShort, role) => {
            let plural = names_count > 1;
            let term_form = match component.form {
                ContributorForm::VerbShort => TermForm::VerbShort,
                _ => TermForm::Verb,
            };
            let term = options
                .locale
                .resolved_role_term(role, plural, &term_form, None);

            (
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        _ => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::apply_label_case;
    use citum_schema::options::titles::TextCase;

    #[test]
    fn capitalize_first_uppercases_label_initial() {
        // given the locale's lowercase short editor term and capitalize-first
        let out = apply_label_case("eds.".to_string(), Some(TextCase::CapitalizeFirst), "en-US");
        // then the IEEE-style capitalized form is produced
        assert_eq!(out, "Eds.");
    }

    #[test]
    fn no_text_case_leaves_term_unchanged() {
        // given no text-case transform
        let out = apply_label_case("eds.".to_string(), None, "en-US");
        // then the term is returned verbatim
        assert_eq!(out, "eds.");
    }
}
