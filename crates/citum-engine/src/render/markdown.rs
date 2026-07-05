/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! CommonMark (Markdown) output format.
//!
//! This renderer is designed for **Pandoc interop**: citum processes citations
//! inline (replacing `[@key]` markers with rendered text) and emits the document
//! body verbatim, so the output can be piped directly to `pandoc` or any other
//! CommonMark-aware formatter. Only citation and bibliography strings are
//! rendered in CommonMark markup; block-level document markup passes through
//! unchanged.
//!
//! # Note styles
//!
//! Note-based styles (Chicago notes, etc.) emit `[^label]` anchors in prose and
//! `[^label]: …` footnote definitions at the end of the document. These follow
//! the Pandoc/GFM footnote extension — **not** core CommonMark. Downstream
//! consumers must enable the extension:
//! `pandoc --from commonmark+footnotes` (or `--from gfm`).

use super::format::{OutputFormat, QuoteMarks};
use citum_schema::template::WrapPunctuation;

/// Escape CommonMark-active characters in raw bibliography data text.
///
/// Backslash-escapes `\`, `*`, `_`, `[`, `]`, `` ` ``, `<`, `>`, and `&`
/// so that data fields (titles, author names, etc.) cannot accidentally
/// activate emphasis, strong, link, code-span, autolink, inline HTML, or
/// HTML-entity syntax. Style-applied markup (`emph`, `strong`, `link`)
/// wraps already-escaped text, so intentional markup is unaffected.
fn escape_commonmark_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for ch in s.chars() {
        match ch {
            '\\' | '*' | '_' | '[' | ']' | '`' | '<' | '>' | '&' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Renders processed citations and bibliography entries as CommonMark markup.
#[derive(Default, Clone)]
pub struct Markdown;

impl OutputFormat for Markdown {
    type Output = String;

    fn text(&self, s: &str) -> Self::Output {
        escape_commonmark_text(s)
    }

    fn join(&self, items: Vec<Self::Output>, delimiter: &str) -> Self::Output {
        items.join(delimiter)
    }

    fn finish(&self, output: Self::Output) -> String {
        output
    }

    /// Render a heading using ATX syntax (`#`, `##`, ...).
    fn heading(&self, level: u8, content: Self::Output) -> Self::Output {
        let marks = "#".repeat(level.max(1) as usize);
        format!("{marks} {content}\n\n")
    }

    /// Render emphasis as `*content*` (CommonMark italic).
    fn emph(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("*{content}*")
    }

    /// Render strong emphasis as `**content**` (CommonMark bold).
    fn strong(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("**{content}**")
    }

    /// Render small caps as raw inline HTML.
    ///
    /// CommonMark has no native small-caps syntax. Raw `<span>` HTML is passed
    /// through by Pandoc's CommonMark reader and most other processors.
    fn small_caps(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("<span style=\"font-variant:small-caps\">{content}</span>")
    }

    /// Render superscript as raw inline HTML.
    ///
    /// CommonMark has no native superscript syntax. Raw `<sup>` HTML is passed
    /// through by Pandoc and most processors.
    fn superscript(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("<sup>{content}</sup>")
    }

    fn quote(&self, content: Self::Output, marks: &QuoteMarks) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        let (open, close) = marks.for_depth(0);
        format!("{open}{content}{close}")
    }

    fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{prefix}{content}{suffix}")
    }

    fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{prefix}{content}{suffix}")
    }

    fn wrap_punctuation(
        &self,
        wrap: &WrapPunctuation,
        content: Self::Output,
        marks: &QuoteMarks,
    ) -> Self::Output {
        match wrap {
            WrapPunctuation::Parentheses => format!("({content})"),
            WrapPunctuation::Brackets => format!("[{content}]"),
            WrapPunctuation::Quotes => self.quote(content, marks),
        }
    }

    /// Render a semantic class as a plain passthrough.
    ///
    /// CommonMark has no attribute syntax. Content is returned unchanged so
    /// citations remain readable plain text. Use `--format html` or `--format djot`
    /// if machine-readable semantic spans are needed.
    fn semantic(&self, _class: &str, content: Self::Output) -> Self::Output {
        content
    }

    fn annotation(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("\n\n{content}")
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
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests"
)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_emph() {
        let fmt = Markdown;
        for (input, expected) in [("", ""), ("text", "*text*")] {
            assert_eq!(fmt.emph(input.to_string()), expected);
        }
    }

    #[test]
    fn test_markdown_strong() {
        let fmt = Markdown;
        for (input, expected) in [("", ""), ("text", "**text**")] {
            assert_eq!(fmt.strong(input.to_string()), expected);
        }
    }

    #[test]
    fn test_markdown_small_caps() {
        let fmt = Markdown;
        assert_eq!(fmt.small_caps(String::new()), "");
        assert_eq!(
            fmt.small_caps("Smith".to_string()),
            "<span style=\"font-variant:small-caps\">Smith</span>"
        );
    }

    #[test]
    fn test_markdown_superscript() {
        let fmt = Markdown;
        assert_eq!(fmt.superscript(String::new()), "");
        assert_eq!(fmt.superscript("2".to_string()), "<sup>2</sup>");
    }

    #[test]
    fn test_markdown_quote() {
        let fmt = Markdown;
        let marks = QuoteMarks::default();
        for (input, expected) in [("", ""), ("text", "\u{201C}text\u{201D}")] {
            assert_eq!(fmt.quote(input.to_string(), &marks), expected);
        }
    }

    #[test]
    fn test_markdown_quote_uses_locale_marks() {
        let fmt = Markdown;
        let marks = QuoteMarks {
            open: "\u{ab}".to_string(),
            close: "\u{bb}".to_string(),
            open_inner: "\u{2039}".to_string(),
            close_inner: "\u{203a}".to_string(),
        };

        assert_eq!(fmt.quote("text".to_string(), &marks), "\u{ab}text\u{bb}");
    }

    #[test]
    fn test_markdown_semantic_passthrough() {
        let fmt = Markdown;
        assert_eq!(fmt.semantic("author", "Jane Doe".to_string()), "Jane Doe");
        assert_eq!(fmt.semantic("title", String::new()), "");
    }

    #[test]
    fn test_markdown_link() {
        let fmt = Markdown;
        assert_eq!(fmt.link("https://example.com", String::new()), "");
        assert_eq!(
            fmt.link("https://example.com", "Example".to_string()),
            "[Example](https://example.com)"
        );
    }

    #[test]
    fn test_markdown_wrap_punctuation() {
        let fmt = Markdown;
        let marks = QuoteMarks::default();
        for (wrap, input, expected) in [
            (WrapPunctuation::Parentheses, "text", "(text)"),
            (WrapPunctuation::Brackets, "text", "[text]"),
            (WrapPunctuation::Quotes, "text", "\u{201C}text\u{201D}"),
        ] {
            assert_eq!(
                fmt.wrap_punctuation(&wrap, input.to_string(), &marks),
                expected
            );
        }
    }

    #[test]
    fn test_markdown_text_escapes_active_chars() {
        let fmt = Markdown;
        assert_eq!(fmt.text("plain"), "plain");
        assert_eq!(fmt.text("A * B"), "A \\* B");
        assert_eq!(fmt.text("use [x]"), "use \\[x\\]");
        assert_eq!(fmt.text("code `foo`"), "code \\`foo\\`");
        assert_eq!(fmt.text("back\\slash"), "back\\\\slash");
        assert_eq!(fmt.text("under_score"), "under\\_score");
        // Angle brackets and ampersand: escape to prevent autolinks,
        // inline HTML, and HTML entity expansion.
        assert_eq!(fmt.text("<doi:10.1/x>"), "\\<doi:10.1/x\\>");
        assert_eq!(fmt.text("Smith & Jones"), "Smith \\& Jones");
        assert_eq!(fmt.text("<em>bold</em>"), "\\<em\\>bold\\</em\\>");
    }
}
