/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use crate::ir::{FormattingOptions, Node};
use citum_schema::template::{DelimiterPunctuation, Rendering, TemplateComponent};
use std::sync::OnceLock;

pub(super) fn map_label_form(form: &crate::ir::LabelForm) -> citum_schema::template::LabelForm {
    match form {
        crate::ir::LabelForm::Long => citum_schema::template::LabelForm::Long,
        crate::ir::LabelForm::Short => citum_schema::template::LabelForm::Short,
        crate::ir::LabelForm::Symbol => citum_schema::template::LabelForm::Symbol,
        // Verb and VerbShort don't exist in template::LabelForm, map to Long
        crate::ir::LabelForm::Verb | crate::ir::LabelForm::VerbShort => {
            citum_schema::template::LabelForm::Long
        }
    }
}

/// Convert `FormattingOptions` to Rendering.
pub(super) fn convert_formatting(fmt: &FormattingOptions) -> Rendering {
    use citum_schema::template::{WrapConfig, WrapPunctuation};

    // Infer wrap from prefix/suffix patterns
    let (mut wrap_punct, remaining_prefix, remaining_suffix) =
        infer_wrap_from_affixes(&fmt.prefix, &fmt.suffix);

    // quotes="true" in CSL maps to wrap: quotes in Citum. This is the
    // single owner of quote rendering; the `quote` field below is left
    // unset so the engine does not apply quotation marks twice.
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
            .map(|s| matches!(s, crate::ir::FontStyle::Italic)),
        strong: fmt
            .font_weight
            .as_ref()
            .map(|w| matches!(w, crate::ir::FontWeight::Bold)),
        small_caps: fmt
            .font_variant
            .as_ref()
            .map(|v| matches!(v, crate::ir::FontVariant::SmallCaps)),
        vertical_align: fmt.vertical_align.clone(),
        // Quote rendering is owned by the `wrap: quotes` path above; emitting
        // it here as well caused doubled quotation marks (`““Title””`).
        quote: None,
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
        _ => {} // List, Term, and future variants - don't modify
    }
}

/// Map a String delimiter to `DelimiterPunctuation`.
/// Preserves custom delimiters that don't match standard patterns.
pub(super) fn map_delimiter(delimiter: &Option<String>) -> Option<DelimiterPunctuation> {
    delimiter
        .as_deref()
        .map(DelimiterPunctuation::from_csl_string)
}

/// Get the rendering options from a component.
pub(super) fn get_component_rendering(component: &TemplateComponent) -> Rendering {
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
pub(super) fn set_component_rendering(component: &mut TemplateComponent, rendering: Rendering) {
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

/// Extracts the `source_order` from a `Node`, if present.
/// Returns the order value or `usize::MAX` if not set (sorts last).
pub(super) fn extract_source_order(node: &Node) -> Option<usize> {
    let order = match node {
        Node::Variable(v) => v.source_order,
        Node::Date(d) => d.source_order,
        Node::Names(n) => n.source_order,
        Node::Group(g) => g.source_order,
        Node::Term(t) => t.source_order,
        _ => None,
    };
    if migrate_debug_enabled() {
        tracing::debug!(
            "TemplateCompiler: extract_source_order({:?}) = {:?}",
            match node {
                Node::Variable(v) => format!("Variable({:?})", v.variable),
                Node::Date(d) => format!("Date({:?})", d.variable),
                Node::Names(n) => format!("Names({:?})", n.variables),
                Node::Group(_) => "Group".to_string(),
                Node::Text { value } => format!("Text({value})"),
                Node::Condition(_) => "Condition".to_string(),
                Node::Term(t) => format!("Term({:?})", t.term),
            },
            order
        );
    }
    order
}

fn migrate_debug_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("CITUM_MIGRATE_DEBUG")
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;
    use citum_schema::locale::GeneralTerm;
    use citum_schema::template::WrapPunctuation;

    /// Regression test for Fix A: infer_wrap_from_affixes correctly identifies parentheses wrap.
    #[test]
    fn test_infer_wrap_from_affixes_parentheses() {
        let (wrap_punct, remaining_prefix, remaining_suffix) =
            infer_wrap_from_affixes(&Some("(".to_string()), &Some(")".to_string()));

        assert_eq!(wrap_punct, Some(WrapPunctuation::Parentheses));
        assert_eq!(remaining_prefix, None);
        assert_eq!(remaining_suffix, None);
    }

    /// Regression test for Fix A: infer_wrap_from_affixes with extra affixes.
    #[test]
    fn test_infer_wrap_with_inner_affixes() {
        let (wrap_punct, remaining_prefix, remaining_suffix) =
            infer_wrap_from_affixes(&Some("text (".to_string()), &Some(") more".to_string()));

        assert_eq!(wrap_punct, Some(WrapPunctuation::Parentheses));
        assert_eq!(remaining_prefix, Some("text ".to_string()));
        assert_eq!(remaining_suffix, Some(" more".to_string()));
    }

    /// Regression test for Fix B: apply_wrap_to_component only applies to specific variants.
    /// Terms should NOT receive wrap (they inherit it via group containment instead).
    #[test]
    fn test_apply_wrap_does_not_modify_term() {
        let group_wrap = (Some(WrapPunctuation::Parentheses), None, None);

        // Create a Term component
        let mut term = TemplateComponent::Term(citum_schema::template::TemplateTerm {
            term: GeneralTerm::In,
            form: None,
            gender: None,
            rendering: Rendering::default(),
            custom: None,
        });

        // Apply wrap to the term
        apply_wrap_to_component(&mut term, &group_wrap);

        // Verify that Term's wrap remains None (not modified)
        if let TemplateComponent::Term(t) = term {
            assert_eq!(
                t.rendering.wrap, None,
                "Term should not receive wrap from apply_wrap_to_component"
            );
        } else {
            panic!("Expected TemplateComponent::Term");
        }
    }

    /// Regression test (csl26-c2um): a CSL `quotes="true"` node maps to a single
    /// quote layer (`wrap: quotes`) and must not also set the `quote` field, which
    /// would make the engine emit doubled quotation marks.
    #[test]
    fn given_quotes_true_when_convert_formatting_then_only_wrap_owns_quotes() {
        let fmt = crate::ir::FormattingOptions {
            quotes: Some(true),
            ..Default::default()
        };

        let rendering = convert_formatting(&fmt);

        assert_eq!(
            rendering.wrap.map(|w| w.punctuation),
            Some(WrapPunctuation::Quotes),
            "quotes=true should produce a quote wrap"
        );
        assert_eq!(
            rendering.quote, None,
            "the redundant quote field must stay unset so quotes are not doubled"
        );
    }
}
