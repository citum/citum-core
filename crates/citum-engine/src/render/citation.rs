/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use crate::render::component::{ProcTemplate, render_component_with_format};
use crate::render::format::OutputFormat;
use crate::render::plain::PlainText;
use citum_schema::template::WrapPunctuation;

/// Render a processed template into a final citation string using PlainText format.
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
            if punctuation_in_quote
                && delim.starts_with(',')
                && (content.ends_with('"') || content.ends_with('\u{201D}'))
            {
                let is_curly = content.ends_with('\u{201D}');
                content.pop();
                content.push(',');
                content.push(if is_curly { '\u{201D}' } else { '"' });
                content.push_str(&delim[1..]);
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

    format!("{}{}{}", open, content, close)
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
}
