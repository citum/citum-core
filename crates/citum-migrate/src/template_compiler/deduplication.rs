use super::*;

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
            if let TemplateComponent::List(list) = item
                && self.has_variable_recursive(&list.items, target)
            {
                return true;
            }
        }
        false
    }

    /// Add type-specific overrides to all items within a List.
    /// This ensures that when a List is created inside a type-specific branch,
    /// all its items get the appropriate suppress=true with type-specific unsuppress.
    #[allow(dead_code)]
    pub(super) fn add_type_overrides_to_list_items(
        &self,
        items: &mut [TemplateComponent],
        current_types: &[ItemType],
    ) {
        for item in items.iter_mut() {
            match item {
                TemplateComponent::List(nested_list) => {
                    // Recursively process nested lists
                    self.add_type_overrides_to_list_items(&mut nested_list.items, current_types);
                }
                _ => {
                    // Add suppress=true to base, with type-specific unsuppress overrides
                    let mut base = self.get_component_rendering(item);
                    base.suppress = Some(true);
                    self.set_component_rendering(item, base.clone());

                    for item_type in current_types {
                        let type_str = self.item_type_to_string(item_type);
                        let mut unsuppressed = base.clone();
                        unsuppressed.suppress = Some(false);
                        self.add_override_to_component(item, type_str, unsuppressed);
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(super) fn add_overrides_recursive(
        &self,
        component: &mut TemplateComponent,
        new_comp: &TemplateComponent,
        current_types: &[ItemType],
    ) {
        if self.same_variable(component, new_comp) {
            if current_types.is_empty() {
                // Empty types means this is the default case - unsuppress the component
                let mut rendering = self.get_component_rendering(component);
                if rendering.suppress == Some(true) {
                    rendering.suppress = Some(false);
                    self.set_component_rendering(component, rendering);
                }
            } else {
                // Add type-specific overrides
                self.add_overrides_to_existing(component, new_comp, current_types);
            }
            return;
        }
        if let TemplateComponent::List(list) = component {
            for item in &mut list.items {
                self.add_overrides_recursive(item, new_comp, current_types);
            }
        }
    }

    /// Get a debug name for a component
    #[allow(dead_code)]
    pub(super) fn get_component_name(&self, comp: &TemplateComponent) -> String {
        match comp {
            TemplateComponent::Contributor(c) => format!("contributor:{:?}", c.contributor),
            TemplateComponent::Date(d) => format!("date:{:?}", d.date),
            TemplateComponent::Title(t) => format!("title:{:?}", t.title),
            TemplateComponent::Number(n) => format!("number:{:?}", n.number),
            TemplateComponent::Variable(v) => format!("variable:{:?}", v.variable),
            TemplateComponent::List(_) => "List".to_string(),
            _ => "unknown".to_string(),
        }
    }

    #[allow(dead_code)]
    pub(super) fn add_overrides_to_existing(
        &self,
        existing: &mut TemplateComponent,
        new_comp: &TemplateComponent,
        current_types: &[ItemType],
    ) {
        let base_rendering = self.get_component_rendering(new_comp);
        let new_overrides = self.get_component_overrides(new_comp);

        use citum_schema::template::ComponentOverride;

        for item_type in current_types {
            let type_str = self.item_type_to_string(item_type);
            use citum_schema::template::TypeSelector;
            let mut override_val = new_overrides
                .as_ref()
                .and_then(|ovr| ovr.get(&TypeSelector::Single(type_str.clone())))
                .cloned()
                .unwrap_or_else(|| ComponentOverride::Rendering(base_rendering.clone()));

            if let ComponentOverride::Rendering(ref mut rendering) = override_val {
                if rendering.suppress.is_none() || rendering.suppress == Some(true) {
                    rendering.suppress = Some(false);
                }
                self.add_override_to_component(existing, type_str, rendering.clone());
            }
        }
    }

    /// Simplified compile that only takes then_branch (for citations).
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
                            if !then_components.is_empty() {
                                components.extend(then_components);
                            } else {
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
    /// Both will render for those types, causing duplication. This method adds
    /// suppress overrides to standalone components for types where the variable
    /// already appears in a List.
    pub(super) fn fix_duplicate_variables(&self, components: &mut [TemplateComponent]) {
        // Step 1: Collect which variables appear in Lists, and for which types
        let mut list_vars: HashMap<String, Vec<String>> = HashMap::new();
        let mut default_list_vars: Vec<String> = Vec::new();

        for component in components.iter() {
            if let TemplateComponent::List(list) = component {
                // Check if this List is visible by default
                let base_rendering = self.get_component_rendering(component);
                let is_default_visible = base_rendering.suppress != Some(true);

                // Extract all variables from this List
                let vars = self.extract_list_vars(list);

                if is_default_visible {
                    for var in &vars {
                        if !default_list_vars.contains(var) {
                            default_list_vars.push(var.clone());
                        }
                    }
                } else {
                    let visible_types = self.get_visible_types_for_component(component);
                    for var in vars {
                        list_vars
                            .entry(var)
                            .or_default()
                            .extend(visible_types.clone());
                    }
                }
            }
        }

        // Step 2: For each standalone component, add suppress overrides for types
        // where it already appears in a List
        for component in components.iter_mut() {
            // Skip Lists - we only care about standalone components
            if matches!(component, TemplateComponent::List(_)) {
                continue;
            }

            // Get the variable key for this component
            if let Some(var_key) = self.get_variable_key(component) {
                // If it appears in a default-visible List, suppress it by default
                if default_list_vars.contains(&var_key) {
                    let mut rendering = self.get_component_rendering(component);
                    rendering.suppress = Some(true);
                    self.set_component_rendering(component, rendering);
                } else if let Some(types_in_lists) = list_vars.get(&var_key) {
                    // Add suppress overrides for those types
                    for type_str in types_in_lists {
                        let mut suppressed = self.get_component_rendering(component);
                        suppressed.suppress = Some(true);
                        self.add_override_to_component(component, type_str.clone(), suppressed);
                    }
                }
            }
        }
    }

    /// Get the list of types for which a component is visible.
    ///
    /// Returns type names where suppress=false (either by default or via overrides).
    pub(super) fn get_visible_types_for_component(
        &self,
        component: &TemplateComponent,
    ) -> Vec<String> {
        let base_rendering = self.get_component_rendering(component);
        let overrides = self.get_component_overrides(component);

        let mut visible_types = Vec::new();

        // If component has suppress=true by default, only count types with suppress=false overrides
        if base_rendering.suppress == Some(true)
            && let Some(ovr) = overrides
        {
            use citum_schema::template::{ComponentOverride, TypeSelector};
            for (selector, ov) in ovr {
                if let ComponentOverride::Rendering(rendering) = ov
                    && rendering.suppress != Some(true)
                {
                    match selector {
                        TypeSelector::Single(s) => visible_types.push(s),
                        TypeSelector::Multiple(types) => {
                            for t in types {
                                visible_types.push(t);
                            }
                        }
                    }
                }
            }
        }
        // If component is visible by default (suppress=false or None),
        // we would need to list all types except those with suppress=true overrides.
        // For now, we skip this case as it would require enumerating all possible types.

        visible_types
    }

    /// Old deduplication method - no longer needed with occurrence-based compilation.
    /// Kept for reference but not used in new code path.
    #[allow(dead_code)]
    pub(super) fn deduplicate_and_flatten(
        &self,
        components: Vec<TemplateComponent>,
    ) -> Vec<TemplateComponent> {
        let mut seen_vars: Vec<String> = Vec::new();
        let mut seen_list_signatures: Vec<String> = Vec::new();
        let mut result: Vec<TemplateComponent> = Vec::new();

        // First pass: add all non-List components and track their keys
        // When encountering duplicates, merge their overrides
        for component in &components {
            if !matches!(component, TemplateComponent::List(_)) {
                if let Some(key) = self.get_variable_key(component) {
                    if let Some(existing_idx) = seen_vars.iter().position(|k| k == &key) {
                        // Duplicate found - merge overrides into existing component
                        self.merge_overrides_into(&mut result[existing_idx], component);
                    } else {
                        seen_vars.push(key);
                        result.push(component.clone());
                    }
                } else {
                    result.push(component.clone());
                }
            }
        }

        // Second pass: process Lists with recursive cleaning
        for component in components {
            if let TemplateComponent::List(list) = component {
                // Recursively clean the list
                if let Some(cleaned) = self.clean_list_recursive(&list, &seen_vars) {
                    // Check if it's a List or was unwrapped
                    if let TemplateComponent::List(cleaned_list) = &cleaned {
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

    #[allow(dead_code)]
    pub(super) fn clean_list_recursive(
        &self,
        list: &TemplateList,
        seen_vars: &[String],
    ) -> Option<TemplateComponent> {
        let mut cleaned_items: Vec<TemplateComponent> = Vec::new();

        for item in &list.items {
            if let TemplateComponent::List(nested) = item {
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

        Some(TemplateComponent::List(TemplateList {
            items: cleaned_items,
            delimiter: list.delimiter.clone(),
            rendering: list.rendering.clone(),
            ..Default::default()
        }))
    }

    /// Extract all variable keys from a List (recursively).
    pub(super) fn extract_list_vars(&self, list: &TemplateList) -> Vec<String> {
        let mut vars = Vec::new();
        for item in &list.items {
            if let Some(key) = self.get_variable_key(item) {
                vars.push(key);
            } else if let TemplateComponent::List(nested) = item {
                vars.extend(self.extract_list_vars(nested));
            }
        }
        vars
    }

    #[allow(dead_code)]
    pub(super) fn merge_overrides_into(
        &self,
        target: &mut TemplateComponent,
        source: &TemplateComponent,
    ) {
        if let Some(source_overrides) = self.get_component_overrides(source) {
            let target_overrides = match target {
                TemplateComponent::Contributor(c) => &mut c.overrides,
                TemplateComponent::Date(d) => &mut d.overrides,
                TemplateComponent::Number(n) => &mut n.overrides,
                TemplateComponent::Title(t) => &mut t.overrides,
                TemplateComponent::Variable(v) => &mut v.overrides,
                TemplateComponent::List(l) => &mut l.overrides,
                _ => return,
            };

            let overrides_map = target_overrides.get_or_insert_with(std::collections::HashMap::new);
            for (k, v) in source_overrides {
                overrides_map.entry(k.clone()).or_insert(v);
            }
        }
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
            TemplateComponent::List(_) => None,
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
