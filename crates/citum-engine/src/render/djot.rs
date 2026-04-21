/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Djot output format.

use super::format::OutputFormat;
use citum_schema::template::WrapPunctuation;

#[derive(Default, Clone)]
/// Renders processed citations and bibliography entries as Djot markup.
pub struct Djot;

impl OutputFormat for Djot {
    type Output = String;

    fn text(&self, s: &str) -> Self::Output {
        // No escaping for Djot as requested.
        s.to_string()
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
        format!("_{content}_")
    }

    fn strong(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("*{content}*")
    }

    fn small_caps(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("[{content}]{{.small-caps}}")
    }

    fn superscript(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("[{content}]{{.superscript}}")
    }

    fn quote(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("\u{201C}{content}\u{201D}")
    }

    fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{prefix}{content}{suffix}")
    }

    fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{prefix}{content}{suffix}")
    }

    fn wrap_punctuation(&self, wrap: &WrapPunctuation, content: Self::Output) -> Self::Output {
        match wrap {
            WrapPunctuation::Parentheses => format!("({content})"),
            WrapPunctuation::Brackets => format!("[{content}]"),
            WrapPunctuation::Quotes => format!("\u{201C}{content}\u{201D}"),
        }
    }

    fn semantic(&self, class: &str, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("[{content}]{{.{class}}}")
    }

    fn link(&self, url: &str, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("[{content}]({url})")
    }

    fn entry(
        &self,
        _id: &str,
        content: Self::Output,
        url: Option<&str>,
        _metadata: &super::format::ProcEntryMetadata,
    ) -> Self::Output {
        if let Some(u) = url {
            self.link(u, content)
        } else {
            content
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_djot_emph() {
        let fmt = Djot;

        for (input, expected) in [("", ""), ("text", "_text_")] {
            assert_eq!(fmt.emph(input.to_string()), expected);
        }
    }

    #[test]
    fn test_djot_strong() {
        let fmt = Djot;

        for (input, expected) in [("", ""), ("text", "*text*")] {
            assert_eq!(fmt.strong(input.to_string()), expected);
        }
    }

    #[test]
    fn test_djot_small_caps() {
        let fmt = Djot;

        for (input, expected) in [("", ""), ("text", "[text]{.small-caps}")] {
            assert_eq!(fmt.small_caps(input.to_string()), expected);
        }
    }

    #[test]
    fn test_djot_quote() {
        let fmt = Djot;

        for (input, expected) in [("", ""), ("text", "\u{201C}text\u{201D}")] {
            assert_eq!(fmt.quote(input.to_string()), expected);
        }
    }

    #[test]
    fn test_djot_semantic() {
        let fmt = Djot;

        for (input, class, expected) in [("", "author", ""), ("text", "author", "[text]{.author}")]
        {
            assert_eq!(fmt.semantic(class, input.to_string()), expected);
        }
    }

    #[test]
    fn test_djot_link() {
        let fmt = Djot;

        for (input, url, expected) in [
            ("", "https://example.com", ""),
            ("text", "https://example.com", "[text](https://example.com)"),
        ] {
            assert_eq!(fmt.link(url, input.to_string()), expected);
        }
    }

    #[test]
    fn test_djot_wrap_punctuation() {
        let fmt = Djot;

        for (wrap, input, expected) in [
            (WrapPunctuation::Parentheses, "text", "(text)"),
            (WrapPunctuation::Brackets, "text", "[text]"),
            (WrapPunctuation::Quotes, "text", "\u{201C}text\u{201D}"),
        ] {
            assert_eq!(fmt.wrap_punctuation(&wrap, input.to_string()), expected);
        }
    }
}
