/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Plain text output format.

use super::format::{OutputFormat, QuoteMarks, realize_wrap};
use crate::values::ScriptClass;
use citum_schema::template::WrapPunctuation;

#[derive(Default, Clone)]
/// Renders processed citations and bibliography entries as plain text.
pub struct PlainText;

impl OutputFormat for PlainText {
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
        format!("**{content}**")
    }

    fn small_caps(&self, content: Self::Output) -> Self::Output {
        content.to_uppercase()
    }

    fn superscript(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("^{content}^")
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

    fn semantic(&self, _class: &str, content: Self::Output) -> Self::Output {
        // Plain text ignores semantic classes
        content
    }

    fn annotation(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }

        format!("\n\n{content}")
    }

    fn link(&self, _url: &str, content: Self::Output) -> Self::Output {
        // Plain text just renders the text content of the link
        content
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
mod tests {
    use super::*;

    #[test]
    fn small_caps_preserves_empty_text() {
        let fmt = PlainText;

        assert_eq!(fmt.small_caps(String::new()), "");
    }

    #[test]
    fn small_caps_uppercases_plain_text() {
        let fmt = PlainText;

        assert_eq!(
            fmt.small_caps("Smith and Lumière".to_string()),
            "SMITH AND LUMIÈRE"
        );
    }
}
