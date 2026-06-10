/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use citum_schema::template::TemplateComponent;
use std::collections::HashSet;

/// Remove suppressed components whose variable also renders live in the
/// same template.
///
/// The occurrence-based compiler keeps components that only appear in
/// type-specific CSL branches as `suppress: true` placeholders in the
/// default template. The engine renders each variable at most once per
/// entry (first occurrence wins), so a suppressed placeholder that
/// precedes a live component referencing the same variable consumes it
/// and the live component renders empty — dropping container titles,
/// dates, and volumes from every type without a type-variant override.
///
/// Placeholders whose variable never renders live are kept: they carry
/// no poison and remain available as small-diff anchors for
/// type-variants.
pub fn strip_suppressed_variable_poison(components: &mut Vec<TemplateComponent>) {
    let live = collect_live_variable_keys(components);
    if live.is_empty() {
        return;
    }
    strip_in_scope(components, &live, false);
}

fn collect_live_variable_keys(components: &[TemplateComponent]) -> HashSet<String> {
    let mut keys = HashSet::new();
    collect_live(components, false, &mut keys);
    keys
}

fn collect_live(
    components: &[TemplateComponent],
    inherited_suppressed: bool,
    keys: &mut HashSet<String>,
) {
    for component in components {
        let suppressed = inherited_suppressed || component.rendering().suppress == Some(true);
        match component {
            TemplateComponent::Group(group) => collect_live(&group.group, suppressed, keys),
            _ => {
                if !suppressed && let Some(key) = variable_key(component) {
                    keys.insert(key);
                }
            }
        }
    }
}

fn strip_in_scope(
    components: &mut Vec<TemplateComponent>,
    live: &HashSet<String>,
    inherited_suppressed: bool,
) {
    components.retain_mut(|component| {
        let suppressed = inherited_suppressed || component.rendering().suppress == Some(true);
        match component {
            TemplateComponent::Group(group) => {
                let was_populated = !group.group.is_empty();
                strip_in_scope(&mut group.group, live, suppressed);
                // A group emptied by stripping renders nothing but would
                // still contribute affixes; drop it with its members.
                !(was_populated && group.group.is_empty())
            }
            _ => !(suppressed && variable_key(component).is_some_and(|key| live.contains(&key))),
        }
    });
}

/// Stable identity of the data a component renders, ignoring presentation.
fn variable_key(component: &TemplateComponent) -> Option<String> {
    match component {
        TemplateComponent::Contributor(inner) => {
            Some(format!("contributor:{:?}", inner.contributor))
        }
        TemplateComponent::Date(inner) => Some(format!("date:{:?}", inner.date)),
        TemplateComponent::Title(inner) => Some(format!("title:{:?}", inner.title)),
        TemplateComponent::Number(inner) => Some(format!("number:{:?}", inner.number)),
        TemplateComponent::Variable(inner) => Some(format!("variable:{:?}", inner.variable)),
        // Groups are keyed by their members; terms and any future
        // component kinds render no reference variable.
        _ => None,
    }
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
        DateVariable, NumberVariable, TemplateDate, TemplateGroup, TemplateNumber, TemplateTitle,
        TitleType,
    };

    fn title(title: TitleType, suppress: Option<bool>) -> TemplateComponent {
        let mut component = TemplateComponent::Title(TemplateTitle {
            title,
            ..Default::default()
        });
        component.rendering_mut().suppress = suppress;
        component
    }

    fn date(suppress: Option<bool>) -> TemplateComponent {
        let mut component = TemplateComponent::Date(TemplateDate {
            date: DateVariable::Issued,
            ..Default::default()
        });
        component.rendering_mut().suppress = suppress;
        component
    }

    fn volume() -> TemplateComponent {
        TemplateComponent::Number(TemplateNumber {
            number: NumberVariable::Volume,
            ..Default::default()
        })
    }

    fn group(members: Vec<TemplateComponent>, suppress: Option<bool>) -> TemplateComponent {
        let mut component = TemplateComponent::Group(TemplateGroup {
            group: members,
            ..Default::default()
        });
        component.rendering_mut().suppress = suppress;
        component
    }

    #[test]
    fn given_suppressed_duplicate_group_when_stripped_then_live_components_survive() {
        // Mirrors the zeitschrift-fur-allgemeinmedizin shape: a suppressed
        // branch group precedes live container/date components.
        let mut template = vec![
            title(TitleType::Primary, None),
            group(
                vec![title(TitleType::ParentSerial, None), date(None)],
                Some(true),
            ),
            group(
                vec![title(TitleType::ParentSerial, None), volume()],
                Some(false),
            ),
            date(Some(false)),
        ];

        strip_suppressed_variable_poison(&mut template);

        // The suppressed group loses both poisoned members and is dropped.
        assert_eq!(template.len(), 3);
        assert!(
            matches!(&template[0], TemplateComponent::Title(t) if t.title == TitleType::Primary)
        );
        assert!(matches!(&template[1], TemplateComponent::Group(g) if g.group.len() == 2));
        assert!(matches!(&template[2], TemplateComponent::Date(_)));
    }

    #[test]
    fn given_type_only_placeholder_when_stripped_then_placeholder_is_kept() {
        // A suppressed component whose variable never renders live is a
        // harmless type-variant anchor and must survive.
        let mut template = vec![
            title(TitleType::Primary, None),
            title(TitleType::ParentMonograph, Some(true)),
        ];

        strip_suppressed_variable_poison(&mut template);

        assert_eq!(template.len(), 2);
    }

    #[test]
    fn given_mixed_suppressed_group_when_stripped_then_only_poisoned_members_removed() {
        // Suppressed group with one poisoned member (date, live later) and
        // one placeholder (parent-monograph, never live): only the poison goes.
        let mut template = vec![
            group(
                vec![title(TitleType::ParentMonograph, None), date(None)],
                Some(true),
            ),
            date(Some(false)),
        ];

        strip_suppressed_variable_poison(&mut template);

        assert_eq!(template.len(), 2);
        let TemplateComponent::Group(group) = &template[0] else {
            panic!("expected group placeholder to survive");
        };
        assert_eq!(group.group.len(), 1);
        assert!(
            matches!(&group.group[0], TemplateComponent::Title(t) if t.title == TitleType::ParentMonograph)
        );
    }

    #[test]
    fn given_no_live_variables_when_stripped_then_template_unchanged() {
        let mut template = vec![title(TitleType::Primary, Some(true))];
        strip_suppressed_variable_poison(&mut template);
        assert_eq!(template.len(), 1);
    }
}
