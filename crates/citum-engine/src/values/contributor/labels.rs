//! Role-label resolution for contributor rendering.

use crate::render::format::OutputFormat;
use crate::values::RenderOptions;
use citum_schema::locale::TermForm;
use citum_schema::options::RoleLabelPreset;
use citum_schema::template::{ContributorForm, ContributorRole, Rendering, TemplateContributor};

/// Resolve a configured role-label preset to `(prefix, suffix)`.
pub(super) fn resolve_role_label_preset<F: OutputFormat<Output = String>>(
    role: &ContributorRole,
    preset: RoleLabelPreset,
    names_count: usize,
    effective_rendering: &Rendering,
    options: &RenderOptions<'_>,
    fmt: &F,
) -> (Option<String>, Option<String>) {
    let plural = names_count > 1;
    match preset {
        RoleLabelPreset::None => (None, None),
        RoleLabelPreset::VerbPrefix => {
            let term = options
                .locale
                .resolved_role_term(role, plural, TermForm::Verb);
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
                .resolved_role_term(role, plural, TermForm::VerbShort);
            (
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        RoleLabelPreset::ShortSuffix => {
            let term = options
                .locale
                .resolved_role_term(role, plural, TermForm::Short);
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, " (", ")")
                }),
            )
        }
        RoleLabelPreset::LongSuffix => {
            let term = options
                .locale
                .resolved_role_term(role, plural, TermForm::Long);
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, ", ", "")
                }),
            )
        }
    }
}

/// Resolve the role-label prefix and suffix for a formatted contributor.
///
/// Returns `(prefix, suffix)` strings to wrap the formatted name list.
/// Precedence: explicit `label` config > configured role presets > form-based defaults.
pub(super) fn resolve_role_labels<F: OutputFormat<Output = String>>(
    component: &TemplateContributor,
    names_count: usize,
    effective_rendering: &Rendering,
    options: &RenderOptions<'_>,
    fmt: &F,
    role_omitted: bool,
) -> (Option<String>, Option<String>) {
    if let Some(label_config) = &component.label {
        use citum_schema::template::{LabelPlacement, RoleLabelForm};

        let plural = names_count > 1;
        let term_form = match label_config.form {
            RoleLabelForm::Short => TermForm::Short,
            RoleLabelForm::Long => TermForm::Long,
        };

        let role = match label_config.term.as_str() {
            "editor" => Some(ContributorRole::Editor),
            "translator" => Some(ContributorRole::Translator),
            _ => Some(component.contributor.clone()),
        };

        let term_text = role.and_then(|r| options.locale.resolved_role_term(&r, plural, term_form));

        return match label_config.placement {
            LabelPlacement::Prefix => (
                term_text.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            ),
            LabelPlacement::Suffix => (
                None,
                term_text.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, ", ", "")
                }),
            ),
        };
    }

    if role_omitted {
        return (None, None);
    }

    if let Some(preset) = options
        .config
        .contributors
        .as_ref()
        .and_then(|contributors| contributors.effective_role_label_preset(&component.contributor))
    {
        return resolve_role_label_preset(
            &component.contributor,
            preset,
            names_count,
            effective_rendering,
            options,
            fmt,
        );
    }

    // Form-based defaults
    match (&component.form, &component.contributor) {
        (ContributorForm::Verb | ContributorForm::VerbShort, role) => {
            let plural = names_count > 1;
            let term_form = match component.form {
                ContributorForm::VerbShort => TermForm::VerbShort,
                _ => TermForm::Verb,
            };
            let term = options.locale.resolved_role_term(role, plural, term_form);
            (
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        (
            ContributorForm::Long,
            ContributorRole::Editor
            | ContributorRole::Translator
            | ContributorRole::Interviewer
            | ContributorRole::Director
            | ContributorRole::Illustrator
            | ContributorRole::Composer,
        ) => {
            let plural = names_count > 1;
            let term =
                options
                    .locale
                    .resolved_role_term(&component.contributor, plural, TermForm::Short);
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, " (", ")")
                }),
            )
        }
        _ => (None, None),
    }
}
