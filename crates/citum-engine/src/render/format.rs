/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Output format trait for pluggable renderers.

use citum_schema::template::WrapPunctuation;

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

    /// Render content enclosed in quotation marks.
    fn quote(&self, content: Self::Output) -> Self::Output;

    /// Apply outer prefix and suffix strings to the content.
    ///
    /// These are typically the "prefix" and "suffix" fields from the Citum style.
    fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output;

    /// Apply inner prefix and suffix strings to the content.
    ///
    /// These are applied inside any wrapping punctuation.
    fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output;

    /// Wrap the content in specific punctuation (parentheses, brackets, or quotes).
    fn wrap_punctuation(&self, wrap: &WrapPunctuation, content: Self::Output) -> Self::Output;

    /// Apply a semantic identifier (class) to the content.
    ///
    /// This is used for machine readability or fine-grained CSS styling.
    /// Examples include "citum-title", "citum-author", "citum-doi".
    fn semantic(&self, class: &str, content: Self::Output) -> Self::Output;

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
        fn quote(&self, content: Self::Output) -> Self::Output {
            format!("quote({content})")
        }
        fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
            format!("{prefix}{content}{suffix}")
        }
        fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
            format!("{prefix}{content}{suffix}")
        }
        fn wrap_punctuation(&self, _wrap: &WrapPunctuation, content: Self::Output) -> Self::Output {
            content
        }
        fn semantic(&self, class: &str, content: Self::Output) -> Self::Output {
            format!("sem[{class}]({content})")
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
