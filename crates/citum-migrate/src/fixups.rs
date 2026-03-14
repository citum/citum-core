#![allow(missing_docs)]

use citum_schema::template::{
    DateVariable, DelimiterPunctuation, Rendering, SimpleVariable, TemplateComponent, TemplateList,
    TemplateVariable, TitleType, TypeSelector, WrapPunctuation,
};
use csl_legacy::model::{CslNode, Layout};
use std::collections::HashSet;

pub fn selector_matches_any(selector: &TypeSelector, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| selector.matches(candidate))
}

pub fn normalize_legal_case_type_template(
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
) {
    let Some(map) = type_templates.as_mut() else {
        return;
    };
    let style_id = legacy_style.info.id.to_lowercase();
    let style_is_elsevier_harvard = style_id.contains("elsevier-harvard");
    let style_is_springer_socpsych = style_id.contains("springer-socpsych-author-date");

    for (selector, template) in map.iter_mut() {
        if !selector.matches("legal_case") && !selector.matches("legal-case") {
            continue;
        }

        let mut seen_locator = false;
        let mut has_issued = false;
        let mut has_parent_serial = false;
        let mut has_reporter = false;
        let mut has_page = false;
        template.retain_mut(|component| {
            if let TemplateComponent::Term(term) = component
                && (matches!(term.term, citum_schema::locale::GeneralTerm::Circa)
                    || matches!(term.term, citum_schema::locale::GeneralTerm::At))
            {
                return false;
            }
            if let TemplateComponent::Term(term) = component
                && matches!(term.term, citum_schema::locale::GeneralTerm::NoDate)
            {
                return false;
            }

            if let TemplateComponent::Date(date_component) = component {
                if date_component.date == DateVariable::Issued {
                    has_issued = true;
                    date_component.rendering.suppress = Some(false);
                    if style_is_springer_socpsych {
                        date_component.form = citum_schema::template::DateForm::Full;
                    }
                } else {
                    return false;
                }
            }

            if let TemplateComponent::Title(title_component) = component
                && title_component.title == TitleType::ParentSerial
            {
                has_parent_serial = true;
            }

            if let TemplateComponent::Variable(variable) = component
                && (variable.variable == SimpleVariable::Locator
                    || variable.variable == SimpleVariable::Url)
            {
                if variable.variable == SimpleVariable::Locator {
                    if seen_locator {
                        return false;
                    }
                    seen_locator = true;
                } else {
                    return false;
                }
            }
            if let TemplateComponent::Variable(variable) = component {
                if variable.variable == SimpleVariable::Reporter {
                    has_reporter = true;
                }
                if variable.variable == SimpleVariable::Page {
                    has_page = true;
                }
                if style_is_elsevier_harvard && variable.variable == SimpleVariable::Authority {
                    return false;
                }
            }

            if let TemplateComponent::Term(term) = component
                && (matches!(term.term, citum_schema::locale::GeneralTerm::Section)
                    || matches!(term.term, citum_schema::locale::GeneralTerm::Accessed))
            {
                return false;
            }

            if let TemplateComponent::Number(number_component) = component
                && number_component.number == citum_schema::template::NumberVariable::Volume
            {
                number_component.rendering.suppress = Some(false);
            }

            true
        });

        if !has_issued {
            let mut date_component = citum_schema::template::TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            };
            if style_is_springer_socpsych {
                date_component.form = citum_schema::template::DateForm::Full;
            }
            template.push(TemplateComponent::Date(date_component));
        }
        if !has_parent_serial {
            template.push(TemplateComponent::Title(
                citum_schema::template::TemplateTitle {
                    title: TitleType::ParentSerial,
                    ..Default::default()
                },
            ));
        }
        if (style_is_elsevier_harvard || style_is_springer_socpsych) && !has_reporter {
            template.push(TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Reporter,
                ..Default::default()
            }));
        }
        if style_is_springer_socpsych && !has_page {
            template.push(TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Page,
                ..Default::default()
            }));
        }
    }
}

pub fn ensure_inferred_media_type_templates(
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
    bibliography_template: &[TemplateComponent],
) {
    let map = type_templates.get_or_insert_with(std::collections::HashMap::new);
    let enable_interview_detail =
        legacy_style_uses_contributor_variable(legacy_style, "interviewer");
    let enable_motion_picture_detail = legacy_style_mentions_motion_picture_term(legacy_style)
        || legacy_style_uses_contributor_variable(legacy_style, "director");

    if enable_motion_picture_detail
        && !map
            .keys()
            .any(|selector| selector.matches("motion_picture"))
    {
        let mut template = base_media_template_from_bibliography(bibliography_template);
        template.push(TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Genre,
            ..Default::default()
        }));
        template.push(TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Medium,
            ..Default::default()
        }));
        template.push(TemplateComponent::Contributor(
            citum_schema::template::TemplateContributor {
                contributor: citum_schema::template::ContributorRole::Director,
                form: citum_schema::template::ContributorForm::Long,
                rendering: Rendering {
                    prefix: Some("Directed by ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
        if template.len() >= 3 {
            map.insert(TypeSelector::Single("motion_picture".to_string()), template);
        }
    }

    if enable_interview_detail && !map.keys().any(|selector| selector.matches("interview")) {
        let mut template = base_media_template_from_bibliography(bibliography_template);
        template.push(TemplateComponent::Contributor(
            citum_schema::template::TemplateContributor {
                contributor: citum_schema::template::ContributorRole::Interviewer,
                form: citum_schema::template::ContributorForm::Long,
                ..Default::default()
            },
        ));
        template.push(TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Medium,
            ..Default::default()
        }));
        template.push(TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Url,
            ..Default::default()
        }));
        if template.len() >= 3 {
            map.insert(TypeSelector::Single("interview".to_string()), template);
        }
    }
}

pub fn base_media_template_from_bibliography(
    bibliography_template: &[TemplateComponent],
) -> Vec<TemplateComponent> {
    let mut template = Vec::new();
    if let Some(author_component) = bibliography_template.iter().find_map(|component| {
        if let TemplateComponent::Contributor(contributor) = component
            && contributor.contributor == citum_schema::template::ContributorRole::Author
        {
            return Some(component.clone());
        }
        None
    }) {
        template.push(author_component);
    }
    if let Some(issued_component) = bibliography_template.iter().find_map(|component| {
        if let TemplateComponent::Date(date_component) = component
            && date_component.date == DateVariable::Issued
        {
            return Some(component.clone());
        }
        None
    }) {
        template.push(issued_component);
    }
    if let Some(title_component) = bibliography_template.iter().find_map(|component| {
        if let TemplateComponent::Title(title_component) = component
            && title_component.title == TitleType::Primary
        {
            return Some(component.clone());
        }
        None
    }) {
        template.push(title_component);
    }
    template
}

pub fn ensure_personal_communication_omitted(
    legacy_style: &csl_legacy::model::Style,
    citation_template: &[TemplateComponent],
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
) {
    if !citation_template_suppresses_personal_communication(citation_template)
        && !legacy_style_omits_personal_communication_in_bibliography(legacy_style)
    {
        return;
    }
    if !legacy_style_mentions_personal_communication(legacy_style) {
        return;
    }
    let map = type_templates.get_or_insert_with(std::collections::HashMap::new);
    map.insert(
        TypeSelector::Single("personal_communication".to_string()),
        Vec::new(),
    );
    map.insert(
        TypeSelector::Single("personal-communication".to_string()),
        Vec::new(),
    );
}

pub fn citation_template_suppresses_personal_communication(template: &[TemplateComponent]) -> bool {
    template
        .iter()
        .any(component_suppresses_personal_communication)
}

pub fn component_suppresses_personal_communication(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Date(date_component) => {
            date_component.overrides.as_ref().is_some_and(|overrides| {
                overrides.iter().any(|(selector, override_component)| {
                    selector_matches_any(
                        selector,
                        &["personal_communication", "personal-communication"],
                    ) && matches!(
                        override_component,
                        citum_schema::template::ComponentOverride::Rendering(rendering)
                            if rendering.suppress == Some(true)
                    )
                })
            })
        }
        TemplateComponent::List(list) => list
            .items
            .iter()
            .any(component_suppresses_personal_communication),
        _ => false,
    }
}

pub fn legacy_style_mentions_personal_communication(style: &csl_legacy::model::Style) -> bool {
    fn node_mentions_personal_communication(node: &CslNode) -> bool {
        match node {
            CslNode::Choose(choose) => {
                let branch_mentions = |branch: &csl_legacy::model::ChooseBranch| {
                    branch.type_.as_ref().is_some_and(|types| {
                        types
                            .split_whitespace()
                            .any(|t| t == "personal_communication")
                    }) || branch
                        .children
                        .iter()
                        .any(node_mentions_personal_communication)
                };
                branch_mentions(&choose.if_branch)
                    || choose.else_if_branches.iter().any(branch_mentions)
                    || choose.else_branch.as_ref().is_some_and(|children| {
                        children.iter().any(node_mentions_personal_communication)
                    })
            }
            CslNode::Group(group) => group
                .children
                .iter()
                .any(node_mentions_personal_communication),
            _ => false,
        }
    }

    style.bibliography.as_ref().is_some_and(|bibliography| {
        bibliography
            .layout
            .children
            .iter()
            .any(node_mentions_personal_communication)
    })
}

pub fn legacy_style_omits_personal_communication_in_bibliography(
    style: &csl_legacy::model::Style,
) -> bool {
    fn node_has_omit_branch(node: &CslNode) -> bool {
        match node {
            CslNode::Choose(choose) => {
                let branch_is_omit = |branch: &csl_legacy::model::ChooseBranch| {
                    branch.type_.as_ref().is_some_and(|types| {
                        types
                            .split_whitespace()
                            .any(|t| t == "personal_communication")
                    }) && branch.children.is_empty()
                        && branch.variable.is_none()
                        && branch.is_numeric.is_none()
                        && branch.is_uncertain_date.is_none()
                        && branch.locator.is_none()
                        && branch.position.is_none()
                };

                branch_is_omit(&choose.if_branch)
                    || choose.else_if_branches.iter().any(branch_is_omit)
                    || choose.if_branch.children.iter().any(node_has_omit_branch)
                    || choose
                        .else_if_branches
                        .iter()
                        .any(|branch| branch.children.iter().any(node_has_omit_branch))
                    || choose
                        .else_branch
                        .as_ref()
                        .is_some_and(|children| children.iter().any(node_has_omit_branch))
            }
            CslNode::Group(group) => group.children.iter().any(node_has_omit_branch),
            _ => false,
        }
    }

    style.bibliography.as_ref().is_some_and(|bibliography| {
        bibliography
            .layout
            .children
            .iter()
            .any(node_has_omit_branch)
    })
}

pub fn legacy_style_uses_contributor_variable(
    style: &csl_legacy::model::Style,
    variable_name: &str,
) -> bool {
    fn node_uses_contributor_variable(node: &CslNode, variable_name: &str) -> bool {
        match node {
            CslNode::Names(names) => names
                .variable
                .split_whitespace()
                .any(|candidate| candidate == variable_name),
            CslNode::Group(group) => group
                .children
                .iter()
                .any(|child| node_uses_contributor_variable(child, variable_name)),
            CslNode::Choose(choose) => {
                choose
                    .if_branch
                    .children
                    .iter()
                    .any(|child| node_uses_contributor_variable(child, variable_name))
                    || choose.else_if_branches.iter().any(|branch| {
                        branch
                            .children
                            .iter()
                            .any(|child| node_uses_contributor_variable(child, variable_name))
                    })
                    || choose.else_branch.as_ref().is_some_and(|children| {
                        children
                            .iter()
                            .any(|child| node_uses_contributor_variable(child, variable_name))
                    })
            }
            _ => false,
        }
    }

    style.macros.iter().any(|macro_def| {
        macro_def
            .children
            .iter()
            .any(|node| node_uses_contributor_variable(node, variable_name))
    }) || style
        .citation
        .layout
        .children
        .iter()
        .any(|node| node_uses_contributor_variable(node, variable_name))
        || style.bibliography.as_ref().is_some_and(|bibliography| {
            bibliography
                .layout
                .children
                .iter()
                .any(|node| node_uses_contributor_variable(node, variable_name))
        })
}

pub fn legacy_style_mentions_motion_picture_term(style: &csl_legacy::model::Style) -> bool {
    fn node_mentions_motion_picture(node: &CslNode) -> bool {
        match node {
            CslNode::Text(text) => text
                .term
                .as_ref()
                .is_some_and(|term| term == "motion_picture"),
            CslNode::Group(group) => group.children.iter().any(node_mentions_motion_picture),
            CslNode::Choose(choose) => {
                choose
                    .if_branch
                    .children
                    .iter()
                    .any(node_mentions_motion_picture)
                    || choose
                        .else_if_branches
                        .iter()
                        .any(|branch| branch.children.iter().any(node_mentions_motion_picture))
                    || choose
                        .else_branch
                        .as_ref()
                        .is_some_and(|children| children.iter().any(node_mentions_motion_picture))
            }
            _ => false,
        }
    }

    style
        .macros
        .iter()
        .any(|macro_def| macro_def.children.iter().any(node_mentions_motion_picture))
        || style
            .citation
            .layout
            .children
            .iter()
            .any(node_mentions_motion_picture)
        || style.bibliography.as_ref().is_some_and(|bibliography| {
            bibliography
                .layout
                .children
                .iter()
                .any(node_mentions_motion_picture)
        })
}

pub fn ensure_inferred_patent_type_template(
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
    bibliography_template: &[TemplateComponent],
) {
    let map = type_templates.get_or_insert_with(std::collections::HashMap::new);
    if map.keys().any(|selector| selector.matches("patent")) {
        return;
    }

    let mut patent_template = base_media_template_from_bibliography(bibliography_template);

    let style_id = legacy_style.info.id.to_lowercase();
    let suppress_patent_number_for_style = style_id.contains("springer-socpsych-author-date");
    if !suppress_patent_number_for_style {
        patent_template.push(TemplateComponent::Number(
            citum_schema::template::TemplateNumber {
                number: citum_schema::template::NumberVariable::Number,
                ..Default::default()
            },
        ));
    }

    if patent_template.len() >= 2 {
        map.insert(TypeSelector::Single("patent".to_string()), patent_template);
    }
}

pub fn ensure_numeric_locator_citation_component(
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

pub fn normalize_wrapped_numeric_locator_citation_component(
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

pub fn find_wrapped_locator_group_format(
    nodes: &[CslNode],
) -> Option<(WrapPunctuation, bool, bool)> {
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

pub fn nodes_have_locator_label_with_stripped_periods(nodes: &[CslNode]) -> bool {
    nodes
        .iter()
        .any(node_has_locator_label_with_stripped_periods)
}

pub fn node_has_locator_label_with_stripped_periods(node: &CslNode) -> bool {
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

pub fn apply_wrapped_locator_formatting(
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

pub fn normalize_author_date_locator_citation_component(
    layout: &Layout,
    macros: &[csl_legacy::model::Macro],
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

pub fn infer_locator_group_delimiter(layout: &Layout) -> Option<String> {
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

pub fn infer_locator_group_delimiter_from_nodes(nodes: &[CslNode]) -> Option<String> {
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

pub fn apply_author_date_locator_formatting(
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

pub fn should_replace_author_date_locator_prefix(
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

pub fn infer_locator_prefix_from_nodes(
    nodes: &[CslNode],
    macros: &[csl_legacy::model::Macro],
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

pub fn move_group_wrap_to_citation_items(
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

pub fn apply_wrap_to_component(component: &mut TemplateComponent, wrap: WrapPunctuation) {
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

pub fn citation_template_has_locator(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_locator)
}

pub fn component_has_locator(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Variable(v) => v.variable == SimpleVariable::Locator,
        TemplateComponent::List(list) => list.items.iter().any(component_has_locator),
        _ => false,
    }
}

pub fn layout_uses_citation_locator(layout: &Layout) -> bool {
    nodes_use_citation_locator(&layout.children)
}

pub fn nodes_use_citation_locator(nodes: &[CslNode]) -> bool {
    nodes.iter().any(node_uses_citation_locator)
}

pub fn node_uses_citation_locator(node: &CslNode) -> bool {
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

pub fn macro_name_indicates_locator(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered.contains("citation-locator") || lowered.contains("locator")
}

pub fn layout_has_group_wrap_for_citation_number(layout: &Layout, wrap: &WrapPunctuation) -> bool {
    let (prefix, suffix) = match wrap {
        WrapPunctuation::Brackets => ("[", "]"),
        WrapPunctuation::Parentheses => ("(", ")"),
        _ => return false,
    };
    nodes_have_wrapped_citation_number_group(&layout.children, prefix, suffix)
}

pub fn nodes_have_wrapped_citation_number_group(
    nodes: &[CslNode],
    prefix: &str,
    suffix: &str,
) -> bool {
    nodes
        .iter()
        .any(|node| node_has_wrapped_citation_number_group(node, prefix, suffix))
}

pub fn node_has_wrapped_citation_number_group(node: &CslNode, prefix: &str, suffix: &str) -> bool {
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

pub fn nodes_contain_citation_number(nodes: &[CslNode]) -> bool {
    nodes.iter().any(node_contains_citation_number)
}

pub fn node_contains_citation_number(node: &CslNode) -> bool {
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

pub fn citation_template_has_citation_number(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_citation_number)
}

pub fn component_has_citation_number(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Number(n) => {
            n.number == citum_schema::template::NumberVariable::CitationNumber
        }
        TemplateComponent::List(list) => list.items.iter().any(component_has_citation_number),
        _ => false,
    }
}

pub fn note_citation_template_is_underfit(template: &[TemplateComponent]) -> bool {
    template.len() == 1 && component_is_contributor_only(&template[0])
}

pub fn component_is_contributor_only(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Contributor(_) => true,
        TemplateComponent::List(list) => list.items.iter().all(component_is_contributor_only),
        _ => false,
    }
}

pub fn citation_template_is_author_year_only(template: &[TemplateComponent]) -> bool {
    let mut has_contributor = false;
    let mut has_date = false;

    for component in template {
        match component {
            TemplateComponent::Contributor(_) => has_contributor = true,
            TemplateComponent::Date(_) => has_date = true,
            TemplateComponent::List(list) => {
                for item in &list.items {
                    match item {
                        TemplateComponent::Contributor(_) => has_contributor = true,
                        TemplateComponent::Date(_) => has_date = true,
                        _ => return false,
                    }
                }
            }
            _ => return false,
        }
    }

    has_contributor && has_date
}

pub fn normalize_contributor_form_to_short(template: &mut [TemplateComponent]) -> bool {
    let mut changed = false;
    for component in template {
        match component {
            TemplateComponent::Contributor(c) => {
                if c.form == citum_schema::template::ContributorForm::Long {
                    c.form = citum_schema::template::ContributorForm::Short;
                    changed = true;
                }
            }
            TemplateComponent::List(list) => {
                if normalize_contributor_form_to_short(&mut list.items) {
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub fn normalize_author_date_inferred_contributors(
    template: &mut [TemplateComponent],
    drop_component_shorten: bool,
) -> bool {
    let mut changed = false;
    for component in template {
        match component {
            TemplateComponent::Contributor(c) => {
                if c.form == citum_schema::template::ContributorForm::Long {
                    c.form = citum_schema::template::ContributorForm::Short;
                    changed = true;
                }
                if c.name_order == Some(citum_schema::template::NameOrder::GivenFirst) {
                    c.name_order = Some(citum_schema::template::NameOrder::FamilyFirst);
                    changed = true;
                }
                if drop_component_shorten && c.shorten.is_some() {
                    c.shorten = None;
                    changed = true;
                }
            }
            TemplateComponent::List(list) => {
                if normalize_author_date_inferred_contributors(
                    &mut list.items,
                    drop_component_shorten,
                ) {
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub fn should_merge_inferred_type_template(
    type_name: &str,
    inferred_template: &[TemplateComponent],
    candidate_template: &[TemplateComponent],
) -> bool {
    match type_name {
        // Patent branches can require structural divergence in numeric styles,
        // but keep only compact candidates to avoid overfitting from verbose
        // fallback templates that are better handled by the inferred default.
        "patent" => candidate_template.len() <= 12,
        // Only merge encyclopedia fallback templates when inferred output does
        // not already carry entry-encyclopedia overrides and the candidate is
        // compact (no parent title chain).
        "entry-encyclopedia" => {
            !template_targets_type(inferred_template, type_name)
                && !template_has_parent_title(candidate_template)
        }
        // Webpage templates are kept only when inferred output does not already
        // target webpages, and the candidate includes accessed-date structure.
        "webpage" => {
            (!template_targets_type(inferred_template, type_name)
                || !template_has_accessed_date(inferred_template))
                && template_has_accessed_date(candidate_template)
        }
        // Case-law citations are structurally distinct in many numeric styles
        // and often need dedicated suppression/order not recoverable from the
        // shared inferred template alone.
        "legal-case" | "legal_case" => !template_targets_type(inferred_template, type_name),
        // Personal communications often have highly specialized fields like recipient
        // and translator/interviewer notes that need dedicated rendering.
        "personal_communication" | "personal-communication" => {
            !template_targets_type(inferred_template, type_name)
        }
        // For common bibliography types, prefer XML type branches when they
        // carry clear structural differences from the inferred global template.
        // This recovers repeated title/container/publisher/volume gaps.
        "article-journal" | "article-magazine" | "article-newspaper" | "book" | "report"
        | "broadcast" | "interview" | "motion_picture" | "motion-picture" => {
            inferred_candidate_structurally_diverges(inferred_template, candidate_template)
        }
        _ => false,
    }
}

pub fn scrub_inferred_literal_artifacts(component: &mut TemplateComponent) {
    match component {
        TemplateComponent::Title(title) => {
            if title.title == TitleType::Primary
                && let Some(prefix) = title.rendering.prefix.as_ref()
                && let Some(cleaned) = scrub_year_only_prefix(prefix)
            {
                title.rendering.prefix = Some(cleaned);
            }
            scrub_overrides_map(title.overrides.as_mut());
        }
        TemplateComponent::Number(number) => {
            if number.number == citum_schema::template::NumberVariable::Pages
                && let Some(prefix) = number.rendering.prefix.as_ref()
                && let Some(cleaned) = scrub_pages_year_literal_prefix(prefix)
            {
                number.rendering.prefix = Some(cleaned);
            }
            scrub_overrides_map(number.overrides.as_mut());
        }
        TemplateComponent::List(list) => {
            for item in &mut list.items {
                scrub_inferred_literal_artifacts(item);
            }
            scrub_overrides_map(list.overrides.as_mut());
        }
        TemplateComponent::Contributor(contributor) => {
            scrub_overrides_map(contributor.overrides.as_mut());
        }
        TemplateComponent::Date(date) => {
            scrub_overrides_map(date.overrides.as_mut());
        }
        TemplateComponent::Variable(variable) => {
            scrub_overrides_map(variable.overrides.as_mut());
        }
        TemplateComponent::Term(term) => {
            scrub_overrides_map(term.overrides.as_mut());
        }
        _ => {}
    }
}

pub fn scrub_component_override_literals(
    override_value: &mut citum_schema::template::ComponentOverride,
) {
    match override_value {
        citum_schema::template::ComponentOverride::Component(component) => {
            scrub_inferred_literal_artifacts(component)
        }
        citum_schema::template::ComponentOverride::Rendering(rendering) => {
            if let Some(prefix) = rendering.prefix.as_ref() {
                if let Some(cleaned) = scrub_year_only_prefix(prefix) {
                    rendering.prefix = Some(cleaned);
                } else if let Some(cleaned) = scrub_pages_year_literal_prefix(prefix) {
                    rendering.prefix = Some(cleaned);
                }
            }
        }
    }
}

pub fn scrub_year_only_prefix(prefix: &str) -> Option<String> {
    let trimmed = prefix.trim();
    if !is_four_digit_year(trimmed) {
        return None;
    }

    if prefix.starts_with(' ') && prefix.ends_with(' ') {
        Some(" ".to_string())
    } else {
        None
    }
}

pub fn scrub_pages_year_literal_prefix(prefix: &str) -> Option<String> {
    if prefix
        .strip_prefix("; ")
        .and_then(|s| s.strip_suffix("; "))
        .is_some_and(|s| is_four_digit_year(s.trim()))
    {
        return Some("; ".to_string());
    }

    if prefix
        .strip_prefix(". ")
        .and_then(|s| s.strip_suffix(": "))
        .is_some_and(|s| is_four_digit_year(s.trim()))
    {
        return Some(": ".to_string());
    }

    None
}

pub fn is_four_digit_year(value: &str) -> bool {
    value.len() == 4
        && value.chars().all(|ch| ch.is_ascii_digit())
        && value
            .parse::<u16>()
            .is_ok_and(|year| (1800..=2100).contains(&year))
}

pub fn template_targets_type(template: &[TemplateComponent], target_type: &str) -> bool {
    template
        .iter()
        .any(|component| component_targets_type(component, target_type))
}

pub fn component_targets_type(component: &TemplateComponent, target_type: &str) -> bool {
    let overrides = match component {
        TemplateComponent::Contributor(c) => c.overrides.as_ref(),
        TemplateComponent::Date(d) => d.overrides.as_ref(),
        TemplateComponent::Title(t) => t.overrides.as_ref(),
        TemplateComponent::Number(n) => n.overrides.as_ref(),
        TemplateComponent::Variable(v) => v.overrides.as_ref(),
        TemplateComponent::List(l) => l.overrides.as_ref(),
        TemplateComponent::Term(t) => t.overrides.as_ref(),
        _ => None,
    };

    if let Some(overrides) = overrides
        && overrides
            .keys()
            .any(|selector| selector.matches(target_type))
    {
        return true;
    }

    if let TemplateComponent::List(list) = component {
        return list
            .items
            .iter()
            .any(|item| component_targets_type(item, target_type));
    }

    false
}

pub fn template_has_parent_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_parent_title)
}

pub fn component_has_parent_title(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Title(t) => {
            t.title == TitleType::ParentMonograph || t.title == TitleType::ParentSerial
        }
        TemplateComponent::List(list) => list.items.iter().any(component_has_parent_title),
        _ => false,
    }
}

pub fn template_has_accessed_date(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_accessed_date)
}

pub fn component_has_accessed_date(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Date(d) => d.date == DateVariable::Accessed,
        TemplateComponent::List(list) => list.items.iter().any(component_has_accessed_date),
        _ => false,
    }
}

pub fn inferred_candidate_structurally_diverges(
    inferred_template: &[TemplateComponent],
    candidate_template: &[TemplateComponent],
) -> bool {
    let inferred_has_primary_title = template_has_primary_title(inferred_template);
    let candidate_has_primary_title = template_has_primary_title(candidate_template);
    let inferred_has_parent_serial = template_has_parent_serial(inferred_template);
    let candidate_has_parent_serial = template_has_parent_serial(candidate_template);
    let inferred_has_publisher = template_has_publisher(inferred_template);
    let candidate_has_publisher = template_has_publisher(candidate_template);
    let inferred_has_volume = template_has_volume(inferred_template);
    let candidate_has_volume = template_has_volume(candidate_template);

    (inferred_has_primary_title && !candidate_has_primary_title)
        || (!inferred_has_parent_serial && candidate_has_parent_serial)
        || (inferred_has_publisher && !candidate_has_publisher)
        || (!inferred_has_volume && candidate_has_volume)
}

pub fn template_has_primary_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_primary_title)
}

pub fn component_has_primary_title(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Title(t) => t.title == TitleType::Primary,
        TemplateComponent::List(list) => list.items.iter().any(component_has_primary_title),
        _ => false,
    }
}

pub fn template_has_parent_serial(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_parent_serial)
}

pub fn component_has_parent_serial(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Title(t) => t.title == TitleType::ParentSerial,
        TemplateComponent::List(list) => list.items.iter().any(component_has_parent_serial),
        _ => false,
    }
}

pub fn template_has_publisher(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_publisher)
}

pub fn component_has_publisher(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Variable(v) => v.variable == SimpleVariable::Publisher,
        TemplateComponent::List(list) => list.items.iter().any(component_has_publisher),
        _ => false,
    }
}

pub fn template_has_volume(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_volume)
}

pub fn component_has_volume(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Number(n) => n.number == citum_schema::template::NumberVariable::Volume,
        TemplateComponent::List(list) => list.items.iter().any(component_has_volume),
        _ => false,
    }
}

/// Scrub literal artifacts from every value in a component overrides map.
pub fn scrub_overrides_map(
    overrides: Option<
        &mut std::collections::HashMap<
            citum_schema::template::TypeSelector,
            citum_schema::template::ComponentOverride,
        >,
    >,
) {
    let Some(map) = overrides else { return };
    for val in map.values_mut() {
        scrub_component_override_literals(val);
    }
}
