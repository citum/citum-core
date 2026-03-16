use citum_schema::options::{TextCase, TitleRendering, TitlesConfig};
use csl_legacy::model::{CslNode, Style};
use std::collections::HashSet;

/// Extracts title formatting configuration from a CSL style.
///
/// This reads title-related rendering intent directly from legacy CSL so the
/// migrated style carries explicit title emphasis, quoting, and text-case
/// behavior in `options.titles`.
#[must_use]
pub fn extract_title_config(style: &Style) -> Option<TitlesConfig> {
    let mut config = TitlesConfig::default();
    let mut has_config = false;

    if let Some(bibliography) = &style.bibliography {
        if let Some(rendering) =
            scan_for_title_rendering(&bibliography.layout.children, style, "container-title")
        {
            config.periodical = Some(rendering.clone());
            config.serial = Some(rendering);
            has_config = true;
        }

        if let Some(rendering) =
            scan_for_title_rendering(&bibliography.layout.children, style, "title")
        {
            if rendering.text_case.is_some() {
                config.component = Some(TitleRendering {
                    text_case: rendering.text_case,
                    ..Default::default()
                });
            }
            config.monograph = Some(rendering);
            has_config = true;
        }
    }

    if config.component.is_none()
        && let Some(rendering) =
            scan_for_title_rendering(&style.citation.layout.children, style, "title")
    {
        config.component = Some(rendering);
        has_config = true;
    }

    if has_config { Some(config) } else { None }
}

fn scan_for_title_rendering(
    nodes: &[CslNode],
    style: &Style,
    var_name: &str,
) -> Option<TitleRendering> {
    let mut visited_macros = HashSet::new();
    let rendering = scan_nodes_for_title_rendering(nodes, style, var_name, &mut visited_macros);
    rendering.filter(has_rendering_signal)
}

fn scan_nodes_for_title_rendering(
    nodes: &[CslNode],
    style: &Style,
    var_name: &str,
    visited_macros: &mut HashSet<String>,
) -> Option<TitleRendering> {
    let mut merged = TitleRendering::default();
    let mut found = false;

    for node in nodes {
        match node {
            CslNode::Text(text) => {
                if text.variable.as_deref() == Some(var_name) {
                    merge_rendering(&mut merged, rendering_from_text_node(text));
                    found = true;
                }

                if let Some(macro_name) = &text.macro_name
                    && visited_macros.insert(macro_name.clone())
                {
                    if let Some(legacy_macro) = style
                        .macros
                        .iter()
                        .find(|candidate| candidate.name == *macro_name)
                        && let Some(nested) = scan_nodes_for_title_rendering(
                            &legacy_macro.children,
                            style,
                            var_name,
                            visited_macros,
                        )
                    {
                        merge_rendering(&mut merged, nested);
                        found = true;
                    }
                    visited_macros.remove(macro_name);
                }
            }
            CslNode::Group(group) => {
                if let Some(nested) =
                    scan_nodes_for_title_rendering(&group.children, style, var_name, visited_macros)
                {
                    merge_rendering(&mut merged, nested);
                    found = true;
                }
            }
            CslNode::Choose(choose) => {
                if let Some(nested) = scan_nodes_for_title_rendering(
                    &choose.if_branch.children,
                    style,
                    var_name,
                    visited_macros,
                ) {
                    merge_rendering(&mut merged, nested);
                    found = true;
                }

                for branch in &choose.else_if_branches {
                    if let Some(nested) = scan_nodes_for_title_rendering(
                        &branch.children,
                        style,
                        var_name,
                        visited_macros,
                    ) {
                        merge_rendering(&mut merged, nested);
                        found = true;
                    }
                }

                if let Some(else_branch) = &choose.else_branch
                    && let Some(nested) =
                        scan_nodes_for_title_rendering(else_branch, style, var_name, visited_macros)
                {
                    merge_rendering(&mut merged, nested);
                    found = true;
                }
            }
            _ => {}
        }
    }

    found.then_some(merged)
}

fn rendering_from_text_node(text: &csl_legacy::model::Text) -> TitleRendering {
    TitleRendering {
        text_case: map_text_case(text.text_case.as_deref()),
        emph: text
            .formatting
            .font_style
            .as_deref()
            .filter(|value| *value == "italic")
            .map(|_| true),
        quote: text.quotes.filter(|quoted| *quoted),
        strong: text
            .formatting
            .font_weight
            .as_deref()
            .filter(|value| *value == "bold")
            .map(|_| true),
        small_caps: text
            .formatting
            .font_variant
            .as_deref()
            .filter(|value| *value == "small-caps")
            .map(|_| true),
        ..Default::default()
    }
}

fn merge_rendering(target: &mut TitleRendering, incoming: TitleRendering) {
    if target.text_case.is_none() {
        target.text_case = incoming.text_case;
    }
    if target.emph.is_none() {
        target.emph = incoming.emph;
    }
    if target.quote.is_none() {
        target.quote = incoming.quote;
    }
    if target.strong.is_none() {
        target.strong = incoming.strong;
    }
    if target.small_caps.is_none() {
        target.small_caps = incoming.small_caps;
    }
}

fn has_rendering_signal(rendering: &TitleRendering) -> bool {
    rendering.text_case.is_some()
        || rendering.emph.is_some()
        || rendering.quote.is_some()
        || rendering.strong.is_some()
        || rendering.small_caps.is_some()
}

fn map_text_case(case: Option<&str>) -> Option<TextCase> {
    match case {
        Some("sentence") => Some(TextCase::Sentence),
        Some("title") => Some(TextCase::Title),
        Some("capitalize-first") => Some(TextCase::CapitalizeFirst),
        Some("lowercase") => Some(TextCase::Lowercase),
        Some("uppercase") => Some(TextCase::Uppercase),
        Some("none") => Some(TextCase::AsIs),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csl_legacy::model::{Citation, Formatting, Info, Layout, Macro, Text};

    fn empty_style(
        citation_children: Vec<CslNode>,
        bibliography_children: Option<Vec<CslNode>>,
    ) -> Style {
        Style {
            version: "1.0".to_string(),
            xmlns: "http://purl.org/net/xbiblio/csl".to_string(),
            class: "in-text".to_string(),
            default_locale: None,
            initialize_with: None,
            initialize_with_hyphen: None,
            names_delimiter: None,
            name_as_sort_order: None,
            sort_separator: None,
            delimiter_precedes_last: None,
            delimiter_precedes_et_al: None,
            demote_non_dropping_particle: None,
            and: None,
            page_range_format: None,
            info: Info::default(),
            locale: Vec::new(),
            macros: Vec::new(),
            citation: Citation {
                layout: Layout {
                    prefix: None,
                    suffix: None,
                    delimiter: None,
                    children: citation_children,
                },
                sort: None,
                et_al_min: None,
                et_al_use_first: None,
                disambiguate_add_year_suffix: None,
                disambiguate_add_names: None,
                disambiguate_add_givenname: None,
            },
            bibliography: bibliography_children.map(|children| csl_legacy::model::Bibliography {
                layout: Layout {
                    prefix: None,
                    suffix: None,
                    delimiter: None,
                    children,
                },
                sort: None,
                et_al_min: None,
                et_al_use_first: None,
                hanging_indent: None,
                subsequent_author_substitute: None,
                subsequent_author_substitute_rule: None,
            }),
        }
    }

    fn title_text(variable: &str, text_case: Option<&str>, italic: bool, quotes: bool) -> CslNode {
        CslNode::Text(Text {
            value: None,
            variable: Some(variable.to_string()),
            macro_name: None,
            term: None,
            form: None,
            prefix: None,
            suffix: None,
            quotes: quotes.then_some(true),
            text_case: text_case.map(ToString::to_string),
            strip_periods: None,
            plural: None,
            macro_call_order: None,
            formatting: Formatting {
                font_style: italic.then_some("italic".to_string()),
                ..Default::default()
            },
        })
    }

    #[test]
    fn extracts_text_case_from_bibliography_titles() {
        let style = empty_style(
            Vec::new(),
            Some(vec![
                title_text("title", Some("sentence"), true, false),
                title_text("container-title", None, true, false),
            ]),
        );

        let config = extract_title_config(&style).expect("titles config");
        assert_eq!(
            config
                .component
                .as_ref()
                .and_then(|rendering| rendering.text_case),
            Some(TextCase::Sentence)
        );
        assert_eq!(
            config
                .monograph
                .as_ref()
                .and_then(|rendering| rendering.text_case),
            Some(TextCase::Sentence)
        );
        assert_eq!(
            config
                .monograph
                .as_ref()
                .and_then(|rendering| rendering.emph),
            Some(true)
        );
        assert_eq!(
            config
                .periodical
                .as_ref()
                .and_then(|rendering| rendering.emph),
            Some(true)
        );
    }

    #[test]
    fn extracts_component_quotes_from_citation_macros() {
        let mut style = empty_style(
            vec![CslNode::Text(Text {
                value: None,
                variable: None,
                macro_name: Some("title-short".to_string()),
                term: None,
                form: None,
                prefix: None,
                suffix: None,
                quotes: None,
                text_case: None,
                strip_periods: None,
                plural: None,
                macro_call_order: None,
                formatting: Formatting::default(),
            })],
            None,
        );
        style.macros.push(Macro {
            name: "title-short".to_string(),
            children: vec![title_text("title", None, false, true)],
        });

        let config = extract_title_config(&style).expect("titles config");
        assert_eq!(
            config
                .component
                .as_ref()
                .and_then(|rendering| rendering.quote),
            Some(true)
        );
    }
}
