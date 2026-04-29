/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! HTML output format.

use super::format::{OutputFormat, SemanticAttribute};
use citum_schema::template::WrapPunctuation;
use std::fmt::Write;

#[derive(Default, Clone)]
/// Renders processed citations and bibliography entries as HTML fragments.
pub struct Html;

impl Html {
    fn sanitize_href(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for ch in value.chars() {
            if ch.is_ascii_control()
                || ch.is_whitespace()
                || matches!(ch, '"' | '\'' | '<' | '>' | '&')
            {
                let mut buf = [0u8; 4];
                for byte in ch.encode_utf8(&mut buf).as_bytes() {
                    escaped.push('%');
                    escaped.push_str(&format!("{byte:02X}"));
                }
            } else {
                escaped.push(ch);
            }
        }
        escaped
    }

    fn escape_attribute_value(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

impl OutputFormat for Html {
    type Output = String;

    fn text(&self, s: &str) -> Self::Output {
        // As requested, we avoid escaping and use raw Unicode.
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
        format!("<i>{content}</i>")
    }

    fn strong(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("<b>{content}</b>")
    }

    fn small_caps(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!(r#"<span style="font-variant:small-caps">{content}</span>"#)
    }

    fn superscript(&self, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!("<sup>{content}</sup>")
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
        format!(r#"<span class="{class}">{content}</span>"#)
    }

    fn semantic_with_attributes(
        &self,
        class: &str,
        content: Self::Output,
        attributes: &[SemanticAttribute],
    ) -> Self::Output {
        if content.is_empty() {
            return content;
        }

        let mut extra_attrs = String::new();
        for attribute in attributes {
            let _ = write!(
                &mut extra_attrs,
                r#" {}="{}""#,
                attribute.name,
                Self::escape_attribute_value(&attribute.value)
            );
        }

        format!(r#"<span class="{class}"{extra_attrs}>{content}</span>"#)
    }

    fn citation(&self, ids: Vec<String>, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        let ids_str = ids.join(" ");
        format!(r#"<span class="citum-citation" data-ref="{ids_str}">{content}</span>"#)
    }

    fn link(&self, url: &str, content: Self::Output) -> Self::Output {
        if content.is_empty() {
            return content;
        }
        format!(r#"<a href="{}">{}</a>"#, Self::sanitize_href(url), content)
    }

    fn format_id(&self, id: &str) -> String {
        format!("ref-{id}")
    }

    fn bibliography(&self, entries: Vec<Self::Output>) -> Self::Output {
        format!(
            r#"<div class="citum-bibliography">
{}
</div>"#,
            self.join(entries, "\n")
        )
    }

    fn entry(
        &self,
        id: &str,
        content: Self::Output,
        url: Option<&str>,
        metadata: &super::format::ProcEntryMetadata,
    ) -> Self::Output {
        let content = if let Some(u) = url {
            self.link(u, content)
        } else {
            content
        };

        let mut attrs = format!(
            r#"id="{}""#,
            Self::escape_attribute_value(&self.format_id(id))
        );
        if let Some(author) = &metadata.author {
            attrs.push_str(r#" data-author=""#);
            attrs.push_str(&Self::escape_attribute_value(author));
            attrs.push('"');
        }
        if let Some(year) = &metadata.year {
            attrs.push_str(r#" data-year=""#);
            attrs.push_str(&Self::escape_attribute_value(year));
            attrs.push('"');
        }
        if let Some(title) = &metadata.title {
            attrs.push_str(r#" data-title=""#);
            attrs.push_str(&Self::escape_attribute_value(title));
            attrs.push('"');
        }

        format!(r#"<div class="citum-entry" {attrs}>{content}</div>"#)
    }
}
