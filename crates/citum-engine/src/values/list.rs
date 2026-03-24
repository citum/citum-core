//! Rendering logic for list components with configurable delimiters.
//!
//! This module handles rendering of lists of template items, with support for
//! different delimiters between items (commas, semicolons, etc.) and rendering modes.

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::template::{DelimiterPunctuation, TemplateGroup};

impl ComponentValues for TemplateGroup {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        let mut has_content = false;
        let fmt = F::default();

        // Collect values from all items, applying their rendering
        let values: Vec<F::Output> = self
            .group
            .iter()
            .filter_map(|item| {
                let v = item.values::<F>(reference, hints, options)?;
                if v.value.is_empty() {
                    return None;
                }

                // Track if we have any "meaningful" content (not just a term)
                if !is_term_based(item) {
                    has_content = true;
                }

                // Use the central rendering logic to apply global config, local settings, and overrides
                let proc_item = crate::render::ProcTemplateComponent {
                    template_component: item.clone(),
                    template_index: options.current_template_index,
                    value: v.value,
                    prefix: v.prefix,
                    suffix: v.suffix,
                    url: v.url,
                    ref_type: Some(reference.ref_type().clone()),
                    config: Some(options.config.clone()),
                    item_language: crate::values::effective_component_language(reference, item),
                    pre_formatted: v.pre_formatted,
                };

                let rendered = crate::render::render_component_with_format_and_renderer::<F>(
                    &proc_item,
                    &fmt,
                    options.show_semantics,
                );
                if rendered.is_empty() {
                    None
                } else {
                    Some(rendered)
                }
            })
            .collect();

        if values.is_empty() || !has_content {
            return None;
        }

        // Join with delimiter
        let delimiter = self
            .delimiter
            .as_ref()
            .unwrap_or(&DelimiterPunctuation::Comma)
            .to_string_with_space();

        Some(ProcValues {
            value: fmt.join(values, &delimiter),
            prefix: None,
            suffix: None,
            url: None,
            substituted_key: None,
            pre_formatted: true,
        })
    }
}

/// Check if a component is purely term-based or a list of such.
fn is_term_based(component: &citum_schema::template::TemplateComponent) -> bool {
    use citum_schema::template::TemplateComponent;
    match component {
        TemplateComponent::Term(_) => true,
        TemplateComponent::Group(l) => l.group.iter().all(is_term_based),
        _ => false,
    }
}
