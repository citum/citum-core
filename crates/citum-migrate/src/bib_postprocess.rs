/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Inferred bibliography template postprocessing and repair.

use citum_migrate::{
    fixups::{
        ensure_inferred_media_type_templates, ensure_inferred_patent_type_template,
        normalize_legal_case_type_template, scrub_inferred_literal_artifacts,
        should_merge_inferred_type_template,
    },
    template_resolver,
};
use citum_schema::{
    locale::GeneralTerm,
    template::{
        ContributorRole, DateVariable, SimpleVariable, TemplateComponent, TitleType, TypeSelector,
    },
};

use super::template_diff::TypeTemplateMap;

/// Extension trait to enumerate concrete type names from a `TypeSelector`.
pub(crate) trait TypeSelectorNames {
    fn type_names(&self) -> Vec<String>;
}

impl TypeSelectorNames for TypeSelector {
    fn type_names(&self) -> Vec<String> {
        match self {
            TypeSelector::Single(name) => vec![name.clone()],
            TypeSelector::Multiple(names) => names.clone(),
        }
    }
}

pub(crate) fn postprocess_inferred_bibliography(
    new_bib: &mut Vec<TemplateComponent>,
    type_templates: &mut Option<TypeTemplateMap>,
    legacy_style: &csl_legacy::model::Style,
) {
    for component in &mut *new_bib {
        scrub_inferred_literal_artifacts(component);
    }
    relax_inferred_bibliography_date_suppression(new_bib);
    if let Some(type_templates) = type_templates.as_mut() {
        for template in type_templates.values_mut() {
            for component in template.iter_mut() {
                scrub_inferred_literal_artifacts(component);
            }
            relax_inferred_bibliography_date_suppression(template);
        }
        repair_inferred_bibliography_type_templates(new_bib, type_templates);
    }
    normalize_legal_case_type_template(legacy_style, type_templates);
    ensure_inferred_media_type_templates(legacy_style, type_templates, new_bib);
    ensure_inferred_patent_type_template(legacy_style, type_templates, new_bib);
}

pub(crate) fn merge_inferred_type_templates(
    xml_fallback: &citum_migrate::compilation::XmlCompilationOutput,
    resolved_bib_template: &[TemplateComponent],
) -> Option<TypeTemplateMap> {
    xml_fallback
        .type_templates
        .clone()
        .map(|type_templates| {
            type_templates
                .into_iter()
                .filter(|(selector, type_template)| {
                    selector.type_names().iter().any(|type_name| {
                        should_merge_inferred_type_template(
                            type_name,
                            resolved_bib_template,
                            type_template,
                        )
                    })
                })
                .collect::<indexmap::IndexMap<_, _>>()
        })
        .filter(|m| !m.is_empty())
}

pub(crate) fn is_inferred_bib_source(source: &template_resolver::TemplateSource) -> bool {
    matches!(
        source,
        template_resolver::TemplateSource::InferredCached(_)
            | template_resolver::TemplateSource::InferredLive
    )
}

pub(crate) fn repair_inferred_bibliography_type_templates(
    default_template: &[TemplateComponent],
    type_templates: &mut TypeTemplateMap,
) {
    let base_title = default_template
        .iter()
        .find(|component| component_is_primary_title(component))
        .cloned();
    let base_publisher = default_template
        .iter()
        .find(|component| component_is_publisher(component))
        .cloned();

    for (selector, template) in type_templates.iter_mut() {
        let type_names = selector.type_names();

        if should_inherit_primary_title(&type_names)
            && !template.iter().any(component_is_primary_title)
            && let Some(base_title) = base_title.clone()
        {
            let insert_at = template
                .iter()
                .position(|component| {
                    component_is_container_title(component) || component_is_in_term(component)
                })
                .or_else(|| {
                    template
                        .iter()
                        .rposition(|component| {
                            component_is_author(component) || component_is_issued_date(component)
                        })
                        .map(|index| index + 1)
                })
                .unwrap_or(0);
            template.insert(insert_at, base_title);
        }

        if should_inherit_publisher(&type_names)
            && !template.iter().any(component_is_publisher)
            && let Some(base_publisher) = base_publisher.clone()
        {
            let insert_at = template
                .iter()
                .rposition(component_is_publisher_place)
                .map(|index| index + 1)
                .or_else(|| {
                    template
                        .iter()
                        .rposition(component_is_primary_title)
                        .map(|index| index + 1)
                })
                .unwrap_or(template.len());
            template.insert(insert_at, base_publisher);
        }
    }
}

fn should_inherit_primary_title(type_names: &[String]) -> bool {
    type_names.iter().any(|type_name| {
        matches!(
            type_name.as_str(),
            "article-magazine"
                | "article-newspaper"
                | "book"
                | "broadcast"
                | "motion_picture"
                | "motion-picture"
                | "report"
        )
    })
}

fn should_inherit_publisher(type_names: &[String]) -> bool {
    type_names.iter().any(|type_name| {
        matches!(
            type_name.as_str(),
            "book" | "motion_picture" | "motion-picture" | "report" | "thesis"
        )
    })
}

pub(crate) fn component_is_primary_title(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if title.title == TitleType::Primary
    )
}

fn component_is_container_title(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if matches!(
                title.title,
                citum_schema::template::TitleType::ParentMonograph
                    | citum_schema::template::TitleType::ParentSerial
            )
    )
}

fn component_is_author(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Contributor(contributor)
            if contributor.contributor == ContributorRole::Author
    )
}

fn component_is_issued_date(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(date)
            if date.date == DateVariable::Issued
    )
}

pub(crate) fn component_is_in_term(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Term(term)
            if term.term == GeneralTerm::In
    )
}

pub(crate) fn component_is_publisher(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable)
            if variable.variable == SimpleVariable::Publisher
    )
}

pub(crate) fn component_is_publisher_place(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable)
            if variable.variable == SimpleVariable::PublisherPlace
    )
}

pub(crate) fn relax_inferred_bibliography_date_suppression(template: &mut [TemplateComponent]) {
    for component in template {
        if let TemplateComponent::Group(list) = component {
            relax_inferred_bibliography_date_suppression(&mut list.group);
        }
    }
}
