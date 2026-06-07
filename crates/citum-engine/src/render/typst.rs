/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
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

    /// Return the length of the longest consecutive backtick run in `s`.
    fn longest_backtick_run(s: &str) -> usize {
        let mut max = 0usize;
        let mut cur = 0usize;
        for ch in s.chars() {
            if ch == '`' {
                cur += 1;
                if cur > max {
                    max = cur;
                }
            } else {
                cur = 0;
            }
        }
        max
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
        format!("#emph[{content}]")
    }

    fn strong(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("#strong[{content}]")
    }

    fn small_caps(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("#smallcaps[{content}]")
    }

    fn superscript(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("#super[{content}]")
    }

    fn quote(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("\u{201C}{content}\u{201D}")
    }

    fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{}{}{}", self.text(prefix), content, self.text(suffix))
    }

    fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{}{}{}", self.text(prefix), content, self.text(suffix))
    }

    fn wrap_punctuation(&self, wrap: &WrapPunctuation, content: Self::Output) -> Self::Output {
        match wrap {
            WrapPunctuation::Parentheses => format!("({content})"),
            WrapPunctuation::Brackets => format!("[{content}]"),
            WrapPunctuation::Quotes => format!("\u{201C}{content}\u{201D}"),
        }
    }

    fn semantic(&self, _class: &str, content: Self::Output) -> Self::Output {
        content
    }

    fn annotation(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("\n#block(class: \"citum-annotation\")[{}]", content)
    }

    fn citation(&self, ids: Vec<String>, content: Self::Output) -> Self::Output {
        if content.is_empty() || ids.len() != 1 {
            return content;
        }

        #[allow(clippy::unwrap_used, reason = "length checked")]
        let id = ids.first().unwrap();
        format!("#link(<{}>)[{}]", self.format_id(id), content)
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

    // ── Block-level body markup methods ────────────────────────────────────

    fn paragraph(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("{content}\n\n")
    }

    fn block_quote(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        let trimmed = content.trim_end();
        format!("#quote(block: true)[\n{trimmed}\n]\n\n")
    }

    fn bullet_list(&self, items: Vec<Self::Output>) -> Self::Output {
        if items.is_empty() {
            return String::new();
        }
        let body = items
            .iter()
            .map(|item| format!("- {}", item.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{body}\n\n")
    }

    fn ordered_list(&self, items: Vec<Self::Output>) -> Self::Output {
        if items.is_empty() {
            return String::new();
        }
        let body = items
            .iter()
            .map(|item| format!("+ {}", item.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{body}\n\n")
    }

    fn heading(&self, level: u8, content: Self::Output) -> Self::Output {
        let marks = "=".repeat(level.max(1) as usize);
        format!("{marks} {content}\n\n")
    }

    fn code_block(&self, lang: Option<&str>, content: Self::Output) -> Self::Output {
        let fence = "`".repeat(Self::longest_backtick_run(&content).max(2) + 1);
        let lang_tag = lang.unwrap_or("");
        format!("{fence}{lang_tag}\n{content}{fence}\n\n")
    }

    fn inline_code(&self, content: Self::Output) -> Self::Output {
        let ticks = "`".repeat(Self::longest_backtick_run(&content) + 1);
        format!("{ticks}{content}{ticks}")
    }

    fn strikeout(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("#strike[{content}]")
    }

    fn hard_break(&self) -> Self::Output {
        "\\\n".to_string()
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
