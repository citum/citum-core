/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use super::{CslnNode, ItemType, TemplateCompiler, TemplateComponent};
use citum_schema::template::{NumberVariable, SimpleVariable};

// The full-corpus scorecard currently has no bibliography root above 500
// recursive components except the known APA 6 XML-fallback outlier. Keeping the
// cutoff above normal styles confines branch-aware cleanup to pathological
// output while preserving established migration shapes for the rest of the
// corpus.
const PATHOLOGICAL_BIBLIOGRAPHY_COMPONENTS: usize = 500;

impl TemplateCompiler {
    #[must_use]
    pub fn compile_bibliography(
        &self,
        nodes: &[CslnNode],
        _is_numeric: bool,
    ) -> Vec<TemplateComponent> {
        // DISABLED: Sorting was needed to work around HashMap's random iteration order.
        // Now that we use IndexMap, we preserve the CSL 1.0 layout order naturally.
        let mut template = self.compile(nodes);
        crate::passes::deduplicate::deduplicate_numbers_in_lists(&mut template);
        crate::passes::deduplicate::deduplicate_dates_in_lists(&mut template);
        crate::passes::deduplicate::remove_redundant_no_date_terms(&mut template);
        self.fix_duplicate_variables(&mut template);
        template
    }

    /// Compile bibliography with type-specific templates.
    ///
    /// Uses the new occurrence-based compilation approach which correctly handles
    /// mutually exclusive conditional branches with proper suppress semantics.
    #[must_use]
    pub fn compile_bibliography_with_types(
        &self,
        nodes: &[CslnNode],
        _is_numeric: bool,
    ) -> (
        Vec<TemplateComponent>,
        indexmap::IndexMap<citum_schema::template::TypeSelector, Vec<TemplateComponent>>,
    ) {
        let mut default_template = self.compile(nodes);

        // DISABLED: Hardcoded sorting doesn't work for all styles (e.g., numeric styles have different order).
        // A general solution requires preserving macro call order from CSL 1.0 during parsing.
        // self.sort_bibliography_components(&mut default_template, _is_numeric);

        // Deduplicate number components (edition, volume, issue) in nested lists
        crate::passes::deduplicate::deduplicate_numbers_in_lists(&mut default_template);

        // Deduplicate date components (issued, accessed) in nested lists
        crate::passes::deduplicate::deduplicate_dates_in_lists(&mut default_template);
        crate::passes::deduplicate::remove_redundant_no_date_terms(&mut default_template);

        // Fix duplicate variables (e.g., date appearing both in List and standalone)
        self.fix_duplicate_variables(&mut default_template);

        let use_pathological_bibliography_cleanup =
            template_component_count(&default_template) > PATHOLOGICAL_BIBLIOGRAPHY_COMPONENTS;

        if use_pathological_bibliography_cleanup {
            // Compile pathological bibliography roots from non-type/default
            // branches only. Type-specific branches are represented below as
            // type variants; keeping them in the root template creates large
            // standalone output with hidden conditional-only components.
            default_template = self.compile_bibliography_default(nodes);
            crate::passes::deduplicate::deduplicate_numbers_in_lists(&mut default_template);
            crate::passes::deduplicate::deduplicate_dates_in_lists(&mut default_template);
            crate::passes::deduplicate::remove_redundant_no_date_terms(&mut default_template);
            self.fix_duplicate_variables(&mut default_template);
            deduplicate_exact_components(&mut default_template);
        }

        // Generate selective type templates for high-impact outlier types where
        // branch-specific structure is often materially different from the
        // default template (and where suppress-only overrides are insufficient).
        //
        // These templates are intentionally scoped to limit migration noise.
        let type_templates: indexmap::IndexMap<
            citum_schema::template::TypeSelector,
            Vec<TemplateComponent>,
        > = self.generate_selective_type_templates(
            nodes,
            &default_template,
            use_pathological_bibliography_cleanup,
        );

        (default_template, type_templates)
    }

    fn compile_bibliography_default(&self, nodes: &[CslnNode]) -> Vec<TemplateComponent> {
        let no_wrap = (None, None, None);
        let mut occurrences = Vec::new();
        self.collect_bibliography_default_occurrences(
            nodes,
            &no_wrap,
            &super::BranchContext::Default,
            &mut occurrences,
        );
        self.merge_occurrences(occurrences)
    }

    pub(super) fn generate_selective_type_templates(
        &self,
        nodes: &[CslnNode],
        default_template: &[TemplateComponent],
        deduplicate_exact_type_components: bool,
    ) -> indexmap::IndexMap<citum_schema::template::TypeSelector, Vec<TemplateComponent>> {
        use citum_schema::template::TypeSelector;

        let mut candidates = self.collect_types_with_branches(nodes);
        if !candidates.contains(&ItemType::EntryEncyclopedia) {
            candidates.push(ItemType::EntryEncyclopedia);
        }

        // Add monograph types unconditionally (Fix D) — these types appear in CSL else branches
        // and need type-specific templates even if not detected by branch collector
        let monograph_types = vec![
            ItemType::Book,
            ItemType::Thesis,
            ItemType::Report,
            ItemType::Chapter,
            ItemType::PaperConference,
            ItemType::Manuscript,
        ];
        for mt in monograph_types {
            if !candidates.contains(&mt) {
                candidates.push(mt);
            }
        }

        candidates.sort_by_key(|t| self.item_type_to_string(t));
        candidates.dedup_by_key(|t| self.item_type_to_string(t));

        let mut type_templates = indexmap::IndexMap::new();
        for item_type in candidates {
            let mut type_template = if deduplicate_exact_type_components {
                self.compile_for_type_with_untyped_else_if_fallback(nodes, &item_type)
            } else {
                self.compile_for_type(nodes, &item_type)
            };
            if type_template.is_empty() {
                continue;
            }

            crate::passes::deduplicate::deduplicate_numbers_in_lists(&mut type_template);
            crate::passes::deduplicate::deduplicate_dates_in_lists(&mut type_template);
            crate::passes::deduplicate::remove_redundant_no_date_terms(&mut type_template);
            self.fix_duplicate_variables(&mut type_template);
            if deduplicate_exact_type_components {
                deduplicate_exact_components(&mut type_template);
            }

            // Post-process legal_case templates: ensure authority variable is
            // present (it appears in complex nested conditions that compile_for_type
            // may not fully resolve) and suppress volume/pages which are inapplicable.
            if matches!(item_type, ItemType::LegalCase) {
                self.postprocess_legal_case_template(&mut type_template);
            }

            if matches!(item_type, ItemType::ArticleJournal)
                && !self.article_journal_template_needs_preservation(&type_template)
            {
                continue;
            }

            if type_template == default_template {
                continue;
            }

            type_templates.insert(
                TypeSelector::Single(self.item_type_to_string(&item_type)),
                type_template,
            );
        }

        type_templates
    }

    fn article_journal_template_needs_preservation(&self, template: &[TemplateComponent]) -> bool {
        let has_doi = template.iter().any(component_has_doi);
        let has_detail_component = template.iter().any(component_has_article_detail);
        has_doi && has_detail_component
    }

    /// Post-process a `legal_case` type template to ensure correct field set.
    ///
    /// Legal case citations follow the pattern: Title, Authority Year.
    /// - Ensures `variable: authority` is inserted after `title: primary`
    /// - Suppresses `number: volume`, `number: pages`, and `number: issue`
    ///   which are inapplicable to legal case citations.
    pub(super) fn postprocess_legal_case_template(&self, template: &mut Vec<TemplateComponent>) {
        use citum_schema::template::{SimpleVariable, TemplateVariable};

        // Suppress volume, pages and issue — inapplicable for legal cases
        for comp in template.iter_mut() {
            match comp {
                TemplateComponent::Number(n)
                    if matches!(
                        n.number,
                        citum_schema::template::NumberVariable::Volume
                            | citum_schema::template::NumberVariable::Pages
                            | citum_schema::template::NumberVariable::Issue
                    ) =>
                {
                    n.rendering.suppress = Some(true);
                }
                _ => {}
            }
        }

        // Inject authority variable after title:primary if not already present
        let has_authority = template.iter().any(|c| {
            matches!(
                c,
                TemplateComponent::Variable(v) if v.variable == SimpleVariable::Authority
            )
        });

        if !has_authority {
            // Find position of title:primary to insert after it
            let insert_pos = template
                .iter()
                .position(|c| {
                    matches!(
                        c,
                        TemplateComponent::Title(t)
                            if t.title == citum_schema::template::TitleType::Primary
                    )
                })
                .map_or(template.len(), |p| p + 1);

            template.insert(
                insert_pos,
                TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::Authority,
                    rendering: citum_schema::template::Rendering::default(),
                    ..Default::default()
                }),
            );
        }
    }
}

fn component_has_doi(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Variable(variable) => variable.variable == SimpleVariable::Doi,
        TemplateComponent::Group(list) => list.group.iter().any(component_has_doi),
        _ => false,
    }
}

fn deduplicate_exact_components(template: &mut Vec<TemplateComponent>) {
    let mut unique = Vec::new();
    for component in template.drain(..) {
        if !unique.iter().any(|seen| seen == &component) {
            unique.push(component);
        }
    }
    *template = unique;
}

fn template_component_count(template: &[TemplateComponent]) -> usize {
    template.iter().map(component_count).sum()
}

fn component_count(component: &TemplateComponent) -> usize {
    match component {
        TemplateComponent::Group(group) => {
            1 + group.group.iter().map(component_count).sum::<usize>()
        }
        _ => 1,
    }
}

fn component_has_article_detail(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Date(date) => date.date == citum_schema::template::DateVariable::Issued,
        TemplateComponent::Number(number) => matches!(
            number.number,
            NumberVariable::Volume | NumberVariable::Issue | NumberVariable::Pages
        ),
        TemplateComponent::Group(list) => list.group.iter().any(component_has_article_detail),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::{
        FormattingOptions, Variable,
        legacy::{ConditionBlock, ElseIfBranch, VariableBlock},
        template::{SimpleVariable, TemplateComponent, TitleType, TypeSelector},
    };
    use std::collections::HashMap;

    fn variable_node(variable: Variable, source_order: usize) -> CslnNode {
        CslnNode::Variable(VariableBlock {
            variable,
            label: None,
            formatting: FormattingOptions::default(),
            overrides: HashMap::new(),
            source_order: Some(source_order),
        })
    }

    fn first_condition(
        if_item_type: Vec<ItemType>,
        if_variables: Vec<Variable>,
        then_branch: Vec<CslnNode>,
        else_branch: Option<Vec<CslnNode>>,
    ) -> CslnNode {
        CslnNode::Condition(ConditionBlock {
            if_item_type,
            if_variables,
            then_branch,
            else_if_branches: vec![],
            else_branch,
        })
    }

    fn condition_with_else_if(
        if_item_type: Vec<ItemType>,
        then_branch: Vec<CslnNode>,
        else_if_branch: ElseIfBranch,
    ) -> CslnNode {
        CslnNode::Condition(ConditionBlock {
            if_item_type,
            if_variables: vec![],
            then_branch,
            else_if_branches: vec![else_if_branch],
            else_branch: None,
        })
    }

    fn template_has_primary_title(template: &[TemplateComponent]) -> bool {
        template.iter().any(|component| {
            matches!(
                component,
                TemplateComponent::Title(title) if title.title == TitleType::Primary
            )
        })
    }

    fn template_has_publisher(template: &[TemplateComponent]) -> bool {
        template.iter().any(|component| {
            matches!(
                component,
                TemplateComponent::Variable(variable)
                    if variable.variable == SimpleVariable::Publisher
            )
        })
    }

    #[test]
    fn bibliography_default_excludes_type_conditioned_then_branch() {
        let nodes = vec![first_condition(
            vec![ItemType::Book],
            vec![],
            vec![variable_node(Variable::Title, 1)],
            Some(vec![variable_node(Variable::Publisher, 2)]),
        )];

        let compiler = TemplateCompiler;
        let default_template = compiler.compile_bibliography_default(&nodes);

        assert!(!template_has_primary_title(&default_template));
        assert!(template_has_publisher(&default_template));
    }

    #[test]
    fn bibliography_type_templates_preserve_matching_branch() {
        let nodes = vec![first_condition(
            vec![ItemType::Book],
            vec![],
            vec![variable_node(Variable::Title, 1)],
            Some(vec![variable_node(Variable::Publisher, 2)]),
        )];

        let compiler = TemplateCompiler;
        let (_, type_templates) = compiler.compile_bibliography_with_types(&nodes, false);
        let book_selector = TypeSelector::Single("book".to_string());
        assert!(type_templates.contains_key(&book_selector));
        if let Some(book_template) = type_templates.get(&book_selector) {
            assert!(template_has_primary_title(book_template));
            assert!(!template_has_publisher(book_template));
        }
    }

    #[test]
    fn bibliography_type_templates_use_untyped_else_if_as_fallback() {
        let nodes = vec![condition_with_else_if(
            vec![ItemType::LegalCase],
            vec![variable_node(Variable::Authority, 1)],
            ElseIfBranch {
                if_item_type: vec![],
                if_variables: vec![Variable::Title],
                children: vec![variable_node(Variable::Title, 2)],
            },
        )];

        let compiler = TemplateCompiler;
        let book_template =
            compiler.compile_for_type_with_untyped_else_if_fallback(&nodes, &ItemType::Book);

        assert!(template_has_primary_title(&book_template));
    }

    #[test]
    fn bibliography_type_templates_prefer_typed_else_if_over_untyped_if_fallback() {
        let nodes = vec![condition_with_else_if(
            vec![],
            vec![variable_node(Variable::Publisher, 1)],
            ElseIfBranch {
                if_item_type: vec![ItemType::Book],
                if_variables: vec![],
                children: vec![variable_node(Variable::Title, 2)],
            },
        )];

        let compiler = TemplateCompiler;
        let book_template =
            compiler.compile_for_type_with_untyped_else_if_fallback(&nodes, &ItemType::Book);

        assert!(template_has_primary_title(&book_template));
        assert!(!template_has_publisher(&book_template));
    }

    #[test]
    fn bibliography_default_keeps_variable_only_conditions() {
        let nodes = vec![first_condition(
            vec![],
            vec![Variable::Title],
            vec![variable_node(Variable::Title, 1)],
            Some(vec![variable_node(Variable::Publisher, 2)]),
        )];

        let compiler = TemplateCompiler;
        let default_template = compiler.compile_bibliography_default(&nodes);

        assert!(template_has_primary_title(&default_template));
        assert!(template_has_publisher(&default_template));
    }

    #[test]
    fn bibliography_default_excludes_type_and_variable_conditioned_branch() {
        let nodes = vec![first_condition(
            vec![ItemType::Book],
            vec![Variable::Title],
            vec![variable_node(Variable::Title, 1)],
            Some(vec![variable_node(Variable::Publisher, 2)]),
        )];

        let compiler = TemplateCompiler;
        let default_template = compiler.compile_bibliography_default(&nodes);

        assert!(!template_has_primary_title(&default_template));
        assert!(template_has_publisher(&default_template));
    }

    #[test]
    fn bibliography_compilation_removes_exact_duplicate_components() {
        let nodes = vec![
            variable_node(Variable::Publisher, 1),
            variable_node(Variable::Publisher, 2),
        ];

        let compiler = TemplateCompiler;
        let (default_template, _) = compiler.compile_bibliography_with_types(&nodes, false);
        let publisher_count = default_template
            .iter()
            .filter(|component| {
                matches!(
                    component,
                    TemplateComponent::Variable(variable)
                        if variable.variable == SimpleVariable::Publisher
                )
            })
            .count();

        assert_eq!(publisher_count, 1);
    }
}
