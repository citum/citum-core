use super::*;

impl TemplateCompiler {
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
                && let Some(mut next_comp) = self.compile_node(&nodes[i + 1])
            {
                // Merge text into prefix
                let mut rendering = self.get_component_rendering(&next_comp);
                let mut new_prefix = value.clone();
                if let Some(p) = rendering.prefix {
                    new_prefix.push_str(&p);
                }
                rendering.prefix = Some(new_prefix);
                self.set_component_rendering(&mut next_comp, rendering);

                // Apply inherited wrap if applicable
                if inherited_wrap.0.is_some() && matches!(&next_comp, TemplateComponent::Date(_)) {
                    self.apply_wrap_to_component(&mut next_comp, inherited_wrap);
                }

                // Extract source_order from the next node
                let source_order = self.extract_source_order(&nodes[i + 1]);
                occurrences.push(ComponentOccurrence {
                    component: next_comp,
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

    // Old compilation method kept for citation compilation (compile_simple)
    #[allow(dead_code)]
    pub(super) fn compile_with_wrap(
        &self,
        nodes: &[CslnNode],
        inherited_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
        current_types: &[ItemType],
    ) -> Vec<TemplateComponent> {
        let mut components = Vec::new();
        let mut i = 0;

        while i < nodes.len() {
            let node = &nodes[i];

            // Lookahead merge for Text nodes
            if let CslnNode::Text { value } = node
                && i + 1 < nodes.len()
            {
                // Try to compile next node
                if let Some(mut next_comp) = self.compile_node(&nodes[i + 1]) {
                    // Merge text into prefix
                    let mut rendering = self.get_component_rendering(&next_comp);
                    let mut new_prefix = value.clone();
                    if let Some(p) = rendering.prefix {
                        new_prefix.push_str(&p);
                    }
                    rendering.prefix = Some(new_prefix);
                    self.set_component_rendering(&mut next_comp, rendering);

                    // Apply inherited wrap if applicable
                    if inherited_wrap.0.is_some()
                        && matches!(&next_comp, TemplateComponent::Date(_))
                    {
                        self.apply_wrap_to_component(&mut next_comp, inherited_wrap);
                    }

                    self.add_or_upgrade_component(&mut components, next_comp, current_types);
                    i += 2;
                    continue;
                }
            }

            if let Some(mut component) = self.compile_node(node) {
                // Apply inherited wrap to date components
                if inherited_wrap.0.is_some() && matches!(&component, TemplateComponent::Date(_)) {
                    self.apply_wrap_to_component(&mut component, inherited_wrap);
                }
                // Add or replace with better-formatted version
                self.add_or_upgrade_component(&mut components, component, current_types);
            } else {
                match node {
                    CslnNode::Group(g) => {
                        // Check if this group has its own wrap
                        let group_wrap = Self::infer_wrap_from_affixes(
                            &g.formatting.prefix,
                            &g.formatting.suffix,
                        );
                        // Use group's wrap if it has one, otherwise inherit from parent
                        let effective_wrap = if group_wrap.0.is_some() {
                            group_wrap.clone()
                        } else {
                            inherited_wrap.clone()
                        };
                        let group_components =
                            self.compile_with_wrap(&g.children, &effective_wrap, current_types);

                        // Only create a List for meaningful structural groups:
                        // - Groups with explicit non-default delimiters (not period/comma)
                        // - AND containing 2-3 components that form a logical unit
                        // Most groups should just be flattened.
                        let meaningful_delimiter = g.delimiter.as_ref().is_some_and(|d| {
                            // Keep lists for special delimiters like none (volume+issue)
                            // or colon (title: subtitle)
                            matches!(d.as_str(), "" | "none" | ": " | " " | ", ")
                        });
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
                            self.add_or_upgrade_component(&mut components, list, current_types);
                        } else {
                            for gc in group_components {
                                self.add_or_upgrade_component(&mut components, gc, current_types);
                            }
                        }
                    }
                    CslnNode::Condition(c) => {
                        // Concatenate current types with if_item_type
                        let mut then_types = current_types.to_vec();
                        then_types.extend(c.if_item_type.clone());

                        // Pass wrap through conditions
                        let then_components =
                            self.compile_with_wrap(&c.then_branch, inherited_wrap, &then_types);
                        for tc in then_components {
                            self.add_or_upgrade_component(&mut components, tc, &then_types);
                        }

                        for else_if in &c.else_if_branches {
                            let mut else_if_types = current_types.to_vec();
                            else_if_types.extend(else_if.if_item_type.clone());

                            let branch_components = self.compile_with_wrap(
                                &else_if.children,
                                inherited_wrap,
                                &else_if_types,
                            );
                            for bc in branch_components {
                                self.add_or_upgrade_component(&mut components, bc, &else_if_types);
                            }
                        }

                        if let Some(ref else_nodes) = c.else_branch {
                            let else_components =
                                self.compile_with_wrap(else_nodes, inherited_wrap, current_types);
                            for ec in else_components {
                                self.add_or_upgrade_component(&mut components, ec, current_types);
                            }
                        }
                    }
                    _ => {}
                }
            }
            i += 1;
        }

        components
    }

    #[allow(dead_code)]
    pub(super) fn add_or_upgrade_component(
        &self,
        components: &mut Vec<TemplateComponent>,
        new_component: TemplateComponent,
        current_types: &[ItemType],
    ) {
        // Recursive search for existing variable
        let mut existing_idx = None;

        for (i, c) in components.iter_mut().enumerate() {
            if self.same_variable(c, &new_component) {
                existing_idx = Some(i);
                break;
            }
            // Also check inside Lists
            if let TemplateComponent::List(list) = c
                && self.has_variable_recursive(&list.items, &new_component)
            {
                // Variable exists but is nested. We can't easily merge top-level into nested
                // without knowing the structure. For now, mark as "found" so we can add overrides.
                // Actually, let's just use a recursive mutation helper.
                self.add_overrides_recursive(c, &new_component, current_types);
                return;
            }
        }

        if let Some(idx) = existing_idx {
            if current_types.is_empty() {
                // ... same global logic ...
                let mut rendering = self.get_component_rendering(&components[idx]);
                if rendering.suppress == Some(true) {
                    rendering.suppress = Some(false);
                    self.set_component_rendering(&mut components[idx], rendering);
                }

                if let (TemplateComponent::Date(existing), TemplateComponent::Date(new)) =
                    (&components[idx], &new_component)
                    && existing.rendering.wrap.is_none()
                    && new.rendering.wrap.is_some()
                {
                    components[idx] = new_component.clone();
                }
            } else {
                // Add overrides to existing top-level component
                self.add_overrides_to_existing(&mut components[idx], &new_component, current_types);
            }
        } else {
            // ... same NEW component logic ...
            let mut component_to_add = new_component;
            if !current_types.is_empty() {
                if let TemplateComponent::List(ref mut list) = component_to_add {
                    // For Lists, propagate type-specific overrides to each item
                    self.add_type_overrides_to_list_items(&mut list.items, current_types);
                } else {
                    let mut base = self.get_component_rendering(&component_to_add);
                    base.suppress = Some(true);
                    self.set_component_rendering(&mut component_to_add, base.clone());

                    for item_type in current_types {
                        let type_str = self.item_type_to_string(item_type);
                        let mut unsuppressed = base.clone();
                        unsuppressed.suppress = Some(false);
                        self.add_override_to_component(
                            &mut component_to_add,
                            type_str,
                            unsuppressed,
                        );
                    }
                }
            }
            components.push(component_to_add);
        }
    }
}
