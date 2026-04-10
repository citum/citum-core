//! Rendering logic for contributors (authors, editors, translators).
//!
//! This module handles contributor rendering with support for name ordering,
//! role labels, et-al formatting, and multilingual name resolution.

mod labels;
pub mod names;
mod substitute;

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderContext, RenderOptions};
use citum_schema::options::{IntegralNameForm, IntegralNameRule};
use citum_schema::template::{ContributorForm, ContributorRole, NameOrder, TemplateContributor};

#[cfg(test)]
pub(crate) use names::{NameFormatContext, format_single_name};
pub use names::{NamesOverrides, format_contributors_short, format_names};

/// Checks if a contributor role label should be omitted for a given reference.
///
/// Returns true if the role appears in the configuration's role.omit list.
pub(super) fn is_role_label_omitted(options: &RenderOptions<'_>, role: &ContributorRole) -> bool {
    options
        .config
        .contributors
        .as_ref()
        .and_then(|c| c.role.as_ref())
        .is_some_and(|role_opts| {
            role_opts
                .omit
                .iter()
                .any(|entry| entry.eq_ignore_ascii_case(role.as_str()))
        })
}

/// Format a role term with period stripping if configured.
///
/// Handles the repeated pattern of checking `should_strip_periods` and formatting
/// the result with a given prefix and suffix pattern.
pub(super) fn format_role_term<F: crate::render::format::OutputFormat<Output = String>>(
    term: &str,
    fmt: &F,
    effective_rendering: &citum_schema::template::Rendering,
    options: &RenderOptions<'_>,
    prefix: &str,
    suffix: &str,
) -> String {
    let term_str = if crate::values::should_strip_periods(effective_rendering, options) {
        crate::values::strip_trailing_periods(term)
    } else {
        term.to_string()
    };
    fmt.text(&format!("{prefix}{term_str}{suffix}"))
}

/// Apply `FullThenShort` integral-citation subsequent-form rewrite to contributor form.
fn apply_integral_subsequent_form(
    component: &mut TemplateContributor,
    hints: &ProcHints,
    options: &RenderOptions<'_>,
) {
    if options.context != RenderContext::Citation {
        return;
    }
    if !matches!(options.mode, citum_schema::citation::CitationMode::Integral) {
        return;
    }
    if !matches!(component.contributor, ContributorRole::Author) {
        return;
    }
    if !matches!(
        hints.integral_name_state,
        Some(citum_schema::citation::IntegralNameState::Subsequent)
    ) {
        return;
    }
    if !options
        .config
        .integral_names
        .as_ref()
        .is_some_and(|cfg| matches!(cfg.resolve().rule, IntegralNameRule::FullThenShort))
    {
        return;
    }
    let subsequent_form = options
        .config
        .integral_names
        .as_ref()
        .map_or(IntegralNameForm::Short, |cfg| cfg.resolve().subsequent_form);
    component.form = match subsequent_form {
        IntegralNameForm::Short => ContributorForm::Short,
        IntegralNameForm::FamilyOnly => ContributorForm::FamilyOnly,
    };
}

/// Build name overrides and format all names for a contributor component.
fn format_contributor_names(
    component: &TemplateContributor,
    names_vec: &[crate::reference::FlatName],
    effective_rendering: &citum_schema::template::Rendering,
    options: &RenderOptions<'_>,
    hints: &ProcHints,
) -> String {
    let effective_name_order = component.name_order.as_ref().or_else(|| {
        options
            .config
            .contributors
            .as_ref()?
            .effective_role_name_order(&component.contributor)
    });
    let effective_shorten = component
        .shorten
        .as_ref()
        .or_else(|| options.config.contributors.as_ref()?.shorten.as_ref());

    // Priority chain for name_form:
    // 1. component.name_form (TemplateContributor-level override - highest priority)
    // 2. effective_rendering.name_form (from overrides, second priority)
    // 3. config (options-level fallback)
    let effective_name_form = component.name_form.or(effective_rendering.name_form);

    let name_overrides = names::NamesOverrides {
        name_order: effective_name_order,
        sort_separator: component.sort_separator.as_ref(),
        shorten: effective_shorten,
        and: component.and.as_ref(),
        initialize_with: effective_rendering.initialize_with.as_ref(),
        name_form: effective_name_form,
    };
    names::format_names(names_vec, &component.form, options, &name_overrides, hints)
}

impl ComponentValues for TemplateContributor {
    #[allow(
        clippy::too_many_lines,
        reason = "large match statement for contributor role dispatch"
    )]
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        let fmt = F::default();

        let mut component = self.clone();
        let mut effective_rendering = self.rendering.clone();

        // Apply integral-citation subsequent-form (FullThenShort rule)
        apply_integral_subsequent_form(&mut component, hints, options);

        // Respect explicit suppression before any contributor substitution logic.
        if effective_rendering.suppress == Some(true) {
            return None;
        }

        // Personal-communication special-case: always given-first with a suffix.
        if options.context == RenderContext::Citation
            && reference.ref_type() == "personal-communication"
            && matches!(component.contributor, ContributorRole::Author)
            && matches!(component.form, ContributorForm::Long)
            && component.name_order.is_none()
            && effective_rendering.suffix.is_none()
        {
            component.form = ContributorForm::Long;
            component.name_order = Some(NameOrder::GivenFirst);
            effective_rendering.suffix = Some(", personal communication".to_string());
        }

        let contributor = match &component.contributor {
            ContributorRole::Author => {
                if options.suppress_author {
                    None
                } else {
                    reference.author()
                }
            }
            ContributorRole::Editor => reference.editor(),
            ContributorRole::Translator => reference.translator(),
            ContributorRole::Recipient => {
                reference.contributor(citum_schema::reference::ContributorRole::Recipient)
            }
            ContributorRole::Chair => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("chair".to_string()),
            ),
            ContributorRole::Interviewer => {
                reference.contributor(citum_schema::reference::ContributorRole::Interviewer)
            }
            ContributorRole::Guest => {
                reference.contributor(citum_schema::reference::ContributorRole::Guest)
            }
            ContributorRole::Director => {
                reference.contributor(citum_schema::reference::ContributorRole::Director)
            }
            ContributorRole::Composer => {
                reference.contributor(citum_schema::reference::ContributorRole::Composer)
            }
            ContributorRole::Illustrator => {
                reference.contributor(citum_schema::reference::ContributorRole::Illustrator)
            }
            ContributorRole::Interviewee => None, // Not a standard reference contributor role
            ContributorRole::Publisher => None, // Publisher is a corporate entity, not a person contributor
            ContributorRole::Inventor => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("inventor".to_string()),
            ),
            ContributorRole::Counsel => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("counsel".to_string()),
            ),
            ContributorRole::CollectionEditor => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("collection-editor".to_string()),
            ),
            ContributorRole::ContainerAuthor => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("container-author".to_string()),
            ),
            ContributorRole::EditorialDirector => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("editorial-director".to_string()),
            ),
            ContributorRole::TextualEditor => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("textual-editor".to_string()),
            ),
            ContributorRole::OriginalAuthor => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("original-author".to_string()),
            ),
            ContributorRole::ReviewedAuthor => reference.contributor(
                citum_schema::reference::ContributorRole::Custom("reviewed-author".to_string()),
            ),
            _ => None, // Handle any future template contributor roles
        };

        // Resolve substitute config once for all substitute/suppression checks below.
        let default_substitute = citum_schema::options::SubstituteConfig::default();
        let substitute_config = options
            .config
            .substitute
            .as_ref()
            .unwrap_or(&default_substitute);
        let substitute = substitute_config.resolve();

        // Check if this role is suppressed by role-substitute configuration
        if substitute::is_role_suppressed_by_substitute(
            &component.contributor,
            &substitute,
            reference,
        ) {
            return None;
        }

        // Resolve multilingual names if configured
        let names_vec = if let Some(contrib) = contributor {
            substitute::resolve_multilingual_for_contrib(&contrib, options)
        } else {
            Vec::new()
        };

        // If author is suppressed, don't attempt substitution or formatting.
        if names_vec.is_empty()
            && matches!(component.contributor, ContributorRole::Author)
            && options.suppress_author
        {
            return None;
        }

        // Handle substitution if author is empty.
        if names_vec.is_empty() && matches!(component.contributor, ContributorRole::Author) {
            return substitute::resolve_author_substitute::<F>(
                &component,
                hints,
                options,
                reference,
                &effective_rendering,
                &fmt,
                &substitute,
            );
        }

        // Handle role-substitute if this role is empty.
        if names_vec.is_empty() {
            return substitute::resolve_role_substitute::<F>(
                &component.contributor,
                &component,
                hints,
                options,
                reference,
                &effective_rendering,
                &fmt,
                &substitute,
            );
        }

        let formatted =
            format_contributor_names(&component, &names_vec, &effective_rendering, options, hints);

        let role_omitted = is_role_label_omitted(options, &component.contributor);
        let (role_prefix, role_suffix) = labels::resolve_role_labels::<F>(
            &component,
            names_vec.len(),
            &effective_rendering,
            options,
            &fmt,
            role_omitted,
        );

        let is_pre_formatted = role_prefix.is_some() || role_suffix.is_some();
        let final_value = if is_pre_formatted {
            fmt.text(&formatted)
        } else {
            formatted
        };

        Some(ProcValues {
            value: final_value,
            prefix: role_prefix,
            suffix: role_suffix,
            url: crate::values::resolve_effective_url(
                component.links.as_ref(),
                options.config.links.as_ref(),
                reference,
                citum_schema::options::LinkAnchor::Component,
            ),
            substituted_key: None,
            pre_formatted: is_pre_formatted,
        })
    }
}
