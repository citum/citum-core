//! Author-substitution logic for contributor rendering.
//!
//! When a reference has no author, this module handles the fallback chain:
//! editor → title → translator, as configured by the style's `substitute` block.

use crate::processor::rendering::get_variable_key;
use crate::reference::Reference;
use crate::render::format::OutputFormat;
use crate::values::{ProcHints, ProcValues, RenderContext, RenderOptions};
use citum_schema::options::{RoleLabelPreset, SubstituteKey};
use citum_schema::template::{ContributorRole, Rendering, TemplateComponent, TemplateContributor};

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

/// Resolve substitute-path role labels for a rendered fallback contributor.
fn resolve_substitute_role_labels<F: OutputFormat<Output = String>>(
    component: &TemplateContributor,
    role: &ContributorRole,
    names_count: usize,
    options: &RenderOptions<'_>,
    effective_rendering: &Rendering,
    fmt: &F,
    substitute: &citum_schema::options::Substitute,
) -> (Option<String>, Option<String>) {
    if options.context != RenderContext::Bibliography || super::is_role_label_omitted(options, role)
    {
        return (None, None);
    }

    let preset = substitute
        .contributor_role_form
        .as_deref()
        .and_then(|form| match form {
            "short" => Some(RoleLabelPreset::ShortSuffix),
            "long" => Some(RoleLabelPreset::LongSuffix),
            _ => None,
        })
        .or_else(|| {
            options
                .config
                .contributors
                .as_ref()
                .and_then(|contributors| contributors.effective_role_label_preset(role))
        });

    preset
        .and_then(|selected| {
            if component.contributor == ContributorRole::Author
                && matches!(
                    selected,
                    RoleLabelPreset::VerbPrefix | RoleLabelPreset::VerbShortPrefix
                )
            {
                return None;
            }

            Some(super::labels::resolve_role_label_preset::<F>(
                role,
                selected,
                names_count,
                effective_rendering,
                options,
                fmt,
            ))
        })
        .unwrap_or((None, None))
}

/// Format a substitute contributor using the current role-aware config path.
#[allow(
    clippy::too_many_arguments,
    reason = "Role-aware substitute formatting needs shared engine state until this module is refactored."
)]
fn resolve_named_substitute<F: OutputFormat<Output = String>>(
    role: ContributorRole,
    contributor: &citum_schema::reference::contributor::Contributor,
    component: &TemplateContributor,
    hints: &ProcHints,
    options: &RenderOptions<'_>,
    reference: &Reference,
    effective_rendering: &Rendering,
    fmt: &F,
    substitute: &citum_schema::options::Substitute,
) -> Option<ProcValues<F::Output>> {
    let names_vec = resolve_multilingual_for_contrib(contributor, options);
    if names_vec.is_empty() {
        return None;
    }

    let effective_name_order = component.name_order.as_ref().or_else(|| {
        options
            .config
            .contributors
            .as_ref()
            .and_then(|contributors| contributors.effective_role_name_order(&role))
    });

    // Priority chain for name_form:
    // 1. component.name_form (TemplateContributor-level override - highest priority)
    // 2. effective_rendering.name_form (from overrides, second priority)
    // 3. config (options-level fallback)
    let effective_name_form = component.name_form.or(effective_rendering.name_form);

    let name_overrides = super::names::NamesOverrides {
        name_order: effective_name_order,
        sort_separator: component.sort_separator.as_ref(),
        shorten: component.shorten.as_ref(),
        and: component.and.as_ref(),
        initialize_with: effective_rendering.initialize_with.as_ref(),
        name_form: effective_name_form,
    };
    let formatted =
        super::names::format_names(&names_vec, &component.form, options, &name_overrides, hints);
    let (prefix, suffix) = resolve_substitute_role_labels::<F>(
        component,
        &role,
        names_vec.len(),
        options,
        effective_rendering,
        fmt,
        substitute,
    );

    let url = crate::values::resolve_effective_url(
        component.links.as_ref(),
        options.config.links.as_ref(),
        reference,
        citum_schema::options::LinkAnchor::Component,
    );

    Some(ProcValues {
        value: fmt.text(&formatted),
        prefix,
        suffix,
        url,
        substituted_key: get_variable_key(&TemplateComponent::Contributor(TemplateContributor {
            contributor: role,
            rendering: component.rendering.clone(),
            ..Default::default()
        })),
        pre_formatted: true,
    })
}

/// Check if a role should be suppressed by role-substitute configuration.
///
/// Returns true if this role appears as a fallback in some other role's chain
/// AND that primary role has data on the reference.
pub(super) fn is_role_suppressed_by_substitute(
    role: &ContributorRole,
    options: &RenderOptions<'_>,
    reference: &Reference,
) -> bool {
    let default_substitute = citum_schema::options::SubstituteConfig::default();
    let substitute_config = options
        .config
        .substitute
        .as_ref()
        .unwrap_or(&default_substitute);
    let substitute = substitute_config.resolve();

    let role_str = role.as_str();

    for (primary_role_str, fallback_chain) in &substitute.role_substitute {
        // Check if this role is in the fallback chain
        if !fallback_chain.iter().any(|s| s == role_str) {
            continue;
        }

        // Check if the primary role has data
        let primary_role: ContributorRole = match primary_role_str.as_str() {
            "container-author" => ContributorRole::ContainerAuthor,
            "editor" => ContributorRole::Editor,
            "translator" => ContributorRole::Translator,
            "director" => ContributorRole::Director,
            "composer" => ContributorRole::Composer,
            "illustrator" => ContributorRole::Illustrator,
            "collection-editor" => ContributorRole::CollectionEditor,
            "editorial-director" => ContributorRole::EditorialDirector,
            "textual-editor" => ContributorRole::TextualEditor,
            "original-author" => ContributorRole::OriginalAuthor,
            "reviewed-author" => ContributorRole::ReviewedAuthor,
            "recipient" => ContributorRole::Recipient,
            "interviewer" => ContributorRole::Interviewer,
            "guest" => ContributorRole::Guest,
            "inventor" => ContributorRole::Inventor,
            "counsel" => ContributorRole::Counsel,
            _ => continue,
        };

        // Check if the primary role has contributors on the reference
        let has_primary = match &primary_role {
            ContributorRole::Editor => reference.editor().is_some(),
            ContributorRole::Translator => reference.translator().is_some(),
            ContributorRole::Director => {
                reference.contributor(citum_schema::reference::ContributorRole::Director).is_some()
            }
            ContributorRole::Composer => {
                reference.contributor(citum_schema::reference::ContributorRole::Composer).is_some()
            }
            ContributorRole::Illustrator => {
                reference.contributor(citum_schema::reference::ContributorRole::Illustrator).is_some()
            }
            ContributorRole::ContainerAuthor => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "container-author".to_string(),
                    ))
                    .is_some()
            }
            ContributorRole::CollectionEditor => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "collection-editor".to_string(),
                    ))
                    .is_some()
            }
            ContributorRole::EditorialDirector => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "editorial-director".to_string(),
                    ))
                    .is_some()
            }
            ContributorRole::TextualEditor => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "textual-editor".to_string(),
                    ))
                    .is_some()
            }
            ContributorRole::OriginalAuthor => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "original-author".to_string(),
                    ))
                    .is_some()
            }
            ContributorRole::ReviewedAuthor => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "reviewed-author".to_string(),
                    ))
                    .is_some()
            }
            ContributorRole::Recipient => {
                reference.contributor(citum_schema::reference::ContributorRole::Recipient).is_some()
            }
            ContributorRole::Interviewer => {
                reference.contributor(citum_schema::reference::ContributorRole::Interviewer).is_some()
            }
            ContributorRole::Guest => {
                reference.contributor(citum_schema::reference::ContributorRole::Guest).is_some()
            }
            ContributorRole::Inventor => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "inventor".to_string(),
                    ))
                    .is_some()
            }
            ContributorRole::Counsel => {
                reference
                    .contributor(citum_schema::reference::ContributorRole::Custom(
                        "counsel".to_string(),
                    ))
                    .is_some()
            }
            _ => false,
        };

        if has_primary {
            return true;
        }
    }

    false
}

/// Attempt to substitute a non-author contributor field via role-substitute fallback chain.
///
/// Returns `Some(ProcValues)` if a substitute from the chain was found, `None` if the chain
/// is exhausted with no result.
#[allow(
    clippy::too_many_arguments,
    reason = "Role-aware role-substitute needs shared engine state."
)]
pub(super) fn resolve_role_substitute<F: OutputFormat<Output = String>>(
    primary_role: &ContributorRole,
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

    let primary_role_str = primary_role.as_str();
    let fallback_chain = substitute.role_substitute.get(primary_role_str)?;

    for fallback_role_str in fallback_chain {
        let fallback_role: ContributorRole = match fallback_role_str.as_str() {
            "editor" => ContributorRole::Editor,
            "translator" => ContributorRole::Translator,
            "director" => ContributorRole::Director,
            "composer" => ContributorRole::Composer,
            "illustrator" => ContributorRole::Illustrator,
            "collection-editor" => ContributorRole::CollectionEditor,
            "editorial-director" => ContributorRole::EditorialDirector,
            "textual-editor" => ContributorRole::TextualEditor,
            "original-author" => ContributorRole::OriginalAuthor,
            "reviewed-author" => ContributorRole::ReviewedAuthor,
            "recipient" => ContributorRole::Recipient,
            "interviewer" => ContributorRole::Interviewer,
            "guest" => ContributorRole::Guest,
            "inventor" => ContributorRole::Inventor,
            "counsel" => ContributorRole::Counsel,
            "container-author" => ContributorRole::ContainerAuthor,
            _ => continue,
        };

        let contributor = match &fallback_role {
            ContributorRole::Editor => reference.editor(),
            ContributorRole::Translator => reference.translator(),
            ContributorRole::Director => {
                reference.contributor(citum_schema::reference::ContributorRole::Director)
            }
            ContributorRole::Composer => {
                reference.contributor(citum_schema::reference::ContributorRole::Composer)
            }
            ContributorRole::Illustrator => {
                reference.contributor(citum_schema::reference::ContributorRole::Illustrator)
            }
            ContributorRole::ContainerAuthor => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "container-author".to_string(),
                ))
            }
            ContributorRole::CollectionEditor => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "collection-editor".to_string(),
                ))
            }
            ContributorRole::EditorialDirector => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "editorial-director".to_string(),
                ))
            }
            ContributorRole::TextualEditor => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "textual-editor".to_string(),
                ))
            }
            ContributorRole::OriginalAuthor => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "original-author".to_string(),
                ))
            }
            ContributorRole::ReviewedAuthor => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "reviewed-author".to_string(),
                ))
            }
            ContributorRole::Recipient => {
                reference.contributor(citum_schema::reference::ContributorRole::Recipient)
            }
            ContributorRole::Interviewer => {
                reference.contributor(citum_schema::reference::ContributorRole::Interviewer)
            }
            ContributorRole::Guest => {
                reference.contributor(citum_schema::reference::ContributorRole::Guest)
            }
            ContributorRole::Inventor => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "inventor".to_string(),
                ))
            }
            ContributorRole::Counsel => {
                reference.contributor(citum_schema::reference::ContributorRole::Custom(
                    "counsel".to_string(),
                ))
            }
            _ => None,
        };

        if let Some(contrib) = contributor {
            return resolve_named_substitute(
                fallback_role,
                &contrib,
                component,
                hints,
                options,
                reference,
                effective_rendering,
                fmt,
                &substitute,
            );
        }
    }

    None
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
                if let Some(editors) = reference.editor()
                    && let Some(result) = resolve_named_substitute(
                        ContributorRole::Editor,
                        &editors,
                        component,
                        hints,
                        options,
                        reference,
                        effective_rendering,
                        fmt,
                        &substitute,
                    )
                {
                    return Some(result);
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
                if let Some(translators) = reference.translator()
                    && let Some(result) = resolve_named_substitute(
                        ContributorRole::Translator,
                        &translators,
                        component,
                        hints,
                        options,
                        reference,
                        effective_rendering,
                        fmt,
                        &substitute,
                    )
                {
                    return Some(result);
                }
            }
        }
    }

    None
}
