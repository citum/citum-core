use super::*;

impl TemplateCompiler {
    fn add_type_overrides_to_list_items(
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
    fn add_overrides_recursive(
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
    fn get_component_name(&self, comp: &TemplateComponent) -> String {
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
    fn add_overrides_to_existing(
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
}
