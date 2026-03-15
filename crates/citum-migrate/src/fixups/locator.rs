use citum_schema::template::{
    DelimiterPunctuation, Rendering, SimpleVariable, TemplateComponent, TemplateList,
    TemplateVariable, WrapPunctuation,
};
use csl_legacy::model::{CslNode, Layout, Macro};
use std::collections::HashSet;

pub(super) fn ensure_numeric_locator_citation_component(
    layout: &Layout,
    template: &mut [TemplateComponent],
) {
    if !layout_uses_citation_locator(layout) || citation_template_has_locator(template) {
        return;
    }

    let locator_component = TemplateComponent::Variable(TemplateVariable {
        variable: SimpleVariable::Locator,
        show_label: Some(true),
        rendering: Rendering {
            prefix: Some(", ".to_string()),
            ..Default::default()
        },
        ..Default::default()
    });

    if let Some(idx) = template.iter().position(component_has_citation_number) {
        match &mut template[idx] {
            TemplateComponent::List(list) => {
                list.items.push(locator_component);
                if list.delimiter.is_none() {
                    list.delimiter = Some(DelimiterPunctuation::None);
                }
            }
            _ => {
                let original = template[idx].clone();
                template[idx] = TemplateComponent::List(TemplateList {
                    items: vec![original, locator_component],
                    delimiter: Some(DelimiterPunctuation::None),
                    ..Default::default()
                });
            }
        }
    }
}

pub(super) fn normalize_wrapped_numeric_locator_citation_component(
    layout: &Layout,
    template: &mut [TemplateComponent],
    citation_delimiter: &mut Option<String>,
) {
    let Some((locator_wrap, no_inner_delimiter, strip_label_periods)) =
        find_wrapped_locator_group_format(&layout.children)
    else {
        return;
    };

    if !citation_template_has_citation_number(template) || !citation_template_has_locator(template)
    {
        return;
    }

    if apply_wrapped_locator_formatting(template, &locator_wrap, strip_label_periods)
        && no_inner_delimiter
    {
        *citation_delimiter = Some(String::new());
    }
}

fn find_wrapped_locator_group_format(nodes: &[CslNode]) -> Option<(WrapPunctuation, bool, bool)> {
    for node in nodes {
        match node {
            CslNode::Group(group) => {
                let wrap = match (group.prefix.as_deref(), group.suffix.as_deref()) {
                    (Some("("), Some(")")) => Some(WrapPunctuation::Parentheses),
                    (Some("["), Some("]")) => Some(WrapPunctuation::Brackets),
                    _ => None,
                };
                if let Some(wrap) = wrap
                    && nodes_use_citation_locator(&group.children)
                {
                    let strip_label_periods =
                        nodes_have_locator_label_with_stripped_periods(&group.children);
                    return Some((wrap, group.delimiter.is_none(), strip_label_periods));
                }

                if let Some(found) = find_wrapped_locator_group_format(&group.children) {
                    return Some(found);
                }
            }
            CslNode::Choose(choose) => {
                if let Some(found) = find_wrapped_locator_group_format(&choose.if_branch.children) {
                    return Some(found);
                }
                for branch in &choose.else_if_branches {
                    if let Some(found) = find_wrapped_locator_group_format(&branch.children) {
                        return Some(found);
                    }
                }
                if let Some(else_branch) = choose.else_branch.as_ref()
                    && let Some(found) = find_wrapped_locator_group_format(else_branch)
                {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

fn nodes_have_locator_label_with_stripped_periods(nodes: &[CslNode]) -> bool {
    nodes
        .iter()
        .any(node_has_locator_label_with_stripped_periods)
}

fn node_has_locator_label_with_stripped_periods(node: &CslNode) -> bool {
    match node {
        CslNode::Label(label) => {
            label.variable.as_deref() == Some("locator") && label.strip_periods == Some(true)
        }
        CslNode::Group(group) => nodes_have_locator_label_with_stripped_periods(&group.children),
        CslNode::Choose(choose) => {
            nodes_have_locator_label_with_stripped_periods(&choose.if_branch.children)
                || choose
                    .else_if_branches
                    .iter()
                    .any(|branch| nodes_have_locator_label_with_stripped_periods(&branch.children))
                || choose.else_branch.as_ref().is_some_and(|children| {
                    nodes_have_locator_label_with_stripped_periods(children)
                })
        }
        _ => false,
    }
}

fn apply_wrapped_locator_formatting(
    template: &mut [TemplateComponent],
    wrap: &WrapPunctuation,
    strip_label_periods: bool,
) -> bool {
    let mut changed = false;
    for component in template {
        match component {
            TemplateComponent::Variable(variable)
                if variable.variable == SimpleVariable::Locator =>
            {
                if variable.show_label != Some(true) {
                    variable.show_label = Some(true);
                    changed = true;
                }
                if strip_label_periods && variable.strip_label_periods != Some(true) {
                    variable.strip_label_periods = Some(true);
                    changed = true;
                }
                if variable.rendering.wrap.as_ref() != Some(wrap) {
                    variable.rendering.wrap = Some(wrap.clone());
                    changed = true;
                }
                if variable.rendering.prefix.is_some() {
                    variable.rendering.prefix = None;
                    changed = true;
                }
                if variable.rendering.suffix.is_some() {
                    variable.rendering.suffix = None;
                    changed = true;
                }
            }
            TemplateComponent::List(list) => {
                if apply_wrapped_locator_formatting(&mut list.items, wrap, strip_label_periods) {
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub(super) fn normalize_author_date_locator_citation_component(
    layout: &Layout,
    macros: &[Macro],
    template: &mut Vec<TemplateComponent>,
) {
    if !layout_uses_citation_locator(layout) {
        return;
    }

    let locator_prefix = infer_locator_group_delimiter(layout)
        .or_else(|| {
            let mut visited = HashSet::new();
            infer_locator_prefix_from_nodes(&layout.children, macros, &mut visited)
        })
        .unwrap_or(" ".to_string());

    if apply_author_date_locator_formatting(template, &locator_prefix) {
        return;
    }

    template.push(TemplateComponent::Variable(TemplateVariable {
        variable: SimpleVariable::Locator,
        show_label: Some(true),
        rendering: Rendering {
            prefix: Some(locator_prefix),
            ..Default::default()
        },
        ..Default::default()
    }));
}

fn infer_locator_group_delimiter(layout: &Layout) -> Option<String> {
    if let Some(delimiter) = layout.delimiter.as_ref()
        && layout
            .children
            .iter()
            .position(node_uses_citation_locator)
            .is_some_and(|index| index > 0)
        && !delimiter.is_empty()
    {
        return Some(delimiter.clone());
    }

    infer_locator_group_delimiter_from_nodes(&layout.children)
}

fn infer_locator_group_delimiter_from_nodes(nodes: &[CslNode]) -> Option<String> {
    for node in nodes {
        match node {
            CslNode::Group(group) => {
                if let Some(delimiter) = group.delimiter.as_ref()
                    && group
                        .children
                        .iter()
                        .position(node_uses_citation_locator)
                        .is_some_and(|index| index > 0)
                    && !delimiter.is_empty()
                {
                    return Some(delimiter.clone());
                }

                if let Some(delimiter) = infer_locator_group_delimiter_from_nodes(&group.children) {
                    return Some(delimiter);
                }
            }
            CslNode::Choose(choose) => {
                if let Some(delimiter) =
                    infer_locator_group_delimiter_from_nodes(&choose.if_branch.children)
                {
                    return Some(delimiter);
                }
                for branch in &choose.else_if_branches {
                    if let Some(delimiter) =
                        infer_locator_group_delimiter_from_nodes(&branch.children)
                    {
                        return Some(delimiter);
                    }
                }
                if let Some(else_branch) = choose.else_branch.as_ref()
                    && let Some(delimiter) = infer_locator_group_delimiter_from_nodes(else_branch)
                {
                    return Some(delimiter);
                }
            }
            _ => {}
        }
    }
    None
}

fn apply_author_date_locator_formatting(
    template: &mut [TemplateComponent],
    locator_prefix: &str,
) -> bool {
    let mut found_locator = false;
    for component in template {
        match component {
            TemplateComponent::Variable(variable)
                if variable.variable == SimpleVariable::Locator =>
            {
                found_locator = true;
                if variable.show_label != Some(true) {
                    variable.show_label = Some(true);
                }
                if should_replace_author_date_locator_prefix(
                    variable.rendering.prefix.as_deref(),
                    locator_prefix,
                ) {
                    variable.rendering.prefix = Some(locator_prefix.to_string());
                }
            }
            TemplateComponent::List(list) => {
                if apply_author_date_locator_formatting(&mut list.items, locator_prefix) {
                    found_locator = true;
                }
            }
            _ => {}
        }
    }
    found_locator
}

fn should_replace_author_date_locator_prefix(
    existing_prefix: Option<&str>,
    preferred_prefix: &str,
) -> bool {
    match existing_prefix {
        None => true,
        Some("") => true,
        Some(prefix) if prefix == preferred_prefix => false,
        Some(prefix) => prefix.trim().is_empty() && preferred_prefix != prefix,
    }
}

fn infer_locator_prefix_from_nodes(
    nodes: &[CslNode],
    macros: &[Macro],
    visited_macros: &mut HashSet<String>,
) -> Option<String> {
    for node in nodes {
        match node {
            CslNode::Text(t) => {
                let is_locator = t.variable.as_deref() == Some("locator")
                    || t.macro_name
                        .as_deref()
                        .is_some_and(macro_name_indicates_locator);
                if !is_locator {
                    continue;
                }

                if let Some(prefix) = t.prefix.as_ref()
                    && !prefix.is_empty()
                {
                    return Some(prefix.clone());
                }

                if let Some(macro_name) = t.macro_name.as_ref()
                    && visited_macros.insert(macro_name.clone())
                    && let Some(macro_def) = macros.iter().find(|m| m.name == *macro_name)
                    && let Some(prefix) =
                        infer_locator_prefix_from_nodes(&macro_def.children, macros, visited_macros)
                {
                    return Some(prefix);
                }
            }
            CslNode::Group(g) => {
                if let Some(prefix) =
                    infer_locator_prefix_from_nodes(&g.children, macros, visited_macros)
                {
                    return Some(prefix);
                }
            }
            CslNode::Choose(c) => {
                if let Some(prefix) =
                    infer_locator_prefix_from_nodes(&c.if_branch.children, macros, visited_macros)
                {
                    return Some(prefix);
                }
                for branch in &c.else_if_branches {
                    if let Some(prefix) =
                        infer_locator_prefix_from_nodes(&branch.children, macros, visited_macros)
                    {
                        return Some(prefix);
                    }
                }
                if let Some(else_branch) = c.else_branch.as_ref()
                    && let Some(prefix) =
                        infer_locator_prefix_from_nodes(else_branch, macros, visited_macros)
                {
                    return Some(prefix);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn move_group_wrap_to_citation_items(
    layout: &Layout,
    template: &mut [TemplateComponent],
    citation_wrap: &mut Option<WrapPunctuation>,
) {
    let Some(wrap) = citation_wrap.clone() else {
        return;
    };

    if !layout_has_group_wrap_for_citation_number(layout, &wrap) {
        return;
    }

    for component in template.iter_mut() {
        if component_has_citation_number(component) {
            apply_wrap_to_component(component, wrap.clone());
        }
    }
    *citation_wrap = None;
}

fn apply_wrap_to_component(component: &mut TemplateComponent, wrap: WrapPunctuation) {
    match component {
        TemplateComponent::Number(n) => {
            if n.rendering.wrap.is_none() {
                n.rendering.wrap = Some(wrap);
            }
        }
        TemplateComponent::List(list) => {
            if list.rendering.wrap.is_none() {
                list.rendering.wrap = Some(wrap);
            }
        }
        _ => {}
    }
}

fn citation_template_has_locator(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_locator)
}

fn component_has_locator(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Variable(v) => v.variable == SimpleVariable::Locator,
        TemplateComponent::List(list) => list.items.iter().any(component_has_locator),
        _ => false,
    }
}

fn layout_uses_citation_locator(layout: &Layout) -> bool {
    nodes_use_citation_locator(&layout.children)
}

fn nodes_use_citation_locator(nodes: &[CslNode]) -> bool {
    nodes.iter().any(node_uses_citation_locator)
}

fn node_uses_citation_locator(node: &CslNode) -> bool {
    match node {
        CslNode::Text(t) => {
            t.variable.as_deref() == Some("locator")
                || t.macro_name
                    .as_deref()
                    .is_some_and(macro_name_indicates_locator)
        }
        CslNode::Group(g) => nodes_use_citation_locator(&g.children),
        CslNode::Choose(c) => {
            nodes_use_citation_locator(&c.if_branch.children)
                || c.else_if_branches
                    .iter()
                    .any(|b| nodes_use_citation_locator(&b.children))
                || c.else_branch
                    .as_ref()
                    .is_some_and(|children| nodes_use_citation_locator(children))
        }
        _ => false,
    }
}

fn macro_name_indicates_locator(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered.contains("citation-locator") || lowered.contains("locator")
}

fn layout_has_group_wrap_for_citation_number(layout: &Layout, wrap: &WrapPunctuation) -> bool {
    let (prefix, suffix) = match wrap {
        WrapPunctuation::Brackets => ("[", "]"),
        WrapPunctuation::Parentheses => ("(", ")"),
        _ => return false,
    };
    nodes_have_wrapped_citation_number_group(&layout.children, prefix, suffix)
}

fn nodes_have_wrapped_citation_number_group(nodes: &[CslNode], prefix: &str, suffix: &str) -> bool {
    nodes
        .iter()
        .any(|node| node_has_wrapped_citation_number_group(node, prefix, suffix))
}

fn node_has_wrapped_citation_number_group(node: &CslNode, prefix: &str, suffix: &str) -> bool {
    match node {
        CslNode::Group(g) => {
            if g.prefix.as_deref() == Some(prefix)
                && g.suffix.as_deref() == Some(suffix)
                && nodes_contain_citation_number(&g.children)
            {
                return true;
            }
            nodes_have_wrapped_citation_number_group(&g.children, prefix, suffix)
        }
        CslNode::Choose(c) => {
            nodes_have_wrapped_citation_number_group(&c.if_branch.children, prefix, suffix)
                || c.else_if_branches
                    .iter()
                    .any(|b| nodes_have_wrapped_citation_number_group(&b.children, prefix, suffix))
                || c.else_branch.as_ref().is_some_and(|children| {
                    nodes_have_wrapped_citation_number_group(children, prefix, suffix)
                })
        }
        _ => false,
    }
}

fn nodes_contain_citation_number(nodes: &[CslNode]) -> bool {
    nodes.iter().any(node_contains_citation_number)
}

fn node_contains_citation_number(node: &CslNode) -> bool {
    match node {
        CslNode::Text(t) => t.variable.as_deref() == Some("citation-number"),
        CslNode::Number(n) => n.variable == "citation-number",
        CslNode::Group(g) => nodes_contain_citation_number(&g.children),
        CslNode::Choose(c) => {
            nodes_contain_citation_number(&c.if_branch.children)
                || c.else_if_branches
                    .iter()
                    .any(|b| nodes_contain_citation_number(&b.children))
                || c.else_branch
                    .as_ref()
                    .is_some_and(|children| nodes_contain_citation_number(children))
        }
        _ => false,
    }
}

pub(super) fn citation_template_has_citation_number(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_citation_number)
}

fn component_has_citation_number(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Number(n) => {
            n.number == citum_schema::template::NumberVariable::CitationNumber
        }
        TemplateComponent::List(list) => list.items.iter().any(component_has_citation_number),
        _ => false,
    }
}
