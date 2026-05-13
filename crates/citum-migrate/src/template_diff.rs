/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Template variant diff computation for bibliography type-variant generation.

use citum_schema::{
    BibliographySpec, Style,
    template::{
        Rendering, TemplateAddOperation, TemplateComponent, TemplateComponentSelector,
        TemplateModifyOperation, TemplateRemoveOperation, TemplateVariant, TemplateVariantDiff,
        TypeSelector,
    },
};
use std::collections::BTreeMap;

/// Per-type template map used throughout the migration pipeline.
pub(crate) type TypeTemplateMap = indexmap::IndexMap<TypeSelector, Vec<TemplateComponent>>;

pub(crate) type TypeVariantMap = indexmap::IndexMap<TypeSelector, TemplateVariant>;

pub(crate) fn build_type_variants(
    default_template: &[TemplateComponent],
    type_templates: TypeTemplateMap,
) -> TypeVariantMap {
    let mut variants = TypeVariantMap::new();
    let mut candidate_parents: Vec<(TypeSelector, Vec<TemplateComponent>)> = Vec::new();

    for (selector, template) in type_templates {
        let variant = template_variant_from_full_template(
            default_template,
            &candidate_parents,
            &selector,
            template.clone(),
        );
        variants.insert(selector.clone(), variant);
        candidate_parents.push((selector, template));
    }

    variants
}

pub(crate) fn template_variant_from_full_template(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    selector: &TypeSelector,
    target_template: Vec<TemplateComponent>,
) -> TemplateVariant {
    let Some(diff) =
        derive_best_template_variant_diff(default_template, candidate_parents, &target_template)
    else {
        return TemplateVariant::Full(target_template);
    };

    if diff_resolves_to_template(
        default_template,
        candidate_parents,
        selector,
        &diff,
        &target_template,
    ) {
        TemplateVariant::Diff(diff)
    } else {
        TemplateVariant::Full(target_template)
    }
}

fn derive_best_template_variant_diff(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    target_template: &[TemplateComponent],
) -> Option<TemplateVariantDiff> {
    let mut best_diff = derive_template_variant_diff(default_template, target_template);
    let mut best_weight = best_diff
        .as_ref()
        .map(diff_operation_weight)
        .unwrap_or(usize::MAX);

    for (parent_selector, parent_template) in candidate_parents {
        let Some(mut parent_diff) = derive_template_variant_diff(parent_template, target_template)
        else {
            continue;
        };
        let weight = diff_operation_weight(&parent_diff);
        if weight >= best_weight {
            continue;
        }
        parent_diff.extends = Some(parent_selector.clone());
        best_diff = Some(parent_diff);
        best_weight = weight;
    }

    best_diff
}

fn diff_operation_weight(diff: &TemplateVariantDiff) -> usize {
    diff.modify.len() + diff.remove.len() + diff.add.len()
}

fn diff_resolves_to_template(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    selector: &TypeSelector,
    diff: &TemplateVariantDiff,
    expected_template: &[TemplateComponent],
) -> bool {
    let mut variants = TypeVariantMap::new();
    for (parent_selector, parent_template) in candidate_parents {
        variants.insert(
            parent_selector.clone(),
            TemplateVariant::Full(parent_template.clone()),
        );
    }
    variants.insert(selector.clone(), TemplateVariant::Diff(diff.clone()));
    let style = Style {
        bibliography: Some(BibliographySpec {
            template: Some(default_template.to_vec()),
            type_variants: Some(variants),
            ..Default::default()
        }),
        ..Default::default()
    };

    style
        .try_into_resolved()
        .ok()
        .and_then(|style| style.bibliography)
        .and_then(|bibliography| bibliography.type_variants)
        .and_then(|variants| variants.get(selector).cloned())
        .and_then(TemplateVariant::into_template)
        .is_some_and(|resolved| resolved == expected_template)
}

fn derive_template_variant_diff(
    default_template: &[TemplateComponent],
    target_template: &[TemplateComponent],
) -> Option<TemplateVariantDiff> {
    if default_template.is_empty() {
        return None;
    }

    let default_keys = component_keys(default_template)?;
    let target_keys = component_keys(target_template)?;
    let common_pairs = lcs_pairs(&default_keys, &target_keys);
    let mut diff = TemplateVariantDiff::default();

    for (index, component) in default_template.iter().enumerate() {
        if !common_pairs
            .iter()
            .any(|(base_index, _)| *base_index == index)
        {
            diff.remove.push(TemplateRemoveOperation {
                match_selector: component_selector(component)?,
            });
        }
    }

    for (base_index, target_index) in &common_pairs {
        let base_component = default_template.get(*base_index)?;
        let target_component = target_template.get(*target_index)?;
        if base_component != target_component {
            if !is_rendering_only_change(base_component, target_component) {
                return None;
            }
            diff.modify.push(TemplateModifyOperation {
                match_selector: component_selector(base_component)?,
                label_form: modified_number_label_form(base_component, target_component),
                rendering: target_component.rendering().clone(),
            });
        }
    }

    let mut last_anchor: Option<TemplateComponentSelector> = None;
    for (target_index, component) in target_template.iter().enumerate() {
        if let Some((base_index, _)) = common_pairs
            .iter()
            .find(|(_, common_target_index)| *common_target_index == target_index)
        {
            last_anchor = default_template
                .get(*base_index)
                .and_then(component_selector);
            continue;
        }

        let next_anchor = common_pairs
            .iter()
            .find(|(_, common_target_index)| *common_target_index > target_index)
            .and_then(|(base_index, _)| default_template.get(*base_index))
            .and_then(component_selector);

        let component_selector = component_selector(component)?;
        let add = if let Some(before) = next_anchor {
            TemplateAddOperation {
                before: Some(before),
                after: None,
                component: component.clone(),
            }
        } else if let Some(after) = last_anchor.clone() {
            TemplateAddOperation {
                before: None,
                after: Some(after),
                component: component.clone(),
            }
        } else {
            return None;
        };
        diff.add.push(add);
        last_anchor = Some(component_selector);
    }

    Some(diff)
}

fn component_keys(template: &[TemplateComponent]) -> Option<Vec<String>> {
    template
        .iter()
        .map(|component| serde_json::to_string(&component_selector(component)?.fields).ok())
        .collect()
}

pub(crate) fn component_selector(
    component: &TemplateComponent,
) -> Option<TemplateComponentSelector> {
    let serde_json::Value::Object(fields) = serde_json::to_value(component).ok()? else {
        return None;
    };
    for key in [
        "contributor",
        "date",
        "title",
        "number",
        "variable",
        "term",
        "group",
    ] {
        if let Some(value) = fields.get(key) {
            let mut selector = BTreeMap::new();
            selector.insert(key.to_string(), value.clone());
            return Some(TemplateComponentSelector { fields: selector });
        }
    }
    None
}

fn modified_number_label_form(
    base: &TemplateComponent,
    target: &TemplateComponent,
) -> Option<citum_schema::template::LabelForm> {
    match (base, target) {
        (TemplateComponent::Number(base_number), TemplateComponent::Number(target_number))
            if base_number.label_form != target_number.label_form =>
        {
            target_number.label_form.clone()
        }
        _ => None,
    }
}

fn is_rendering_only_change(base: &TemplateComponent, target: &TemplateComponent) -> bool {
    let mut normalized_base = base.clone();
    let mut normalized_target = target.clone();
    *normalized_base.rendering_mut() = Rendering::default();
    *normalized_target.rendering_mut() = Rendering::default();

    match (&mut normalized_base, &mut normalized_target) {
        (TemplateComponent::Number(base_number), TemplateComponent::Number(target_number))
            if base_number.label_form != target_number.label_form =>
        {
            if target_number.label_form.is_none() {
                return false;
            }
            base_number.label_form = None;
            target_number.label_form = None;
        }
        _ => {}
    }

    normalized_base == normalized_target
}

fn lcs_pairs(left: &[String], right: &[String]) -> Vec<(usize, usize)> {
    let mut lengths = vec![vec![0usize; right.len() + 1]; left.len() + 1];

    for i in (0..left.len()).rev() {
        for j in (0..right.len()).rev() {
            let diagonal = lengths
                .get(i + 1)
                .and_then(|row| row.get(j + 1))
                .copied()
                .unwrap_or(0);
            let down = lengths
                .get(i + 1)
                .and_then(|row| row.get(j))
                .copied()
                .unwrap_or(0);
            let right_value = lengths
                .get(i)
                .and_then(|row| row.get(j + 1))
                .copied()
                .unwrap_or(0);
            let value = if left.get(i) == right.get(j) {
                diagonal + 1
            } else {
                down.max(right_value)
            };
            if let Some(cell) = lengths.get_mut(i).and_then(|row| row.get_mut(j)) {
                *cell = value;
            }
        }
    }

    let mut pairs = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < left.len() && j < right.len() {
        if left.get(i) == right.get(j) {
            pairs.push((i, j));
            i += 1;
            j += 1;
            continue;
        }

        let down = lengths
            .get(i + 1)
            .and_then(|row| row.get(j))
            .copied()
            .unwrap_or(0);
        let right_value = lengths
            .get(i)
            .and_then(|row| row.get(j + 1))
            .copied()
            .unwrap_or(0);
        if down >= right_value {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}
