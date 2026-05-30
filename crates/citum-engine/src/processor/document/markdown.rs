/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Markdown document parsing for Pandoc-style citations.

use super::djot::parsing::parse_frontmatter;
use super::{CitationParser, CitationPlacement, CitationStructure, ParsedCitation, ParsedDocument};
use crate::{Citation, CitationItem};
use citum_schema::citation::{CitationMode, normalize_locator_text};
use citum_schema::locale::Locale;
use std::collections::HashSet;

/// A parser for Markdown documents with Pandoc-style citation syntax.
///
/// This parser currently supports inline prose citations and maps them into the
/// shared document-processing pipeline. Markdown-specific footnotes, document
/// metadata, and inline bibliography blocks remain future work.
pub struct MarkdownParser;

impl Default for MarkdownParser {
    fn default() -> Self {
        Self
    }
}

impl CitationParser for MarkdownParser {
    /// Convert Markdown body markup to HTML after citation splicing.
    ///
    /// NUL placeholder tokens (`\x00CITUMHTML…TOKEN…\x00`) are temporarily
    /// re-encoded as HTML comments before the Markdown parser runs, because
    /// pulldown-cmark normalises `\x00` to U+FFFD. The comments survive the
    /// conversion verbatim and are swapped back so that the caller's
    /// `HtmlPlaceholderRegistry::apply()` can still locate them.
    fn finalize_html_output(&self, rendered: &str) -> String {
        use pulldown_cmark::{Options, html};

        let (remapped, token_map) = remap_nul_tokens(rendered);
        let parser = pulldown_cmark::Parser::new_ext(&remapped, Options::ENABLE_STRIKETHROUGH);
        let mut out = String::new();
        html::push_html(&mut out, parser);

        // Restore original NUL tokens so HtmlPlaceholderRegistry::apply() works.
        for (comment, original) in token_map {
            out = out.replace(&comment, &original);
        }
        out
    }

    /// Convert Markdown body markup to the target terminal format (Typst, LaTeX)
    /// after citation placeholder tokens have been spliced in.
    fn render_body_markup<F>(&self, body: &str, fmt: &F) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        crate::render::markup::render_markdown_body(body, fmt)
    }

    fn parse_document(&self, content: &str, locale: &Locale) -> ParsedDocument {
        let (frontmatter_result, body) = parse_frontmatter(content);
        let body_start = content.len() - body.len();
        let (frontmatter, frontmatter_error) = match frontmatter_result {
            Ok(fm) => (fm, None),
            Err(e) => (None, Some(e)),
        };
        let frontmatter_options = frontmatter.as_ref().and_then(|fm| fm.options.clone());
        // Legacy top-level fields are superseded by their `options.*` counterparts.
        let frontmatter_integral_name_memory = frontmatter
            .as_ref()
            .and_then(|fm| fm.integral_name_memory.clone())
            .filter(|_| {
                frontmatter_options
                    .as_ref()
                    .and_then(|o| o.integral_name_memory.as_ref())
                    .is_none()
            });
        let frontmatter_org_abbreviation_memory = frontmatter
            .and_then(|fm| fm.org_abbreviation_memory)
            .filter(|_| {
                frontmatter_options
                    .as_ref()
                    .and_then(|o| o.org_abbreviation_memory.as_ref())
                    .is_none()
            });

        let citations = find_citations(body, locale)
            .into_iter()
            .map(|(start, end, citation)| ParsedCitation {
                start: body_start + start,
                end: body_start + end,
                citation,
                placement: CitationPlacement::InlineProse,
                structure: CitationStructure::default(),
            })
            .collect();

        ParsedDocument {
            citations,
            manual_note_order: Vec::new(),
            manual_note_references: Vec::new(),
            manual_note_labels: HashSet::new(),
            bibliography_blocks: Vec::new(),
            frontmatter_groups: None,
            frontmatter_integral_name_memory,
            frontmatter_org_abbreviation_memory,
            frontmatter_options,
            frontmatter_error,
            body_start,
        }
    }
}

#[allow(
    clippy::string_slice,
    clippy::unreachable,
    reason = "Markdown scanning logic"
)]
fn find_citations(content: &str, locale: &Locale) -> Vec<(usize, usize, Citation)> {
    let mut results = Vec::new();
    let mut offset = 0;

    while offset < content.len() {
        let remaining = &content[offset..];
        let next_at = remaining.find('@');
        let next_bracket = remaining.find('[');

        let (relative_start, kind) = match (next_at, next_bracket) {
            (Some(at), Some(bracket)) if bracket <= at => (bracket, ScanKind::Bracket),
            (Some(at), Some(bracket)) if at < bracket => (at, ScanKind::Textual),
            (Some(at), None) => (at, ScanKind::Textual),
            (None, Some(bracket)) => (bracket, ScanKind::Bracket),
            (None, None) => break,
            _ => unreachable!(),
        };

        let start = offset + relative_start;
        let candidate = &content[start..];

        let parsed = match kind {
            ScanKind::Bracket => parse_bracketed_citation(candidate, locale),
            ScanKind::Textual => parse_textual_citation(content, start, locale),
        };

        if let Some((consumed, citation)) = parsed {
            results.push((start, start + consumed, citation));
            offset = start + consumed;
        } else if matches!(kind, ScanKind::Bracket) {
            offset = start + candidate.find(']').map_or(1, |idx| idx + 1);
        } else {
            offset = start + 1;
        }
    }

    results
}

#[derive(Debug, Clone, Copy)]
enum ScanKind {
    Bracket,
    Textual,
}

#[allow(clippy::string_slice, reason = "Brackets and @ are 1-byte ASCII")]
fn parse_bracketed_citation(input: &str, locale: &Locale) -> Option<(usize, Citation)> {
    if !input.starts_with('[') {
        return None;
    }

    let closing = input.find(']')?;
    let inner = input[1..closing].trim();
    if inner.is_empty() || !inner.contains('@') {
        return None;
    }

    let mut items = Vec::new();
    let mut suppress_author = None;

    for segment in inner.split(';') {
        let (item, suppress) = parse_bracketed_item(segment, locale)?;
        if let Some(existing) = suppress_author {
            if existing != suppress {
                return None;
            }
        } else {
            suppress_author = Some(suppress);
        }
        items.push(item);
    }

    Some((
        closing + 1,
        Citation {
            items,
            suppress_author: suppress_author.unwrap_or(false),
            ..Default::default()
        },
    ))
}

#[allow(
    clippy::string_slice,
    clippy::indexing_slicing,
    reason = "Citations are ASCII-heavy; indices from find() are on char boundaries"
)]
fn parse_bracketed_item(segment: &str, locale: &Locale) -> Option<(CitationItem, bool)> {
    let segment = segment.trim();
    let at_pos = segment.find('@')?;
    let mut suppress_author = false;
    let prefix_end = if at_pos > 0 && segment.as_bytes()[at_pos - 1] == b'-' {
        suppress_author = true;
        at_pos - 1
    } else {
        at_pos
    };

    let prefix = normalize_prefix(&segment[..prefix_end]);
    let after_at = &segment[at_pos + 1..];
    let key_end = cite_key_len(after_at)?;
    let key = &after_at[..key_end];
    let remainder = after_at[key_end..].trim_start();

    let mut item = CitationItem {
        id: key.to_string(),
        prefix,
        ..Default::default()
    };

    if let Some(rest) = remainder.strip_prefix(',') {
        let rest = rest.trim();
        if !rest.is_empty() {
            item.locator = normalize_locator_text(rest, locale);
            if item.locator.is_none() {
                item.suffix = Some(rest.to_string());
            }
        }
    } else if !remainder.is_empty() {
        item.suffix = Some(remainder.trim().to_string());
    }

    Some((item, suppress_author))
}

#[allow(clippy::string_slice, reason = "@ and indices from find() are safe")]
fn parse_textual_citation(
    content: &str,
    start: usize,
    locale: &Locale,
) -> Option<(usize, Citation)> {
    if !is_valid_textual_start(content, start) {
        return None;
    }

    let after_at = &content[start + 1..];
    let key_end = cite_key_len(after_at)?;
    let key = &after_at[..key_end];
    let mut consumed = 1 + key_end;

    let mut item = CitationItem {
        id: key.to_string(),
        ..Default::default()
    };

    let trailing = &content[start + consumed..];
    if let Some((locator_consumed, locator)) = parse_textual_locator_suffix(trailing, locale) {
        item.locator = Some(locator);
        consumed += locator_consumed;
    }

    Some((
        consumed,
        Citation {
            mode: CitationMode::Integral,
            items: vec![item],
            ..Default::default()
        },
    ))
}

#[allow(clippy::string_slice, reason = "Brackets and @ are 1-byte ASCII")]
fn parse_textual_locator_suffix(
    input: &str,
    locale: &Locale,
) -> Option<(usize, citum_schema::citation::CitationLocator)> {
    let whitespace_len = input.len() - input.trim_start_matches(char::is_whitespace).len();
    let rest = &input[whitespace_len..];
    if !rest.starts_with('[') {
        return None;
    }

    let closing = rest.find(']')?;
    let inner = rest[1..closing].trim();
    if inner.is_empty() || inner.contains('@') {
        return None;
    }

    let locator = normalize_locator_text(inner, locale)?;
    Some((whitespace_len + closing + 1, locator))
}

fn cite_key_len(input: &str) -> Option<usize> {
    let len = input
        .char_indices()
        .take_while(
            |(_, ch)| matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | ':' | '.'),
        )
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()
        .unwrap_or(0);

    if len == 0 { None } else { Some(len) }
}

fn normalize_prefix(prefix: &str) -> Option<String> {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("{trimmed} "))
    }
}

#[allow(clippy::string_slice, reason = "start index from find() is safe")]
fn is_valid_textual_start(content: &str, start: usize) -> bool {
    let prev = content[..start].chars().next_back();
    !matches!(prev, Some(ch) if ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | '@'))
}

/// Re-encode NUL placeholder tokens as HTML comments and return a mapping.
///
/// pulldown-cmark normalises `\x00` to U+FFFD, which would corrupt the
/// `HtmlPlaceholderRegistry` tokens. Replacing them with HTML comments
/// (`<!--CITUM-TOKEN-N-->`) before parsing lets them pass through as
/// `InlineHtml` or `Html` events and survive the conversion intact.
/// The returned pairs map each comment back to the original token so the
/// caller can restore them after `push_html` runs.
fn remap_nul_tokens(s: &str) -> (String, Vec<(String, String)>) {
    let mut result = String::with_capacity(s.len());
    let mut map: Vec<(String, String)> = Vec::new();
    let mut outside = true;
    let mut token_body = String::new();
    for ch in s.chars() {
        if ch == '\x00' {
            if outside {
                // Opening NUL: start accumulating the token body.
                token_body.clear();
            } else {
                // Closing NUL: emit the comment placeholder.
                let idx = map.len();
                let comment = format!("<!--CITUM-TOKEN-{idx}-->");
                let original = format!("\x00{token_body}\x00");
                result.push_str(&comment);
                map.push((comment, original));
            }
            outside = !outside;
        } else if outside {
            result.push(ch);
        } else {
            token_body.push(ch);
        }
    }
    (result, map)
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
    use citum_schema::citation::{CitationLocator, LocatorType};

    #[test]
    fn test_parse_bracketed_multi_cite() {
        let parser = MarkdownParser;
        let citations =
            parser.parse_citations("See [@kuhn1962; @watson1953, ch. 2].", &Locale::en_us());

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert_eq!(citation.items.len(), 2);
        assert_eq!(citation.items[0].id, "kuhn1962");
        assert_eq!(
            citation.items[1].locator,
            Some(CitationLocator::single(LocatorType::Chapter, "2"))
        );
    }

    #[test]
    fn test_parse_bracketed_prefix_and_suppress_author() {
        let parser = MarkdownParser;
        let citations = parser.parse_citations("[see -@kuhn1962, p. 10]", &Locale::en_us());

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert!(citation.suppress_author);
        assert_eq!(citation.items[0].prefix.as_deref(), Some("see "));
        assert_eq!(
            citation.items[0].locator,
            Some(CitationLocator::single(LocatorType::Page, "10"))
        );
    }

    #[test]
    fn test_parse_textual_citation() {
        let parser = MarkdownParser;
        let citations = parser.parse_citations(
            "Kuhn argued that @kuhn1962 changed science.",
            &Locale::en_us(),
        );

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert_eq!(citation.mode, CitationMode::Integral);
        assert_eq!(citation.items[0].id, "kuhn1962");
    }

    #[test]
    fn test_parse_textual_citation_with_locator_suffix() {
        let parser = MarkdownParser;
        let citations =
            parser.parse_citations("@kuhn1962 [p. 10] argues this point.", &Locale::en_us());

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert_eq!(citation.mode, CitationMode::Integral);
        assert_eq!(
            citation.items[0].locator,
            Some(CitationLocator::single(LocatorType::Page, "10"))
        );
    }

    #[test]
    fn test_parse_document_marks_citations_as_inline_prose() {
        let parser = MarkdownParser;
        let parsed = parser.parse_document("Text [@kuhn1962].", &Locale::en_us());

        assert_eq!(parsed.citations.len(), 1);
        assert_eq!(
            parsed.citations[0].placement,
            CitationPlacement::InlineProse
        );
        assert!(parsed.manual_note_order.is_empty());
        assert!(parsed.bibliography_blocks.is_empty());
    }

    #[test]
    fn test_does_not_parse_email_address() {
        let parser = MarkdownParser;
        let citations =
            parser.parse_citations("Contact test@example.com for details.", &Locale::en_us());

        assert!(citations.is_empty());
    }

    #[test]
    fn test_unsupported_bracket_cluster_does_not_fall_back_to_textual_citations() {
        let parser = MarkdownParser;
        let citations =
            parser.parse_citations("Mixed [@kuhn1962; -@watson1953] cluster.", &Locale::en_us());

        assert!(citations.is_empty());
    }

    #[test]
    fn given_markdown_body_when_finalize_html_output_then_markup_is_converted_to_html() {
        let parser = MarkdownParser;
        let input = "**bold** and _em_ text.";
        let output = parser.finalize_html_output(input);
        assert!(
            output.contains("<strong>bold</strong>"),
            "expected <strong>bold</strong> in: {output}"
        );
        assert!(
            output.contains("<em>em</em>"),
            "expected <em>em</em> in: {output}"
        );
    }

    #[test]
    fn given_markdown_with_nul_tokens_when_finalize_html_output_then_tokens_survive_conversion() {
        let parser = MarkdownParser;
        // NUL tokens stand in for spliced citation HTML; they must survive the
        // pulldown-cmark pass so HtmlPlaceholderRegistry::apply() can substitute them.
        let token = "\x00CITUMHTMLINLINETOKEN0\x00";
        let input = format!("Some prose with {token} inline.");
        let output = parser.finalize_html_output(&input);
        assert!(
            output.contains(token),
            "NUL token must survive pulldown-cmark conversion; output: {output}"
        );
    }

    #[test]
    fn given_markdown_blockquote_when_finalize_html_output_then_blockquote_element_emitted() {
        let parser = MarkdownParser;
        let input = "> block quote with *italic* text";
        let output = parser.finalize_html_output(input);
        assert!(
            output.contains("<blockquote>"),
            "expected <blockquote> in: {output}"
        );
        assert!(
            output.contains("<em>italic</em>"),
            "expected <em>italic</em> in: {output}"
        );
    }
}
