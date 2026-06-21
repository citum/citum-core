/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Bounded mutation operators for the template synthesis loop.
//!
//! Each operator family enumerates single named mutations of one template in
//! a fixed deterministic order: component order moves first, then affix
//! edits, label form changes, and group boundary moves, each in component
//! index order. A proposal is one mutation applied to the source template —
//! proposals are never recombined — so the loop in
//! `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md` stays non-combinatorial.
//! No-op proposals are filtered and `CandidateBudget::max_per_family` caps
//! each family.

use crate::measured_citation::CandidateBudget;
use citum_schema::template::{
    LabelForm, Rendering, RoleLabelForm, Template, TemplateComponent, WrapConfig, WrapPunctuation,
};
use std::collections::BTreeMap;

/// Mutation operator families enumerated per synthesis round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MutationFamily {
    /// Swap adjacent top-level components.
    ComponentOrder,
    /// Clear a prefix or suffix, or toggle a parentheses wrap.
    AffixEdit,
    /// Change a number label form or a contributor role label form.
    LabelForm,
    /// Flatten a group or move a leading/trailing child out of it.
    GroupBoundary,
}

impl MutationFamily {
    /// Stable family label used in debug output and selection metadata.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ComponentOrder => "component-order",
            Self::AffixEdit => "affix-edit",
            Self::LabelForm => "label-form",
            Self::GroupBoundary => "group-boundary",
        }
    }
}

/// A single named mutation of a source template.
#[derive(Debug, Clone)]
pub(crate) struct MutationProposal {
    /// Operator family that produced this proposal.
    pub(crate) family: MutationFamily,
    /// Stable proposal name, unique within one enumeration.
    pub(crate) name: String,
    /// The mutated template.
    pub(crate) template: Template,
}

/// Enumerate one bounded round of mutation proposals for `template`.
///
/// Families and component indices are visited in a fixed order, so identical
/// inputs always enumerate identical proposals. Proposals equal to the
/// source template are dropped; `budget.max_per_family` caps each family.
pub(crate) fn enumerate_mutations(
    template: &[TemplateComponent],
    budget: CandidateBudget,
) -> Vec<MutationProposal> {
    let mut proposals = Vec::new();
    push_component_order_moves(template, &mut proposals);
    push_affix_edits(template, &mut proposals);
    push_label_form_changes(template, &mut proposals);
    push_group_boundary_moves(template, &mut proposals);

    let mut family_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    proposals.retain(|proposal| {
        if proposal.template.as_slice() == template {
            return false;
        }
        let count = family_counts.entry(proposal.family.as_str()).or_insert(0);
        *count += 1;
        *count <= budget.max_per_family
    });
    proposals
}

/// Propose swapping each adjacent top-level component pair.
fn push_component_order_moves(
    template: &[TemplateComponent],
    proposals: &mut Vec<MutationProposal>,
) {
    for index in 0..template.len().saturating_sub(1) {
        let mut mutated = template.to_vec();
        mutated.swap(index, index + 1);
        proposals.push(MutationProposal {
            family: MutationFamily::ComponentOrder,
            name: format!("order-swap-{index}-{}", index + 1),
            template: mutated,
        });
    }
}

/// Propose bounded affix edits per component: clear an existing prefix or
/// suffix, and toggle a parentheses wrap.
fn push_affix_edits(template: &[TemplateComponent], proposals: &mut Vec<MutationProposal>) {
    for (index, component) in template.iter().enumerate() {
        let rendering = component.rendering();
        if rendering.prefix.is_some() {
            proposals.push(affix_edit(template, index, "clear-prefix", |rendering| {
                rendering.prefix = None;
            }));
        }
        if rendering.suffix.is_some() {
            proposals.push(affix_edit(template, index, "clear-suffix", |rendering| {
                rendering.suffix = None;
            }));
        }
        if rendering.wrap.is_some() {
            proposals.push(affix_edit(template, index, "clear-wrap", |rendering| {
                rendering.wrap = None;
            }));
        } else {
            proposals.push(affix_edit(
                template,
                index,
                "wrap-parentheses",
                |rendering| {
                    rendering.wrap = Some(WrapConfig {
                        punctuation: WrapPunctuation::Parentheses,
                        inner_prefix: None,
                        inner_suffix: None,
                    });
                },
            ));
        }
    }
}

/// Build one affix-edit proposal by mutating the rendering at `index`.
fn affix_edit<F>(
    template: &[TemplateComponent],
    index: usize,
    edit: &str,
    apply: F,
) -> MutationProposal
where
    F: FnOnce(&mut Rendering),
{
    let mut mutated = template.to_vec();
    if let Some(component) = mutated.get_mut(index) {
        apply(component.rendering_mut());
    }
    MutationProposal {
        family: MutationFamily::AffixEdit,
        name: format!("affix-{edit}-{index}"),
        template: mutated,
    }
}

/// Propose label form alternatives for number components and the toggled
/// role label form for contributor components that render a role label.
fn push_label_form_changes(template: &[TemplateComponent], proposals: &mut Vec<MutationProposal>) {
    for (index, component) in template.iter().enumerate() {
        match component {
            TemplateComponent::Number(number) => {
                let current = number.label_form.clone().unwrap_or_default();
                let alternatives = [
                    (LabelForm::Long, "long"),
                    (LabelForm::Short, "short"),
                    (LabelForm::Symbol, "symbol"),
                ];
                for (form, form_name) in alternatives {
                    if form == current {
                        continue;
                    }
                    let mut mutated = template.to_vec();
                    if let Some(TemplateComponent::Number(target)) = mutated.get_mut(index) {
                        target.label_form = Some(form);
                    }
                    proposals.push(MutationProposal {
                        family: MutationFamily::LabelForm,
                        name: format!("label-form-{index}-{form_name}"),
                        template: mutated,
                    });
                }
            }
            TemplateComponent::Contributor(contributor) => {
                let Some(role_label) = contributor.label.as_ref() else {
                    continue;
                };
                let (form, form_name) = match role_label.form {
                    RoleLabelForm::Short => (RoleLabelForm::Long, "long"),
                    RoleLabelForm::Long => (RoleLabelForm::Short, "short"),
                };
                let mut mutated = template.to_vec();
                if let Some(TemplateComponent::Contributor(target)) = mutated.get_mut(index)
                    && let Some(label) = target.label.as_mut()
                {
                    label.form = form;
                }
                proposals.push(MutationProposal {
                    family: MutationFamily::LabelForm,
                    name: format!("role-label-form-{index}-{form_name}"),
                    template: mutated,
                });
            }
            _ => {}
        }
    }
}

/// Propose group boundary moves: flatten each group into its parent, and
/// move the leading or trailing child out of groups with two or more
/// children.
fn push_group_boundary_moves(
    template: &[TemplateComponent],
    proposals: &mut Vec<MutationProposal>,
) {
    for (index, component) in template.iter().enumerate() {
        let TemplateComponent::Group(group) = component else {
            continue;
        };

        let mut flattened = template.to_vec();
        flattened.splice(index..=index, group.group.iter().cloned());
        proposals.push(MutationProposal {
            family: MutationFamily::GroupBoundary,
            name: format!("group-flatten-{index}"),
            template: flattened,
        });

        if group.group.len() < 2 {
            continue;
        }
        let mut first_out = template.to_vec();
        if let Some(TemplateComponent::Group(target)) = first_out.get_mut(index) {
            let child = target.group.remove(0);
            first_out.insert(index, child);
        }
        proposals.push(MutationProposal {
            family: MutationFamily::GroupBoundary,
            name: format!("group-first-out-{index}"),
            template: first_out,
        });

        let mut last_out = template.to_vec();
        if let Some(TemplateComponent::Group(target)) = last_out.get_mut(index)
            && let Some(child) = target.group.pop()
        {
            last_out.insert(index + 1, child);
        }
        proposals.push(MutationProposal {
            family: MutationFamily::GroupBoundary,
            name: format!("group-last-out-{index}"),
            template: last_out,
        });
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "Panicking and direct indexing are acceptable in tests."
)]
mod tests {
    use super::{MutationFamily, enumerate_mutations};
    use crate::measured_citation::CandidateBudget;
    use citum_schema::template::{
        ContributorRole, LabelForm, NumberVariable, RoleLabel, RoleLabelForm, SimpleVariable,
        TemplateComponent, TemplateContributor, TemplateGroup, TemplateNumber, TemplateVariable,
    };
    use std::collections::BTreeMap;

    fn variable(variable: SimpleVariable) -> TemplateComponent {
        TemplateComponent::Variable(TemplateVariable {
            variable,
            ..Default::default()
        })
    }

    fn suffixed_variable(variable: SimpleVariable, suffix: &str) -> TemplateComponent {
        let mut component = self::variable(variable);
        component.rendering_mut().suffix = Some(suffix.to_string());
        component
    }

    fn sample_template() -> Vec<TemplateComponent> {
        vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Editor,
                label: Some(RoleLabel {
                    term: "editor".to_string(),
                    form: RoleLabelForm::Short,
                    placement: Default::default(),
                    text_case: None,
                }),
                ..Default::default()
            }),
            TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::Volume,
                ..Default::default()
            }),
            suffixed_variable(SimpleVariable::Doi, ". "),
            TemplateComponent::Group(TemplateGroup {
                group: vec![
                    variable(SimpleVariable::Url),
                    variable(SimpleVariable::Isbn),
                ],
                ..Default::default()
            }),
        ]
    }

    #[test]
    fn enumerate_mutations_is_deterministic_across_runs() {
        let template = sample_template();
        let budget = CandidateBudget::default();

        let first: Vec<String> = enumerate_mutations(&template, budget)
            .into_iter()
            .map(|proposal| proposal.name)
            .collect();
        let second: Vec<String> = enumerate_mutations(&template, budget)
            .into_iter()
            .map(|proposal| proposal.name)
            .collect();

        assert!(!first.is_empty());
        assert_eq!(first, second);
    }

    #[test]
    fn component_order_moves_swap_each_adjacent_pair() {
        let template = sample_template();

        let proposals = enumerate_mutations(&template, CandidateBudget::default());

        let swap = proposals
            .iter()
            .find(|proposal| proposal.name == "order-swap-1-2")
            .expect("adjacent swap proposal should be enumerated");
        assert_eq!(swap.family, MutationFamily::ComponentOrder);
        assert_eq!(swap.template[1], template[2]);
        assert_eq!(swap.template[2], template[1]);
    }

    #[test]
    fn affix_edits_clear_suffix_and_toggle_parentheses_wrap() {
        let template = sample_template();

        let proposals = enumerate_mutations(&template, CandidateBudget::default());

        let cleared = proposals
            .iter()
            .find(|proposal| proposal.name == "affix-clear-suffix-2")
            .expect("clear-suffix proposal should be enumerated");
        assert_eq!(cleared.template[2].rendering().suffix, None);
        let wrapped = proposals
            .iter()
            .find(|proposal| proposal.name == "affix-wrap-parentheses-0")
            .expect("wrap-parentheses proposal should be enumerated");
        assert!(wrapped.template[0].rendering().wrap.is_some());
    }

    #[test]
    fn label_form_changes_skip_the_current_effective_form() {
        let template = sample_template();

        let proposals = enumerate_mutations(&template, CandidateBudget::default());

        let number_forms: Vec<&str> = proposals
            .iter()
            .filter(|proposal| proposal.name.starts_with("label-form-1-"))
            .map(|proposal| proposal.name.as_str())
            .collect();
        assert_eq!(
            number_forms,
            vec!["label-form-1-long", "label-form-1-symbol"]
        );
        let toggled = proposals
            .iter()
            .find(|proposal| proposal.name == "role-label-form-0-long")
            .expect("role label toggle should be enumerated");
        let TemplateComponent::Contributor(contributor) = &toggled.template[0] else {
            panic!("toggled component should remain a contributor");
        };
        assert_eq!(
            contributor.label.as_ref().map(|label| label.form.clone()),
            Some(RoleLabelForm::Long)
        );
    }

    #[test]
    fn number_label_form_proposals_set_an_explicit_form() {
        let template = sample_template();

        let proposals = enumerate_mutations(&template, CandidateBudget::default());

        let long_form = proposals
            .iter()
            .find(|proposal| proposal.name == "label-form-1-long")
            .expect("long label form proposal should be enumerated");
        let TemplateComponent::Number(number) = &long_form.template[1] else {
            panic!("mutated component should remain a number");
        };
        assert_eq!(number.label_form, Some(LabelForm::Long));
    }

    #[test]
    fn group_boundary_moves_flatten_and_extract_children() {
        let template = sample_template();

        let proposals = enumerate_mutations(&template, CandidateBudget::default());

        let flattened = proposals
            .iter()
            .find(|proposal| proposal.name == "group-flatten-3")
            .expect("group flatten proposal should be enumerated");
        assert_eq!(flattened.template.len(), 5);
        assert_eq!(flattened.template[3], variable(SimpleVariable::Url));
        assert_eq!(flattened.template[4], variable(SimpleVariable::Isbn));

        let first_out = proposals
            .iter()
            .find(|proposal| proposal.name == "group-first-out-3")
            .expect("group first-out proposal should be enumerated");
        assert_eq!(first_out.template[3], variable(SimpleVariable::Url));
        let TemplateComponent::Group(rest) = &first_out.template[4] else {
            panic!("remaining group should follow the extracted child");
        };
        assert_eq!(rest.group, vec![variable(SimpleVariable::Isbn)]);

        let last_out = proposals
            .iter()
            .find(|proposal| proposal.name == "group-last-out-3")
            .expect("group last-out proposal should be enumerated");
        assert_eq!(last_out.template[4], variable(SimpleVariable::Isbn));
    }

    #[test]
    fn noop_proposals_are_filtered_from_the_enumeration() {
        let template = vec![variable(SimpleVariable::Doi), variable(SimpleVariable::Doi)];

        let proposals = enumerate_mutations(&template, CandidateBudget::default());

        assert!(
            proposals
                .iter()
                .all(|proposal| proposal.template != template),
            "every proposal must differ from the source template"
        );
        assert!(
            !proposals
                .iter()
                .any(|proposal| proposal.name.starts_with("order-swap")),
            "swapping identical components is a no-op and must be filtered"
        );
    }

    #[test]
    fn per_family_budget_caps_each_operator_family() {
        let template = sample_template();
        let budget = CandidateBudget {
            max_per_family: 1,
            max_total: 32,
        };

        let proposals = enumerate_mutations(&template, budget);

        let mut family_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
        for proposal in &proposals {
            *family_counts.entry(proposal.family.as_str()).or_insert(0) += 1;
        }
        for (family, count) in family_counts {
            assert!(count <= 1, "family {family} exceeds the per-family cap");
        }
    }
}
