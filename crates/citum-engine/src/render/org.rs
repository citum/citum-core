/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Org-mode output format.

use super::format::OutputFormat;
use citum_schema::template::WrapPunctuation;

/// Renders processed citations and bibliography entries as org-mode markup.
#[derive(Default, Clone)]
pub struct OrgOutputFormat;

impl OutputFormat for OrgOutputFormat {
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

    /// Render content with emphasis (italics in org-mode: /text/).
    fn emph(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("/{content}/")
    }

    /// Render content with strong emphasis (bold in org-mode: *text*).
    fn strong(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("*{content}*")
    }

    /// Render content in small capitals (org-mode uses ~text~).
    fn small_caps(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("~{content}~")
    }

    fn superscript(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("^{content}^")
    }

    fn quote(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        // Org-mode doesn't have native quotation marks, use as-is with Unicode
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

    fn semantic(&self, _class: &str, content: Self::Output) -> Self::Output {
        // Org-mode doesn't support semantic classes; just return the content
        content
    }

    /// Render a hyperlink in org-mode format: `[[url][text]]`
    fn link(&self, url: &str, content: Self::Output) -> Self::Output {
        format!("[[{url}][{content}]]")
    }

    fn entry(
        &self,
        _id: &str,
        content: Self::Output,
        _url: Option<&str>,
        _metadata: &super::format::ProcEntryMetadata,
    ) -> Self::Output {
        content
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
    fn test_org_emph() {
        let fmt = OrgOutputFormat;
        let result = fmt.emph(fmt.text("italic text"));
        assert_eq!(result, "/italic text/");
    }

    #[test]
    fn test_org_strong() {
        let fmt = OrgOutputFormat;
        let result = fmt.strong(fmt.text("bold text"));
        assert_eq!(result, "*bold text*");
    }

    #[test]
    fn test_org_small_caps() {
        let fmt = OrgOutputFormat;
        let result = fmt.small_caps(fmt.text("small caps"));
        assert_eq!(result, "~small caps~");
    }

    #[test]
    fn test_org_link() {
        let fmt = OrgOutputFormat;
        let result = fmt.link("https://example.com", fmt.text("Example"));
        assert_eq!(result, "[[https://example.com][Example]]");
    }

    #[test]
    fn test_org_empty_content() {
        let fmt = OrgOutputFormat;
        let result = fmt.emph(fmt.text(""));
        assert_eq!(result, "");
    }
}
