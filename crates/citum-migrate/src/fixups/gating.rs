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
        ContributorRole, DateVariable, DelimiterPunctuation, SimpleVariable, TemplateComponent,
        TemplateGroup, TitleType, TypeSelector,
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

/// Reference types that citeproc-js gates url/accessed on.
///
/// Matches the `type="webpage post post-weblog"` conditional in CSL source.
const WEB_TYPES: &[&str] = &["webpage", "post", "post-weblog"];

/// Gate url and accessed components to web-only type templates.
///
/// citeproc-js renders url and the `accessed` term/date only when
/// `type="webpage post post-weblog"`. The converter flattens the wrapping
/// `<if type="webpage">` gate into the base template, so url and accessed
/// leak into every entry type. This fixup restores the gate:
///
/// - **Base and non-web type-templates** (`webpage`, `post`, `post-weblog`
///   absent from the selector): url variable, accessed date, and bare
///   `accessed` label terms are removed. An orphan `accessed` term with no
///   accompanying accessed date is purely spurious and is dropped.
/// - **Web type-templates** (selector matches a web type): left untouched so
///   the engine keeps its url/accessed rendering.
///
/// Must be called **before** `build_type_variants` so that the cleaned base
/// and type-template components are captured in diffs consistently.
pub fn gate_web_only_url_accessed(
    base_template: &mut Vec<TemplateComponent>,
    type_templates: &mut indexmap::IndexMap<TypeSelector, Vec<TemplateComponent>>,
) {
    strip_url_accessed(base_template);
    for (selector, template) in type_templates.iter_mut() {
        if !super::selector_matches_any(selector, WEB_TYPES) {
            strip_url_accessed(template);
        }
    }
}

/// Removes url variable, accessed date, and bare accessed label terms from a template.
///
/// An `accessed` term immediately followed by an accessed date is a label/date
/// pair introduced by the CSL webpage block; both are dropped. An orphan
/// `accessed` term with no accompanying accessed date is also dropped.
fn strip_url_accessed(template: &mut Vec<TemplateComponent>) {
    let original = std::mem::take(template);
    let mut out: Vec<TemplateComponent> = Vec::with_capacity(original.len());
    let mut iter = original.into_iter().peekable();

    while let Some(component) = iter.next() {
        if is_url_variable(&component) || is_accessed_date(&component) {
            // Drop url and standalone accessed-date components unconditionally.
            continue;
        }
        if is_accessed_term(&component) {
            // Consume the accessed term and any immediately following accessed
            // date (the pair introduced by the CSL webpage accessed block).
            // Neither renders correctly on non-web types; both are dropped.
            if iter.peek().is_some_and(is_accessed_date) {
                iter.next(); // drop the accompanying date too
            }
            continue;
        }
        // Recurse into groups so nested url/accessed are also cleaned.
        let mut component = component;
        if let TemplateComponent::Group(ref mut group) = component {
            strip_url_accessed(&mut group.group);
            // If the group is now empty after stripping, omit it entirely.
            if group.group.is_empty() {
                continue;
            }
        }
        out.push(component);
    }

    *template = out;
}

fn is_url_variable(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(v) if v.variable == SimpleVariable::Url
    )
}

fn is_accessed_date(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(d) if d.date == DateVariable::Accessed
    )
}

fn is_accessed_term(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Term(t)
            if t.term == GeneralTerm::Accessed && component.rendering().suppress != Some(true)
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
            TitleType::ContainerTitle | TitleType::ParentSerial | TitleType::ParentMonograph
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

    // ── gate_web_only_url_accessed tests ────────────────────────────────────

    fn url_variable() -> TemplateComponent {
        use citum_schema::template::{SimpleVariable, TemplateVariable};
        TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Url,
            ..Default::default()
        })
    }

    fn accessed_date() -> TemplateComponent {
        TemplateComponent::Date(TemplateDate {
            date: DateVariable::Accessed,
            ..Default::default()
        })
    }

    fn accessed_term() -> TemplateComponent {
        TemplateComponent::Term(TemplateTerm {
            term: GeneralTerm::Accessed,
            ..Default::default()
        })
    }

    #[test]
    fn given_base_template_with_url_and_accessed_when_gated_then_all_stripped() {
        // jar-style base: issued_date, url_variable, accessed_term, accessed_date
        // citeproc-js only emits these for webpage/post types; non-web base must
        // not carry them.
        let mut base = vec![
            issued_date(),
            url_variable(),
            accessed_term(),
            accessed_date(),
        ];
        let mut type_templates = indexmap::IndexMap::new();
        gate_web_only_url_accessed(&mut base, &mut type_templates);
        assert_eq!(base.len(), 1, "only issued_date should remain in the base");
        assert!(
            matches!(&base[0], TemplateComponent::Date(d) if d.date == DateVariable::Issued),
            "remaining component should be the issued date"
        );
    }

    #[test]
    fn given_non_web_type_template_with_url_when_gated_then_url_removed() {
        // An article-journal type-template must not carry url/accessed even if
        // it was flattened from a source conditional.
        let mut base = vec![issued_date()];
        let mut type_templates = indexmap::IndexMap::new();
        type_templates.insert(
            TypeSelector::Single("article-journal".to_string()),
            vec![issued_date(), url_variable()],
        );
        gate_web_only_url_accessed(&mut base, &mut type_templates);
        let article = type_templates
            .get(&TypeSelector::Single("article-journal".to_string()))
            .expect("article-journal template should still exist");
        assert_eq!(
            article.len(),
            1,
            "url should be removed from article-journal template"
        );
        assert!(
            matches!(&article[0], TemplateComponent::Date(d) if d.date == DateVariable::Issued),
            "issued date should remain in article-journal template"
        );
    }

    #[test]
    fn given_webpage_type_template_with_url_when_gated_then_url_retained() {
        // webpage entries should keep url/accessed intact — that is the correct
        // citeproc-js behaviour for web sources.
        let mut base = vec![issued_date()];
        let mut type_templates = indexmap::IndexMap::new();
        type_templates.insert(
            TypeSelector::Single("webpage".to_string()),
            vec![
                issued_date(),
                url_variable(),
                accessed_term(),
                accessed_date(),
            ],
        );
        gate_web_only_url_accessed(&mut base, &mut type_templates);
        let webpage = type_templates
            .get(&TypeSelector::Single("webpage".to_string()))
            .expect("webpage template should still exist");
        assert_eq!(
            webpage.len(),
            4,
            "url, accessed-term and accessed-date should be retained for webpage type"
        );
    }

    #[test]
    fn given_orphan_accessed_term_with_no_date_when_gated_then_term_dropped() {
        // A bare accessed term with no following accessed date is purely
        // spurious: there is no date to gate the label on, so drop it.
        let mut base = vec![issued_date(), accessed_term()];
        let mut type_templates = indexmap::IndexMap::new();
        gate_web_only_url_accessed(&mut base, &mut type_templates);
        assert_eq!(base.len(), 1, "orphan accessed term should be dropped");
        assert!(
            matches!(&base[0], TemplateComponent::Date(d) if d.date == DateVariable::Issued),
            "only issued date should remain"
        );
    }
}
