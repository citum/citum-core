use super::*;

impl TemplateCompiler {
    /// Attempt to merge a Text node with the next component.
    /// If next node compiles to a component, prepend text to its prefix.
    /// Apply inherited wrap to Date components if applicable.
    /// Returns (component, source_order).
    fn merge_text_lookahead(
        &self,
        text_value: &str,
        next_node: &CslnNode,
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
    ) -> Option<(TemplateComponent, Option<usize>)> {
        let mut next_comp = self.compile_node(next_node)?;

        // Merge text into prefix
        let mut rendering = self.get_component_rendering(&next_comp);
        let mut new_prefix = text_value.to_string();
        if let Some(p) = rendering.prefix {
            new_prefix.push_str(&p);
        }
        rendering.prefix = Some(new_prefix);
        self.set_component_rendering(&mut next_comp, rendering);

        // Apply inherited wrap if applicable
        if inherited_wrap.0.is_some() && matches!(&next_comp, TemplateComponent::Date(_)) {
            self.apply_wrap_to_component(&mut next_comp, inherited_wrap);
        }

        let source_order = self.extract_source_order(next_node);
        Some((next_comp, source_order))
    }

    #[allow(clippy::too_many_lines)] // FIXME: csl26-44gu
    pub(super) fn collect_occurrences(
        &self,
        nodes: &[CslnNode],
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
            let node = &nodes[i];

            // Lookahead merge for Text nodes
            if let CslnNode::Text { value } = node
                && i + 1 < nodes.len()
                && let Some((component, source_order)) =
                    self.merge_text_lookahead(value, &nodes[i + 1], inherited_wrap)
            {
                occurrences.push(ComponentOccurrence {
                    component,
                    context: context.clone(),
                    source_order,
                });
                i += 2;
                continue;
            }

            if let Some(mut component) = self.compile_node(node) {
                // Apply inherited wrap to date components
                if inherited_wrap.0.is_some() && matches!(&component, TemplateComponent::Date(_)) {
                    self.apply_wrap_to_component(&mut component, inherited_wrap);
                }
                let source_order = self.extract_source_order(node);
                occurrences.push(ComponentOccurrence {
                    component,
                    context: context.clone(),
                    source_order,
                });
            } else {
                match node {
                    CslnNode::Group(g) => {
                        // Check if this group has its own wrap
                        let group_wrap = Self::infer_wrap_from_affixes(
                            &g.formatting.prefix,
                            &g.formatting.suffix,
                        );
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

                        // Decide if this should be a List
                        let meaningful_delimiter = g
                            .delimiter
                            .as_ref()
                            .is_some_and(|d| matches!(d.as_str(), "" | "none" | ": " | " " | ", "));
                        let is_small_structural_group =
                            group_components.len() >= 2 && group_components.len() <= 3;
                        let should_be_list = meaningful_delimiter
                            && is_small_structural_group
                            && group_wrap.0.is_none();

                        if should_be_list && !group_components.is_empty() {
                            let list = TemplateComponent::List(TemplateList {
                                items: group_components,
                                delimiter: self.map_delimiter(&g.delimiter),
                                rendering: self.convert_formatting(&g.formatting),
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
                    CslnNode::Condition(c) => {
                        // THEN branch: type-specific if types specified
                        let then_context = if c.if_item_type.is_empty() {
                            BranchContext::Default
                        } else {
                            BranchContext::TypeSpecific(c.if_item_type.clone())
                        };
                        self.collect_occurrences(
                            &c.then_branch,
                            inherited_wrap,
                            &then_context,
                            occurrences,
                        );

                        // ELSE_IF branches: each is type-specific
                        for else_if in &c.else_if_branches {
                            let else_if_context = if else_if.if_item_type.is_empty() {
                                BranchContext::Default
                            } else {
                                BranchContext::TypeSpecific(else_if.if_item_type.clone())
                            };
                            self.collect_occurrences(
                                &else_if.children,
                                inherited_wrap,
                                &else_if_context,
                                occurrences,
                            );
                        }

                        // ELSE branch: always default context
                        if let Some(ref else_nodes) = c.else_branch {
                            self.collect_occurrences(
                                else_nodes,
                                inherited_wrap,
                                &BranchContext::Default,
                                occurrences,
                            );
                        }
                    }
                    _ => {}
                }
            }
            i += 1;
        }
    }

    /// Merge component occurrences with smart suppress semantics.
    ///
    /// Key logic:
    /// - If component appears in DEFAULT branch → base suppress: false (visible by default)
    /// - If component ONLY in type-specific branches → base suppress: true + type overrides
    /// - Collect all type-specific occurrences as overrides with suppress: false
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
            } else if let TemplateComponent::List(ref list) = occurrence.component {
                // Use consistent signature with deduplicate pass
                format!("list:{}", crate::passes::deduplicate::list_signature(list))
            } else {
                // Other non-variable components - give unique key
                list_counter += 1;
                format!("other:{}", list_counter)
            };

            grouped.entry(key).or_default().push(occurrence);
        }

        // Merge each group
        for (_key, mut group) in grouped {
            if group.is_empty() {
                continue;
            }

            // Sort by source_order to preserve macro call order from CSL 1.0.
            // Components without source_order (usize::MAX) sort last.
            // Stable sort preserves existing order for components with same source_order.
            group.sort_by_key(|occ| occ.source_order.unwrap_or(usize::MAX));

            // Check if any occurrence is in Default context
            let has_default = group
                .iter()
                .any(|occ| matches!(occ.context, BranchContext::Default));

            // Start with the first component as the base
            let mut merged = group[0].component.clone();

            // For Lists, propagate type overrides to each item from all branches
            if let TemplateComponent::List(ref mut list) = merged {
                for occurrence in &group {
                    if let BranchContext::TypeSpecific(types) = &occurrence.context {
                        self.add_type_overrides_to_list_items(&mut list.items, types);
                    }
                }
            }

            if has_default {
                // Component appears in default branch → visible by default
                let mut base_rendering = self.get_component_rendering(&merged);
                base_rendering.suppress = Some(false);
                self.set_component_rendering(&mut merged, base_rendering);

                // Add type-specific overrides for any TypeSpecific contexts
                for occurrence in &group {
                    if let BranchContext::TypeSpecific(types) = &occurrence.context {
                        for item_type in types {
                            let type_str = self.item_type_to_string(item_type);
                            let mut rendering = self.get_component_rendering(&occurrence.component);
                            rendering.suppress = Some(false); // Explicitly visible for this type
                            self.add_override_to_component(&mut merged, type_str, rendering);
                        }
                    }
                }
            } else {
                // Component ONLY in type-specific branches → hidden by default
                let mut base_rendering = self.get_component_rendering(&merged);
                base_rendering.suppress = Some(true);
                self.set_component_rendering(&mut merged, base_rendering.clone());

                // Add overrides for each type-specific occurrence
                for occurrence in &group {
                    if let BranchContext::TypeSpecific(types) = &occurrence.context {
                        for item_type in types {
                            let type_str = self.item_type_to_string(item_type);
                            let mut rendering = self.get_component_rendering(&occurrence.component);
                            rendering.suppress = Some(false); // Show for this type
                            self.add_override_to_component(&mut merged, type_str, rendering);
                        }
                    }
                }
            }

            // Track minimum source_order for this merged component
            let min_order = group.iter().filter_map(|occ| occ.source_order).min();
            result.push((merged, min_order));
        }

        if super::migrate_debug_enabled() {
            eprintln!("=== Component source orders before sorting ===");
            for (comp, order) in &result {
                let comp_type = match comp {
                    TemplateComponent::Contributor(c) => {
                        format!("Contributor({:?})", c.contributor)
                    }
                    TemplateComponent::Date(d) => format!("Date({:?})", d.date),
                    TemplateComponent::Title(t) => format!("Title({:?})", t.title),
                    TemplateComponent::Number(n) => format!("Number({:?})", n.number),
                    TemplateComponent::Variable(v) => format!("Variable({:?})", v.variable),
                    TemplateComponent::List(_) => "List".to_string(),
                    _ => "Other".to_string(),
                };
                eprintln!("  {} -> order: {:?}", comp_type, order);
            }
        }

        // Sort result by source_order to preserve macro call order
        result.sort_by_key(|(_, order)| order.unwrap_or(usize::MAX));

        if super::migrate_debug_enabled() {
            eprintln!("=== After sorting ===");
            for (comp, order) in &result {
                let comp_type = match comp {
                    TemplateComponent::Contributor(c) => {
                        format!("Contributor({:?})", c.contributor)
                    }
                    TemplateComponent::Date(d) => format!("Date({:?})", d.date),
                    TemplateComponent::Title(t) => format!("Title({:?})", t.title),
                    _ => "...".to_string(),
                };
                eprintln!("  {} -> order: {:?}", comp_type, order);
            }
        }

        // Extract just the components (drop the ordering metadata)
        result.into_iter().map(|(comp, _)| comp).collect()
    }
}
