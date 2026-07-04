/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Djot-specific parsing logic using winnow.

use super::super::{
    CitationStructure, DocumentIntegralNameOverride, DocumentOptionsOverride, ManualNoteReference,
    ParsedCitation,
};
use super::BibliographyBlock;
use crate::{Citation, CitationItem};
use citum_schema::citation::{CitationMode, normalize_locator_text};
use citum_schema::grouping::{BibliographyGroup, GroupHeading, GroupSelector};
use citum_schema::locale::Locale;
use citum_schema::template::TypeSelector;
use jotdown::{Attributes, Container, Event, Parser};
use serde::Deserialize;
use std::collections::HashSet;
use std::ops::Range;
use winnow::Parser as WinnowParser;
use winnow::ascii::space0;
use winnow::combinator::{opt, repeat};
use winnow::error::ContextError;
use winnow::token::{take_until, take_while};

#[derive(Debug, Clone)]
pub(crate) struct FootnoteDefinitionRange {
    pub label: String,
    pub content: Range<usize>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DocumentFrontmatter {
    pub bibliography: Option<Vec<BibliographyGroup>>,
    pub integral_name_memory: Option<DocumentIntegralNameOverride>,
    pub org_abbreviation_memory: Option<super::super::DocumentOrgAbbreviationOverride>,
    pub options: Option<DocumentOptionsOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeChange {
    offset: usize,
    structure: CitationStructure,
}

#[derive(Debug, Default)]
struct ScopeTracker {
    current_chapter: Option<String>,
    current_section: Option<String>,
    chapter_stack: Vec<(Option<String>, Option<String>)>,
    section_stack: Vec<Option<String>>,
}

fn parse_suppress_author_modifier(input: &mut &str) -> winnow::Result<bool, ContextError> {
    let modifier: Option<char> = opt('-').parse_next(input)?;
    Ok(modifier.is_some())
}

fn parse_integral_modifier(input: &mut &str) -> winnow::Result<bool, ContextError> {
    let modifier: Option<char> = opt('+').parse_next(input)?;
    Ok(modifier.is_some())
}

pub(crate) fn scan_manual_notes(
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
            Event::FootnoteReference(label) if footnote_stack.is_empty() => {
                manual_note_references.push(ManualNoteReference {
                    label: label.to_string(),
                    start: range.start,
                });
                manual_note_labels.insert(label.to_string());
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

pub(crate) fn annotate_citation_structures(content: &str, citations: &mut [ParsedCitation]) {
    if citations.is_empty() {
        return;
    }

    let changes = collect_scope_changes(content);
    for citation in citations {
        citation.structure = scope_for_offset(&changes, citation.start);
    }
}

fn collect_scope_changes(content: &str) -> Vec<ScopeChange> {
    let mut tracker = ScopeTracker::default();
    let mut changes = vec![ScopeChange {
        offset: 0,
        structure: tracker.current_structure(),
    }];

    for (event, range) in Parser::new(content).into_offset_iter() {
        match event {
            Event::Start(Container::Div { class }, _) => {
                let classes: Vec<&str> = class.split_whitespace().collect();
                if classes.contains(&"chapter") {
                    tracker.enter_chapter(format!("chapter-div-{}", range.start));
                    push_scope_change(&mut changes, range.start, tracker.current_structure());
                }
                if classes.contains(&"section") {
                    tracker.enter_section(format!("section-div-{}", range.start));
                    push_scope_change(&mut changes, range.start, tracker.current_structure());
                }
            }
            Event::End(Container::Div { class }) => {
                let classes: Vec<&str> = class.split_whitespace().collect();
                if classes.contains(&"section") {
                    tracker.exit_section();
                    push_scope_change(&mut changes, range.end, tracker.current_structure());
                }
                if classes.contains(&"chapter") {
                    tracker.exit_chapter();
                    push_scope_change(&mut changes, range.end, tracker.current_structure());
                }
            }
            Event::Start(
                Container::Heading {
                    level,
                    id,
                    has_section: _,
                },
                _,
            ) => {
                let scope_id = if id.is_empty() {
                    format!("heading-{level}-{}", range.start)
                } else {
                    id.to_string()
                };
                tracker.enter_heading(level, scope_id);
                push_scope_change(&mut changes, range.start, tracker.current_structure());
            }
            _ => {}
        }
    }

    changes
}

fn push_scope_change(changes: &mut Vec<ScopeChange>, offset: usize, structure: CitationStructure) {
    if changes
        .last()
        .is_none_or(|change| change.structure != structure)
    {
        changes.push(ScopeChange { offset, structure });
    }
}

fn scope_for_offset(changes: &[ScopeChange], offset: usize) -> CitationStructure {
    let index = changes.partition_point(|change| change.offset <= offset);
    changes
        .get(index.saturating_sub(1))
        .map(|change| change.structure.clone())
        .unwrap_or_default()
}

pub(crate) fn find_citations(content: &str, locale: &Locale) -> Vec<(usize, usize, Citation)> {
    let mut results = Vec::new();
    let mut input = content;
    let mut offset = 0;

    while !input.is_empty() {
        let Some(start_pos) = input.find('[') else {
            break;
        };

        #[allow(clippy::string_slice, reason = "start_pos is found via find('[')")]
        let potential = &input[start_pos..];
        let mut p_input = potential;

        if let Ok(citation) = parse_parenthetical_citation(&mut p_input, locale) {
            let consumed = potential.len() - p_input.len();
            let end_pos = start_pos + consumed;
            results.push((offset + start_pos, offset + end_pos, citation));

            let shift = end_pos;
            #[allow(clippy::string_slice, reason = "shift is a valid boundary")]
            let next_input = &input[shift..];
            input = next_input;
            offset += shift;
        } else {
            let shift = start_pos + 1;
            #[allow(
                clippy::string_slice,
                reason = "shift is valid (start_pos + '[' length)"
            )]
            let next_input = &input[shift..];
            input = next_input;
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
        #[allow(clippy::string_slice, reason = "comma_pos is found via find(',')")]
        let locator_part = after_key[comma_pos + 1..].trim();
        item.locator = normalize_locator_text(locator_part, locale);
    } else {
        *input = checkpoint;
    }

    Ok(item)
}

impl ScopeTracker {
    fn current_structure(&self) -> CitationStructure {
        const DEFAULT_STRUCTURE: &str = "document";
        let chapter_scope = self
            .current_chapter
            .clone()
            .unwrap_or_else(|| DEFAULT_STRUCTURE.to_string());
        let section_scope = self
            .current_section
            .clone()
            .unwrap_or_else(|| chapter_scope.clone());
        CitationStructure {
            chapter_scope,
            section_scope,
        }
    }

    fn enter_chapter(&mut self, scope_id: String) {
        self.chapter_stack
            .push((self.current_chapter.clone(), self.current_section.clone()));
        self.current_chapter = Some(scope_id);
        self.current_section = None;
    }

    fn exit_chapter(&mut self) {
        if let Some((chapter, section)) = self.chapter_stack.pop() {
            self.current_chapter = chapter;
            self.current_section = section;
        }
    }

    fn enter_section(&mut self, scope_id: String) {
        self.section_stack.push(self.current_section.clone());
        self.current_section = Some(scope_id);
    }

    fn exit_section(&mut self) {
        if let Some(section) = self.section_stack.pop() {
            self.current_section = section;
        }
    }

    fn enter_heading(&mut self, level: u16, scope_id: String) {
        if level == 1 {
            if self.chapter_stack.is_empty() {
                self.current_chapter = Some(scope_id);
                self.current_section = None;
            } else {
                self.current_section = Some(scope_id);
            }
        } else {
            self.current_section = Some(scope_id);
        }
    }
}

/// Whether `line` (a single line, excluding its trailing newline) is a
/// frontmatter delimiter: a line consisting solely of `---`, tolerating
/// trailing whitespace or a `\r` (CRLF line endings) before the newline.
///
/// Anchoring to whole lines (rather than a bare substring match) avoids two
/// pitfalls: a leading `----` thematic break is not a delimiter, and a YAML
/// value containing `---` mid-line (e.g. `title: a --- b`) does not count
/// as one either.
fn is_frontmatter_delimiter_line(line: &str) -> bool {
    line.trim_end() == "---"
}

/// Parse YAML frontmatter from content.
///
/// Returns `(result, remaining_content)` where `result` is:
/// - `Ok(None)` when no `---` block is present
/// - `Ok(Some(fm))` on a successful parse
/// - `Err(msg)` when a `---` block is present but fails to deserialize (e.g.
///   unknown field rejected by `deny_unknown_fields`)
///
/// Both the opening and closing delimiters must be a line consisting solely
/// of `---` (see [`is_frontmatter_delimiter_line`]); a bare substring match
/// would misread a leading thematic break as frontmatter, or close the block
/// early on an embedded `---` inside a YAML value.
///
/// Callers must surface the error; they must not silently proceed without
/// frontmatter data when the user authored an invalid block.
#[allow(clippy::string_slice, reason = "'---' is 1-byte ASCII")]
pub(crate) fn parse_frontmatter(
    content: &str,
) -> (Result<Option<DocumentFrontmatter>, String>, &str) {
    let trimmed = content.trim_start();

    let opening_line_len = trimmed.find('\n').unwrap_or(trimmed.len());
    if !is_frontmatter_delimiter_line(&trimmed[..opening_line_len]) {
        return (Ok(None), content);
    }
    let after_opening = if opening_line_len < trimmed.len() {
        &trimmed[opening_line_len + 1..]
    } else {
        ""
    };

    // Scan line-by-line for the closing delimiter.
    let mut cursor = 0usize;
    loop {
        let remaining = &after_opening[cursor..];
        let line_len = remaining.find('\n').unwrap_or(remaining.len());
        let line = &remaining[..line_len];
        let at_end = cursor + line_len >= after_opening.len();

        if is_frontmatter_delimiter_line(line) {
            let frontmatter_content = &after_opening[..cursor];
            let remaining_start = if at_end {
                after_opening.len()
            } else {
                cursor + line_len + 1
            };
            let remaining_body = after_opening[remaining_start..].trim_start();

            let result = serde_yaml::from_str::<DocumentFrontmatter>(frontmatter_content)
                .map(Some)
                .map_err(|e| e.to_string());
            return (result, remaining_body);
        }

        if at_end {
            return (Ok(None), content);
        }
        cursor += line_len + 1;
    }
}

/// Scan document for inline bibliography blocks (`::: bibliography :::`)
/// and extract their metadata from attributes.
pub(crate) fn scan_bibliography_blocks(content: &str) -> Vec<BibliographyBlock> {
    let mut blocks = Vec::new();
    let mut div_stack: Vec<(usize, BibliographyGroup)> = Vec::new();

    for (event, range) in Parser::new(content).into_offset_iter() {
        match event {
            Event::Start(Container::Div { class }, attrs) if class.contains("bibliography") => {
                div_stack.push((range.start, extract_group_from_attrs(&class, attrs)));
            }
            Event::End(Container::Div { class }) => {
                if class.contains("bibliography")
                    && let Some((start, group_id)) = div_stack.pop()
                {
                    blocks.push(BibliographyBlock {
                        start,
                        end: range.end,
                        group: group_id,
                    });
                }
            }
            _ => {}
        }
    }

    blocks
}

/// Extract bibliography group definition from div attributes.
fn extract_group_from_attrs(_class: &str, attrs: Attributes) -> BibliographyGroup {
    let mut title: Option<String> = None;
    let mut ref_type: Option<String> = None;
    let mut not_ref_type: Option<String> = None;

    for (kind, value) in attrs {
        if let Some(key) = kind.key() {
            match key {
                "title" => title = Some(value.to_string()),
                "type" => ref_type = Some(value.to_string()),
                "not-type" | "exclude-type" => not_ref_type = Some(value.to_string()),
                _ => {}
            }
        }
    }

    let mut selector = GroupSelector {
        ref_type: ref_type.as_deref().map(parse_type_selector),
        ..Default::default()
    };

    if let Some(not_ref_type) = not_ref_type {
        selector.not = Some(Box::new(GroupSelector {
            ref_type: Some(parse_type_selector(&not_ref_type)),
            ..Default::default()
        }));
    }

    BibliographyGroup {
        id: "default".to_string(),
        heading: title.map(|literal| GroupHeading::Literal { literal }),
        selector,
        ..Default::default()
    }
}

fn parse_type_selector(value: &str) -> TypeSelector {
    match value.parse::<TypeSelector>() {
        Ok(selector) => selector,
        Err(err) => match err {},
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::{parse_frontmatter, scan_bibliography_blocks};
    use citum_schema::grouping::GroupHeading;
    use citum_schema::template::TypeSelector;

    #[test]
    fn given_leading_thematic_break_when_parse_frontmatter_then_not_treated_as_frontmatter() {
        // A leading "----" thematic break must not be misread as a
        // frontmatter opening delimiter (only an exact "---" line opens one).
        let content = "----\n\nBody text.";
        let (result, remaining) = parse_frontmatter(content);

        assert!(result.unwrap().is_none());
        assert_eq!(remaining, content);
    }

    #[test]
    fn given_embedded_dashes_in_value_when_parse_frontmatter_then_block_does_not_close_early() {
        // A YAML value containing "---" mid-line must not be mistaken for
        // the closing delimiter.
        let content = "---\ntitle: a --- b\n---\n\nBody text.";
        let (result, remaining) = parse_frontmatter(content);

        assert!(
            result.unwrap().is_some(),
            "well-formed frontmatter block should parse"
        );
        assert_eq!(remaining, "Body text.");
    }

    #[test]
    fn given_normal_frontmatter_when_parse_frontmatter_then_body_is_returned() {
        let content = "---\ntitle: T\n---\n\nBody text.";
        let (result, remaining) = parse_frontmatter(content);

        assert!(result.unwrap().is_some(), "frontmatter should parse");
        assert_eq!(remaining, "Body text.");
    }

    #[test]
    fn bibliography_block_attrs_parse_negated_type_selector() {
        let blocks = scan_bibliography_blocks(
            r#"{ not-type="manuscript" title="Secondary Sources" }
::: bibliography
:::
"#,
        );

        assert_eq!(blocks.len(), 1);
        let Some(block) = blocks.first() else {
            return;
        };
        assert_eq!(
            block.group.heading,
            Some(GroupHeading::Literal {
                literal: "Secondary Sources".to_string(),
            })
        );
        assert_eq!(
            block
                .group
                .selector
                .not
                .as_ref()
                .and_then(|selector| selector.ref_type.as_ref()),
            Some(&TypeSelector::Single("manuscript".to_string()))
        );
    }
}
