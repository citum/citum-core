/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
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
            let delim_first = delim.chars().next();
            let content_last = content.chars().last();

            if punctuation_in_quote
                && delim_first == Some(',')
                && (content.ends_with('"') || content.ends_with('\u{201D}'))
            {
                let is_curly = content.ends_with('\u{201D}');
                content.pop();
                content.push(',');
                content.push(if is_curly { '\u{201D}' } else { '"' });
                content.push_str(&delim[1..]);
            } else if let (Some(last), Some(first)) = (content_last, delim_first)
                && is_terminal_punctuation(last)
                && is_terminal_punctuation(first)
            {
                content.pop();
                content.push_str(&resolve_punctuation_collision(last, first));
                content.push_str(&delim[first.len_utf8()..]);
            } else {
                content.push_str(delim);
            }
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
                value: "Kuhn".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: None,
                url: None,
                item_language: None,
                pre_formatted: false,
            },
            ProcTemplateComponent {
                template_component: TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                value: "1962".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: None,
                url: None,
                item_language: None,
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
                value: "colon".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config.clone()),
                url: None,
                item_language: None,
                pre_formatted: false,
            },
            ProcTemplateComponent {
                template_component: TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                value: "period".to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config),
                url: None,
                item_language: None,
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
                value: value.to_string(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config.clone()),
                url: None,
                item_language: None,
                pre_formatted: false,
            },
            ProcTemplateComponent {
                template_component: TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    form: DateForm::Year,
                    rendering: Rendering::default(),
                    ..Default::default()
                }),
                value: heading["ENDING IN ".len()..].to_ascii_lowercase(),
                prefix: None,
                suffix: None,
                ref_type: None,
                config: Some(config.clone()),
                url: None,
                item_language: None,
                pre_formatted: false,
            },
        ]
    }
}
