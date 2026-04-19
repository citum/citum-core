//! Author-substitution logic for contributor rendering.
//!
//! When a reference has no author, this module handles the fallback chain:
//! editor → title → translator, as configured by the style's `substitute` block.

use crate::processor::rendering::get_variable_key;
use crate::reference::Reference;
use crate::render::format::OutputFormat;
use crate::values::{ProcHints, ProcValues, RenderContext, RenderOptions};
use citum_schema::options::{RoleLabelPreset, SubstituteKey};
use citum_schema::reference::Title;
use citum_schema::template::{ContributorRole, Rendering, TemplateComponent, TemplateContributor};

enum ResolvedRole {
    BuiltIn(ContributorRole),
    Custom(String),
}

impl ResolvedRole {
    fn key(&self) -> &str {
        match self {
            Self::BuiltIn(role) => role.as_str(),
            Self::Custom(role) => role.as_str(),
        }
    }

    fn built_in(&self) -> Option<&ContributorRole> {
        match self {
            Self::BuiltIn(role) => Some(role),
            Self::Custom(_) => None,
        }
    }
}

fn normalize_role_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let canonical = trimmed
        .chars()
        .map(|ch| match ch {
            '_' => '-',
            other => other.to_ascii_lowercase(),
        })
        .collect::<String>();

    canonical
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        .then_some(canonical)
}

fn parse_known_role(value: &str) -> Option<ContributorRole> {
    Some(match value {
        "author" => ContributorRole::Author,
        "chair" => ContributorRole::Chair,
        "editor" => ContributorRole::Editor,
        "translator" => ContributorRole::Translator,
        "director" => ContributorRole::Director,
        "composer" => ContributorRole::Composer,
        "illustrator" => ContributorRole::Illustrator,
        "collection-editor" => ContributorRole::CollectionEditor,
        "container-author" => ContributorRole::ContainerAuthor,
        "editorial-director" => ContributorRole::EditorialDirector,
        "textual-editor" => ContributorRole::TextualEditor,
        "original-author" => ContributorRole::OriginalAuthor,
        "reviewed-author" => ContributorRole::ReviewedAuthor,
        "recipient" => ContributorRole::Recipient,
        "interviewer" => ContributorRole::Interviewer,
        "guest" => ContributorRole::Guest,
        "inventor" => ContributorRole::Inventor,
        "counsel" => ContributorRole::Counsel,
        _ => return None,
    })
}

fn resolve_role_key(value: &str) -> Option<ResolvedRole> {
    let canonical = normalize_role_key(value)?;
    Some(
        parse_known_role(&canonical)
            .map(ResolvedRole::BuiltIn)
            .unwrap_or(ResolvedRole::Custom(canonical)),
    )
}

fn lookup_role_contributor(
    reference: &Reference,
    role: &ResolvedRole,
) -> Option<citum_schema::reference::contributor::Contributor> {
    use citum_schema::reference::ContributorRole as DataRole;

    match role {
        ResolvedRole::BuiltIn(ContributorRole::Editor) => reference.editor(),
        ResolvedRole::BuiltIn(ContributorRole::Translator) => reference.translator(),
        ResolvedRole::BuiltIn(ContributorRole::Director) => {
            reference.contributor(DataRole::Director)
        }
        ResolvedRole::BuiltIn(ContributorRole::Composer) => {
            reference.contributor(DataRole::Composer)
        }
        ResolvedRole::BuiltIn(ContributorRole::Illustrator) => {
            reference.contributor(DataRole::Illustrator)
        }
        ResolvedRole::BuiltIn(ContributorRole::ContainerAuthor) => {
            reference.contributor(DataRole::Custom("container-author".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::CollectionEditor) => {
            reference.contributor(DataRole::Custom("collection-editor".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::EditorialDirector) => {
            reference.contributor(DataRole::Custom("editorial-director".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::TextualEditor) => {
            reference.contributor(DataRole::Custom("textual-editor".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::OriginalAuthor) => {
            reference.contributor(DataRole::Custom("original-author".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::ReviewedAuthor) => {
            reference.contributor(DataRole::Custom("reviewed-author".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::Recipient) => {
            reference.contributor(DataRole::Recipient)
        }
        ResolvedRole::BuiltIn(ContributorRole::Interviewer) => {
            reference.contributor(DataRole::Interviewer)
        }
        ResolvedRole::BuiltIn(ContributorRole::Guest) => reference.contributor(DataRole::Guest),
        ResolvedRole::BuiltIn(ContributorRole::Chair) => {
            reference.contributor(DataRole::Custom("chair".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::Inventor) => {
            reference.contributor(DataRole::Custom("inventor".to_string()))
        }
        ResolvedRole::BuiltIn(ContributorRole::Counsel) => {
            reference.contributor(DataRole::Custom("counsel".to_string()))
        }
        ResolvedRole::Custom(role) => match role.as_str() {
            "compiler" => reference.contributor(DataRole::Compiler),
            "performer" => reference.contributor(DataRole::Performer),
            "narrator" => reference.contributor(DataRole::Narrator),
            "host" => reference.contributor(DataRole::Host),
            "producer" | "executive-producer" => reference.contributor(DataRole::Producer),
            "writer" => reference.contributor(DataRole::Writer),
            _ => reference.contributor(DataRole::Custom(role.clone())),
        },
        _ => None,
    }
}

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
    role: &ResolvedRole,
    names_count: usize,
    options: &RenderOptions<'_>,
    effective_rendering: &Rendering,
    fmt: &F,
    substitute: &citum_schema::options::Substitute,
) -> (Option<String>, Option<String>) {
    if options.context != RenderContext::Bibliography
        || role
            .built_in()
            .is_some_and(|known| super::is_role_label_omitted(options, known))
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
                .and_then(|contributors| {
                    role.built_in()
                        .and_then(|known| contributors.effective_role_label_preset(known))
                })
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

            role.built_in().map(|known| {
                super::labels::resolve_role_label_preset::<F>(
                    known,
                    selected,
                    names_count,
                    None,
                    effective_rendering,
                    options,
                    fmt,
                )
            })
        })
        .unwrap_or((None, None))
}

/// Format a substitute contributor using the current role-aware config path.
#[allow(
    clippy::too_many_arguments,
    reason = "Role-aware substitute formatting needs shared engine state until this module is refactored."
)]
fn resolve_named_substitute<F: OutputFormat<Output = String>>(
    role: &ResolvedRole,
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
        role.built_in().and_then(|known| {
            options
                .config
                .contributors
                .as_ref()
                .and_then(|contributors| contributors.effective_role_name_order(known))
        })
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
        role,
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

    let substituted_key = role.built_in().map_or_else(
        || Some(format!("contributor:{}", role.key())),
        |known| {
            get_variable_key(&TemplateComponent::Contributor(TemplateContributor {
                contributor: known.clone(),
                rendering: component.rendering.clone(),
                ..Default::default()
            }))
        },
    );

    Some(ProcValues {
        value: fmt.text(&formatted),
        prefix,
        suffix,
        url,
        substituted_key,
        pre_formatted: true,
    })
}

/// Check if a role should be suppressed by role-substitute configuration.
///
/// Returns true if this role appears as a fallback in some other role's chain
/// AND that primary role has data on the reference.
pub(super) fn is_role_suppressed_by_substitute(
    role: &ContributorRole,
    substitute: &citum_schema::options::Substitute,
    reference: &Reference,
) -> bool {
    let role_str = role.as_str();

    for (primary_role_str, fallback_chain) in &substitute.role_substitute {
        // Check if this role is in the fallback chain
        if !fallback_chain
            .iter()
            .filter_map(|entry| resolve_role_key(entry))
            .any(|entry| entry.key() == role_str)
        {
            continue;
        }

        if let Some(primary_role) = resolve_role_key(primary_role_str)
            && lookup_role_contributor(reference, &primary_role).is_some()
        {
            return true;
        }
    }

    false
}

fn find_role_substitute_chain<'a>(
    substitute: &'a citum_schema::options::Substitute,
    primary_role: &ContributorRole,
) -> Option<&'a Vec<String>> {
    let primary_role_str = primary_role.as_str();

    substitute
        .role_substitute
        .get(primary_role_str)
        .or_else(|| {
            substitute
                .role_substitute
                .iter()
                .find_map(|(configured_role, fallback_chain)| {
                    resolve_role_key(configured_role)
                        .filter(|resolved| resolved.key() == primary_role_str)
                        .map(|_| fallback_chain)
                })
        })
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
    substitute: &citum_schema::options::Substitute,
) -> Option<ProcValues<F::Output>> {
    let fallback_chain = find_role_substitute_chain(substitute, primary_role)?;

    for fallback_role_str in fallback_chain {
        let Some(fallback_role) = resolve_role_key(fallback_role_str) else {
            continue;
        };

        if let Some(contrib) = lookup_role_contributor(reference, &fallback_role) {
            return resolve_named_substitute(
                &fallback_role,
                &contrib,
                component,
                hints,
                options,
                reference,
                effective_rendering,
                fmt,
                substitute,
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
    substitute: &citum_schema::options::Substitute,
) -> Option<ProcValues<F::Output>> {
    for key in &substitute.template {
        match key {
            SubstituteKey::Editor => {
                if let Some(editors) = reference.editor()
                    && let Some(result) = resolve_named_substitute(
                        &ResolvedRole::BuiltIn(ContributorRole::Editor),
                        &editors,
                        component,
                        hints,
                        options,
                        reference,
                        effective_rendering,
                        fmt,
                        substitute,
                    )
                {
                    return Some(result);
                }
            }
            SubstituteKey::Title => {
                if let Some(title) = reference.title() {
                    // In citation context use a short-form title (main title only,
                    // no subtitle) so the substitute doesn't bloat the in-text cite.
                    // In bibliography use the full display form.
                    let title_str = match options.context {
                        RenderContext::Citation => match title {
                            Title::Structured(s) => s.main.clone(),
                            Title::MultiStructured(v) => {
                                v.first().map(|(_, s)| s.main.clone()).unwrap_or_default()
                            }
                            Title::Shorthand(abbr, _) => abbr.clone(),
                            _ => title.to_string(),
                        },
                        _ => title.to_string(),
                    };
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
                        &ResolvedRole::BuiltIn(ContributorRole::Translator),
                        &translators,
                        component,
                        hints,
                        options,
                        reference,
                        effective_rendering,
                        fmt,
                        substitute,
                    )
                {
                    return Some(result);
                }
            }
        }
    }

    None
}
