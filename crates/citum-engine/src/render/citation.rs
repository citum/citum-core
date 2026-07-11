/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use crate::render::component::{ProcTemplate, render_component_with_format};
use crate::render::format::OutputFormat;
use crate::render::plain::PlainText;
use citum_schema::template::WrapPunctuation;

fn is_terminal_punctuation(ch: char) -> bool {
    matches!(ch, ':' | '.' | ';' | '!' | '?' | ',')
}

fn resolve_punctuation_collision(first: char, second: char) -> String {
    match (first, second) {
        (':', ':') => ":".to_string(),
        ('.', ':') => ".:".to_string(),
        (';', ':') => ";".to_string(),
        ('!', ':') => "!".to_string(),
        ('?', ':') => "?".to_string(),
        (',', ':') => ",:".to_string(),
        (':', '.') => ":".to_string(),
        ('.', '.') => ".".to_string(),
        (';', '.') => ";".to_string(),
        ('!', '.') => "!".to_string(),
        ('?', '.') => "?".to_string(),
        (',', '.') => ",.".to_string(),
        (':', ';') => ":;".to_string(),
        ('.', ';') => ".;".to_string(),
        (';', ';') => ";".to_string(),
        ('!', ';') => "!;".to_string(),
        ('?', ';') => "?;".to_string(),
        (',', ';') => ",;".to_string(),
        (':', '!') => "!".to_string(),
        ('.', '!') => ".!".to_string(),
        (';', '!') => "!".to_string(),
        ('!', '!') => "!".to_string(),
        ('?', '!') => "?!".to_string(),
        (',', '!') => ",!".to_string(),
        (':', '?') => "?".to_string(),
        ('.', '?') => ".?".to_string(),
        (';', '?') => "?".to_string(),
        ('!', '?') => "!?".to_string(),
        ('?', '?') => "?".to_string(),
        (',', '?') => ",?".to_string(),
        (':', ',') => ":,".to_string(),
        ('.', ',') => ".,".to_string(),
        (';', ',') => ";,".to_string(),
        ('!', ',') => "!,".to_string(),
        ('?', ',') => "?,".to_string(),
        (',', ',') => ",".to_string(),
        _ => format!("{first}{second}"),
    }
}

/// Append `delim` to `content`, applying house-style punctuation rules at the join point.
///
/// Three cases are handled in priority order:
/// 1. **Punctuation-in-quote** – when `punctuation_in_quote` is set and `delim` starts with
///    a comma, the comma is pulled *inside* a preceding closing quotation mark (`"` or `\u{201D}`)
///    before appending the rest of the delimiter. Quote marks are literal Unicode characters
///    in every backend, not markup, so this case looks at the raw last char directly.
/// 2. **Punctuation collision** – when format `F`'s *visible* last char of `content` and the
///    first char of `delim` are both terminal punctuation, the pair is resolved via
///    [`resolve_punctuation_collision`] (e.g. `".` + `". "` → `". "` rather than `".. "`). If
///    the raw content genuinely ends with that char, it's popped and merged as before; if the
///    visible terminal punctuation is hidden behind trailing markup (e.g. a LaTeX `\emph{...}`
///    close brace), the raw markup is left alone and the delimiter's redundant leading
///    punctuation is dropped instead.
/// 3. **Default** – append `delim` verbatim.
#[inline]
#[allow(
    clippy::string_slice,
    reason = "UTF-8 safe slicing based on char boundary checks"
)]
fn push_delimiter<F: OutputFormat<Output = String>>(
    content: &mut String,
    delim: &str,
    punctuation_in_quote: bool,
) {
    let delim_first = delim.chars().next();

    if punctuation_in_quote
        && delim_first == Some(',')
        && (content.ends_with('"') || content.ends_with('\u{201D}'))
    {
        // Case 1: pull the leading comma inside the closing quotation mark.
        let is_curly = content.ends_with('\u{201D}');
        content.pop();
        content.push(',');
        content.push(if is_curly { '\u{201D}' } else { '"' });
        content.push_str(&delim[1..]);
        return;
    }

    let Some(first) = delim_first else {
        content.push_str(delim);
        return;
    };
    let Some(visible_last) = F::default().visible_text(content).chars().last() else {
        content.push_str(delim);
        return;
    };

    if !is_terminal_punctuation(visible_last) || !is_terminal_punctuation(first) {
        // Case 3: no special rule — append the delimiter verbatim.
        content.push_str(delim);
    } else if content.ends_with(visible_last) {
        // Case 2a: raw content genuinely ends with the visible terminal char — merge as before.
        content.pop();
        content.push_str(&resolve_punctuation_collision(visible_last, first));
        content.push_str(&delim[first.len_utf8()..]);
    } else {
        // Case 2b: the visible terminal punctuation is behind trailing markup (e.g. LaTeX
        // `}`) — leave the raw markup alone and just drop the delimiter's redundant leading
        // punctuation instead of popping from `content`.
        content.push_str(&delim[first.len_utf8()..]);
    }
}

/// Render a processed template into a final citation string using `PlainText` format.
#[must_use]
pub fn citation_to_string(
    proc_template: &ProcTemplate,
    wrap: Option<&WrapPunctuation>,
    prefix: Option<&str>,
    suffix: Option<&str>,
    delimiter: Option<&str>,
) -> String {
    citation_to_string_with_format::<PlainText>(proc_template, wrap, prefix, suffix, delimiter)
}

/// Render a processed template into a final citation string using a specific format.
#[must_use]
pub fn citation_to_string_with_format<F: OutputFormat<Output = String>>(
    proc_template: &ProcTemplate,
    wrap: Option<&WrapPunctuation>,
    prefix: Option<&str>,
    suffix: Option<&str>,
    delimiter: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    for component in proc_template {
        let rendered = render_component_with_format::<F>(component);
        if !rendered.is_empty() {
            parts.push(rendered);
        }
    }

    let delim = delimiter.unwrap_or("");
    let punctuation_in_quote = proc_template
        .first()
        .and_then(|c| c.config.as_ref())
        .is_some_and(|cfg| cfg.punctuation_in_quote);

    let mut content = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            push_delimiter::<F>(&mut content, delim, punctuation_in_quote);
        }
        content.push_str(part);
    }

    let (open, close) = match wrap {
        Some(WrapPunctuation::Parentheses) => ("(", ")"),
        Some(WrapPunctuation::Brackets) => ("[", "]"),
        Some(WrapPunctuation::Quotes) => ("\u{201C}", "\u{201D}"),
        _ => (prefix.unwrap_or(""), suffix.unwrap_or("")),
    };

    format!("{open}{content}{close}")
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
    use crate::render::typst::Typst;
    use citum_schema::options::Config;
    use citum_schema::template::{
        ContributorForm, ContributorRole, DateForm, DateVariable, Rendering, TemplateComponent,
        TemplateContributor, TemplateDate, TemplateTitle, TitleType,
    };

    #[test]
    fn test_citation_to_string() {
        let template = vec![
            ProcTemplateComponent {
                template_component: TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Short,
                    name_order: None,
                    delimiter: None,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                template_index: None,
                value: "Kuhn".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: None,
                bibliography_config: None,
                url: None,
                item_language: None,
                quote_marks: Default::default(),
                sentence_initial: false,
                pre_formatted: false,
            },
            ProcTemplateComponent {
                template_component: TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                template_index: None,
                value: "1962".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: None,
                bibliography_config: None,
                url: None,
                item_language: None,
                quote_marks: Default::default(),
                sentence_initial: false,
                pre_formatted: false,
            },
        ];

        let result = citation_to_string(
            &template,
            Some(&WrapPunctuation::Parentheses),
            None,
            None,
            Some(", "),
        );
        assert_eq!(result, "(Kuhn, 1962)");
    }

    #[test]
    fn test_punctuation_in_quote_moves_comma_inside_closing_quote() {
        let config = Config {
            punctuation_in_quote: true,
            ..Default::default()
        };
        let template = vec![
            ProcTemplateComponent {
                template_component: TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    rendering: Rendering {
                        quote: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                template_index: None,
                value: "colon".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config.clone().into()),
                bibliography_config: None,
                url: None,
                item_language: None,
                quote_marks: Default::default(),
                sentence_initial: false,
                pre_formatted: false,
            },
            ProcTemplateComponent {
                template_component: TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                template_index: None,
                value: "period".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config.into()),
                bibliography_config: None,
                url: None,
                item_language: None,
                quote_marks: Default::default(),
                sentence_initial: false,
                pre_formatted: false,
            },
        ];

        let plain = citation_to_string(&template, None, None, None, Some(", "));
        let typst =
            citation_to_string_with_format::<Typst>(&template, None, None, None, Some(", "));

        assert_eq!(plain, "“colon,” period");
        assert_eq!(typst, "“colon,” period");
    }

    #[test]
    fn test_punctuation_outside_quotes_preserves_full_monty_matrix() {
        let config = Config {
            punctuation_in_quote: false,
            ..Default::default()
        };
        let suffixes = [
            ("colon", ":"),
            ("period", "."),
            ("semicolon", ";"),
            ("exclamation", "!"),
            ("question", "?"),
            ("comma", ","),
        ];
        let delimiters = [
            ("ENDING IN COLON", ": "),
            ("ENDING IN PERIOD", ". "),
            ("ENDING IN SEMICOLON", "; "),
            ("ENDING IN EXCLAMATION", "! "),
            ("ENDING IN QUESTION", "? "),
            ("ENDING IN COMMA", ", "),
        ];

        let mut lines = Vec::new();
        for (heading, delimiter) in delimiters {
            lines.push(heading.to_string());
            for (value, suffix) in suffixes {
                let template = full_monty_template(&config, heading, value, suffix);
                lines.push(citation_to_string(
                    &template,
                    None,
                    None,
                    None,
                    Some(delimiter),
                ));
            }
        }

        let plain = lines.join("\n");
        let expected = r"ENDING IN COLON
“colon”: colon
“period”.: colon
“semicolon”; colon
“exclamation”! colon
“question”? colon
“comma”,: colon
ENDING IN PERIOD
“colon”: period
“period”. period
“semicolon”; period
“exclamation”! period
“question”? period
“comma”,. period
ENDING IN SEMICOLON
“colon”:; semicolon
“period”.; semicolon
“semicolon”; semicolon
“exclamation”!; semicolon
“question”?; semicolon
“comma”,; semicolon
ENDING IN EXCLAMATION
“colon”! exclamation
“period”.! exclamation
“semicolon”! exclamation
“exclamation”! exclamation
“question”?! exclamation
“comma”,! exclamation
ENDING IN QUESTION
“colon”? question
“period”.? question
“semicolon”? question
“exclamation”!? question
“question”? question
“comma”,? question
ENDING IN COMMA
“colon”:, comma
“period”., comma
“semicolon”;, comma
“exclamation”!, comma
“question”?, comma
“comma”, comma";

        let mut typst_lines = Vec::new();
        for (heading, delimiter) in delimiters {
            typst_lines.push(heading.to_string());
            for (value, suffix) in suffixes {
                let template = full_monty_template(&config, heading, value, suffix);
                typst_lines.push(citation_to_string_with_format::<Typst>(
                    &template,
                    None,
                    None,
                    None,
                    Some(delimiter),
                ));
            }
        }
        let typst = typst_lines.join("\n");

        assert_eq!(plain, expected);
        assert_eq!(typst, expected);
    }

    fn full_monty_template(
        config: &Config,
        heading: &str,
        value: &str,
        suffix: &str,
    ) -> Vec<ProcTemplateComponent> {
        vec![
            ProcTemplateComponent {
                template_component: TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    rendering: Rendering {
                        quote: Some(true),
                        suffix: Some(suffix.to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                template_index: None,
                value: value.to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config.clone().into()),
                bibliography_config: None,
                url: None,
                item_language: None,
                quote_marks: Default::default(),
                sentence_initial: false,
                pre_formatted: false,
            },
            ProcTemplateComponent {
                template_component: TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                template_index: None,
                value: {
                    #[allow(
                        clippy::string_slice,
                        reason = "heading is guaranteed to start with prefix"
                    )]
                    let val = heading["ENDING IN ".len()..].to_ascii_lowercase();
                    val
                },
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config.clone().into()),
                bibliography_config: None,
                url: None,
                item_language: None,
                quote_marks: Default::default(),
                sentence_initial: false,
                pre_formatted: false,
            },
        ]
    }
}
