/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Pure predicates over `TemplateComponent` and `Reference` used by grouped
//! citation rendering and template-policy logic.

use crate::reference::Reference;
use citum_schema::{
    reference::NumOrStr,
    template::{DateVariable, NumberVariable, SimpleVariable, TemplateComponent},
};

/// Returns the first type-variant template whose selector matches `ref_type`,
/// or `None` if there are no variants or none match.
pub(super) fn resolve_type_variant<'a>(
    type_variants: Option<
        &'a indexmap::IndexMap<citum_schema::template::TypeSelector, citum_schema::TemplateVariant>,
    >,
    ref_type: &str,
) -> Option<&'a [TemplateComponent]> {
    let selector_candidates = aliased_type_selector_candidates(ref_type);
    type_variants?.iter().find_map(|(selector, variant)| {
        if selector_candidates
            .iter()
            .any(|candidate| selector.matches(candidate))
        {
            variant.as_template()
        } else {
            None
        }
    })
}

pub(super) fn aliased_type_selector_candidates(ref_type: &str) -> Vec<&str> {
    match ref_type {
        "chapter" => vec!["chapter", "entry-dictionary"],
        _ => vec![ref_type],
    }
}

pub(super) fn is_term_only_component(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Term(_) => true,
        TemplateComponent::Group(group) => group.group.iter().all(is_term_only_component),
        _ => false,
    }
}

pub(super) fn is_primary_title_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if title.title == citum_schema::template::TitleType::Primary
    )
}

pub(super) fn is_parent_container_title_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if matches!(
                title.title,
                citum_schema::template::TitleType::ContainerTitle
                    | citum_schema::template::TitleType::ParentSerial
                    | citum_schema::template::TitleType::ParentMonograph
                    | citum_schema::template::TitleType::CollectionTitle
            )
    )
}

pub(super) fn is_parent_monograph_title_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if title.title == citum_schema::template::TitleType::ParentMonograph
    )
}

pub(super) fn is_issued_date_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(date) if date.date == DateVariable::Issued
    )
}

pub(super) fn is_volume_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Number(number) if number.number == NumberVariable::Volume
    )
}

pub(super) fn is_url_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Url
    )
}

pub(super) fn is_doi_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Doi
    )
}

pub(super) fn is_article_detail_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(date) if date.date == DateVariable::Issued
    ) || matches!(
        component,
        TemplateComponent::Number(number)
            if matches!(
                number.number,
                NumberVariable::Volume | NumberVariable::Issue | NumberVariable::Pages
            )
    )
}

pub(super) fn reference_has_pages(reference: &Reference) -> bool {
    match reference.pages() {
        Some(NumOrStr::Str(pages)) => !pages.trim().is_empty(),
        Some(NumOrStr::Number(_)) => true,
        None => false,
    }
}

pub(super) fn reference_has_doi(reference: &Reference) -> bool {
    reference.doi().is_some_and(|doi| !doi.trim().is_empty())
}

pub(super) fn reference_has_url(reference: &Reference) -> bool {
    reference.url().is_some()
}

pub(super) fn reference_has_online_access(reference: &Reference) -> bool {
    reference_has_doi(reference) || reference_has_url(reference)
}
