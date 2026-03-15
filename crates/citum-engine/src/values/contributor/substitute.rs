//! Author-substitution logic for contributor rendering.
//!
//! When a reference has no author, this module handles the fallback chain:
//! editor → title → translator, as configured by the style's `substitute` block.

use crate::reference::Reference;
use crate::render::format::OutputFormat;
use crate::values::{ProcHints, ProcValues, RenderContext, RenderOptions};
use citum_schema::options::SubstituteKey;
use citum_schema::template::{ContributorRole, Rendering, TemplateContributor};

/// Resolve all multilingual names for a contributor using the current options.
///
/// Eliminates the copy-paste resolution pattern across Editor, Translator, and
/// primary-contributor paths.
pub(super) fn resolve_multilingual_for_contrib(
    contrib: &citum_schema::reference::contributor::Contributor,
    options: &RenderOptions<'_>,
) -> Vec<crate::reference::FlatName> {
    let mode = options
        .config
        .multilingual
        .as_ref()
        .and_then(|m| m.name_mode.as_ref());
    let preferred_transliteration = options
        .config
        .multilingual
        .as_ref()
        .and_then(|m| m.preferred_transliteration.as_deref());
    let preferred_script = options
        .config
        .multilingual
        .as_ref()
        .and_then(|m| m.preferred_script.as_ref());
    crate::values::resolve_multilingual_name(
        contrib,
        mode,
        preferred_transliteration,
        preferred_script,
        &options.locale.locale,
    )
}

/// Attempt to substitute an empty author field with editor, title, or translator.
///
/// Returns `Some(ProcValues)` if a substitute was found, `None` if the chain
/// is exhausted with no result (caller should then return `None` from `values()`).
pub(super) fn resolve_author_substitute<F: OutputFormat<Output = String>>(
    component: &TemplateContributor,
    hints: &ProcHints,
    options: &RenderOptions<'_>,
    reference: &Reference,
    effective_rendering: &Rendering,
    fmt: &F,
) -> Option<ProcValues<F::Output>> {
    let default_substitute = citum_schema::options::SubstituteConfig::default();
    let substitute_config = options
        .config
        .substitute
        .as_ref()
        .unwrap_or(&default_substitute);
    let substitute = substitute_config.resolve();

    for key in &substitute.template {
        match key {
            SubstituteKey::Editor => {
                if let Some(editors) = reference.editor() {
                    let names_vec = resolve_multilingual_for_contrib(&editors, options);
                    if !names_vec.is_empty() {
                        let effective_name_order = component.name_order.as_ref().or_else(|| {
                            options
                                .config
                                .contributors
                                .as_ref()?
                                .role
                                .as_ref()?
                                .roles
                                .as_ref()?
                                .get(component.contributor.as_str())?
                                .name_order
                                .as_ref()
                        });

                        let formatted = super::names::format_names(
                            &names_vec,
                            &component.form,
                            options,
                            effective_name_order,
                            component.sort_separator.as_ref(),
                            component.shorten.as_ref(),
                            component.and.as_ref(),
                            effective_rendering.initialize_with.as_ref(),
                            hints,
                        );

                        // Add role suffix in bibliography context only.
                        // In citations, substituted editors look identical to authors.
                        let suffix = if options.context == RenderContext::Bibliography {
                            if super::is_role_label_omitted(options, &ContributorRole::Editor) {
                                None
                            } else {
                                substitute.contributor_role_form.as_ref().and_then(|form| {
                                    let plural = names_vec.len() > 1;
                                    let term_form = match form.as_str() {
                                        "short" => citum_schema::locale::TermForm::Short,
                                        "verb" => citum_schema::locale::TermForm::Verb,
                                        "verb-short" => citum_schema::locale::TermForm::VerbShort,
                                        _ => citum_schema::locale::TermForm::Short,
                                    };
                                    options
                                        .locale
                                        .role_term(&ContributorRole::Editor, plural, term_form)
                                        .map(|term| {
                                            super::format_role_term::<F>(
                                                term,
                                                fmt,
                                                effective_rendering,
                                                options,
                                                " (",
                                                ")",
                                            )
                                        })
                                })
                            }
                        } else {
                            None
                        };

                        let url = crate::values::resolve_effective_url(
                            component.links.as_ref(),
                            options.config.links.as_ref(),
                            reference,
                            citum_schema::options::LinkAnchor::Component,
                        );

                        return Some(ProcValues {
                            value: fmt.text(&formatted),
                            prefix: None,
                            suffix,
                            url,
                            substituted_key: Some("contributor:Editor".to_string()),
                            pre_formatted: true,
                        });
                    }
                }
            }
            SubstituteKey::Title => {
                if let Some(title) = reference.title() {
                    let title_str = title.to_string();
                    // In citations: quote the title per CSL conventions.
                    // In bibliography: use title as-is (will be styled normally).
                    let value = if options.context == RenderContext::Citation {
                        fmt.quote(fmt.text(&title_str))
                    } else {
                        fmt.text(&title_str)
                    };

                    let url = crate::values::resolve_effective_url(
                        component.links.as_ref(),
                        options.config.links.as_ref(),
                        reference,
                        citum_schema::options::LinkAnchor::Title,
                    );

                    return Some(ProcValues {
                        value,
                        prefix: None,
                        suffix: None,
                        url,
                        substituted_key: Some("title:Primary".to_string()),
                        pre_formatted: true,
                    });
                }
            }
            SubstituteKey::Translator => {
                if let Some(translators) = reference.translator() {
                    let names_vec = resolve_multilingual_for_contrib(&translators, options);
                    if !names_vec.is_empty() {
                        let formatted = super::names::format_names(
                            &names_vec,
                            &component.form,
                            options,
                            component.name_order.as_ref(),
                            component.sort_separator.as_ref(),
                            component.shorten.as_ref(),
                            component.and.as_ref(),
                            effective_rendering.initialize_with.as_ref(),
                            hints,
                        );

                        let url = crate::values::resolve_effective_url(
                            component.links.as_ref(),
                            options.config.links.as_ref(),
                            reference,
                            citum_schema::options::LinkAnchor::Component,
                        );

                        return Some(ProcValues {
                            value: fmt.text(&formatted),
                            prefix: None,
                            suffix: Some(fmt.text(" (Trans.)")),
                            url,
                            substituted_key: None,
                            pre_formatted: true,
                        });
                    }
                }
            }
        }
    }

    None
}
