/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use super::{
    BranchContext, ComponentOccurrence, IndexMap, Node, TemplateCompiler, TemplateComponent,
    TemplateGroup,
    formatting::{
        apply_wrap_to_component, convert_formatting, extract_source_order, get_component_rendering,
        infer_wrap_from_affixes, map_delimiter, set_component_rendering,
    },
};

impl TemplateCompiler {
    /// Attempt to merge a Text node with the next component.
    /// If next node compiles to a component, prepend text to its prefix.
    /// Apply inherited wrap to Date components if applicable.
    /// Returns (component, `source_order`).
    fn merge_text_lookahead(
        &self,
        text_value: &str,
        next_node: &Node,
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
    ) -> Option<(TemplateComponent, Option<usize>)> {
        let mut next_comp = self.compile_node(next_node)?;

        // Merge text into prefix
        let mut rendering = get_component_rendering(&next_comp);
        let mut new_prefix = text_value.to_string();
        if let Some(p) = rendering.prefix {
            new_prefix.push_str(&p);
        }
        rendering.prefix = Some(new_prefix);
        set_component_rendering(&mut next_comp, rendering);

        // Apply inherited wrap if applicable
        if inherited_wrap.0.is_some() && matches!(&next_comp, TemplateComponent::Date(_)) {
            apply_wrap_to_component(&mut next_comp, inherited_wrap);
        }

        let source_order = extract_source_order(next_node);
        Some((next_comp, source_order))
    }

    pub(super) fn collect_occurrences(
        &self,
        nodes: &[Node],
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
        context: &BranchContext,
        occurrences: &mut Vec<ComponentOccurrence>,
    ) {
        let mut i = 0;

        while i < nodes.len() {
            #[allow(clippy::indexing_slicing, reason = "i < nodes.len()")]
            let node = &nodes[i];

            // Lookahead merge for Text nodes
            if let Node::Text { value } = node
                && i + 1 < nodes.len()
                && {
                    #[allow(clippy::indexing_slicing, reason = "i + 1 < nodes.len()")]
                    let next = &nodes[i + 1];
                    let merged = self.merge_text_lookahead(value, next, inherited_wrap);
                    if let Some((component, source_order)) = merged {
                        occurrences.push(ComponentOccurrence {
                            component,
                            context: context.clone(),
                            source_order,
                        });
                        i += 2;
                        true
                    } else {
                        false
                    }
                }
            {
                continue;
            }

            if let Some(mut component) = self.compile_node(node) {
                // Apply inherited wrap to date components
                if inherited_wrap.0.is_some() && matches!(&component, TemplateComponent::Date(_)) {
                    apply_wrap_to_component(&mut component, inherited_wrap);
                }
                let source_order = extract_source_order(node);
                occurrences.push(ComponentOccurrence {
                    component,
                    context: context.clone(),
                    source_order,
                });
            } else {
                match node {
                    Node::Group(g) => {
                        self.collect_group_occurrences(g, inherited_wrap, context, occurrences);
                    }
                    Node::Condition(c) => {
                        self.collect_condition_occurrences(c, inherited_wrap, context, occurrences);
                    }
                    _ => {}
                }
            }
            i += 1;
        }
    }

    pub(super) fn collect_bibliography_default_occurrences(
        &self,
        nodes: &[Node],
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
        context: &BranchContext,
        occurrences: &mut Vec<ComponentOccurrence>,
    ) {
        let mut i = 0;

        while i < nodes.len() {
            #[allow(clippy::indexing_slicing, reason = "i < nodes.len()")]
            let node = &nodes[i];

            if let Node::Text { value } = node
                && i + 1 < nodes.len()
                && {
                    #[allow(clippy::indexing_slicing, reason = "i + 1 < nodes.len()")]
                    let next = &nodes[i + 1];
                    let merged = self.merge_text_lookahead(value, next, inherited_wrap);
                    if let Some((component, source_order)) = merged {
                        occurrences.push(ComponentOccurrence {
                            component,
                            context: context.clone(),
                            source_order,
                        });
                        i += 2;
                        true
                    } else {
                        false
                    }
                }
            {
                continue;
            }

            if let Some(mut component) = self.compile_node(node) {
                if inherited_wrap.0.is_some() && matches!(&component, TemplateComponent::Date(_)) {
                    apply_wrap_to_component(&mut component, inherited_wrap);
                }
                let source_order = extract_source_order(node);
                occurrences.push(ComponentOccurrence {
                    component,
                    context: context.clone(),
                    source_order,
                });
            } else {
                match node {
                    Node::Group(g) => {
                        self.collect_bibliography_default_group_occurrences(
                            g,
                            inherited_wrap,
                            context,
                            occurrences,
                        );
                    }
                    Node::Condition(c) => {
                        self.collect_bibliography_default_condition_occurrences(
                            c,
                            inherited_wrap,
                            occurrences,
                        );
                    }
                    _ => {}
                }
            }
            i += 1;
        }
    }

    fn collect_group_occurrences(
        &self,
        g: &crate::ir::GroupBlock,
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
        context: &BranchContext,
        occurrences: &mut Vec<ComponentOccurrence>,
    ) {
        // Check if this group has its own wrap
        let group_wrap = infer_wrap_from_affixes(&g.formatting.prefix, &g.formatting.suffix);
        let effective_wrap = if group_wrap.0.is_some() {
            group_wrap.clone()
        } else {
            inherited_wrap.clone()
        };

        // Collect group components into a temporary list
        let mut group_occurrences = Vec::new();
        self.collect_occurrences(
            &g.children,
            &effective_wrap,
            context,
            &mut group_occurrences,
        );

        // Extract components from occurrences for grouping logic
        let group_components: Vec<TemplateComponent> = group_occurrences
            .iter()
            .map(|o| o.component.clone())
            .collect();

        // Check if any compiled component is a Term (handles nested conditions and groups)
        let has_term_node = group_components
            .iter()
            .any(|c| matches!(c, TemplateComponent::Term(_)));

        // Decide if this should be a List
        let meaningful_delimiter = g
            .delimiter
            .as_ref()
            .is_some_and(|d| matches!(d.as_str(), "" | "none" | ": " | " " | ", "));
        let is_small_structural_group = group_components.len() >= 2 && group_components.len() <= 3;
        let should_be_list = meaningful_delimiter
            && is_small_structural_group
            && group_wrap.0.is_none()
            && !has_term_node;

        // Never flatten if group has wrap (Fix A) or contains Term nodes (Fix B)
        let must_preserve_as_group = group_wrap.0.is_some() || has_term_node;

        if (should_be_list || must_preserve_as_group) && !group_components.is_empty() {
            let list = TemplateComponent::Group(TemplateGroup {
                group: group_components,
                delimiter: map_delimiter(&g.delimiter),
                rendering: convert_formatting(&g.formatting),
                ..Default::default()
            });
            let source_order = g.source_order;
            occurrences.push(ComponentOccurrence {
                component: list,
                context: context.clone(),
                source_order,
            });
        } else {
            // Flatten - add all group occurrences directly
            occurrences.extend(group_occurrences);
        }
    }

    fn collect_bibliography_default_group_occurrences(
        &self,
        g: &crate::ir::GroupBlock,
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
        context: &BranchContext,
        occurrences: &mut Vec<ComponentOccurrence>,
    ) {
        let group_wrap = infer_wrap_from_affixes(&g.formatting.prefix, &g.formatting.suffix);
        let effective_wrap = if group_wrap.0.is_some() {
            group_wrap.clone()
        } else {
            inherited_wrap.clone()
        };

        let mut group_occurrences = Vec::new();
        self.collect_bibliography_default_occurrences(
            &g.children,
            &effective_wrap,
            context,
            &mut group_occurrences,
        );

        let group_components: Vec<TemplateComponent> = group_occurrences
            .iter()
            .map(|o| o.component.clone())
            .collect();
        let has_term_node = group_components
            .iter()
            .any(|c| matches!(c, TemplateComponent::Term(_)));

        let meaningful_delimiter = g
            .delimiter
            .as_ref()
            .is_some_and(|d| matches!(d.as_str(), "" | "none" | ": " | " " | ", "));
        let is_small_structural_group = group_components.len() >= 2 && group_components.len() <= 3;
        let should_be_list = meaningful_delimiter
            && is_small_structural_group
            && group_wrap.0.is_none()
            && !has_term_node;
        let must_preserve_as_group = group_wrap.0.is_some() || has_term_node;

        if (should_be_list || must_preserve_as_group) && !group_components.is_empty() {
            let list = TemplateComponent::Group(TemplateGroup {
                group: group_components,
                delimiter: map_delimiter(&g.delimiter),
                rendering: convert_formatting(&g.formatting),
                ..Default::default()
            });
            occurrences.push(ComponentOccurrence {
                component: list,
                context: context.clone(),
                source_order: g.source_order,
            });
        } else {
            occurrences.extend(group_occurrences);
        }
    }

    fn collect_condition_occurrences(
        &self,
        c: &crate::ir::ConditionBlock,
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
        inherited_context: &BranchContext,
        occurrences: &mut Vec<ComponentOccurrence>,
    ) {
        // A type-less branch inherits the surrounding context; it must never
        // narrow Conditional back to Default. A variable condition nested
        // inside a type condition (e.g. `if variable="URL"` inside
        // `if type="webpage"`) is still type-conditional output.
        let then_context = if c.if_item_type.is_empty() {
            inherited_context.clone()
        } else {
            BranchContext::Conditional
        };
        self.collect_occurrences(&c.then_branch, inherited_wrap, &then_context, occurrences);

        // ELSE_IF branches: type-specific if types specified
        for else_if in &c.else_if_branches {
            let else_if_context = if else_if.if_item_type.is_empty() {
                inherited_context.clone()
            } else {
                BranchContext::Conditional
            };
            self.collect_occurrences(
                &else_if.children,
                inherited_wrap,
                &else_if_context,
                occurrences,
            );
        }

        // ELSE branch: inherits the surrounding context
        if let Some(ref else_nodes) = c.else_branch {
            self.collect_occurrences(else_nodes, inherited_wrap, inherited_context, occurrences);
        }
    }

    fn collect_bibliography_default_condition_occurrences(
        &self,
        c: &crate::ir::ConditionBlock,
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
        occurrences: &mut Vec<ComponentOccurrence>,
    ) {
        if !condition_has_type_branch(c) {
            self.collect_condition_occurrences(
                c,
                inherited_wrap,
                &BranchContext::Default,
                occurrences,
            );
            return;
        }

        if c.if_item_type.is_empty() {
            self.collect_bibliography_default_occurrences(
                &c.then_branch,
                inherited_wrap,
                &BranchContext::Default,
                occurrences,
            );
        }

        for else_if in &c.else_if_branches {
            if else_if.if_item_type.is_empty() {
                self.collect_bibliography_default_occurrences(
                    &else_if.children,
                    inherited_wrap,
                    &BranchContext::Default,
                    occurrences,
                );
            }
        }

        if let Some(ref else_nodes) = c.else_branch {
            self.collect_bibliography_default_occurrences(
                else_nodes,
                inherited_wrap,
                &BranchContext::Default,
                occurrences,
            );
        }
    }

    /// Merge component occurrences with suppress semantics for the default template.
    ///
    /// Key logic:
    /// - If component appears in DEFAULT branch → base suppress: false (visible by default)
    /// - If component ONLY in type-specific branches → base suppress: true
    ///
    /// Type-specific behavior is handled entirely by `type-variants` at the spec
    /// level (via `compile_for_type`), not per-component overrides.
    #[allow(clippy::cognitive_complexity, reason = "complex merge logic")]
    pub(super) fn merge_occurrences(
        &self,
        occurrences: Vec<ComponentOccurrence>,
    ) -> Vec<TemplateComponent> {
        let mut result: Vec<(TemplateComponent, Option<usize>)> = Vec::new();

        // Group occurrences by variable key (including Lists)
        let mut grouped: IndexMap<String, Vec<ComponentOccurrence>> = IndexMap::new();
        let mut list_counter = 0;

        for occurrence in occurrences {
            let key = if let Some(var_key) = self.get_variable_key(&occurrence.component) {
                var_key
            } else if let TemplateComponent::Group(ref list) = occurrence.component {
                // Use consistent signature with deduplicate pass
                format!("list:{}", crate::passes::deduplicate::list_signature(list))
            } else {
                // Other non-variable components - give unique key
                list_counter += 1;
                format!("other:{list_counter}")
            };

            grouped.entry(key).or_default().push(occurrence);
        }

        // Merge each group
        for (_key, mut group) in grouped {
            if group.is_empty() {
                continue;
            }

            // Sort Default-context occurrences first, then by source_order, so the
            // base component takes its shape (form, labels, affixes) and position
            // from the default branch rather than from whichever type-specific
            // branch happens to appear earliest in the CSL source. Components
            // without source_order (usize::MAX) sort last; the stable sort
            // preserves existing order for ties.
            group.sort_by_key(|occ| {
                (
                    !matches!(occ.context, BranchContext::Default),
                    occ.source_order.unwrap_or(usize::MAX),
                )
            });

            // Check if any occurrence is in Default context
            let has_default = group
                .iter()
                .any(|occ| matches!(occ.context, BranchContext::Default));

            // Start with the first component as the base
            #[allow(clippy::indexing_slicing, reason = "group is not empty")]
            let mut merged = group[0].component.clone();

            if has_default {
                // Component appears in default branch → visible by default
                let mut base_rendering = get_component_rendering(&merged);
                base_rendering.suppress = Some(false);
                set_component_rendering(&mut merged, base_rendering);
            } else {
                // Component ONLY in type-specific branches → hidden by default
                // Type-variants entries (from compile_for_type) handle per-type visibility.
                let mut base_rendering = get_component_rendering(&merged);
                base_rendering.suppress = Some(true);
                set_component_rendering(&mut merged, base_rendering);
            }

            // Position the merged component by its default-branch occurrence when
            // one exists; conditional-only components fall back to their earliest
            // occurrence in any branch.
            let min_order = if has_default {
                group
                    .iter()
                    .filter(|occ| matches!(occ.context, BranchContext::Default))
                    .filter_map(|occ| occ.source_order)
                    .min()
            } else {
                group.iter().filter_map(|occ| occ.source_order).min()
            };
            result.push((merged, min_order));
        }

        if super::migrate_debug_enabled() {
            tracing::debug!("=== Component source orders before sorting ===");
            for (comp, order) in &result {
                let comp_type = match comp {
                    TemplateComponent::Contributor(c) => {
                        format!("Contributor({:?})", c.contributor)
                    }
                    TemplateComponent::Date(d) => format!("Date({:?})", d.date),
                    TemplateComponent::Title(t) => format!("Title({:?})", t.title),
                    TemplateComponent::Number(n) => format!("Number({:?})", n.number),
                    TemplateComponent::Variable(v) => format!("Variable({:?})", v.variable),
                    TemplateComponent::Group(_) => "Group".to_string(),
                    _ => "Other".to_string(),
                };
                tracing::debug!("  {comp_type} -> order: {order:?}");
            }
        }

        // Sort result by source_order to preserve macro call order
        result.sort_by_key(|(_, order)| order.unwrap_or(usize::MAX));

        if super::migrate_debug_enabled() {
            tracing::debug!("=== After sorting ===");
            for (comp, order) in &result {
                let comp_type = match comp {
                    TemplateComponent::Contributor(c) => {
                        format!("Contributor({:?})", c.contributor)
                    }
                    TemplateComponent::Date(d) => format!("Date({:?})", d.date),
                    TemplateComponent::Title(t) => format!("Title({:?})", t.title),
                    _ => "...".to_string(),
                };
                tracing::debug!("  {comp_type} -> order: {order:?}");
            }
        }

        // Extract just the components (drop the ordering metadata)
        result.into_iter().map(|(comp, _)| comp).collect()
    }
}

fn condition_has_type_branch(c: &crate::ir::ConditionBlock) -> bool {
    !c.if_item_type.is_empty()
        || c.else_if_branches
            .iter()
            .any(|else_if| !else_if.if_item_type.is_empty())
}
