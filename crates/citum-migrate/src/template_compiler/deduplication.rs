use super::{CslnNode, Rendering, TemplateCompiler, TemplateComponent, TemplateGroup};

impl TemplateCompiler {
    pub(super) fn has_variable_recursive(
        &self,
        items: &[TemplateComponent],
        target: &TemplateComponent,
    ) -> bool {
        for item in items {
            if self.same_variable(item, target) {
                return true;
            }
            if let TemplateComponent::Group(list) = item
                && self.has_variable_recursive(&list.group, target)
            {
                return true;
            }
        }
        false
    }

    /// Simplified compile that only takes `then_branch` (for citations).
    /// This avoids pulling in type-specific variations from else branches.
    pub(super) fn compile_simple(&self, nodes: &[CslnNode]) -> Vec<TemplateComponent> {
        use citum_schema::ItemType;
        let mut components = Vec::new();

        for node in nodes {
            if let Some(component) = self.compile_node(node) {
                components.push(component);
            } else {
                match node {
                    CslnNode::Group(g) => {
                        components.extend(self.compile_simple(&g.children));
                    }
                    CslnNode::Condition(c) => {
                        // For citations, prefer else_branch for uncommon type conditions
                        let uncommon_types = [
                            ItemType::PersonalCommunication,
                            ItemType::Interview,
                            ItemType::LegalCase,
                            ItemType::Legislation,
                            ItemType::Bill,
                            ItemType::Treaty,
                        ];
                        let is_uncommon_type = !c.if_item_type.is_empty()
                            && c.if_item_type.iter().any(|t| uncommon_types.contains(t));

                        if is_uncommon_type {
                            // Prefer else_branch for common/default case
                            // Check else_if_branches first for common types
                            let mut found = false;
                            for else_if in &c.else_if_branches {
                                let has_common_types = else_if.if_item_type.is_empty()
                                    || else_if
                                        .if_item_type
                                        .iter()
                                        .any(|t| !uncommon_types.contains(t));
                                if has_common_types {
                                    components.extend(self.compile_simple(&else_if.children));
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                if let Some(ref else_nodes) = c.else_branch {
                                    components.extend(self.compile_simple(else_nodes));
                                } else {
                                    components.extend(self.compile_simple(&c.then_branch));
                                }
                            }
                        } else {
                            // Take then_branch, but fall back to else_if/else_branch if empty
                            let then_components = self.compile_simple(&c.then_branch);
                            if then_components.is_empty() {
                                // Try else_if branches first
                                let mut found = false;
                                for else_if in &c.else_if_branches {
                                    let branch_components = self.compile_simple(&else_if.children);
                                    if !branch_components.is_empty() {
                                        components.extend(branch_components);
                                        found = true;
                                        break;
                                    }
                                }
                                if !found && let Some(ref else_nodes) = c.else_branch {
                                    components.extend(self.compile_simple(else_nodes));
                                }
                            } else {
                                components.extend(then_components);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        components
    }

    /// Fix duplicate variables that appear both in Lists and as standalone components.
    ///
    /// When a variable (like date:issued) appears:
    /// 1. Inside a List for specific types (e.g., article-journal)
    /// 2. As a standalone component for all types
    ///
    /// Both will render for those types, causing duplication. This method
    /// suppresses standalone components when they duplicate List contents.
    /// Type-specific behavior is handled by `type-variants` at the spec level.
    pub(super) fn fix_duplicate_variables(&self, components: &mut [TemplateComponent]) {
        // Collect variables that appear in default-visible Lists
        let mut default_list_vars: Vec<String> = Vec::new();

        for component in components.iter() {
            if let TemplateComponent::Group(list) = component {
                let base_rendering = self.get_component_rendering(component);
                let is_default_visible = base_rendering.suppress != Some(true);

                if is_default_visible {
                    let vars = self.extract_list_vars(list);
                    for var in vars {
                        if !default_list_vars.contains(&var) {
                            default_list_vars.push(var);
                        }
                    }
                }
            }
        }

        // Suppress standalone components that duplicate List contents
        for component in components.iter_mut() {
            if matches!(component, TemplateComponent::Group(_)) {
                continue;
            }

            if let Some(var_key) = self.get_variable_key(component)
                && default_list_vars.contains(&var_key)
            {
                let mut rendering = self.get_component_rendering(component);
                rendering.suppress = Some(true);
                self.set_component_rendering(component, rendering);
            }
        }
    }

    /// Old deduplication method - no longer needed with occurrence-based compilation.
    /// Kept for reference but not used in new code path.
    #[allow(dead_code, reason = "helper functions")]
    pub(super) fn deduplicate_and_flatten(
        &self,
        components: Vec<TemplateComponent>,
    ) -> Vec<TemplateComponent> {
        let mut seen_vars: Vec<String> = Vec::new();
        let mut seen_list_signatures: Vec<String> = Vec::new();
        let mut result: Vec<TemplateComponent> = Vec::new();

        // First pass: add all non-List components and track their keys
        // When encountering duplicates, keep first occurrence
        for component in &components {
            if !matches!(component, TemplateComponent::Group(_)) {
                if let Some(key) = self.get_variable_key(component) {
                    if seen_vars.contains(&key) {
                        continue; // Skip duplicate
                    }
                    seen_vars.push(key);
                }
                result.push(component.clone());
            }
        }

        // Second pass: process Lists with recursive cleaning
        for component in components {
            if let TemplateComponent::Group(list) = component {
                // Recursively clean the list
                if let Some(cleaned) = self.clean_list_recursive(&list, &seen_vars) {
                    // Check if it's a List or was unwrapped
                    if let TemplateComponent::Group(cleaned_list) = &cleaned {
                        // Create signature for duplicate detection
                        let list_vars = self.extract_list_vars(cleaned_list);
                        let mut signature_parts = list_vars.clone();
                        signature_parts.sort();
                        let signature = signature_parts.join("|");

                        // Skip duplicate lists
                        if seen_list_signatures.contains(&signature) {
                            continue;
                        }
                        seen_list_signatures.push(signature);

                        // Track variables in this list
                        for var in list_vars {
                            if !seen_vars.contains(&var) {
                                seen_vars.push(var);
                            }
                        }
                    } else if let Some(key) = self.get_variable_key(&cleaned) {
                        // If it was unwrapped to a single component, check if already seen
                        if seen_vars.contains(&key) {
                            continue;
                        }
                        seen_vars.push(key);
                    }

                    result.push(cleaned);
                }
            }
        }

        result
    }

    #[allow(dead_code, reason = "helper functions")]
    pub(super) fn clean_list_recursive(
        &self,
        list: &TemplateGroup,
        seen_vars: &[String],
    ) -> Option<TemplateComponent> {
        let mut cleaned_items: Vec<TemplateComponent> = Vec::new();

        for item in &list.group {
            if let TemplateComponent::Group(nested) = item {
                // Recursively clean nested lists
                if let Some(cleaned) = self.clean_list_recursive(nested, seen_vars) {
                    cleaned_items.push(cleaned);
                }
            } else if let Some(key) = self.get_variable_key(item) {
                // Only keep if not already seen
                if !seen_vars.contains(&key) {
                    cleaned_items.push(item.clone());
                }
            } else {
                // Keep other items (shouldn't happen often)
                cleaned_items.push(item.clone());
            }
        }

        // Skip empty lists
        if cleaned_items.is_empty() {
            return None;
        }

        // If only one item remains and no special rendering, unwrap it
        if cleaned_items.len() == 1
            && list.delimiter.is_none()
            && list.rendering == Rendering::default()
        {
            return Some(cleaned_items.remove(0));
        }

        Some(TemplateComponent::Group(TemplateGroup {
            group: cleaned_items,
            delimiter: list.delimiter.clone(),
            rendering: list.rendering.clone(),
            ..Default::default()
        }))
    }

    /// Extract all variable keys from a List (recursively).
    pub(super) fn extract_list_vars(&self, list: &TemplateGroup) -> Vec<String> {
        let mut vars = Vec::new();
        for item in &list.group {
            if let Some(key) = self.get_variable_key(item) {
                vars.push(key);
            } else if let TemplateComponent::Group(nested) = item {
                vars.extend(self.extract_list_vars(nested));
            }
        }
        vars
    }

    /// Get a unique key for a component for deduplication purposes.
    pub(super) fn get_variable_key(&self, component: &TemplateComponent) -> Option<String> {
        match component {
            TemplateComponent::Contributor(c) => Some(format!("contributor:{:?}", c.contributor)),
            TemplateComponent::Date(d) => Some(format!("date:{:?}", d.date)),
            TemplateComponent::Title(t) => Some(format!("title:{:?}", t.title)),
            TemplateComponent::Number(n) => Some(format!("number:{:?}", n.number)),
            TemplateComponent::Variable(v) => Some(format!("variable:{:?}", v.variable)),
            TemplateComponent::Term(t) => Some(format!("term:{:?}", t.term)),
            // Lists don't have a single key - they contain multiple variables
            TemplateComponent::Group(_) => None,
            _ => None,
        }
    }

    /// Check if two components refer to the same variable.
    pub(super) fn same_variable(&self, a: &TemplateComponent, b: &TemplateComponent) -> bool {
        match (a, b) {
            (TemplateComponent::Contributor(c1), TemplateComponent::Contributor(c2)) => {
                c1.contributor == c2.contributor
            }
            (TemplateComponent::Date(d1), TemplateComponent::Date(d2)) => d1.date == d2.date,
            (TemplateComponent::Title(t1), TemplateComponent::Title(t2)) => t1.title == t2.title,
            (TemplateComponent::Number(n1), TemplateComponent::Number(n2)) => {
                n1.number == n2.number
            }
            (TemplateComponent::Variable(v1), TemplateComponent::Variable(v2)) => {
                v1.variable == v2.variable
            }
            (TemplateComponent::Term(t1), TemplateComponent::Term(t2)) => t1.term == t2.term,
            _ => false,
        }
    }
}
