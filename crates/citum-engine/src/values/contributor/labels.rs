//! Role-label resolution for contributor rendering.

use crate::reference::Reference;
use crate::render::format::OutputFormat;
use crate::values::RenderOptions;
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

fn requested_role_gender(
    component: &TemplateContributor,
    reference: &Reference,
) -> Option<GrammaticalGender> {
    if let Some(gender) = component.gender {
        return Some(gender);
    }

    let data_role = match component.contributor {
        ContributorRole::Author => citum_schema::reference::ContributorRole::Author,
        ContributorRole::Editor => citum_schema::reference::ContributorRole::Editor,
        ContributorRole::Translator => citum_schema::reference::ContributorRole::Translator,
        ContributorRole::Director => citum_schema::reference::ContributorRole::Director,
        ContributorRole::Composer => citum_schema::reference::ContributorRole::Composer,
        ContributorRole::Illustrator => citum_schema::reference::ContributorRole::Illustrator,
        ContributorRole::Recipient => citum_schema::reference::ContributorRole::Recipient,
        ContributorRole::Interviewer => citum_schema::reference::ContributorRole::Interviewer,
        ContributorRole::Guest => citum_schema::reference::ContributorRole::Guest,
        ContributorRole::Chair => {
            citum_schema::reference::ContributorRole::Custom("chair".to_string())
        }
        _ => return None,
    };

    let entries = reference.contributor_entries(&data_role);
    let mut genders = entries
        .iter()
        .filter_map(|entry| entry.gender.map(map_contributor_gender));
    let first = genders.next()?;

    if genders.all(|gender| gender == first) {
        Some(first)
    } else {
        Some(GrammaticalGender::Common)
    }
}

/// Resolve a configured role-label preset to `(prefix, suffix)`.
pub(super) fn resolve_role_label_preset<F: OutputFormat<Output = String>>(
    role: &ContributorRole,
    preset: RoleLabelPreset,
    names_count: usize,
    requested_gender: Option<GrammaticalGender>,
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
                .resolved_role_term(role, plural, TermForm::Verb, None);
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
                .resolved_role_term(role, plural, TermForm::VerbShort, None);
            (
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        RoleLabelPreset::ShortSuffix => {
            let term =
                options
                    .locale
                    .resolved_role_term(role, plural, TermForm::Short, requested_gender);
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(&t, fmt, effective_rendering, options, " (", ")")
                }),
            )
        }
        RoleLabelPreset::LongSuffix => {
            let term =
                options
                    .locale
                    .resolved_role_term(role, plural, TermForm::Long, requested_gender);
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
    reference: &Reference,
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
            "chair" => Some(ContributorRole::Chair),
            "editor" => Some(ContributorRole::Editor),
            "translator" => Some(ContributorRole::Translator),
            _ => Some(component.contributor.clone()),
        };

        let requested_gender = requested_role_gender(component, reference);
        let term_text = role.and_then(|r| {
            options
                .locale
                .resolved_role_term(&r, plural, term_form, requested_gender)
        });

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
        let requested_gender = requested_role_gender(component, reference);
        return resolve_role_label_preset(
            &component.contributor,
            preset,
            names_count,
            requested_gender,
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
            let term = options
                .locale
                .resolved_role_term(role, plural, term_form, None);
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
            | ContributorRole::Chair
            | ContributorRole::Translator
            | ContributorRole::Interviewer
            | ContributorRole::Director
            | ContributorRole::Illustrator
            | ContributorRole::Composer,
        ) => {
            let plural = names_count > 1;
            let requested_gender = requested_role_gender(component, reference);
            let term = options.locale.resolved_role_term(
                &component.contributor,
                plural,
                TermForm::Short,
                requested_gender,
            );
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
