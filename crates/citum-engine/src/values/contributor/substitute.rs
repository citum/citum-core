/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

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
use citum_schema::reference::{ClassExtension, ContributorRole as DataRole};
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
    match role {
        ResolvedRole::BuiltIn(ContributorRole::Editor) => reference.editor(),
        ResolvedRole::BuiltIn(ContributorRole::Translator) => reference.translator(),
        ResolvedRole::BuiltIn(role) => {
            data_role_for_builtin(role).and_then(|data_role| reference.contributor(data_role))
        }
        ResolvedRole::Custom(role) => reference.contributor(data_role_for_custom(role)),
    }
}

fn data_role_for_builtin(role: &ContributorRole) -> Option<DataRole> {
    Some(match role {
        ContributorRole::Director => DataRole::Director,
        ContributorRole::Composer => DataRole::Composer,
        ContributorRole::Illustrator => DataRole::Illustrator,
        ContributorRole::Recipient => DataRole::Recipient,
        ContributorRole::Interviewer => DataRole::Interviewer,
        ContributorRole::Guest => DataRole::Guest,
        ContributorRole::ContainerAuthor
        | ContributorRole::CollectionEditor
        | ContributorRole::EditorialDirector
        | ContributorRole::TextualEditor
        | ContributorRole::OriginalAuthor
        | ContributorRole::ReviewedAuthor
        | ContributorRole::Chair
        | ContributorRole::Inventor
        | ContributorRole::Counsel => DataRole::Unknown(role.as_str().to_string()),
        ContributorRole::Unknown(role) => DataRole::Unknown(role.clone()),
        ContributorRole::Author
        | ContributorRole::Editor
        | ContributorRole::Translator
        | ContributorRole::Publisher
        | ContributorRole::Interviewee => return None,
        _ => return None,
    })
}

fn data_role_for_custom(role: &str) -> DataRole {
    match role {
        "compiler" => DataRole::Compiler,
        "performer" => DataRole::Performer,
        "narrator" => DataRole::Narrator,
        "host" => DataRole::Host,
        "producer" | "executive-producer" => DataRole::Producer,
        "writer" => DataRole::Writer,
        _ => DataRole::Unknown(role.to_string()),
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
            "short-comma" => Some(RoleLabelPreset::ShortSuffixComma),
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
                    super::labels::RoleLabelTermOptions {
                        gender: None,
                        text_case: substitute.contributor_role_case,
                    },
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

#[allow(
    clippy::too_many_arguments,
    reason = "Substitute lookup and formatting share the same rendering state."
)]
fn resolve_contributor_substitute_for_role<F: OutputFormat<Output = String>>(
    role: &ResolvedRole,
    component: &TemplateContributor,
    hints: &ProcHints,
    options: &RenderOptions<'_>,
    reference: &Reference,
    effective_rendering: &Rendering,
    fmt: &F,
    substitute: &citum_schema::options::Substitute,
) -> Option<ProcValues<F::Output>> {
    let contributor = lookup_role_contributor(reference, role)?;
    resolve_named_substitute(
        role,
        &contributor,
        component,
        hints,
        options,
        reference,
        effective_rendering,
        fmt,
        substitute,
    )
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

        if let Some(result) = resolve_contributor_substitute_for_role(
            &fallback_role,
            component,
            hints,
            options,
            reference,
            effective_rendering,
            fmt,
            substitute,
        ) {
            return Some(result);
        }
    }

    None
}

fn resolve_title_substitute<F: OutputFormat<Output = String>>(
    title: Title,
    component: &TemplateContributor,
    options: &RenderOptions<'_>,
    reference: &Reference,
    fmt: &F,
    substituted_key: &'static str,
    quote_in_citation: bool,
) -> ProcValues<F::Output> {
    let title_str = title_substitute_text(title, options.context);
    let value = if options.context == RenderContext::Citation && quote_in_citation {
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

    ProcValues {
        value,
        prefix: None,
        suffix: None,
        url,
        substituted_key: Some(substituted_key.to_string()),
        pre_formatted: true,
    }
}

fn title_substitute_text(title: Title, context: RenderContext) -> String {
    if context != RenderContext::Citation {
        return title.to_string();
    }

    match title {
        Title::Structured(s) => s.main,
        Title::MultiStructured(v) => v
            .into_iter()
            .next()
            .map(|(_, s)| s.main)
            .unwrap_or_default(),
        Title::Shorthand(abbr, _) => abbr,
        _ => title.to_string(),
    }
}

fn resolve_parent_serial_title(reference: &Reference) -> Option<Title> {
    match reference.extension() {
        ClassExtension::SerialComponent(_)
        | ClassExtension::LegalCase(_)
        | ClassExtension::Treaty(_) => reference.container_title(),
        _ => None,
    }
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
            SubstituteKey::CollectionEditor => {
                if let Some(result) = resolve_contributor_substitute_for_role(
                    &ResolvedRole::BuiltIn(ContributorRole::CollectionEditor),
                    component,
                    hints,
                    options,
                    reference,
                    effective_rendering,
                    fmt,
                    substitute,
                ) {
                    return Some(result);
                }
            }
            SubstituteKey::Editor => {
                if let Some(result) = resolve_contributor_substitute_for_role(
                    &ResolvedRole::BuiltIn(ContributorRole::Editor),
                    component,
                    hints,
                    options,
                    reference,
                    effective_rendering,
                    fmt,
                    substitute,
                ) {
                    return Some(result);
                }
            }
            SubstituteKey::ParentSerial => {
                if let Some(title) = resolve_parent_serial_title(reference) {
                    return Some(resolve_title_substitute(
                        title,
                        component,
                        options,
                        reference,
                        fmt,
                        "title:ParentSerial",
                        false,
                    ));
                }
            }
            SubstituteKey::Title => {
                if let Some(title) = reference.title() {
                    return Some(resolve_title_substitute(
                        title,
                        component,
                        options,
                        reference,
                        fmt,
                        "title:Primary",
                        true,
                    ));
                }
            }
            SubstituteKey::Translator => {
                if let Some(result) = resolve_contributor_substitute_for_role(
                    &ResolvedRole::BuiltIn(ContributorRole::Translator),
                    component,
                    hints,
                    options,
                    reference,
                    effective_rendering,
                    fmt,
                    substitute,
                ) {
                    return Some(result);
                }
            }
        }
    }

    None
}
