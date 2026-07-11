/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Output format trait for pluggable renderers.

use std::borrow::Cow;
use std::ops::Range;

use citum_schema::locale::GrammarOptions;
use citum_schema::template::WrapPunctuation;

/// Return Unicode quote marks for a nesting depth.
///
/// Even depths use outer double quotes; odd depths use inner single quotes.
#[must_use]
pub fn unicode_quote_marks(depth: usize) -> (&'static str, &'static str) {
    if depth.is_multiple_of(2) {
        ("\u{201C}", "\u{201D}")
    } else {
        ("\u{2018}", "\u{2019}")
    }
}

/// Locale-resolved quote mark characters, threaded from
/// [`GrammarOptions`](citum_schema::locale::GrammarOptions) through to rendering so that
/// styles using non-English quotation conventions (e.g. fr-FR guillemets) render correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteMarks {
    /// Opening outer quotation mark.
    pub open: String,
    /// Closing outer quotation mark.
    pub close: String,
    /// Opening inner (nested) quotation mark.
    pub open_inner: String,
    /// Closing inner (nested) quotation mark.
    pub close_inner: String,
}

impl QuoteMarks {
    /// Return the opening and closing quote delimiters for a nesting depth.
    ///
    /// Depth 0 (and other even depths) use the outer pair; odd depths use the inner pair.
    #[must_use]
    pub fn for_depth(&self, depth: usize) -> (&str, &str) {
        if depth.is_multiple_of(2) {
            (&self.open, &self.close)
        } else {
            (&self.open_inner, &self.close_inner)
        }
    }
}

impl Default for QuoteMarks {
    /// The historical hardcoded English fallback, used when no resolved locale is available.
    fn default() -> Self {
        let (open, close) = unicode_quote_marks(0);
        let (open_inner, close_inner) = unicode_quote_marks(1);
        Self {
            open: open.to_string(),
            close: close.to_string(),
            open_inner: open_inner.to_string(),
            close_inner: close_inner.to_string(),
        }
    }
}

impl From<&GrammarOptions> for QuoteMarks {
    fn from(options: &GrammarOptions) -> Self {
        Self {
            open: options.open_quote.clone(),
            close: options.close_quote.clone(),
            open_inner: options.open_inner_quote.clone(),
            close_inner: options.close_inner_quote.clone(),
        }
    }
}

/// Extra attributes applied to semantic wrappers when a renderer supports them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticAttribute {
    /// The attribute name.
    pub name: &'static str,
    /// The attribute value.
    pub value: String,
}

/// Trait for defining how to render template components into a specific format.
///
/// Implementations of this trait define how various formatting instructions
/// (emphasis, quotes, links, etc.) are translated into specific markup or text.
pub trait OutputFormat: Default + Clone {
    /// The type used for intermediate rendered content.
    ///
    /// For simple text formats, this is usually `String`. More complex formats
    /// might use an AST or a specialized builder type.
    type Output;

    /// Convert a raw string into the format's output type.
    ///
    /// The implementation should handle any necessary character escaping
    /// required by the target format.
    fn text(&self, s: &str) -> Self::Output;

    /// Join multiple outputs into a single output using a delimiter.
    fn join(&self, items: Vec<Self::Output>, delimiter: &str) -> Self::Output;

    /// Convert the intermediate output into the final result string.
    ///
    /// This is called exactly once at the end of the rendering process
    /// for a top-level component (citation or bibliography entry).
    fn finish(&self, output: Self::Output) -> String;

    /// Render content with emphasis (typically italics).
    fn emph(&self, content: Self::Output) -> Self::Output;

    /// Render content with strong emphasis (typically bold).
    fn strong(&self, content: Self::Output) -> Self::Output;

    /// Render content in small capitals.
    fn small_caps(&self, content: Self::Output) -> Self::Output;

    /// Render content as superscript text.
    fn superscript(&self, content: Self::Output) -> Self::Output;

    /// Return the opening and closing quote delimiters for a nesting depth.
    ///
    /// Depth 0 is an outer quote pair, depth 1 is the first inner quote pair,
    /// and deeper levels alternate between those two pairs. `marks` carries the
    /// locale-resolved quote characters; callers with no resolved locale can pass
    /// `&QuoteMarks::default()` to keep the historical English fallback.
    fn quote_marks<'a>(&self, depth: usize, marks: &'a QuoteMarks) -> (&'a str, &'a str) {
        marks.for_depth(depth)
    }

    /// Render content enclosed in quotation marks at a specific nesting depth.
    fn quote_with_depth(
        &self,
        content: Self::Output,
        depth: usize,
        marks: &QuoteMarks,
    ) -> Self::Output {
        let (open, close) = self.quote_marks(depth, marks);
        self.affix(open, content, close)
    }

    /// Render content enclosed in outer quotation marks.
    fn quote(&self, content: Self::Output, marks: &QuoteMarks) -> Self::Output {
        self.quote_with_depth(content, 0, marks)
    }

    /// Apply outer prefix and suffix strings to the content.
    ///
    /// These are typically the "prefix" and "suffix" fields from the Citum style.
    fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output;

    /// Apply inner prefix and suffix strings to the content.
    ///
    /// These are applied inside any wrapping punctuation.
    fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output;

    /// Wrap the content in specific punctuation (parentheses, brackets, or quotes).
    ///
    /// `marks` supplies the locale-resolved quote characters for the `Quotes` variant.
    fn wrap_punctuation(
        &self,
        wrap: &WrapPunctuation,
        content: Self::Output,
        marks: &QuoteMarks,
    ) -> Self::Output;

    /// Apply a semantic identifier (class) to the content.
    ///
    /// This is used for machine readability or fine-grained CSS styling.
    /// Examples include "citum-title", "citum-author", "citum-doi".
    fn semantic(&self, class: &str, content: Self::Output) -> Self::Output;

    /// Render an annotation block.
    ///
    /// This is typically called at the end of a bibliography entry to render
    /// reader-supplied notes.
    fn annotation(&self, content: Self::Output) -> Self::Output;

    // ── Block-level methods (used by the body markup renderer) ─────────────
    // Defaults produce plain passthrough so existing format impls need not change.

    /// Render a paragraph block.
    fn paragraph(&self, content: Self::Output) -> Self::Output {
        content
    }

    /// Render a block quotation.
    fn block_quote(&self, content: Self::Output) -> Self::Output {
        content
    }

    /// Render an unordered (bullet) list from pre-rendered item strings.
    fn bullet_list(&self, items: Vec<Self::Output>) -> Self::Output {
        self.join(items, "\n")
    }

    /// Render an ordered (numbered) list from pre-rendered item strings.
    fn ordered_list(&self, items: Vec<Self::Output>) -> Self::Output {
        self.join(items, "\n")
    }

    /// Render a list item.
    fn list_item(&self, content: Self::Output) -> Self::Output {
        content
    }

    /// Render a heading at the given level (1 = top-level).
    fn heading(&self, _level: u8, content: Self::Output) -> Self::Output {
        content
    }

    /// Render an unnumbered heading at the given level.
    ///
    /// Used for generated section headings (e.g. bibliography group
    /// headings) that must not participate in document section numbering.
    /// Defaults to [`Self::heading`]; formats with numbered headings
    /// (LaTeX) override this with their unnumbered variants.
    fn unnumbered_heading(&self, level: u8, content: Self::Output) -> Self::Output {
        self.heading(level, content)
    }

    /// Render a fenced or indented code block with an optional language hint.
    ///
    /// `content` is the raw (unescaped) code text.
    fn code_block(&self, _lang: Option<&str>, content: Self::Output) -> Self::Output {
        content
    }

    /// Render inline code.
    fn inline_code(&self, content: Self::Output) -> Self::Output {
        content
    }

    /// Render strikethrough text.
    fn strikeout(&self, content: Self::Output) -> Self::Output {
        content
    }

    /// Render a hard line break.
    fn hard_break(&self) -> Self::Output {
        self.text(" ")
    }

    /// Apply a semantic identifier plus optional attributes to the content.
    ///
    /// Formats that do not support extra attributes can ignore them and reuse
    /// [`Self::semantic`].
    fn semantic_with_attributes(
        &self,
        class: &str,
        content: Self::Output,
        _attributes: &[SemanticAttribute],
    ) -> Self::Output {
        self.semantic(class, content)
    }

    /// Render a full citation container with one or more reference IDs.
    fn citation(&self, _ids: Vec<String>, content: Self::Output) -> Self::Output {
        content
    }

    // ── Visible-text methods ────────────────────────────────────────────────
    // Used by bibliography/citation punctuation-boundary logic so separator
    // and dedup decisions look at logical text, not backend markup (the
    // "backends differ only in markup" rule — see DESIGN_PRINCIPLES §7).

    /// Byte ranges of `fragment` that are visible (non-markup) text, in order.
    ///
    /// The default treats the whole fragment as visible, which is correct
    /// for [`PlainText`](crate::render::plain::PlainText) and safe for any
    /// third-party format that hasn't implemented a lexer: boundary logic
    /// simply falls back to looking at raw characters, as it always has.
    /// Backends whose inline methods (`emph`, `link`, `wrap_punctuation`,
    /// ...) emit markup should override this to exclude it.
    fn visible_runs(&self, fragment: &str) -> Vec<Range<usize>> {
        let mut runs = Vec::new();
        if !fragment.is_empty() {
            runs.push(0..fragment.len());
        }
        runs
    }

    /// The visible (markup-stripped) text of a rendered fragment.
    ///
    /// Borrows `fragment` unchanged when it is entirely visible (the common
    /// case); otherwise stitches the visible runs into an owned `String`.
    fn visible_text<'a>(&self, fragment: &'a str) -> Cow<'a, str> {
        let runs = self.visible_runs(fragment);
        if runs.len() == 1 && runs.first() == Some(&(0..fragment.len())) {
            return Cow::Borrowed(fragment);
        }
        let mut owned = String::with_capacity(fragment.len());
        for run in runs {
            if let Some(slice) = fragment.get(run) {
                owned.push_str(slice);
            }
        }
        Cow::Owned(owned)
    }

    /// Hyperlink the content to a URL.
    fn link(&self, url: &str, content: Self::Output) -> Self::Output;

    /// Format a reference ID for use as a target or link (e.g. adding a prefix).
    fn format_id(&self, id: &str) -> String {
        id.to_string()
    }

    /// Render a full bibliography container.
    ///
    /// The default implementation joins the entries with double newlines.
    fn bibliography(&self, entries: Vec<Self::Output>) -> Self::Output {
        self.join(entries, "\n\n")
    }

    /// Render a single bibliography entry with its unique identifier and optional link.
    ///
    /// The default implementation just returns the content.
    fn entry(
        &self,
        _id: &str,
        content: Self::Output,
        _url: Option<&str>,
        _metadata: &ProcEntryMetadata,
    ) -> Self::Output {
        content
    }
}

/// Metadata for a processed bibliography entry, used for interactivity.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcEntryMetadata {
    /// Rendered primary author(s) string.
    pub author: Option<String>,
    /// Rendered year string.
    pub year: Option<String>,
    /// Rendered title string.
    pub title: Option<String>,
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

    #[derive(Default, Clone)]
    struct DummyFormat;

    impl OutputFormat for DummyFormat {
        type Output = String;
        fn text(&self, s: &str) -> Self::Output {
            s.to_string()
        }
        fn join(&self, items: Vec<Self::Output>, delimiter: &str) -> Self::Output {
            items.join(delimiter)
        }
        fn finish(&self, output: Self::Output) -> String {
            output
        }
        fn emph(&self, content: Self::Output) -> Self::Output {
            format!("emph({content})")
        }
        fn strong(&self, content: Self::Output) -> Self::Output {
            format!("strong({content})")
        }
        fn small_caps(&self, content: Self::Output) -> Self::Output {
            format!("sc({content})")
        }
        fn superscript(&self, content: Self::Output) -> Self::Output {
            format!("sup({content})")
        }
        fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
            format!("{prefix}{content}{suffix}")
        }
        fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
            format!("{prefix}{content}{suffix}")
        }
        fn wrap_punctuation(
            &self,
            _wrap: &WrapPunctuation,
            content: Self::Output,
            _marks: &QuoteMarks,
        ) -> Self::Output {
            content
        }
        fn semantic(&self, class: &str, content: Self::Output) -> Self::Output {
            format!("sem[{class}]({content})")
        }
        fn annotation(&self, content: Self::Output) -> Self::Output {
            format!("annot({content})")
        }
        fn link(&self, url: &str, content: Self::Output) -> Self::Output {
            format!("link[{url}]({content})")
        }
    }

    #[test]
    fn test_default_methods() {
        let fmt = DummyFormat;
        assert_eq!(
            fmt.semantic_with_attributes("test", "content".to_string(), &[]),
            "sem[test](content)"
        );
        assert_eq!(
            fmt.citation(vec!["id1".to_string()], "content".to_string()),
            "content"
        );
        assert_eq!(fmt.format_id("id1"), "id1");
        assert_eq!(
            fmt.bibliography(vec!["entry1".to_string(), "entry2".to_string()]),
            "entry1\n\nentry2"
        );
        assert_eq!(
            fmt.entry(
                "id1",
                "content".to_string(),
                None,
                &ProcEntryMetadata::default()
            ),
            "content"
        );
    }
}
