/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
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

/// Builds all template variants for a given reference type from a type-template map.
///
/// This runs before lineage/wrapper conversion, so raw template equality is not
/// final semantic equality. Preserve the original selector keys here; any
/// selector grouping must happen only after resolved parent-aware templates are
/// known.
pub(crate) fn build_type_variants(
    default_template: &[TemplateComponent],
    type_templates: TypeTemplateMap,
) -> TypeVariantMap {
    let mut variants = TypeVariantMap::new();
    let mut candidate_parents: Vec<(TypeSelector, Vec<TemplateComponent>)> = Vec::new();
    let intended_templates = type_templates.clone();

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

    engine_validate_variants(default_template, variants, &intended_templates)
}

/// Validate all derived variants through the engine's authoritative resolver.
///
/// Constructs a throwaway [`Style`] that mirrors what the engine will load,
/// resolves it with the same logic the engine uses at render time, and
/// compares each engine-resolved template against the original intended target.
/// Any variant whose engine-resolved form diverges from the intent is demoted
/// to [`TemplateVariant::Full`] so the engine can use it without reconstruction.
///
/// This eliminates the latent migrate-vs-engine diff-resolver mismatch where
/// migrate's private pairwise check accepted diffs that the engine's recursive
/// `extends` resolution would apply differently (e.g. corrupting `legal_case`
/// when the shared base template was modified by a fixup).
pub(crate) fn engine_validate_variants(
    default_template: &[TemplateComponent],
    mut variants: TypeVariantMap,
    intended_templates: &TypeTemplateMap,
) -> TypeVariantMap {
    // Build a minimal throwaway style that carries exactly the section template
    // and the candidate variant map — no extends, no version, no citum-version
    // constraint — so try_into_resolved() only runs resolve_style_template_variants.
    let probe = Style {
        bibliography: Some(BibliographySpec {
            template: Some(default_template.to_vec()),
            type_variants: Some(variants.clone()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let engine_resolved_variants = probe
        .try_into_resolved()
        .ok()
        .and_then(|s| s.bibliography)
        .and_then(|bib| bib.type_variants);

    for (selector, variant) in &mut variants {
        if !matches!(variant, TemplateVariant::Diff(_)) {
            continue;
        }
        let Some(target) = intended_templates.get(selector) else {
            continue;
        };
        let engine_template = engine_resolved_variants
            .as_ref()
            .and_then(|m| m.get(selector))
            .and_then(TemplateVariant::as_template);
        if engine_template != Some(target.as_slice()) {
            // Engine resolves this diff to a different template than intended:
            // demote to Full so the engine uses the correct template verbatim.
            *variant = TemplateVariant::Full(target.clone());
        }
    }

    variants
}

/// Extracts a single variant from a full template definition, computing a minimal diff if possible.
pub(crate) fn template_variant_from_full_template(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    _selector: &TypeSelector,
    target_template: Vec<TemplateComponent>,
) -> TemplateVariant {
    let Some(diff) =
        derive_best_template_variant_diff(default_template, candidate_parents, &target_template)
    else {
        return TemplateVariant::Full(target_template);
    };

    if diff_resolves_to_template(default_template, candidate_parents, &diff, &target_template) {
        TemplateVariant::Diff(diff)
    } else {
        TemplateVariant::Full(target_template)
    }
}

/// Selects the lowest-weight diff across all computed variant diffs.
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

/// Assigns a numeric cost to a diff operation for selecting the lowest-weight variant diff.
fn diff_operation_weight(diff: &TemplateVariantDiff) -> usize {
    diff.modify.len() + diff.remove.len() + diff.add.len()
}

/// Round-trip correctness check: applies the computed diff and verifies it reproduces the target.
fn diff_resolves_to_template(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    diff: &TemplateVariantDiff,
    expected_template: &[TemplateComponent],
) -> bool {
    let mut resolved = diff
        .extends
        .as_ref()
        .and_then(|extends| {
            candidate_parents
                .iter()
                .find(|(selector, _)| selector == extends)
                .map(|(_, template)| template.clone())
        })
        .unwrap_or_else(|| default_template.to_vec());

    apply_template_variant_diff(&mut resolved, diff).is_some_and(|()| resolved == expected_template)
}

/// Computes a diff between a single template variant and the target template.
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

/// Extracts the sequence of component keys from a template for LCS alignment.
fn component_keys(template: &[TemplateComponent]) -> Option<Vec<String>> {
    template.iter().map(component_key).collect()
}

/// Builds a selector keyed on a component's type discriminant and identifying value (e.g. `variable: title`).
pub(crate) fn component_selector(
    component: &TemplateComponent,
) -> Option<TemplateComponentSelector> {
    let (key, value) = component_selector_value(component)?;
    let mut selector = BTreeMap::new();
    selector.insert(key.to_string(), value);
    Some(TemplateComponentSelector { fields: selector })
}

/// Returns the unique key for a single template component.
fn component_key(component: &TemplateComponent) -> Option<String> {
    let (key, value) = component_selector_value(component)?;
    let value = serde_json::to_string(&value).ok()?;
    Some(format!("{key}:{value}"))
}

/// Returns the value portion of a component selector.
fn component_selector_value(
    component: &TemplateComponent,
) -> Option<(&'static str, serde_json::Value)> {
    match component {
        TemplateComponent::Contributor(inner) => Some((
            "contributor",
            serde_json::to_value(&inner.contributor).ok()?,
        )),
        TemplateComponent::Date(inner) => Some(("date", serde_json::to_value(&inner.date).ok()?)),
        TemplateComponent::Title(inner) => {
            Some(("title", serde_json::to_value(&inner.title).ok()?))
        }
        TemplateComponent::Number(inner) => {
            Some(("number", serde_json::to_value(&inner.number).ok()?))
        }
        TemplateComponent::Variable(inner) => {
            Some(("variable", serde_json::to_value(&inner.variable).ok()?))
        }
        TemplateComponent::Term(inner) => Some(("term", serde_json::to_value(&inner.term).ok()?)),
        TemplateComponent::Group(inner) => {
            Some(("group", serde_json::to_value(&inner.group).ok()?))
        }
        _ => None,
    }
}

/// Applies a variant diff to produce a modified template.
fn apply_template_variant_diff(
    template: &mut Vec<TemplateComponent>,
    diff: &TemplateVariantDiff,
) -> Option<()> {
    // Apply ops in dependency order: modify in place first, then remove, then insert.
    // The `?` on each anchor lookup aborts early, preventing partial application.
    for op in &diff.modify {
        let index = find_unique_anchor(template, &op.match_selector)?;
        let component = template.get_mut(index)?;
        if let Some(label_form) = op.label_form.clone()
            && let TemplateComponent::Number(number) = component
        {
            number.label_form = Some(label_form);
        }
        component.rendering_mut().merge(&op.rendering);
    }

    for op in &diff.remove {
        let index = find_unique_anchor(template, &op.match_selector)?;
        template.remove(index);
    }

    for op in &diff.add {
        // Exactly one of `before` or `after` must be set; any other combination is invalid.
        let (selector, insert_after) = match (&op.before, &op.after) {
            (Some(selector), None) => (selector, false),
            (None, Some(selector)) => (selector, true),
            _ => return None,
        };
        let anchor_index = find_unique_anchor(template, selector)?;
        let insert_at = if insert_after {
            anchor_index.saturating_add(1)
        } else {
            anchor_index
        };
        template.insert(insert_at, op.component.clone());
    }

    Some(())
}

/// Finds a component key that appears exactly once in both sequences, usable as an LCS anchor.
fn find_unique_anchor(
    template: &[TemplateComponent],
    selector: &TemplateComponentSelector,
) -> Option<usize> {
    if selector.is_empty() {
        return None;
    }
    let mut matches = template
        .iter()
        .enumerate()
        .filter_map(|(index, component)| selector.matches(component).then_some(index));
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

/// Returns the modified label form for a number component if it differs between default and target.
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

/// Returns true if the diff touches only rendering options, not structural layout.
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

/// Computes longest common subsequence index pairs used to align default and target template components.
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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;
    use citum_schema::template::{
        SimpleVariable, TemplateComponent, TemplateTitle, TemplateVariable, TemplateVariant,
        TemplateVariantDiff, TitleType, TypeSelector,
    };

    fn title_component() -> TemplateComponent {
        TemplateComponent::Title(TemplateTitle {
            title: TitleType::Primary,
            ..Default::default()
        })
    }

    fn url_component() -> TemplateComponent {
        use citum_schema::template::SimpleVariable;
        TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Url,
            ..Default::default()
        })
    }

    fn url_selector() -> TemplateComponentSelector {
        let mut fields = BTreeMap::new();
        fields.insert(
            "variable".to_string(),
            serde_json::to_value(SimpleVariable::Url).unwrap(),
        );
        TemplateComponentSelector { fields }
    }

    #[test]
    fn given_diff_with_anchor_absent_from_base_when_engine_validated_then_demoted_to_full() {
        // Regression guard for the migrate-vs-engine diff-resolver mismatch:
        // a Diff whose remove operation targets an anchor not present in the
        // default template must be caught by engine_validate_variants and
        // demoted to Full so the engine can use the correct template verbatim.
        //
        // This mirrors what can happen when a base-fixup removes a component
        // (e.g. url) after a Diff was derived against the old base — the stored
        // Diff would reference the now-absent component, causing the engine to
        // fail resolution and producing wrong output.
        let default_template = vec![title_component()];

        // "article" intended: [title_component] (url was never there after fixup)
        // but a hand-crafted Diff still tries to remove url, which the engine
        // cannot find in the section template.
        let bad_diff = TemplateVariant::Diff(TemplateVariantDiff {
            remove: vec![citum_schema::template::TemplateRemoveOperation {
                match_selector: url_selector(),
            }],
            ..Default::default()
        });

        let mut variants = TypeVariantMap::new();
        variants.insert(TypeSelector::Single("article".to_string()), bad_diff);

        let mut intended = TypeTemplateMap::new();
        intended.insert(
            TypeSelector::Single("article".to_string()),
            vec![title_component()],
        );

        let result = engine_validate_variants(&default_template, variants, &intended);
        let article = result
            .get(&TypeSelector::Single("article".to_string()))
            .expect("article variant should be present");

        assert!(
            matches!(article, TemplateVariant::Full(_)),
            "bad Diff with absent anchor should be demoted to Full by engine_validate_variants"
        );
        let TemplateVariant::Full(template) = article else {
            panic!("variant should be Full after demotion");
        };
        assert_eq!(
            template,
            &vec![title_component()],
            "demoted Full template should match the original intended template"
        );
    }

    #[test]
    fn given_correct_diff_when_engine_validated_then_diff_retained() {
        // A Diff that the engine can round-trip correctly must not be demoted.
        let default_template = vec![title_component(), url_component()];

        // "article" drops url — a valid diff the engine can apply.
        let mut type_templates = TypeTemplateMap::new();
        type_templates.insert(
            TypeSelector::Single("article".to_string()),
            vec![title_component()],
        );

        let variants = build_type_variants(&default_template, type_templates);
        let article = variants
            .get(&TypeSelector::Single("article".to_string()))
            .expect("article variant should be present");

        assert!(
            matches!(article, TemplateVariant::Diff(_)),
            "a valid Diff should be kept as Diff — engine_validate_variants must not demote correct diffs"
        );
    }

    #[test]
    fn given_raw_equal_templates_when_variants_built_then_original_selectors_are_preserved() {
        // Regression guard for csl26-sfi3: this function runs before
        // lineage/wrapper resolution, where a raw-equal `book` template can
        // require a different parent-aware result than `bill`. Raw equality
        // must therefore not be collapsed into a broad multi-selector here.
        let default_template = vec![title_component(), url_component()];
        let raw_variant = vec![title_component()];
        let bill_selector = TypeSelector::Single("bill".to_string());
        let book_selector = TypeSelector::Single("book".to_string());
        let mut type_templates = TypeTemplateMap::new();
        type_templates.insert(bill_selector.clone(), raw_variant.clone());
        type_templates.insert(book_selector.clone(), raw_variant);

        let variants = build_type_variants(&default_template, type_templates);

        assert!(
            variants.contains_key(&bill_selector),
            "bill must stay addressable as its original selector"
        );
        assert!(
            variants.contains_key(&book_selector),
            "book must stay addressable as its original selector"
        );
        assert!(
            !variants
                .keys()
                .any(|selector| matches!(selector, TypeSelector::Multiple(_))),
            "pre-wrapper variant construction must not synthesize grouped selectors"
        );
    }
}
