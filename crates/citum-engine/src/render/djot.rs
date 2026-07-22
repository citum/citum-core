/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Djot output format.

use std::ops::Range;

use super::format::{OutputFormat, QuoteMarks, realize_wrap};
use super::visible_scan::{RunBuilder, find_matching, skip_balanced};
use crate::values::ScriptClass;
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

    fn heading(&self, level: u8, content: Self::Output) -> Self::Output {
        let marks = "#".repeat(level.max(1) as usize);
        format!("{marks} {content}\n\n")
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
        script: ScriptClass,
        realization: Option<&citum_schema::options::PunctuationRealization>,
    ) -> Self::Output {
        match realize_wrap(wrap, script, realization) {
            Some((open, close)) => {
                format!("{}{}{}", self.text(&open), content, self.text(&close))
            }
            None => self.quote(content, marks),
        }
    }

    fn semantic(&self, class: &str, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("[{content}]{{.{class}}}")
    }

    fn annotation(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("\n\n::: citum-annotation\n{content}\n:::")
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

    /// Strip `_`/`*` emphasis delimiters and a `[content]{.class}` span's or
    /// `[content](url)` link's bracket-plus-attribute markup, keeping the
    /// bracketed content visible. A bracket pair followed by neither `{` nor
    /// `(` (i.e. from [`Self::wrap_punctuation`]'s `WrapPunctuation::Brackets`)
    /// keeps its brackets visible, since those are house-style punctuation.
    ///
    /// Djot's [`Self::text`] does not escape its input ("no escaping for
    /// Djot as requested"), so a data field containing a literal `_`, `*`,
    /// `[`, or `]` is inherently ambiguous with markup here — the same
    /// ambiguity the Djot renderer itself already has.
    fn visible_runs(&self, fragment: &str) -> Vec<Range<usize>> {
        let mut runs = RunBuilder::default();
        let chars: Vec<(usize, char)> = fragment.char_indices().collect();
        let mut i = 0;
        let mut pending_close: Option<usize> = None;
        while let Some(&(pos, ch)) = chars.get(i) {
            match ch {
                '_' | '*' => i += 1,
                '[' => {
                    let close_i = find_matching(&chars, i, '[', ']', false);
                    let after = close_i.and_then(|c| chars.get(c + 1).map(|&(_, next)| next));
                    if matches!(after, Some('{' | '(')) {
                        pending_close = close_i;
                        i += 1;
                    } else {
                        runs.push_visible(pos, pos + 1);
                        i += 1;
                    }
                }
                ']' => {
                    if pending_close == Some(i) {
                        pending_close = None;
                        let mut j = i + 1;
                        match chars.get(j).map(|&(_, c)| c) {
                            Some('{') => j = skip_balanced(&chars, j, '{', '}', false),
                            Some('(') => j = skip_balanced(&chars, j, '(', ')', false),
                            _ => {}
                        }
                        i = j;
                    } else {
                        runs.push_visible(pos, pos + 1);
                        i += 1;
                    }
                }
                _ => {
                    runs.push_visible(pos, pos + ch.len_utf8());
                    i += 1;
                }
            }
        }
        runs.finish()
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
        let marks = QuoteMarks::default();

        for (input, expected) in [("", ""), ("text", "\u{201C}text\u{201D}")] {
            assert_eq!(fmt.quote(input.to_string(), &marks), expected);
        }
    }

    #[test]
    fn test_djot_quote_uses_locale_marks() {
        let fmt = Djot;
        let marks = QuoteMarks {
            open: "\u{ab}".to_string(),
            close: "\u{bb}".to_string(),
            open_inner: "\u{2039}".to_string(),
            close_inner: "\u{203a}".to_string(),
            punctuation_realization: None,
        };

        assert_eq!(fmt.quote("text".to_string(), &marks), "\u{ab}text\u{bb}");
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
        let marks = QuoteMarks::default();

        for (wrap, script, input, expected) in [
            (
                WrapPunctuation::Parentheses,
                ScriptClass::Latin,
                "text",
                "(text)",
            ),
            (
                WrapPunctuation::Brackets,
                ScriptClass::Latin,
                "text",
                "[text]",
            ),
            (
                WrapPunctuation::Quotes,
                ScriptClass::Latin,
                "text",
                "\u{201C}text\u{201D}",
            ),
            (
                WrapPunctuation::Parentheses,
                ScriptClass::Cjk,
                "text",
                "\u{ff08}text\u{ff09}",
            ),
            (
                WrapPunctuation::Brackets,
                ScriptClass::Cjk,
                "text",
                "\u{3010}text\u{3011}",
            ),
        ] {
            assert_eq!(
                fmt.wrap_punctuation(&wrap, input.to_string(), &marks, script, None),
                expected
            );
        }
    }

    #[test]
    fn visible_text_strips_emph_and_strong_delimiters() {
        let fmt = Djot;
        assert_eq!(fmt.visible_text("_Title._"), "Title.");
        assert_eq!(fmt.visible_text("*Title.*"), "Title.");
    }

    #[test]
    fn visible_text_strips_semantic_span_attributes() {
        let fmt = Djot;
        assert_eq!(fmt.visible_text("[Smith]{.author}"), "Smith");
    }

    #[test]
    fn visible_text_hides_link_url_keeps_text() {
        let fmt = Djot;
        assert_eq!(
            fmt.visible_text("[Example](https://example.com/a.b)"),
            "Example"
        );
    }

    #[test]
    fn visible_text_keeps_literal_wrap_brackets_visible() {
        let fmt = Djot;
        // WrapPunctuation::Brackets: bare `[content]`, no `{...}`/`(...)` follows.
        assert_eq!(fmt.visible_text("[Dataset]"), "[Dataset]");
    }
}
