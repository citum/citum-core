//! Role-label resolution for contributor rendering.

use crate::render::format::OutputFormat;
use crate::values::RenderOptions;
use citum_schema::locale::TermForm;
use citum_schema::options::EditorLabelFormat;
use citum_schema::template::{ContributorForm, ContributorRole, Rendering, TemplateContributor};

/// Resolve role label from global `editor_label_format` config.
///
/// Returns `(prefix, suffix)` using verb-prefix, short-suffix, or long-suffix format.
fn resolve_editor_format_label<F: OutputFormat<Output = String>>(
    format: EditorLabelFormat,
    component: &TemplateContributor,
    names_count: usize,
    effective_rendering: &Rendering,
    options: &RenderOptions<'_>,
    fmt: &F,
) -> (Option<String>, Option<String>) {
    let plural = names_count > 1;
    match format {
        EditorLabelFormat::VerbPrefix => {
            let term = options
                .locale
                .role_term(&component.contributor, plural, TermForm::Verb);
            (
                term.map(|t| {
                    super::format_role_term::<F>(t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        EditorLabelFormat::ShortSuffix => {
            let term = options
                .locale
                .role_term(&component.contributor, plural, TermForm::Short);
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(t, fmt, effective_rendering, options, " (", ")")
                }),
            )
        }
        EditorLabelFormat::LongSuffix => {
            let term = options
                .locale
                .role_term(&component.contributor, plural, TermForm::Long);
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(t, fmt, effective_rendering, options, ", ", "")
                }),
            )
        }
    }
}

/// Resolve the role-label prefix and suffix for a formatted contributor.
///
/// Returns `(prefix, suffix)` strings to wrap the formatted name list.
/// Precedence: explicit `label` config > global `editor_label_format` > form-based defaults.
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

        let term_text = role.and_then(|r| options.locale.role_term(&r, plural, term_form));

        return match label_config.placement {
            LabelPlacement::Prefix => (term_text.map(|t| fmt.text(&format!("{t} "))), None),
            LabelPlacement::Suffix => (None, term_text.map(|t| fmt.text(&format!(", {t}")))),
        };
    }

    if role_omitted {
        return (None, None);
    }

    // Fall back to global editor_label_format configuration
    let editor_format = options
        .config
        .contributors
        .as_ref()
        .and_then(|c| c.editor_label_format);

    if let Some(format) = editor_format {
        if matches!(
            component.contributor,
            ContributorRole::Editor | ContributorRole::Translator
        ) {
            return resolve_editor_format_label(
                format,
                component,
                names_count,
                effective_rendering,
                options,
                fmt,
            );
        }
        return (None, None);
    }

    // Form-based defaults
    match (&component.form, &component.contributor) {
        (ContributorForm::Verb | ContributorForm::VerbShort, role) => {
            let plural = names_count > 1;
            let term_form = match component.form {
                ContributorForm::VerbShort => TermForm::VerbShort,
                _ => TermForm::Verb,
            };
            let term = options.locale.role_term(role, plural, term_form);
            (
                term.map(|t| {
                    super::format_role_term::<F>(t, fmt, effective_rendering, options, "", " ")
                }),
                None,
            )
        }
        (ContributorForm::Long, ContributorRole::Editor | ContributorRole::Translator) => {
            let plural = names_count > 1;
            let term = options
                .locale
                .role_term(&component.contributor, plural, TermForm::Short);
            (
                None,
                term.map(|t| {
                    super::format_role_term::<F>(t, fmt, effective_rendering, options, " (", ")")
                }),
            )
        }
        _ => (None, None),
    }
}
