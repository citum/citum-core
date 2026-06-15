/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Re-bind or drop the leaked `in` term left bare by template specialization.
//!
//! citeproc-js only renders the `in` preposition when it sits inside a group
//! that also carries container data (editors, translators, a parent title);
//! that group suppresses when the container produces no output. Flattening a
//! base template into a type-variant can strip the enclosing group, leaving a
//! bare [`TemplateComponent::Term`] at the template root that the engine
//! renders unconditionally — the leaked `in.` seen on theses, reports, and
//! personal communications, which have no host container. This fixup restores
//! the binding so the engine's existing term-only group suppression applies
//! again.

use citum_schema::{
    locale::GeneralTerm,
    template::{
        ContributorRole, DelimiterPunctuation, TemplateComponent, TemplateGroup, TitleType,
    },
};

/// Re-group or drop the leaked `in` term at a template's root level.
///
/// The engine already suppresses a group whose only rendered children are
/// terms, so an `in` term *inside* a group is handled correctly. It leaks only
/// where it is flattened to the template root, with no enclosing group to
/// suppress it. For each such term, the contiguous run of container companions
/// it introduces is wrapped together with it into a single group; a term with
/// no companion is purely spurious and is dropped. Terms already correctly
/// scoped inside a group are left untouched.
pub fn gate_leaked_in_term(template: &mut Vec<TemplateComponent>) {
    let original = std::mem::take(template);
    let mut out: Vec<TemplateComponent> = Vec::with_capacity(original.len());
    let mut iter = original.into_iter().peekable();

    while let Some(component) = iter.next() {
        // Only act on a root-level `in` term that actually renders; a
        // suppressed term is inert, and inside any group the engine's
        // term-only suppression already gates the term on the group's
        // variables.
        if !is_live_in_term(&component) {
            out.push(component);
            continue;
        }

        // Collect the contiguous run of container components this term introduces.
        let mut run: Vec<TemplateComponent> = Vec::new();
        while iter.peek().is_some_and(is_container_component) {
            if let Some(next) = iter.next() {
                run.push(next);
            }
        }

        if run.is_empty() {
            // Orphan `in` with no container: it can never render correctly
            // (citeproc only emits it inside a populated group). Drop it.
            continue;
        }

        let mut members = Vec::with_capacity(run.len() + 1);
        members.push(component);
        members.extend(run);
        out.push(TemplateComponent::Group(TemplateGroup {
            group: members,
            delimiter: Some(DelimiterPunctuation::Space),
            ..Default::default()
        }));
    }

    *template = out;
}

/// Whether `component` is an `in` term that would render (not suppressed).
fn is_live_in_term(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Term(term)
            if term.term == GeneralTerm::In && component.rendering().suppress != Some(true)
    )
}

/// Whether `component` carries container/host data introduced by an `in` term.
fn is_container_component(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Contributor(contributor) => matches!(
            contributor.contributor,
            ContributorRole::Editor
                | ContributorRole::Translator
                | ContributorRole::ContainerAuthor
                | ContributorRole::CollectionEditor
                | ContributorRole::EditorialDirector
        ),
        TemplateComponent::Title(title) => matches!(
            title.title,
            TitleType::ParentSerial | TitleType::ParentMonograph
        ),
        _ => false,
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
        DateVariable, TemplateContributor, TemplateDate, TemplateTerm, TemplateTitle,
    };

    fn in_term() -> TemplateComponent {
        TemplateComponent::Term(TemplateTerm {
            term: GeneralTerm::In,
            ..Default::default()
        })
    }

    fn editor() -> TemplateComponent {
        TemplateComponent::Contributor(TemplateContributor {
            contributor: ContributorRole::Editor,
            ..Default::default()
        })
    }

    fn parent_serial() -> TemplateComponent {
        TemplateComponent::Title(TemplateTitle {
            title: TitleType::ParentSerial,
            ..Default::default()
        })
    }

    fn issued_date() -> TemplateComponent {
        TemplateComponent::Date(TemplateDate {
            date: DateVariable::Issued,
            ..Default::default()
        })
    }

    #[test]
    fn given_root_in_term_with_no_container_when_gated_then_term_dropped() {
        // china thesis/report/personal-communication shape: `in` flattened to
        // the root with no following container.
        let mut template = vec![in_term(), issued_date()];
        gate_leaked_in_term(&mut template);
        assert_eq!(template.len(), 1);
        assert!(matches!(
            &template[0],
            TemplateComponent::Date(date) if date.date == DateVariable::Issued
        ));
    }

    #[test]
    fn given_root_in_term_before_container_when_gated_then_wrapped_in_group() {
        let mut template = vec![in_term(), editor(), parent_serial(), issued_date()];
        gate_leaked_in_term(&mut template);
        assert_eq!(template.len(), 2);
        let TemplateComponent::Group(group) = &template[0] else {
            panic!("expected leading in-term group");
        };
        assert_eq!(group.group.len(), 3);
        assert!(matches!(
            &group.group[0],
            TemplateComponent::Term(term) if term.term == GeneralTerm::In
        ));
        assert!(matches!(&template[1], TemplateComponent::Date(_)));
    }

    #[test]
    fn given_in_term_inside_group_when_gated_then_left_for_engine_suppression() {
        // Inside a group the engine's term-only suppression already gates the
        // `in` term, so the pass must not touch it (touching it perturbs
        // type-variant diff derivation).
        let mut template = vec![TemplateComponent::Group(TemplateGroup {
            group: vec![in_term(), editor()],
            ..Default::default()
        })];
        gate_leaked_in_term(&mut template);
        let TemplateComponent::Group(group) = &template[0] else {
            panic!("expected outer group");
        };
        assert_eq!(group.group.len(), 2);
        assert!(matches!(
            &group.group[0],
            TemplateComponent::Term(term) if term.term == GeneralTerm::In
        ));
    }

    #[test]
    fn given_suppressed_in_term_when_gated_then_left_untouched() {
        let mut suppressed = in_term();
        suppressed.rendering_mut().suppress = Some(true);
        let mut template = vec![suppressed, editor()];
        gate_leaked_in_term(&mut template);
        assert_eq!(template.len(), 2);
        assert!(matches!(
            &template[0],
            TemplateComponent::Term(term) if term.term == GeneralTerm::In
        ));
    }
}
