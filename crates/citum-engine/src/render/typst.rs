/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Typst output format.

use super::format::OutputFormat;
use citum_schema::template::WrapPunctuation;

/// Typst renderer.
#[derive(Debug, Clone, Default)]
pub struct Typst;

impl Typst {
    fn escape_text(input: &str) -> String {
        let mut escaped = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '\\' => escaped.push_str("\\\\"),
                '#' | '[' | ']' | '<' | '>' | '*' | '_' | '@' | '$' => {
                    escaped.push('\\');
                    escaped.push(ch);
                }
                _ => escaped.push(ch),
            }
        }
        escaped
    }

    fn escape_string(input: &str) -> String {
        let mut escaped = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\\""),
                _ => escaped.push(ch),
            }
        }
        escaped
    }
}

impl OutputFormat for Typst {
    type Output = String;

    fn text(&self, s: &str) -> Self::Output {
        Self::escape_text(s)
    }

    fn join(&self, items: Vec<Self::Output>, delimiter: &str) -> Self::Output {
        items.join(delimiter)
    }

    fn finish(&self, output: Self::Output) -> String {
        output
    }

    fn emph(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("_{}_", content)
    }

    fn strong(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("*{}*", content)
    }

    fn small_caps(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("#smallcaps[{}]", content)
    }

    fn code(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("`{}`", content)
    }

    fn verbatim(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("#raw(\"{}\")", Self::escape_string(&content))
    }

    fn quote(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("\u{201C}{}\u{201D}", content)
    }

    fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{}{}{}", self.text(prefix), content, self.text(suffix))
    }

    fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{}{}{}", self.text(prefix), content, self.text(suffix))
    }

    fn wrap_punctuation(&self, wrap: &WrapPunctuation, content: Self::Output) -> Self::Output {
        match wrap {
            WrapPunctuation::Parentheses => format!("({})", content),
            WrapPunctuation::Brackets => format!("[{}]", content),
            WrapPunctuation::Quotes => format!("\u{201C}{}\u{201D}", content),
            WrapPunctuation::None => content,
        }
    }

    fn semantic(&self, _class: &str, content: Self::Output) -> Self::Output {
        content
    }

    fn citation(&self, ids: Vec<String>, content: Self::Output) -> Self::Output {
        if content.is_empty() || ids.len() != 1 {
            return content;
        }

        format!("#link(<{}>)[{}]", self.format_id(&ids[0]), content)
    }

    fn link(&self, url: &str, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }

        if let Some(label) = url.strip_prefix('#') {
            format!("#link(<{}>)[{}]", self.format_id(label), content)
        } else {
            format!(r#"#link("{}")[{}]"#, Self::escape_string(url), content)
        }
    }

    fn format_id(&self, id: &str) -> String {
        let mut normalized = String::with_capacity(id.len() + 4);
        normalized.push_str("ref-");
        for ch in id.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.') {
                normalized.push(ch);
            } else {
                normalized.push('-');
            }
        }
        normalized
    }

    fn bibliography(&self, entries: Vec<Self::Output>) -> Self::Output {
        self.join(entries, "\n\n")
    }

    fn entry(
        &self,
        id: &str,
        content: Self::Output,
        url: Option<&str>,
        _metadata: &super::format::ProcEntryMetadata,
    ) -> Self::Output {
        let content = if let Some(u) = url {
            self.link(u, content)
        } else {
            content
        };

        format!("{} <{}>", content, self.format_id(id))
    }
}
