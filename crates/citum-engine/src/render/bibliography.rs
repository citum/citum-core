/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use std::collections::HashMap;
use std::fmt::Write;

use crate::io::{AnnotationFormat, AnnotationStyle, ParagraphBreak};
use crate::render::component::{ProcEntry, ProcTemplateComponent, render_component_with_format};
use crate::render::format::OutputFormat;
use crate::render::plain::PlainText;
use crate::render::rich_text::{render_djot_inline, render_org_inline};

/// Check if a character is a final punctuation mark (not a space).
/// This distinguishes between intentional component suffixes and separator duplication.
fn is_final_punctuation(c: char) -> bool {
    matches!(c, '.' | ',' | ':' | ';' | '!' | '?')
}

fn is_sentence_ending_punctuation(c: char) -> bool {
    matches!(c, '.' | '!' | '?')
}

fn visible_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn first_visible_char(input: &str) -> Option<char> {
    visible_text(input).chars().next()
}

fn last_visible_non_space_char(input: &str) -> Option<char> {
    visible_text(input)
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
}

fn ends_with_sentence_ending_visible_punctuation(input: &str) -> bool {
    let visible = visible_text(input);
    let mut chars = visible.chars().rev().filter(|ch| !ch.is_whitespace());
    match chars.next() {
        Some(ch) if is_sentence_ending_punctuation(ch) => true,
        Some('"' | '\u{201D}') => chars.next().is_some_and(is_sentence_ending_punctuation),
        _ => false,
    }
}

/// Returns true when the next rendered component should be treated as sentence-initial
/// under the same join semantics used by bibliography rendering.
#[must_use]
pub(crate) fn component_starts_new_sentence(
    entry_output: &str,
    rendered: &str,
    default_separator: &str,
    punctuation_in_quote: bool,
) -> bool {
    if entry_output.is_empty() {
        return true;
    }

    let first_char = first_visible_char(rendered).unwrap_or(' ');
    let starts_with_separator = matches!(first_char, ',' | ';' | ':' | ' ' | '.' | '(');

    if starts_with_separator {
        return false;
    }

    if ends_with_sentence_ending_visible_punctuation(entry_output) {
        return true;
    }

    let last_char = entry_output.chars().last().unwrap_or(' ');
    let trimmed_last = last_visible_non_space_char(entry_output).unwrap_or(' ');
    if !last_char.is_whitespace()
        && !first_char.is_whitespace()
        && !is_final_punctuation(trimmed_last)
        && default_separator
            .chars()
            .next()
            .is_some_and(is_sentence_ending_punctuation)
    {
        return true;
    }

    punctuation_in_quote
        && default_separator.starts_with('.')
        && (entry_output.ends_with('"') || entry_output.ends_with('\u{201D}'))
}

/// Render processed templates into a final bibliography string using `PlainText` format.
#[must_use]
pub fn refs_to_string(proc_entries: Vec<ProcEntry>) -> String {
    refs_to_string_with_format::<PlainText>(proc_entries, None, None)
}

/// Render one processed bibliography entry body without outer entry/bibliography wrappers.
#[must_use]
pub fn render_entry_body_with_format<F: OutputFormat<Output = String>>(
    entry: &ProcEntry,
) -> String {
    render_entry_body_components_with_format::<F>(&entry.template)
}

/// Render processed bibliography components without outer entry/bibliography wrappers.
#[must_use]
pub(crate) fn render_entry_body_components_with_format<F: OutputFormat<Output = String>>(
    proc_template: &[ProcTemplateComponent],
) -> String {
    let mut entry_output = String::new();
    let mut pending_component: Option<(
        usize,
        &crate::render::component::ProcTemplateComponent,
        String,
    )> = None;

    // Check locale option for punctuation placement in quotes.
    let punctuation_in_quote = proc_template
        .first()
        .and_then(|c| c.config.as_ref())
        .is_some_and(|cfg| cfg.punctuation_in_quote);

    // Get the bibliography separator from the config, defaulting to ". "
    let default_separator = proc_template
        .first()
        .and_then(|c| c.bibliography_config.as_ref())
        .and_then(|bib| bib.separator.as_deref())
        .unwrap_or(". ");

    for (index, component) in proc_template.iter().enumerate() {
        let rendered = render_component_with_format::<F>(component);
        if rendered.is_empty() {
            continue;
        }

        if let Some((_, _, previous)) = pending_component.replace((index, component, rendered)) {
            append_rendered_component(
                &mut entry_output,
                &previous,
                default_separator,
                punctuation_in_quote,
            );
        }
    }

    if let Some((last_index, last_component, rendered)) = pending_component {
        let final_rendered = if last_index + 1 < proc_template.len() {
            let mut trimmed_component = last_component.clone();
            let rendering = trimmed_component.template_component.rendering_mut();
            rendering.suffix = None;
            if let Some(ref mut wrap_config) = rendering.wrap {
                wrap_config.inner_suffix = None;
            }
            trimmed_component.suffix = None;
            render_component_with_format::<F>(&trimmed_component)
        } else {
            rendered
        };
        append_rendered_component(
            &mut entry_output,
            &final_rendered,
            default_separator,
            punctuation_in_quote,
        );
    }

    let bib_cfg = proc_template
        .first()
        .and_then(|c| c.bibliography_config.as_ref());
    let entry_suffix = bib_cfg.and_then(|bib| bib.entry_suffix.as_deref());
    match entry_suffix {
        Some(suffix) if !suffix.is_empty() => {
            let ends_with_url = ends_with_url_or_doi(&entry_output);
            if !ends_with_url && !entry_output.ends_with(suffix.chars().next().unwrap_or('.')) {
                if suffix == "."
                    && punctuation_in_quote
                    && (entry_output.ends_with('"') || entry_output.ends_with('\u{201D}'))
                {
                    let is_curly = entry_output.ends_with('\u{201D}');
                    entry_output.pop();
                    entry_output.push_str(if is_curly { ".\u{201D}" } else { ".\"" });
                } else {
                    entry_output.push_str(suffix);
                }
            }
        }
        _ => {}
    }

    cleanup_dangling_punctuation(&mut entry_output);
    entry_output
}

pub(crate) fn append_rendered_component(
    entry_output: &mut String,
    rendered: &str,
    default_separator: &str,
    punctuation_in_quote: bool,
) {
    if !entry_output.is_empty() {
        let last_char = entry_output.chars().last().unwrap_or(' ');
        let first_char = first_visible_char(rendered).unwrap_or(' ');
        let sep_first_char = default_separator.chars().next().unwrap_or('.');
        let trimmed_last = last_visible_non_space_char(entry_output).unwrap_or(' ');
        let ends_with_punctuation = is_final_punctuation(trimmed_last);
        let starts_with_separator = matches!(first_char, ',' | ';' | ':' | ' ' | '.' | '(');

        if starts_with_separator {
            if first_char == '(' && !last_char.is_whitespace() && last_char != '[' {
                entry_output.push(' ');
            }
        } else if ends_with_punctuation {
            if !last_char.is_whitespace() {
                entry_output.push(' ');
            }
        } else if punctuation_in_quote
            && (last_char == '"' || last_char == '\u{201D}')
            && sep_first_char == '.'
        {
            entry_output.pop();
            let quote_str = if last_char == '\u{201D}' {
                ".\u{201D} "
            } else {
                ".\" "
            };
            entry_output.push_str(quote_str);
        } else if !last_char.is_whitespace() && !first_char.is_whitespace() {
            entry_output.push_str(default_separator);
        } else if !last_char.is_whitespace()
            && first_char.is_whitespace()
            && default_separator.starts_with('.')
            && !ends_with_punctuation
        {
            entry_output.push('.');
        }
    }

    let _ = write!(entry_output, "{rendered}");
}

/// Render processed templates into a final bibliography string using a specific format.
#[must_use]
pub fn refs_to_string_with_format<F: OutputFormat<Output = String>>(
    proc_entries: Vec<ProcEntry>,
    annotations: Option<&HashMap<String, String>>,
    annotation_style: Option<&AnnotationStyle>,
) -> String {
    let fmt = F::default();
    let mut rendered_entries = Vec::new();

    for entry in &proc_entries {
        let mut entry_output = render_entry_body_with_format::<F>(entry);
        let proc_template = &entry.template;

        // Apply annotation if present
        if let Some(annotations) = annotations
            && let Some(annotation_text) = annotations.get(&entry.id)
        {
            let style = annotation_style.cloned().unwrap_or_default();
            let separator = match style.paragraph_break {
                ParagraphBreak::BlankLine => "\n\n",
                ParagraphBreak::SingleLine => "\n",
            };
            let indent_prefix = if style.indent { "    " } else { "" };

            // Render annotation text through markup format if enabled
            let rendered = match style.format {
                AnnotationFormat::Djot => render_djot_inline(annotation_text, &fmt),
                AnnotationFormat::Plain => annotation_text.clone(),
                AnnotationFormat::Org => render_org_inline(annotation_text, &fmt),
            };

            // Apply indentation to each line (preserving blank lines for paragraph breaks)
            let indented_text = rendered
                .lines()
                .map(|line| {
                    if line.trim().is_empty() {
                        line.to_string()
                    } else {
                        let indented_line = format!("{indent_prefix}{line}");
                        if style.italic {
                            fmt.finish(fmt.emph(fmt.text(&indented_line)))
                        } else {
                            indented_line
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            entry_output.push_str(separator);
            entry_output.push_str(&indented_text);
        }

        if visible_text(&entry_output).trim().is_empty() {
            continue;
        }

        // Resolve entry URL if whole-entry linking is enabled
        let entry_url = proc_template
            .first()
            .and_then(|c| c.config.as_ref())
            .and_then(|cfg| cfg.links.as_ref())
            .and_then(|links| {
                use citum_schema::options::LinkAnchor;
                if matches!(links.anchor, Some(LinkAnchor::Entry)) {
                    // We need the reference to resolve the URL.
                    // This is a bit tricky as ProcEntry doesn't have the reference.
                    // But we can look it up from the bibliography if we had access to it.
                    // For now, let's see if any component in the template has a URL resolved.
                    proc_template.iter().find_map(|c| c.url.clone())
                } else {
                    None
                }
            });

        rendered_entries.push(fmt.entry(
            &entry.id,
            entry_output,
            entry_url.as_deref(),
            &entry.metadata,
        ));
    }

    fmt.finish(fmt.bibliography(rendered_entries))
}

/// Check if the output ends with a URL or DOI (to suppress trailing period).
fn ends_with_url_or_doi(output: &str) -> bool {
    let visible = visible_text(output);
    let trimmed = visible.trim_end_matches('.');
    let trimmed = trimmed.trim_end();
    // Check if the last "word" looks like a URL or DOI
    if let Some(last_segment) = trimmed.rsplit_once(' ') {
        let last = last_segment.1;
        last.starts_with("https://") || last.starts_with("http://") || last.starts_with("doi.org/")
    } else {
        trimmed.starts_with("https://")
            || trimmed.starts_with("http://")
            || trimmed.starts_with("doi.org/")
    }
}

fn cleanup_dangling_punctuation(output: &mut String) {
    let patterns = [
        (", .", "."),
        (", ,", ","),
        (": .", "."),
        ("; .", "."),
        // NOTE: Removed (".,", ".") pattern - it was too aggressive and removed legitimate
        // component suffixes like "S.," from author initials. In Citum, component suffixes are
        // explicit and well-defined, so we don't have the CSL 1.0 dual-punctuation issue.
        (" ,", ","),
        (" ;", ";"),
        (" :", ":"),
        (" .", "."),
        (",  ", ", "),
        (". .", "."),
        (".. ", ". "),
        ("..", "."),
        ("  ", " "), // Double space to single
    ];

    let mut changed = true;
    while changed {
        changed = false;
        for (pattern, replacement) in &patterns {
            if output.contains(pattern) {
                *output = output.replace(pattern, replacement);
                changed = true;
            }
        }
    }
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
    use crate::render::component::ProcTemplateComponent;
    use citum_schema::template::{Rendering, TemplateComponent, WrapConfig, WrapPunctuation};

    #[test]
    fn test_component_starts_new_sentence_at_entry_start() {
        assert!(component_starts_new_sentence(
            "",
            "Edited by Grimm, Jacob",
            ". ",
            false
        ));
    }

    #[test]
    fn test_component_starts_new_sentence_after_period() {
        assert!(component_starts_new_sentence(
            "Collected Essays.",
            "edited by Grimm, Jacob",
            ". ",
            false
        ));
    }

    #[test]
    fn test_component_does_not_start_new_sentence_after_colon() {
        assert!(!component_starts_new_sentence(
            "Collected Essays:",
            "edited by Grimm, Jacob",
            ". ",
            false
        ));
    }

    #[test]
    fn test_bibliography_separator_suppression() {
        use citum_schema::options::{BibliographyConfig, Config};

        let config = Config::default();
        let bibliography_config = BibliographyConfig {
            separator: Some(". ".to_string()),
            entry_suffix: Some(String::new()),
            ..Default::default()
        };

        let c1 = ProcTemplateComponent {
            template_component: TemplateComponent::Variable(
                citum_schema::template::TemplateVariable {
                    variable: citum_schema::template::SimpleVariable::Publisher,
                    rendering: Rendering::default(),
                    ..Default::default()
                },
            ),
            template_index: None,
            value: "Publisher1".to_string(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config.clone()),
            bibliography_config: Some(bibliography_config.clone()),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let c2 = ProcTemplateComponent {
            template_component: TemplateComponent::Variable(
                citum_schema::template::TemplateVariable {
                    variable: citum_schema::template::SimpleVariable::PublisherPlace,
                    rendering: Rendering {
                        prefix: Some(". ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
            template_index: None,
            value: "Place".to_string(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config),
            bibliography_config: Some(bibliography_config),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let entries = vec![ProcEntry {
            id: "id1".to_string(),
            template: vec![c1, c2],
            metadata: crate::render::format::ProcEntryMetadata::default(),
        }];
        let result = refs_to_string(entries);
        assert_eq!(result, "Publisher1. Place");
    }

    #[test]
    fn test_no_suppression_after_parenthesis() {
        use citum_schema::options::{BibliographyConfig, Config};

        let config = Config::default();
        let bibliography_config = BibliographyConfig {
            separator: Some(", ".to_string()),
            entry_suffix: Some(String::new()),
            ..Default::default()
        };

        let c1 = ProcTemplateComponent {
            template_component: TemplateComponent::Contributor(
                citum_schema::template::TemplateContributor {
                    contributor: citum_schema::template::ContributorRole::Editor,
                    rendering: Rendering {
                        wrap: Some(WrapConfig {
                            punctuation: WrapPunctuation::Parentheses,
                            inner_prefix: None,
                            inner_suffix: None,
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
            template_index: None,
            value: "Eds.".to_string(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config.clone()),
            bibliography_config: Some(bibliography_config.clone()),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let c2 = ProcTemplateComponent {
            template_component: TemplateComponent::Title(citum_schema::template::TemplateTitle {
                title: citum_schema::template::TitleType::Primary,
                rendering: Rendering::default(),
                ..Default::default()
            }),
            template_index: None,
            value: "Title".to_string(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config),
            bibliography_config: Some(bibliography_config),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let entries = vec![ProcEntry {
            id: "id1".to_string(),
            template: vec![c1, c2],
            metadata: crate::render::format::ProcEntryMetadata::default(),
        }];
        let result = refs_to_string(entries);
        assert_eq!(result, "(Eds.), Title");
    }

    #[test]
    fn test_html_bibliography_structure() {
        use crate::render::html::Html;
        use citum_schema::template::TemplateTerm;

        let c1 = ProcTemplateComponent {
            template_component: TemplateComponent::Term(TemplateTerm::default()),
            value: "Reference Content".to_string(),
            ..Default::default()
        };

        let entries = vec![ProcEntry {
            id: "ref-1".to_string(),
            template: vec![c1],
            metadata: crate::render::format::ProcEntryMetadata::default(),
        }];

        let result = refs_to_string_with_format::<Html>(entries, None, None);
        assert_eq!(
            result,
            r#"<div class="citum-bibliography">
<div class="citum-entry" id="ref-ref-1">Reference Content</div>
</div>"#
        );
    }

    #[test]
    fn test_component_suffix_preserved_elsevier_harvard() {
        use citum_schema::options::{BibliographyConfig, Config};

        // Elsevier Harvard: author component has suffix `, ` and date has suffix `.`
        // Expected: "Hawking, S., 1988." (comma from author suffix preserved)
        let config = Config::default();
        let bibliography_config = BibliographyConfig {
            separator: Some(". ".to_string()),
            entry_suffix: Some(".".to_string()),
            ..Default::default()
        };

        let c1 = ProcTemplateComponent {
            template_component: TemplateComponent::Contributor(
                citum_schema::template::TemplateContributor {
                    contributor: citum_schema::template::ContributorRole::Author,
                    rendering: Rendering {
                        suffix: Some(", ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
            template_index: None,
            value: "Hawking, S.".to_string(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config.clone()),
            bibliography_config: Some(bibliography_config.clone()),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let c2 = ProcTemplateComponent {
            template_component: TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: citum_schema::template::DateVariable::Issued,
                rendering: Rendering {
                    suffix: Some(".".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            template_index: None,
            value: "1988".to_string(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config),
            bibliography_config: Some(bibliography_config),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let entries = vec![ProcEntry {
            id: "hawking1988".to_string(),
            template: vec![c1, c2],
            metadata: crate::render::format::ProcEntryMetadata::default(),
        }];
        let result = refs_to_string(entries);
        // The comma from author's suffix should be preserved
        assert_eq!(result, "Hawking, S., 1988.");
    }

    #[test]
    fn test_terminal_component_suffix_suppressed_when_following_component_is_empty() {
        use citum_schema::options::{BibliographyConfig, Config};

        let config = Config::default();
        let bibliography_config = BibliographyConfig {
            separator: Some(". ".to_string()),
            entry_suffix: Some(String::new()),
            ..Default::default()
        };

        let date = ProcTemplateComponent {
            template_component: TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: citum_schema::template::DateVariable::Issued,
                rendering: Rendering {
                    suffix: Some(", ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            template_index: None,
            value: "2024".to_string(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config.clone()),
            bibliography_config: Some(bibliography_config.clone()),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let pages = ProcTemplateComponent {
            template_component: TemplateComponent::Number(citum_schema::template::TemplateNumber {
                number: citum_schema::template::NumberVariable::Pages,
                rendering: Rendering::default(),
                ..Default::default()
            }),
            template_index: None,
            value: String::new(),
            prefix: None,
            suffix: None,
            ref_type: None,
            config: Some(config),
            bibliography_config: Some(bibliography_config),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let result = refs_to_string(vec![ProcEntry {
            id: "book-without-pages".to_string(),
            template: vec![date, pages],
            metadata: crate::render::format::ProcEntryMetadata::default(),
        }]);

        assert_eq!(result, "2024");
    }

    #[allow(
        clippy::too_many_lines,
        reason = "rendering fixture exercises a full punctuation case"
    )]
    #[test]
    fn test_html_separator_logic_uses_visible_punctuation() {
        use crate::render::html::Html;
        use citum_schema::options::{BibliographyConfig, Config};
        use citum_schema::template::{
            NumberVariable, SimpleVariable, TemplateNumber, TemplateVariable,
        };

        let config = Config {
            ..Default::default()
        };
        let bibliography_config = BibliographyConfig {
            separator: Some(". ".to_string()),
            entry_suffix: Some(String::new()),
            ..Default::default()
        };

        let volume_issue = ProcTemplateComponent {
            template_component: TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::Volume,
                rendering: Rendering {
                    emph: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            }),
            template_index: None,
            value: "322(10)".to_string(),
            prefix: None,
            suffix: None,
            ref_type: Some("article-journal".to_string()),
            config: Some(config.clone()),
            bibliography_config: Some(bibliography_config.clone()),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let pages = ProcTemplateComponent {
            template_component: TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::Pages,
                rendering: Rendering {
                    prefix: Some(", ".to_string()),
                    suffix: Some(".".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            template_index: None,
            value: "891–921".to_string(),
            prefix: None,
            suffix: None,
            ref_type: Some("article-journal".to_string()),
            config: Some(config.clone()),
            bibliography_config: Some(bibliography_config.clone()),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let doi = ProcTemplateComponent {
            template_component: TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Doi,
                rendering: Rendering {
                    prefix: Some("https://doi.org/".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            template_index: None,
            value: "10.1002/andp.19053221004".to_string(),
            prefix: None,
            suffix: None,
            ref_type: Some("article-journal".to_string()),
            config: Some(config),
            bibliography_config: Some(bibliography_config),
            url: None,
            item_language: None,
            sentence_initial: false,
            pre_formatted: false,
        };

        let result = refs_to_string_with_format::<Html>(
            vec![ProcEntry {
                id: "einstein1905".to_string(),
                template: vec![volume_issue, pages, doi],
                metadata: crate::render::format::ProcEntryMetadata::default(),
            }],
            None,
            None,
        );

        assert!(
            !result.contains("322(10)</i></span>. <span class=\"citum-pages\">, 891–921."),
            "separator should not inject a period before pages: {result}"
        );
        assert!(
            !result.contains("891–921.</span>. <span class=\"citum-doi\">"),
            "separator should not inject a period before DOI: {result}"
        );
        assert!(
            result.contains(
                "<span class=\"citum-pages\">, 891–921.</span><span class=\"citum-doi\">"
            ) || result.contains(
                "<span class=\"citum-pages\">, 891–921.</span> <span class=\"citum-doi\">"
            ),
            "HTML output should preserve pages punctuation without duplicate separators: {result}"
        );
    }

    fn make_entry(id: &str, value: &str) -> ProcEntry {
        ProcEntry {
            id: id.to_string(),
            template: vec![ProcTemplateComponent {
                template_component: TemplateComponent::Variable(
                    citum_schema::template::TemplateVariable {
                        variable: citum_schema::template::SimpleVariable::Publisher,
                        rendering: Rendering::default(),
                        ..Default::default()
                    },
                ),
                template_index: None,
                value: value.to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: None,
                url: None,
                bibliography_config: None,
                item_language: None,
                sentence_initial: false,
                pre_formatted: false,
            }],
            metadata: crate::render::format::ProcEntryMetadata::default(),
        }
    }

    #[test]
    fn test_annotation_appended_after_entry() {
        let mut annotations = HashMap::new();
        annotations.insert(
            "ref1".to_string(),
            "A useful overview of the topic.".to_string(),
        );

        let style = AnnotationStyle::default(); // indent=true, no italic, blank line

        let result = refs_to_string_with_format::<PlainText>(
            vec![make_entry("ref1", "Some Publisher")],
            Some(&annotations),
            Some(&style),
        );

        assert!(
            result.contains("Some Publisher"),
            "entry text should appear: {result}"
        );
        assert!(
            result.contains("A useful overview of the topic."),
            "annotation should appear: {result}"
        );
        // Blank line separator: entry text followed by \n\n then indent
        assert!(
            result.contains("\n\n    A useful overview"),
            "annotation should be separated by blank line and indented: {result}"
        );
    }

    #[test]
    fn test_no_annotation_when_id_absent() {
        let mut annotations = HashMap::new();
        annotations.insert(
            "other-ref".to_string(),
            "Annotation for someone else.".to_string(),
        );

        let style = AnnotationStyle::default();

        let result = refs_to_string_with_format::<PlainText>(
            vec![make_entry("ref1", "Some Publisher")],
            Some(&annotations),
            Some(&style),
        );

        assert!(
            !result.contains("Annotation for someone else."),
            "annotation for a different ref should not appear: {result}"
        );
    }

    #[test]
    fn test_annotation_single_line_break() {
        let mut annotations = HashMap::new();
        annotations.insert("ref1".to_string(), "Short note.".to_string());

        let style = AnnotationStyle {
            italic: false,
            indent: false,
            paragraph_break: ParagraphBreak::SingleLine,
            format: AnnotationFormat::Plain,
        };

        let result = refs_to_string_with_format::<PlainText>(
            vec![make_entry("ref1", "Publisher")],
            Some(&annotations),
            Some(&style),
        );

        assert!(
            result.contains("\nShort note."),
            "single line break should precede annotation: {result}"
        );
        assert!(
            !result.contains("\n\n"),
            "should not have blank line with SingleLine break: {result}"
        );
    }

    #[test]
    fn test_no_annotations_when_none_supplied() {
        let result = refs_to_string_with_format::<PlainText>(
            vec![make_entry("ref1", "Some Publisher")],
            None,
            None,
        );

        assert!(
            result.contains("Some Publisher"),
            "entry should render normally: {result}"
        );
        // No extra blank lines beyond entry separator
        let blank_line_count = result.matches("\n\n").count();
        assert!(
            blank_line_count <= 1,
            "should not have spurious blank lines: {result}"
        );
    }
}
