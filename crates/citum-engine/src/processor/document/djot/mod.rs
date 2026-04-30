/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Djot document parsing and HTML conversion adapter.

mod parsing;

use super::{BibliographyBlock, CitationParser, CitationPlacement, ParsedDocument};
use citum_schema::locale::Locale;
use parsing::{
    FootnoteDefinitionRange, annotate_citation_structures, find_citations, parse_frontmatter,
    scan_bibliography_blocks, scan_manual_notes,
};
use std::collections::HashSet;

/// A parser for Djot citations using winnow.
/// Syntax: `[@key]`, `[+@key]`, or `[-@key]`. Multi-cites: `[@key1; @key2]`.
#[derive(Default)]
pub struct DjotParser;

impl CitationParser for DjotParser {
    fn parse_document(&self, content: &str, locale: &Locale) -> ParsedDocument {
        // Try to parse frontmatter and get remaining content
        let (frontmatter, remaining_content) = parse_frontmatter(content);
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

        let mut citations: Vec<_> = find_citations(remaining_content, locale)
            .into_iter()
            .map(|(start, end, citation)| super::ParsedCitation {
                start,
                end,
                citation,
                placement: citation_placement(start, end, &footnote_definitions),
                structure: Default::default(),
            })
            .collect();
        annotate_citation_structures(remaining_content, &mut citations);

        // Scan for inline bibliography blocks in remaining content
        let bibliography_blocks = scan_bibliography_blocks(remaining_content);

        ParsedDocument {
            citations,
            manual_note_order,
            manual_note_references,
            manual_note_labels,
            bibliography_blocks,
            frontmatter_groups: frontmatter
                .as_ref()
                .and_then(|frontmatter| frontmatter.bibliography.clone()),
            frontmatter_integral_names: frontmatter
                .and_then(|frontmatter| frontmatter.integral_names),
            body_start,
        }
    }

    /// Convert Djot markup to HTML using jotdown after citation splicing.
    fn finalize_html_output(&self, rendered: &str) -> String {
        djot_to_html(rendered)
    }
}

/// Determine the citation placement within the document.
fn citation_placement(
    start: usize,
    end: usize,
    footnote_definitions: &[FootnoteDefinitionRange],
) -> CitationPlacement {
    footnote_definitions
        .iter()
        .find(|definition| definition.content.start <= start && end <= definition.content.end)
        .map_or(CitationPlacement::InlineProse, |definition| {
            CitationPlacement::ManualFootnote {
                label: definition.label.clone(),
            }
        })
}

/// Convert Djot markup to HTML using jotdown.
#[must_use]
pub fn djot_to_html(djot: &str) -> String {
    let events = jotdown::Parser::new(djot);
    jotdown::html::render_to_string(events)
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
    use citum_schema::citation::{CitationLocator, CitationMode, LocatorType};

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

    #[test]
    fn test_djot_finalize_html_output_converts_to_html() {
        // DjotParser explicitly overrides finalize_html_output to run jotdown,
        // converting Djot markup to HTML. This is adapter-specific behavior;
        // other parsers (e.g. MarkdownParser) return markup unchanged.
        let parser = DjotParser;
        let result = parser.finalize_html_output("{_em_}");
        assert!(
            result.contains("<em>em</em>"),
            "unexpected output: {result}"
        );
    }
}
