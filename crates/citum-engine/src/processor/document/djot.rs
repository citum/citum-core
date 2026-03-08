/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Djot document parsing and HTML conversion.

use super::{
    CitationParser, CitationPlacement, ManualNoteReference, ParsedCitation, ParsedDocument,
};
use crate::{Citation, CitationItem};
use citum_schema::citation::{CitationMode, normalize_locator_text};
use citum_schema::grouping::BibliographyGroup;
use citum_schema::locale::Locale;
use jotdown::{Attributes, Container, Event, Parser};
use std::collections::HashSet;
use std::ops::Range;
use winnow::Parser as WinnowParser;
use winnow::ascii::space0;
use winnow::combinator::{opt, repeat};
use winnow::error::ContextError;
use winnow::token::{take_until, take_while};

#[derive(Debug, Clone)]
struct FootnoteDefinitionRange {
    label: String,
    content: Range<usize>,
}

/// A bibliography block parsed from djot source.
#[derive(Debug, Clone)]
pub struct BibliographyBlock {
    /// Byte offset of the opening `:::` in source.
    pub start: usize,
    /// Byte offset past the closing `:::`.
    pub end: usize,
    /// The bibliography group for this block.
    pub group: BibliographyGroup,
}

/// A parser for Djot citations using winnow.
/// Syntax: `[@key]`, `[+@key]`, or `[-@key]`. Multi-cites: `[@key1; @key2]`.
pub struct DjotParser;

impl Default for DjotParser {
    fn default() -> Self {
        Self
    }
}

fn parse_suppress_author_modifier(input: &mut &str) -> winnow::Result<bool, ContextError> {
    let modifier: Option<char> = opt('-').parse_next(input)?;
    Ok(modifier.is_some())
}

fn parse_integral_modifier(input: &mut &str) -> winnow::Result<bool, ContextError> {
    let modifier: Option<char> = opt('+').parse_next(input)?;
    Ok(modifier.is_some())
}

impl CitationParser for DjotParser {
    fn parse_document(&self, content: &str, locale: &Locale) -> ParsedDocument {
        // Try to parse frontmatter and get remaining content
        let (frontmatter_groups, remaining_content) = parse_frontmatter(content);
        let body_start = content.len() - remaining_content.len();

        let (manual_note_references, manual_note_labels, footnote_definitions) =
            scan_manual_notes(remaining_content);

        let mut manual_note_order = Vec::new();
        let mut seen_manual = HashSet::new();
        for note in &manual_note_references {
            if seen_manual.insert(note.label.clone()) {
                manual_note_order.push(note.label.clone());
            }
        }

        let citations = find_citations(remaining_content, locale)
            .into_iter()
            .map(|(start, end, citation)| ParsedCitation {
                start,
                end,
                citation,
                placement: citation_placement(start, end, &footnote_definitions),
            })
            .collect();

        // Scan for inline bibliography blocks in remaining content
        let bibliography_blocks = scan_bibliography_blocks(remaining_content);

        ParsedDocument {
            citations,
            manual_note_order,
            manual_note_references,
            manual_note_labels,
            bibliography_blocks,
            frontmatter_groups,
            body_start,
        }
    }
}

fn scan_manual_notes(
    content: &str,
) -> (
    Vec<ManualNoteReference>,
    HashSet<String>,
    Vec<FootnoteDefinitionRange>,
) {
    let mut manual_note_references = Vec::new();
    let mut manual_note_labels = HashSet::new();
    let mut footnote_definitions = Vec::new();
    let mut footnote_stack: Vec<(String, usize)> = Vec::new();

    for (event, range) in Parser::new(content).into_offset_iter() {
        match event {
            Event::FootnoteReference(label) => {
                if footnote_stack.is_empty() {
                    manual_note_references.push(ManualNoteReference {
                        label: label.to_string(),
                        start: range.start,
                    });
                    manual_note_labels.insert(label.to_string());
                }
            }
            Event::Start(Container::Footnote { label }, ..) => {
                manual_note_labels.insert(label.to_string());
                footnote_stack.push((label.to_string(), range.end));
            }
            Event::End(Container::Footnote { label }) => {
                if let Some((open_label, content_start)) = footnote_stack.pop() {
                    debug_assert_eq!(open_label, label);
                    footnote_definitions.push(FootnoteDefinitionRange {
                        label: open_label,
                        content: content_start..range.start,
                    });
                }
            }
            _ => {}
        }
    }

    (
        manual_note_references,
        manual_note_labels,
        footnote_definitions,
    )
}

fn citation_placement(
    start: usize,
    end: usize,
    footnote_definitions: &[FootnoteDefinitionRange],
) -> CitationPlacement {
    footnote_definitions
        .iter()
        .find(|definition| definition.content.start <= start && end <= definition.content.end)
        .map(|definition| CitationPlacement::ManualFootnote {
            label: definition.label.clone(),
        })
        .unwrap_or(CitationPlacement::InlineProse)
}

fn find_citations(content: &str, locale: &Locale) -> Vec<(usize, usize, Citation)> {
    let mut results = Vec::new();
    let mut input = content;
    let mut offset = 0;

    while !input.is_empty() {
        let next_bracket = input.find('[');
        let start_pos = match next_bracket {
            Some(b) => b,
            None => break,
        };

        let potential = &input[start_pos..];
        let mut p_input = potential;

        if let Ok(citation) = parse_parenthetical_citation(&mut p_input, locale) {
            let consumed = potential.len() - p_input.len();
            let end_pos = start_pos + consumed;
            results.push((offset + start_pos, offset + end_pos, citation));

            let shift = end_pos;
            input = &input[shift..];
            offset += shift;
        } else {
            let shift = start_pos + 1;
            input = &input[shift..];
            offset += shift;
        }
    }

    results
}

/// Parse `[content]`
fn parse_parenthetical_citation(
    input: &mut &str,
    locale: &Locale,
) -> winnow::Result<Citation, ContextError> {
    let _ = '['.parse_next(input)?;
    let citation = parse_citation_content(input, locale)?;
    let _ = ']'.parse_next(input)?;
    Ok(citation)
}

fn parse_citation_content(
    input: &mut &str,
    locale: &Locale,
) -> winnow::Result<Citation, ContextError> {
    let mut citation = Citation::default();
    let mut detected_integral = false;
    let mut suppress_author = false;

    let inner: &str = take_until(0.., ']').parse_next(input)?;

    let items: Vec<CitationItem> = repeat(1.., |input: &mut &str| {
        let _ = space0.parse_next(input)?;
        let is_integral = parse_integral_modifier.parse_next(input).unwrap_or(false);
        if is_integral {
            detected_integral = true;
        }
        let suppress = parse_suppress_author_modifier(input)?;
        if suppress {
            suppress_author = true;
        }
        let item = parse_citation_item_no_integral(input, locale)?;
        let _ = opt(';').parse_next(input)?;
        let _ = space0.parse_next(input)?;
        Ok(item)
    })
    .parse_next(&mut inner.trim())?;

    citation.items = items;
    citation.suppress_author = suppress_author;
    if detected_integral {
        citation.mode = CitationMode::Integral;
    }

    Ok(citation)
}

fn parse_citation_item_no_integral(
    input: &mut &str,
    locale: &Locale,
) -> winnow::Result<CitationItem, ContextError> {
    let _ = space0.parse_next(input)?;
    let _: char = '@'.parse_next(input)?;
    let key: &str =
        take_while(1.., |c: char| c.is_alphanumeric() || c == '_' || c == '-').parse_next(input)?;

    let mut item = CitationItem {
        id: key.to_string(),
        ..Default::default()
    };

    let _ = space0.parse_next(input)?;

    let checkpoint = *input;
    let after_key: &str = take_while(0.., |c: char| c != ';' && c != ']').parse_next(input)?;

    if let Some(comma_pos) = after_key.find(',') {
        let locator_part = after_key[comma_pos + 1..].trim();
        item.locator = normalize_locator_text(locator_part, locale);
    } else {
        *input = checkpoint;
    }

    Ok(item)
}

/// Parse YAML frontmatter from content.
/// Returns (Option<groups>, remaining_content).
fn parse_frontmatter(content: &str) -> (Option<Vec<BibliographyGroup>>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    let after_opening = &trimmed[3..];
    if let Some(closing_pos) = after_opening.find("---") {
        let frontmatter_content = &after_opening[..closing_pos];
        let remaining = &after_opening[closing_pos + 3..].trim_start();

        match serde_yaml::from_str::<serde_yaml::Value>(frontmatter_content) {
            Ok(value) => {
                if let Some(bib_value) = value.get("bibliography")
                    && let Ok(groups) =
                        serde_yaml::from_value::<Vec<BibliographyGroup>>(bib_value.clone())
                {
                    return (Some(groups), remaining);
                }
                (None, remaining)
            }
            Err(_) => (None, remaining),
        }
    } else {
        (None, content)
    }
}

/// Scan document for inline bibliography blocks (`::: bibliography :::`)
/// and extract their metadata from attributes.
fn scan_bibliography_blocks(content: &str) -> Vec<BibliographyBlock> {
    let mut blocks = Vec::new();
    let mut div_stack: Vec<(usize, String)> = Vec::new();

    for (event, range) in Parser::new(content).into_offset_iter() {
        match event {
            Event::Start(Container::Div { class }, attrs) => {
                if class.contains("bibliography") {
                    div_stack.push((range.start, extract_group_from_attrs(class, attrs)));
                }
            }
            Event::End(Container::Div { class }) => {
                if class.contains("bibliography")
                    && let Some((start, group_id)) = div_stack.pop()
                    && let Ok(group) = serde_yaml::from_str::<BibliographyGroup>(&group_id)
                {
                    blocks.push(BibliographyBlock {
                        start,
                        end: range.end,
                        group,
                    });
                }
            }
            _ => {}
        }
    }

    blocks
}

/// Extract bibliography group definition from div attributes.
fn extract_group_from_attrs(_class: &str, attrs: Attributes) -> String {
    let mut title: Option<String> = None;
    let mut ref_type: Option<String> = None;

    for (kind, value) in attrs {
        if let Some(key) = kind.key() {
            let val = value.to_string();
            match key {
                "title" => title = Some(val),
                "type" => ref_type = Some(val),
                _ => {}
            }
        }
    }

    let mut yaml = String::from("id: default\n");

    if let Some(t) = title {
        yaml.push_str(&format!("heading:\n  literal: \"{t}\"\n"));
    }

    match ref_type {
        Some(t) => yaml.push_str(&format!("selector:\n  type: {t}\n")),
        None => yaml.push_str("selector: {}\n"),
    }

    yaml
}

/// Convert Djot markup to HTML using jotdown.
pub fn djot_to_html(djot: &str) -> String {
    let events = Parser::new(djot);
    jotdown::html::render_to_string(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::citation::{CitationLocator, LocatorType};

    #[test]
    fn test_parse_multi_cite_with_locators() {
        let parser = DjotParser;
        let content = "[@kuhn1962; @watson1953, ch. 2]";
        let citations = parser.parse_citations(content, &Locale::en_us());

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert_eq!(citation.items.len(), 2);
        assert_eq!(citation.items[0].id, "kuhn1962");
        assert_eq!(citation.items[1].id, "watson1953");
        assert_eq!(
            citation.items[1].locator,
            Some(CitationLocator::single(LocatorType::Chapter, "2"))
        );
    }

    #[test]
    fn test_parse_structured_locator() {
        let parser = DjotParser;
        let content = "[@kuhn1962, section: 5]";
        let citations = parser.parse_citations(content, &Locale::en_us());

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert_eq!(
            citation.items[0].locator,
            Some(CitationLocator::single(LocatorType::Section, "5"))
        );
    }

    #[test]
    fn test_parse_compound_locator() {
        let parser = DjotParser;
        let content = "[@kuhn1962, chapter: 2, page: 10]";
        let citations = parser.parse_citations(content, &Locale::en_us());

        let (_, _, citation) = &citations[0];
        let locator = citation.items[0].locator.as_ref().unwrap();
        assert!(locator.is_compound());
        assert_eq!(locator.segments()[0].label, LocatorType::Chapter);
        assert_eq!(locator.segments()[1].label, LocatorType::Page);
    }

    #[test]
    fn test_parse_suppress_author() {
        let parser = DjotParser;
        let content = "[-@kuhn1962]";
        let citations = parser.parse_citations(content, &Locale::en_us());

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert_eq!(citation.items[0].id, "kuhn1962");
        assert!(citation.suppress_author);
    }

    #[test]
    fn test_parse_bracketed_integral_citation() {
        let parser = DjotParser;
        let content = "[+@kuhn1962]";
        let citations = parser.parse_citations(content, &Locale::en_us());

        assert_eq!(citations.len(), 1);
        let (_, _, citation) = &citations[0];
        assert_eq!(citation.mode, CitationMode::Integral);
        assert_eq!(citation.items[0].id, "kuhn1962");
        assert!(!citation.suppress_author);
    }

    #[test]
    fn test_parse_semicolon_without_citation() {
        let parser = DjotParser;
        let content = "[foo; bar]";
        let citations = parser.parse_citations(content, &Locale::en_us());

        assert_eq!(citations.len(), 0);
    }

    #[test]
    fn test_parse_document_tracks_manual_footnotes() {
        let parser = DjotParser;
        let content = "Text[^m1].\n\n[^m1]: See [@kuhn1962].";
        let parsed = parser.parse_document(content, &Locale::en_us());

        assert_eq!(parsed.manual_note_order, vec!["m1".to_string()]);
        assert_eq!(parsed.manual_note_references.len(), 1);
        assert_eq!(parsed.citations.len(), 1);
        assert_eq!(
            parsed.citations[0].placement,
            CitationPlacement::ManualFootnote {
                label: "m1".to_string()
            }
        );
    }

    #[test]
    fn test_parse_document_marks_prose_citations_as_inline() {
        let parser = DjotParser;
        let content = "Text [@kuhn1962].";
        let parsed = parser.parse_document(content, &Locale::en_us());

        assert_eq!(parsed.citations.len(), 1);
        assert_eq!(
            parsed.citations[0].placement,
            CitationPlacement::InlineProse
        );
    }
}
