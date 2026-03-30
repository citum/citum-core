use super::{
    CslnNode, DelimiterPunctuation, FormattingOptions, Rendering, TemplateCompiler,
    TemplateComponent,
};

impl TemplateCompiler {
    pub(super) fn map_label_form(
        &self,
        form: &citum_schema::LabelForm,
    ) -> citum_schema::template::LabelForm {
        match form {
            citum_schema::LabelForm::Long => citum_schema::template::LabelForm::Long,
            citum_schema::LabelForm::Short => citum_schema::template::LabelForm::Short,
            citum_schema::LabelForm::Symbol => citum_schema::template::LabelForm::Symbol,
            // Verb and VerbShort don't exist in template::LabelForm, map to Long
            citum_schema::LabelForm::Verb | citum_schema::LabelForm::VerbShort => {
                citum_schema::template::LabelForm::Long
            }
        }
    }

    /// Convert `FormattingOptions` to Rendering.
    pub(super) fn convert_formatting(&self, fmt: &FormattingOptions) -> Rendering {
        use citum_schema::template::{WrapConfig, WrapPunctuation};

        // Infer wrap from prefix/suffix patterns
        let (mut wrap_punct, remaining_prefix, remaining_suffix) =
            Self::infer_wrap_from_affixes(&fmt.prefix, &fmt.suffix);

        // quotes="true" in CSL maps to wrap: quotes in Citum
        if fmt.quotes == Some(true) {
            wrap_punct = Some(WrapPunctuation::Quotes);
        }

        // If wrap is detected, remaining affixes are INNER.
        // If no wrap, affixes are OUTER (default prefix/suffix).
        let (prefix, suffix, wrap) = if let Some(punct) = wrap_punct {
            (
                None,
                None,
                Some(WrapConfig {
                    punctuation: punct,
                    inner_prefix: remaining_prefix,
                    inner_suffix: remaining_suffix,
                }),
            )
        } else {
            (remaining_prefix, remaining_suffix, None)
        };

        Rendering {
            text_case: None,
            emph: fmt
                .font_style
                .as_ref()
                .map(|s| matches!(s, citum_schema::FontStyle::Italic)),
            strong: fmt
                .font_weight
                .as_ref()
                .map(|w| matches!(w, citum_schema::FontWeight::Bold)),
            small_caps: fmt
                .font_variant
                .as_ref()
                .map(|v| matches!(v, citum_schema::FontVariant::SmallCaps)),
            quote: fmt.quotes,
            prefix,
            suffix,
            wrap,
            suppress: None,
            initialize_with: None,
            name_form: None,
            strip_periods: fmt.strip_periods,
        }
    }

    /// Infer wrap type from prefix/suffix patterns.
    ///
    /// CSL 1.0 uses `prefix="("` and `suffix=")"` for parentheses wrapping.
    /// Citum prefers explicit `wrap: parentheses` for cleaner representation.
    ///
    /// Returns (wrap, `remaining_prefix`, `remaining_suffix`) where the wrap chars
    /// have been extracted and remaining affixes are returned.
    pub(super) fn infer_wrap_from_affixes(
        prefix: &Option<String>,
        suffix: &Option<String>,
    ) -> (
        Option<citum_schema::template::WrapPunctuation>,
        Option<String>,
        Option<String>,
    ) {
        use citum_schema::template::WrapPunctuation;

        match (prefix.as_deref(), suffix.as_deref()) {
            // Clean parentheses: prefix ends with "(", suffix starts with ")"
            (Some(p), Some(s)) if p.ends_with('(') && s.starts_with(')') => {
                let remaining_prefix = p
                    .strip_suffix('(')
                    .map(std::string::ToString::to_string)
                    .filter(|s| !s.is_empty());
                let remaining_suffix = s
                    .strip_prefix(')')
                    .map(std::string::ToString::to_string)
                    .filter(|s| !s.is_empty());
                (
                    Some(WrapPunctuation::Parentheses),
                    remaining_prefix,
                    remaining_suffix,
                )
            }
            // Clean brackets
            (Some(p), Some(s)) if p.ends_with('[') && s.starts_with(']') => {
                let remaining_prefix = p
                    .strip_suffix('[')
                    .map(std::string::ToString::to_string)
                    .filter(|s| !s.is_empty());
                let remaining_suffix = s
                    .strip_prefix(']')
                    .map(std::string::ToString::to_string)
                    .filter(|s| !s.is_empty());
                (
                    Some(WrapPunctuation::Brackets),
                    remaining_prefix,
                    remaining_suffix,
                )
            }
            // No wrap pattern found - keep original affixes
            _ => (None, prefix.clone(), suffix.clone()),
        }
    }

    /// Apply wrap formatting from a parent group to a component.
    ///
    /// When a group with `prefix="(" suffix=")"` wraps a date, the date
    /// should inherit the wrap property since groups are flattened.
    pub(super) fn apply_wrap_to_component(
        &self,
        component: &mut TemplateComponent,
        group_wrap: &(
            Option<citum_schema::template::WrapPunctuation>,
            Option<String>,
            Option<String>,
        ),
    ) {
        use citum_schema::template::WrapConfig;

        let (wrap_punct, prefix, suffix) = group_wrap;

        // Helper to apply rendering
        let apply = |rendering: &mut Rendering| {
            if rendering.wrap.is_none() && wrap_punct.is_some() {
                rendering.wrap = wrap_punct.clone().map(|punct| WrapConfig {
                    punctuation: punct,
                    inner_prefix: prefix.clone(),
                    inner_suffix: suffix.clone(),
                });
            }

            // If no wrap is being applied, affixes are outer.
            if wrap_punct.is_none() {
                if rendering.prefix.is_none() && prefix.is_some() {
                    rendering.prefix = prefix.clone();
                }
                if rendering.suffix.is_none() && suffix.is_some() {
                    rendering.suffix = suffix.clone();
                }
            }
        };

        match component {
            TemplateComponent::Date(d) => apply(&mut d.rendering),
            TemplateComponent::Contributor(c) => apply(&mut c.rendering),
            TemplateComponent::Title(t) => apply(&mut t.rendering),
            TemplateComponent::Number(n) => apply(&mut n.rendering),
            TemplateComponent::Variable(v) => apply(&mut v.rendering),
            _ => {} // List and future variants - don't modify
        }
    }
    /// Map a String delimiter to `DelimiterPunctuation`.
    /// Preserves custom delimiters that don't match standard patterns.
    pub(super) fn map_delimiter(&self, delimiter: &Option<String>) -> Option<DelimiterPunctuation> {
        delimiter
            .as_deref()
            .map(DelimiterPunctuation::from_csl_string)
    }

    /// Get the rendering options from a component.
    pub(super) fn get_component_rendering(&self, component: &TemplateComponent) -> Rendering {
        match component {
            TemplateComponent::Contributor(c) => c.rendering.clone(),
            TemplateComponent::Date(d) => d.rendering.clone(),
            TemplateComponent::Number(n) => n.rendering.clone(),
            TemplateComponent::Title(t) => t.rendering.clone(),
            TemplateComponent::Variable(v) => v.rendering.clone(),
            TemplateComponent::Group(l) => l.rendering.clone(),
            TemplateComponent::Term(t) => t.rendering.clone(),
            _ => Rendering::default(),
        }
    }

    /// Set the rendering options for a component.
    pub(super) fn set_component_rendering(
        &self,
        component: &mut TemplateComponent,
        rendering: Rendering,
    ) {
        match component {
            TemplateComponent::Contributor(c) => c.rendering = rendering,
            TemplateComponent::Date(d) => d.rendering = rendering,
            TemplateComponent::Number(n) => n.rendering = rendering,
            TemplateComponent::Title(t) => t.rendering = rendering,
            TemplateComponent::Variable(v) => v.rendering = rendering,
            TemplateComponent::Group(l) => l.rendering = rendering,
            TemplateComponent::Term(t) => t.rendering = rendering,
            _ => {}
        }
    }

    /// Extracts the `source_order` from a `CslnNode`, if present.
    /// Returns the order value or `usize::MAX` if not set (sorts last).
    pub(super) fn extract_source_order(&self, node: &CslnNode) -> Option<usize> {
        let order = match node {
            CslnNode::Variable(v) => v.source_order,
            CslnNode::Date(d) => d.source_order,
            CslnNode::Names(n) => n.source_order,
            CslnNode::Group(g) => g.source_order,
            CslnNode::Term(t) => t.source_order,
            _ => None,
        };
        if super::migrate_debug_enabled() {
            eprintln!(
                "TemplateCompiler: extract_source_order({:?}) = {:?}",
                match node {
                    CslnNode::Variable(v) => format!("Variable({:?})", v.variable),
                    CslnNode::Date(d) => format!("Date({:?})", d.variable),
                    CslnNode::Names(n) => format!("Names({:?})", n.variable),
                    CslnNode::Group(_) => "Group".to_string(),
                    CslnNode::Text { value } => format!("Text({value})"),
                    CslnNode::Condition(_) => "Condition".to_string(),
                    CslnNode::Term(t) => format!("Term({:?})", t.term),
                },
                order
            );
        }
        order
    }
}

#[cfg(test)]
mod tests {}
