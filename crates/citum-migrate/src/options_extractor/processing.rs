/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use citum_schema::options::{
    GivennameRule, Group, LabelConfig, LabelPreset, Processing, ProcessingCustom, Sort, SortEntry,
    SortKey, SortSpec,
};
use citum_schema::presets::SortPreset;
use csl_legacy::model::{CslNode, Style};
use std::collections::HashSet;

/// Detects the citation processing mode from a CSL style.
///
/// Analyzes style attributes and layout patterns to determine if the style
/// uses author-date, numeric, note-based, or label-based citation processing.
pub fn detect_processing_mode(style: &Style) -> Option<Processing> {
    // 0. Label (trigraph) styles render the generated `citation-label`
    // variable (e.g. `[Kuhn62]`). The engine only emits citation labels under
    // `Processing::Label`, so the mode must be detected here or the label
    // renders empty.
    fn has_citation_label(nodes: &[csl_legacy::model::CslNode]) -> bool {
        use csl_legacy::model::CslNode;
        nodes.iter().any(|node| match node {
            CslNode::Text(t) => t.variable.as_deref() == Some("citation-label"),
            CslNode::Group(g) => has_citation_label(&g.children),
            _ => false,
        })
    }

    if has_citation_label(&style.citation.layout.children) {
        return Some(Processing::Label(LabelConfig {
            preset: LabelPreset::Ams,
            ..Default::default()
        }));
    }

    // 0b. Note styles are explicit in CSL and should map directly when they
    // do not render generated citation labels.
    if style.class == "note" {
        return Some(Processing::Note);
    }

    // 1. Explicitly numeric style
    // Check if bibliography uses second-field-align (heuristic for numeric labels)
    // Actually, check if it's APA (not numeric) or check common markers
    // Since 'second_field_align' is missing in my model read, I'll use a safer heuristic.

    // Helper to recursively search for citation-number in layout nodes
    fn has_citation_number(nodes: &[csl_legacy::model::CslNode]) -> bool {
        use csl_legacy::model::CslNode;
        nodes.iter().any(|node| match node {
            CslNode::Number(n) => n.variable == "citation-number",
            CslNode::Group(g) => has_citation_number(&g.children),
            CslNode::Text(t) if t.variable.as_deref() == Some("citation-number") => true,
            _ => false,
        })
    }

    let is_numeric =
        style.class == "in-text" && has_citation_number(&style.citation.layout.children);

    if is_numeric {
        return Some(Processing::Numeric);
    }

    // 2. Author-date style
    // Some styles hide date/year logic in nested macro trees. Follow macro calls
    // recursively so we don't miss author-date processing config extraction.
    let mut visited_macros = HashSet::new();
    let is_author_date =
        nodes_have_author_date_signal(&style.citation.layout.children, style, &mut visited_macros);

    if is_author_date {
        let mut custom = Processing::AuthorDate.config();

        let sort = style.citation.sort.as_ref().and_then(extract_sort);
        if let Some(sort) = sort {
            // Preset sorts already carry their canonical group via the base
            // config; only derive group from explicit (non-preset) sort keys.
            if let SortEntry::Explicit(explicit_sort) = &sort {
                custom.group = extract_group_from_sort(explicit_sort);
            }
            custom.sort = Some(sort);
        }

        if let Some(disamb) = custom.disambiguate.as_mut() {
            if let Some(names) = style.citation.disambiguate_add_names {
                disamb.names = names;
            }
            if let Some(add_givenname) = style.citation.disambiguate_add_givenname {
                disamb.add_givenname = add_givenname;
            }
            if let Some(year_suffix) = style.citation.disambiguate_add_year_suffix {
                disamb.year_suffix = year_suffix;
            }
            disamb.givenname_rule = match style.citation.disambiguate_givenname_rule.as_deref() {
                Some("primary-name") => GivennameRule::PrimaryName,
                Some("primary-name-with-initials") => GivennameRule::PrimaryNameWithInitials,
                Some("all-names-with-initials") => GivennameRule::AllNamesWithInitials,
                Some("all-names") => GivennameRule::AllNames,
                _ => GivennameRule::default(),
            };
        }

        return Some(fold_to_named_processing(custom));
    }

    None
}

/// Substitute a `Processing::Custom` for the named variant whose
/// canonical `config()` matches it. Keeps the migrated YAML idiomatic
/// (`processing: author-date`) instead of dumping a `!custom` block when
/// the derived config is just the named-variant default.
fn fold_to_named_processing(custom: ProcessingCustom) -> Processing {
    for candidate in [
        Processing::AuthorDate,
        Processing::AuthorDateGivenname,
        Processing::AuthorDateNames,
        Processing::AuthorDateFull,
        Processing::Numeric,
        Processing::Note,
    ] {
        if candidate.config() == custom {
            return candidate;
        }
    }
    Processing::Custom(custom)
}

fn nodes_have_author_date_signal(
    nodes: &[CslNode],
    style: &Style,
    visited_macros: &mut HashSet<String>,
) -> bool {
    nodes
        .iter()
        .any(|node| node_has_author_date_signal(node, style, visited_macros))
}

fn node_has_author_date_signal(
    node: &CslNode,
    style: &Style,
    visited_macros: &mut HashSet<String>,
) -> bool {
    match node {
        CslNode::Date(_) => true,
        CslNode::Text(t) => {
            if t.variable.as_deref().is_some_and(|v| {
                matches!(
                    v,
                    "issued" | "original-date" | "event-date" | "accessed" | "year-suffix"
                )
            }) {
                return true;
            }

            if let Some(macro_name) = &t.macro_name {
                let lowered = macro_name.to_ascii_lowercase();
                if lowered.contains("year") || lowered.contains("date") {
                    return true;
                }

                if visited_macros.insert(macro_name.clone())
                    && let Some(macro_def) = style.macros.iter().find(|m| m.name == *macro_name)
                    && nodes_have_author_date_signal(&macro_def.children, style, visited_macros)
                {
                    return true;
                }
            }

            false
        }
        CslNode::Group(g) => nodes_have_author_date_signal(&g.children, style, visited_macros),
        CslNode::Choose(c) => {
            nodes_have_author_date_signal(&c.if_branch.children, style, visited_macros)
                || c.else_if_branches
                    .iter()
                    .any(|b| nodes_have_author_date_signal(&b.children, style, visited_macros))
                || c.else_branch.as_ref().is_some_and(|nodes| {
                    nodes_have_author_date_signal(nodes, style, visited_macros)
                })
        }
        CslNode::Names(n) => nodes_have_author_date_signal(&n.children, style, visited_macros),
        _ => false,
    }
}

fn extract_sort(legacy_sort: &csl_legacy::model::Sort) -> Option<SortEntry> {
    let template = deduplicate_sort_specs(
        legacy_sort
            .keys
            .iter()
            .filter_map(|key| {
                let key_kind = key
                    .variable
                    .as_ref()
                    .and_then(|name| parse_sort_key(name))
                    .or_else(|| {
                        key.macro_name
                            .as_ref()
                            .and_then(|name| parse_sort_key(name))
                    })?;

                let ascending = key.sort.as_deref() != Some("descending");
                Some(SortSpec {
                    key: key_kind,
                    ascending,
                })
            })
            .collect(),
    );

    if template.is_empty() {
        None
    } else if let Some(preset) = sort_preset_for_specs(&template) {
        Some(SortEntry::Preset(preset))
    } else {
        Some(SortEntry::Explicit(Sort {
            shorten_names: false,
            render_substitutions: false,
            template,
        }))
    }
}

fn deduplicate_sort_specs(template: Vec<SortSpec>) -> Vec<SortSpec> {
    let mut deduplicated = Vec::new();

    for spec in template {
        if let Some(existing) = deduplicated
            .iter_mut()
            .find(|existing: &&mut SortSpec| existing.key == spec.key)
        {
            existing.ascending &= spec.ascending;
            continue;
        }
        deduplicated.push(spec);
    }

    deduplicated
}

fn sort_preset_for_specs(template: &[SortSpec]) -> Option<SortPreset> {
    if template.iter().any(|spec| !spec.ascending) {
        return None;
    }

    let keys: Vec<&SortKey> = template.iter().map(|spec| &spec.key).collect();
    match keys.as_slice() {
        [SortKey::Author]
        | [SortKey::Author, SortKey::Year]
        | [SortKey::Author, SortKey::Year, SortKey::Title] => Some(SortPreset::AuthorDateTitle),
        [SortKey::Author, SortKey::Title] | [SortKey::Author, SortKey::Title, SortKey::Year] => {
            Some(SortPreset::AuthorTitleDate)
        }
        [SortKey::CitationNumber] => Some(SortPreset::CitationNumber),
        _ => None,
    }
}

fn extract_group_from_sort(sort: &Sort) -> Option<Group> {
    let mut keys: Vec<SortKey> = Vec::new();

    for spec in &sort.template {
        match spec.key {
            SortKey::Author | SortKey::Year | SortKey::Title if !keys.contains(&spec.key) => {
                keys.push(spec.key.clone());
            }
            SortKey::CitationNumber => {}
            _ => {}
        }
    }

    if keys.is_empty() {
        None
    } else {
        Some(Group { template: keys })
    }
}

fn parse_sort_key(name: &str) -> Option<SortKey> {
    let lowered = name.to_ascii_lowercase();

    if lowered == "citation-number" || lowered.contains("citation-number") {
        Some(SortKey::CitationNumber)
    } else if lowered == "author" || lowered.contains("author") {
        Some(SortKey::Author)
    } else if lowered == "issued"
        || lowered == "year"
        || lowered.contains("year")
        || lowered.contains("date")
    {
        Some(SortKey::Year)
    } else if lowered == "title" || lowered.contains("title") {
        Some(SortKey::Title)
    } else {
        None
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;
    use roxmltree::Document;

    fn parse(xml: &str) -> Style {
        let doc = Document::parse(xml).expect("test style XML should parse");
        csl_legacy::parser::parse_style(doc.root_element()).expect("legacy style should parse")
    }

    fn style_with_citation_layout(class: &str, layout_body: &str) -> Style {
        parse(&format!(
            r#"<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="{class}">
  <info><title>t</title><id>https://example.org/t</id></info>
  <citation><layout prefix="[" suffix="]">{layout_body}</layout></citation>
</style>"#
        ))
    }

    #[test]
    fn detects_label_mode_from_citation_label_variable() {
        // given an in-text style whose citation renders the generated label
        let style = style_with_citation_layout("in-text", r#"<text variable="citation-label"/>"#);

        // when the processing mode is detected
        let mode = detect_processing_mode(&style);

        // then it is Label with CSL-compatible label parameters
        assert!(
            matches!(mode, Some(Processing::Label(_))),
            "expected Processing::Label, got: {mode:?}"
        );
        if let Some(Processing::Label(config)) = mode {
            let params = config.effective_params();
            assert_eq!(params.single_author_chars, 4);
            assert_eq!(params.et_al_min, 5);
            assert_eq!(params.et_al_marker, "");
            assert_eq!(params.et_al_names, 4);
        }
    }

    #[test]
    fn detects_label_mode_before_note_mode() {
        // given a note style whose citation renders the generated label
        let style = style_with_citation_layout("note", r#"<text variable="citation-label"/>"#);

        // then label processing wins because note processing cannot emit citation-label
        let mode = detect_processing_mode(&style);
        assert!(
            matches!(mode, Some(Processing::Label(_))),
            "expected Processing::Label, got: {mode:?}"
        );
    }

    #[test]
    fn citation_number_style_is_not_misdetected_as_label() {
        // given a numeric style (no citation-label), Label must not steal it
        let style = style_with_citation_layout("in-text", r#"<text variable="citation-number"/>"#);

        let mode = detect_processing_mode(&style);

        assert!(
            matches!(mode, Some(Processing::Numeric)),
            "expected Processing::Numeric, got: {mode:?}"
        );
    }
}
