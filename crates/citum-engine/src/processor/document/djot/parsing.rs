//! Djot-specific parsing logic using winnow.

use super::super::{
    CitationStructure, DocumentIntegralNameOverride, ManualNoteReference, ParsedCitation,
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

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DocumentFrontmatter {
    pub bibliography: Option<Vec<BibliographyGroup>>,
    pub integral_names: Option<DocumentIntegralNameOverride>,
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

/// Parse YAML frontmatter from content.
/// Returns (frontmatter, `remaining_content`).
#[allow(clippy::string_slice, reason = "'---' is 1-byte ASCII")]
pub(crate) fn parse_frontmatter(content: &str) -> (Option<DocumentFrontmatter>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    let after_opening = &trimmed[3..];
    if let Some(closing_pos) = after_opening.find("---") {
        let frontmatter_content = &after_opening[..closing_pos];
        let remaining = &after_opening[closing_pos + 3..].trim_start();

        (
            serde_yaml::from_str::<DocumentFrontmatter>(frontmatter_content).ok(),
            remaining,
        )
    } else {
        (None, content)
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
                div_stack.push((range.start, extract_group_from_attrs(class, attrs)));
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

    for (kind, value) in attrs {
        if let Some(key) = kind.key() {
            match key {
                "title" => title = Some(value.to_string()),
                "type" => ref_type = Some(value.to_string()),
                _ => {}
            }
        }
    }

    BibliographyGroup {
        id: "default".to_string(),
        heading: title.map(|literal| GroupHeading::Literal { literal }),
        selector: GroupSelector {
            ref_type: ref_type.map(TypeSelector::Single),
            ..Default::default()
        },
        ..Default::default()
    }
}
