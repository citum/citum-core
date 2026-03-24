use super::{CslnNode, ItemType, TemplateCompiler, TemplateComponent};
use citum_schema::template::{NumberVariable, SimpleVariable};

impl TemplateCompiler {
    #[must_use]
    pub fn compile_bibliography(
        &self,
        nodes: &[CslnNode],
        _is_numeric: bool,
    ) -> Vec<TemplateComponent> {
        // DISABLED: Sorting was needed to work around HashMap's random iteration order.
        // Now that we use IndexMap, we preserve the CSL 1.0 layout order naturally.
        self.compile(nodes)
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
        // Compile using the new occurrence-based approach
        // This handles suppress semantics correctly without needing deduplication
        let mut default_template = self.compile(nodes);

        // DISABLED: Hardcoded sorting doesn't work for all styles (e.g., numeric styles have different order).
        // A general solution requires preserving macro call order from CSL 1.0 during parsing.
        // self.sort_bibliography_components(&mut default_template, _is_numeric);

        // Deduplicate number components (edition, volume, issue) in nested lists
        crate::passes::deduplicate::deduplicate_numbers_in_lists(&mut default_template);

        // Deduplicate date components (issued, accessed) in nested lists
        crate::passes::deduplicate::deduplicate_dates_in_lists(&mut default_template);

        // Fix duplicate variables (e.g., date appearing both in List and standalone)
        self.fix_duplicate_variables(&mut default_template);

        // Generate selective type templates for high-impact outlier types where
        // branch-specific structure is often materially different from the
        // default template (and where suppress-only overrides are insufficient).
        //
        // These templates are intentionally scoped to limit migration noise.
        let type_templates: indexmap::IndexMap<
            citum_schema::template::TypeSelector,
            Vec<TemplateComponent>,
        > = self.generate_selective_type_templates(nodes, &default_template);

        (default_template, type_templates)
    }

    pub(super) fn generate_selective_type_templates(
        &self,
        nodes: &[CslnNode],
        default_template: &[TemplateComponent],
    ) -> indexmap::IndexMap<citum_schema::template::TypeSelector, Vec<TemplateComponent>> {
        use citum_schema::template::TypeSelector;

        let mut candidates = self.collect_types_with_branches(nodes);
        if !candidates.contains(&ItemType::EntryEncyclopedia) {
            candidates.push(ItemType::EntryEncyclopedia);
        }

        candidates.sort_by_key(|t| self.item_type_to_string(t));
        candidates.dedup_by_key(|t| self.item_type_to_string(t));

        let mut type_templates = indexmap::IndexMap::new();
        for item_type in candidates {
            let mut type_template = self.compile_for_type(nodes, &item_type);
            if type_template.is_empty() {
                continue;
            }

            crate::passes::deduplicate::deduplicate_numbers_in_lists(&mut type_template);
            crate::passes::deduplicate::deduplicate_dates_in_lists(&mut type_template);
            self.fix_duplicate_variables(&mut type_template);

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
