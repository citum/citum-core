/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Document-level citation processing.

pub mod djot;

pub use djot::BibliographyBlock;

#[cfg(test)]
mod tests;

use crate::Citation;
use crate::processor::Processor;
use crate::processor::rendering::{CompoundRenderData, Renderer};
use citum_schema::locale::Locale;
use citum_schema::options::{
    IntegralNameConfig, IntegralNameContexts, IntegralNameRule, IntegralNameScope,
    NoteConfig as StyleNoteConfig, NoteMarkerOrder, NoteNumberPlacement, NoteQuotePlacement,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const GENERATED_NOTE_LABEL_PREFIX: &str = "citum-auto-";
const MOVABLE_PUNCTUATION: [char; 6] = ['.', ',', ';', ':', '!', '?'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteSide {
    Inside,
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoteOrder {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PunctuationRule {
    Inside,
    Outside,
    Adaptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberRule {
    Inside,
    Outside,
    Same,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NoteRule {
    punctuation: PunctuationRule,
    number: NumberRule,
    order: NoteOrder,
}

#[derive(Debug, Default)]
struct LeftContext {
    punctuation: Option<char>,
    quote: Option<char>,
}

#[derive(Debug, Default)]
struct RightContext {
    punctuation: Option<char>,
    quote: Option<char>,
    consumed_len: usize,
}

/// Describes where a parsed citation appears in the source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationPlacement {
    /// The citation marker appears in prose and should become a generated note
    /// reference for note styles.
    InlineProse,
    /// The citation marker appears inside a manually authored footnote
    /// definition and should render in place.
    ManualFootnote {
        /// The source footnote label that identifies the manual note block.
        label: String,
    },
}

/// Structural citation scope metadata derived from the source document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CitationStructure {
    /// The resolved chapter-level scope key for this citation location.
    pub chapter_scope: String,
    /// The resolved section-level scope key for this citation location.
    pub section_scope: String,
}

/// Document-level integral-name override parsed from frontmatter.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DocumentIntegralNameOverride {
    /// Whether the integral-name policy is enabled for this document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// The name-memory rule to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<citum_schema::options::IntegralNameRule>,
    /// Where name-memory resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<citum_schema::options::IntegralNameScope>,
    /// Which document contexts participate in the policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<citum_schema::options::IntegralNameContexts>,
    /// The contributor form used after the first mention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_form: Option<citum_schema::options::IntegralNameForm>,
}

impl DocumentIntegralNameOverride {
    fn apply_to(&self, base: Option<&IntegralNameConfig>) -> Option<IntegralNameConfig> {
        if self.enabled == Some(false) {
            return None;
        }

        let mut result = base.cloned().unwrap_or_default();
        if self.rule.is_some() {
            result.rule = self.rule;
        }
        if self.scope.is_some() {
            result.scope = self.scope;
        }
        if self.contexts.is_some() {
            result.contexts = self.contexts;
        }
        if self.subsequent_form.is_some() {
            result.subsequent_form = self.subsequent_form;
        }
        Some(result)
    }
}

/// A citation marker parsed from a document.
#[derive(Debug, Clone)]
pub struct ParsedCitation {
    /// Byte offset where the citation marker starts in the source document.
    pub start: usize,
    /// Byte offset immediately after the citation marker in the source document.
    pub end: usize,
    /// The parsed citation payload and its items.
    pub citation: Citation,
    /// Where the citation was found in the source document.
    pub placement: CitationPlacement,
    /// Structural scope metadata for this citation location.
    pub structure: CitationStructure,
}

#[derive(Debug, Clone)]
pub(crate) struct ManualNoteReference {
    pub label: String,
    pub start: usize,
}

/// Structured output from a document parser.
#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    /// Citation markers discovered in source order.
    pub citations: Vec<ParsedCitation>,
    /// Manual footnote labels in the order they appear in the document.
    pub manual_note_order: Vec<String>,
    pub(crate) manual_note_references: Vec<ManualNoteReference>,
    pub(crate) manual_note_labels: HashSet<String>,
    /// Bibliography blocks found in the document.
    pub bibliography_blocks: Vec<djot::BibliographyBlock>,
    /// Bibliography groups from YAML frontmatter.
    pub frontmatter_groups: Option<Vec<citum_schema::grouping::BibliographyGroup>>,
    /// Integral-name override from YAML frontmatter.
    pub frontmatter_integral_names: Option<DocumentIntegralNameOverride>,
    /// Byte offset where the document body starts (past any frontmatter).
    pub body_start: usize,
}

/// A trait for document parsers that can identify citations.
pub trait CitationParser {
    /// Parse the document into citation placements and note metadata.
    fn parse_document(&self, content: &str, locale: &Locale) -> ParsedDocument;

    /// Find and extract citations from a document string.
    /// Returns a list of (start_index, end_index, citation_model) tuples.
    fn parse_citations(&self, content: &str, locale: &Locale) -> Vec<(usize, usize, Citation)> {
        self.parse_document(content, locale)
            .citations
            .into_iter()
            .map(|parsed| (parsed.start, parsed.end, parsed.citation))
            .collect()
    }
}

/// Document output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    /// Plain text (raw markup).
    Plain,
    /// Djot markup.
    Djot,
    /// HTML output.
    Html,
    /// LaTeX output.
    Latex,
    /// Typst output.
    Typst,
}

#[derive(Debug, Clone)]
struct GeneratedNote {
    citation_index: usize,
    label: String,
    note_number: u32,
}

#[derive(Debug, Default)]
struct HtmlPlaceholderRegistry {
    next_index: usize,
    inline_replacements: Vec<(String, String)>,
    block_replacements: Vec<(String, String)>,
}

impl HtmlPlaceholderRegistry {
    fn push_inline(&mut self, html: String) -> String {
        let token = self.next_token("INLINE");
        self.inline_replacements.push((token.clone(), html));
        token
    }

    fn push_block(&mut self, html: String) -> String {
        let token = self.next_token("BLOCK");
        self.block_replacements.push((token.clone(), html));
        token
    }

    fn apply(self, rendered: String) -> String {
        let mut output = rendered;

        for (token, html) in self.block_replacements {
            let paragraph = format!("<p>{token}</p>");
            output = output.replace(&paragraph, &html);
            output = output.replace(&token, &html);
        }

        for (token, html) in self.inline_replacements {
            output = output.replace(&token, &html);
        }

        output
    }

    fn next_token(&mut self, kind: &str) -> String {
        let token = format!("CITUMHTML{kind}TOKEN{}", self.next_index);
        self.next_index = self.next_index.saturating_add(1);
        token
    }
}

#[derive(Debug, Clone)]
enum NoteOccurrence {
    Manual { label: String, start: usize },
    Generated { citation_index: usize, start: usize },
}

impl NoteOccurrence {
    fn start(&self) -> usize {
        match self {
            Self::Manual { start, .. } | Self::Generated { start, .. } => *start,
        }
    }
}

#[derive(Debug, Clone)]
struct IntegralNameContext {
    placement: CitationPlacement,
    structure: CitationStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeenIntegralNameState {
    Unseen,
    NoteOnlySeen,
    BodySeen,
}

impl Processor {
    fn normalize_inline_document_citations(&self, parsed: &ParsedDocument) -> Vec<Citation> {
        let mut normalized: Vec<Citation> = parsed
            .citations
            .iter()
            .map(|parsed| parsed.citation.clone())
            .collect();
        let ordered_indices = build_integral_name_order_indices(parsed);
        let mut ordered_citations: Vec<Citation> = ordered_indices
            .iter()
            .map(|index| normalized[*index].clone())
            .collect();
        let ordered_contexts: Vec<_> = ordered_indices
            .iter()
            .map(|index| IntegralNameContext {
                placement: parsed.citations[*index].placement.clone(),
                structure: parsed.citations[*index].structure.clone(),
            })
            .collect();
        self.annotate_integral_name_states(&mut ordered_citations, &ordered_contexts);
        for (citation, index) in ordered_citations
            .into_iter()
            .zip(ordered_indices.into_iter())
        {
            normalized[index] = citation;
        }
        self.normalize_note_context(&normalized)
    }

    fn processor_with_document_integral_name_override(
        &self,
        override_config: Option<&DocumentIntegralNameOverride>,
    ) -> Option<Self> {
        let override_config = override_config?;
        let mut style = self.style.clone();
        let base = style
            .options
            .as_ref()
            .and_then(|options| options.integral_names.as_ref());
        let applied = override_config.apply_to(base);
        style
            .options
            .get_or_insert_with(Default::default)
            .integral_names = applied;
        Some(Self::with_locale_and_compound_sets(
            style,
            self.bibliography.clone(),
            self.locale.clone(),
            self.compound_sets.clone(),
        ))
    }

    /// Process citations in a document and append a bibliography.
    pub fn process_document<P, F>(
        &self,
        content: &str,
        parser: &P,
        format: DocumentFormat,
    ) -> String
    where
        P: CitationParser,
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let parsed = parser.parse_document(content, &self.locale);
        let owned_processor = self.processor_with_document_integral_name_override(
            parsed.frontmatter_integral_names.as_ref(),
        );
        let processor = owned_processor.as_ref().unwrap_or(self);

        // Strip any frontmatter from the content passed to rendering.
        let body = &content[parsed.body_start..];

        // Check what mode we're in before consuming parsed
        let has_frontmatter = parsed.frontmatter_groups.is_some();
        let has_blocks = !parsed.bibliography_blocks.is_empty();

        // Handle frontmatter groups if present (check before inline blocks)
        if has_frontmatter {
            let groups = parsed.frontmatter_groups.as_ref().unwrap().clone();
            let rendered = if matches!(format, DocumentFormat::Html) {
                let mut placeholders = HtmlPlaceholderRegistry::default();
                let rendered = if processor.is_note_style() {
                    processor.process_note_document_html(body, parsed, &mut placeholders)
                } else {
                    processor.process_inline_document_html(body, parsed, &mut placeholders)
                };
                let bib_content = rewrite_group_headings_for_document(
                    processor.render_with_custom_groups::<F>(
                        &processor.process_references().bibliography,
                        &groups,
                    ),
                    format,
                );
                let bib_heading = "\n\n# Bibliography\n\n";
                let mut result = rendered;
                result.push_str(bib_heading);
                result.push_str(&placeholders.push_block(bib_content));
                let html = self::djot::djot_to_html(&result);
                return placeholders.apply(html);
            } else if processor.is_note_style() {
                processor.process_note_document::<F>(body, parsed)
            } else {
                processor.process_inline_document::<F>(body, parsed)
            };

            let bib_content = rewrite_group_headings_for_document(
                processor.render_with_custom_groups::<F>(
                    &processor.process_references().bibliography,
                    &groups,
                ),
                format,
            );
            let bib_heading = match format {
                DocumentFormat::Latex => "\n\n\\section*{Bibliography}\n\n",
                DocumentFormat::Typst => "\n\n= Bibliography\n\n",
                _ => "\n\n# Bibliography\n\n",
            };
            let mut result = rendered;
            result.push_str(bib_heading);
            result.push_str(&bib_content);
            let result = rewrite_document_markup_for_typst(result, format);
            return match format {
                DocumentFormat::Html => self::djot::djot_to_html(&result),
                DocumentFormat::Djot
                | DocumentFormat::Plain
                | DocumentFormat::Latex
                | DocumentFormat::Typst => result,
            };
        }

        // Handle inline bibliography blocks
        if has_blocks {
            let blocks = parsed.bibliography_blocks.clone();

            // Replace each block with a stable placeholder before citation rendering so
            // that citation-text length changes don't corrupt the block byte offsets.
            let mut staged = body.to_string();
            for (i, block) in blocks.iter().enumerate().rev() {
                let placeholder = format!("\x00BIBBLOCK{i}\x00");
                staged.replace_range(block.start..block.end, &placeholder);
            }

            // Re-parse on the placeholder content so citation offsets are correct.
            let parsed_staged = parser.parse_document(&staged, &self.locale);
            let rendered = if matches!(format, DocumentFormat::Html) {
                let mut placeholders = HtmlPlaceholderRegistry::default();
                let rendered = if processor.is_note_style() {
                    processor.process_note_document_html(&staged, parsed_staged, &mut placeholders)
                } else {
                    processor.process_inline_document_html(
                        &staged,
                        parsed_staged,
                        &mut placeholders,
                    )
                };

                let mut result = rendered;
                for (i, block) in blocks.iter().enumerate() {
                    let placeholder = format!("\x00BIBBLOCK{i}\x00");
                    let mut headingless = block.group.clone();
                    let heading = headingless.heading.take();
                    let bib_content = processor.render_bibliography_for_group::<F>(&headingless);
                    let bib_token = placeholders.push_block(bib_content);
                    let replacement = if let Some(h) = heading {
                        let heading_text = processor.resolve_group_heading(&h).unwrap_or_default();
                        format!("## {heading_text}\n\n{bib_token}\n")
                    } else {
                        format!("{bib_token}\n")
                    };
                    result = result.replace(&placeholder, &replacement);
                }

                let html = self::djot::djot_to_html(&result);
                return placeholders.apply(html);
            } else if processor.is_note_style() {
                processor.process_note_document::<F>(&staged, parsed_staged)
            } else {
                processor.process_inline_document::<F>(&staged, parsed_staged)
            };

            // Swap placeholders for rendered bibliographies.
            let mut result = rendered;
            for (i, block) in blocks.iter().enumerate() {
                let placeholder = format!("\x00BIBBLOCK{i}\x00");
                // Render without heading so render_with_custom_groups doesn't emit one;
                // we emit the heading ourselves at the correct document level (##).
                let mut headingless = block.group.clone();
                let heading = headingless.heading.take();
                let bib_content = processor.render_bibliography_for_group::<F>(&headingless);
                let replacement = if let Some(h) = heading {
                    let heading_text = processor.resolve_group_heading(&h).unwrap_or_default();
                    let prefix = match format {
                        DocumentFormat::Latex => format!("\\subsection*{{{heading_text}}}\n\n"),
                        DocumentFormat::Typst => format!("== {heading_text}\n\n"),
                        _ => format!("## {heading_text}\n\n"),
                    };
                    format!("{prefix}{bib_content}\n")
                } else {
                    format!("{bib_content}\n")
                };
                result = result.replace(&placeholder, &replacement);
            }

            let result = rewrite_document_markup_for_typst(result, format);
            return match format {
                DocumentFormat::Html => self::djot::djot_to_html(&result),
                DocumentFormat::Djot
                | DocumentFormat::Plain
                | DocumentFormat::Latex
                | DocumentFormat::Typst => result,
            };
        }

        // Default behavior: append bibliography with heading
        let rendered = if matches!(format, DocumentFormat::Html) {
            let mut placeholders = HtmlPlaceholderRegistry::default();
            let rendered = if processor.is_note_style() {
                processor.process_note_document_html(body, parsed, &mut placeholders)
            } else {
                processor.process_inline_document_html(body, parsed, &mut placeholders)
            };

            let mut result = rendered;
            result.push_str("\n\n# Bibliography\n\n");
            result.push_str(
                &placeholders.push_block(processor.render_grouped_bibliography_with_format::<F>()),
            );
            let html = self::djot::djot_to_html(&result);
            return placeholders.apply(html);
        } else if processor.is_note_style() {
            processor.process_note_document::<F>(body, parsed)
        } else {
            processor.process_inline_document::<F>(body, parsed)
        };

        let bib_heading = match format {
            DocumentFormat::Latex => "\n\n\\section*{Bibliography}\n\n",
            DocumentFormat::Typst => "\n\n= Bibliography\n\n",
            _ => "\n\n# Bibliography\n\n",
        };
        let mut result = rendered;
        result.push_str(bib_heading);
        result.push_str(&processor.render_grouped_bibliography_with_format::<F>());
        let result = rewrite_document_markup_for_typst(result, format);

        match format {
            DocumentFormat::Html => self::djot::djot_to_html(&result),
            DocumentFormat::Djot
            | DocumentFormat::Plain
            | DocumentFormat::Latex
            | DocumentFormat::Typst => result,
        }
    }

    fn annotate_integral_name_states(
        &self,
        citations: &mut [Citation],
        contexts: &[IntegralNameContext],
    ) {
        let citation_config = self.get_citation_config();
        let Some(config) = citation_config
            .integral_names
            .as_ref()
            .map(|cfg| cfg.resolve())
        else {
            return;
        };
        if !matches!(config.rule, IntegralNameRule::FullThenShort) {
            return;
        }

        let mut seen: HashMap<(String, String), SeenIntegralNameState> = HashMap::new();
        for (citation, context) in citations.iter_mut().zip(contexts.iter()) {
            if !matches!(
                citation.mode,
                citum_schema::citation::CitationMode::Integral
            ) {
                continue;
            }

            let is_body = matches!(context.placement, CitationPlacement::InlineProse);
            if matches!(config.contexts, IntegralNameContexts::BodyOnly) && !is_body {
                continue;
            }

            let scope_key = match config.scope {
                IntegralNameScope::Document => "document".to_string(),
                IntegralNameScope::Chapter => context.structure.chapter_scope.clone(),
                IntegralNameScope::Section => context.structure.section_scope.clone(),
            };

            for item in &mut citation.items {
                if item.integral_name_state.is_some() {
                    continue;
                }

                let key = (scope_key.clone(), item.id.clone());
                let state = seen
                    .get(&key)
                    .copied()
                    .unwrap_or(SeenIntegralNameState::Unseen);
                let derived = match config.contexts {
                    IntegralNameContexts::BodyOnly => {
                        if matches!(state, SeenIntegralNameState::BodySeen) {
                            citum_schema::citation::IntegralNameState::Subsequent
                        } else {
                            seen.insert(key, SeenIntegralNameState::BodySeen);
                            citum_schema::citation::IntegralNameState::First
                        }
                    }
                    IntegralNameContexts::BodyAndNotes => {
                        if is_body {
                            match state {
                                SeenIntegralNameState::BodySeen => {
                                    citum_schema::citation::IntegralNameState::Subsequent
                                }
                                SeenIntegralNameState::Unseen
                                | SeenIntegralNameState::NoteOnlySeen => {
                                    seen.insert(key, SeenIntegralNameState::BodySeen);
                                    citum_schema::citation::IntegralNameState::First
                                }
                            }
                        } else {
                            match state {
                                SeenIntegralNameState::Unseen => {
                                    seen.insert(key, SeenIntegralNameState::NoteOnlySeen);
                                    citum_schema::citation::IntegralNameState::First
                                }
                                SeenIntegralNameState::NoteOnlySeen
                                | SeenIntegralNameState::BodySeen => {
                                    citum_schema::citation::IntegralNameState::Subsequent
                                }
                            }
                        }
                    }
                };
                item.integral_name_state = Some(derived);
            }
        }
    }

    fn process_inline_document<F>(&self, content: &str, parsed: ParsedDocument) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut result = String::new();
        let mut last_idx = 0;
        let normalized = self.normalize_inline_document_citations(&parsed);

        for (parsed, citation) in parsed.citations.iter().zip(normalized.into_iter()) {
            result.push_str(&content[last_idx..parsed.start]);
            match self.process_citation_with_format::<F>(&citation) {
                Ok(rendered) => result.push_str(&rendered),
                Err(_) => result.push_str(&content[parsed.start..parsed.end]),
            }
            last_idx = parsed.end;
        }

        result.push_str(&content[last_idx..]);
        result
    }

    fn process_inline_document_html(
        &self,
        content: &str,
        parsed: ParsedDocument,
        placeholders: &mut HtmlPlaceholderRegistry,
    ) -> String {
        let mut result = String::new();
        let mut last_idx = 0;
        let normalized = self.normalize_inline_document_citations(&parsed);

        for (parsed, citation) in parsed.citations.iter().zip(normalized.into_iter()) {
            result.push_str(&content[last_idx..parsed.start]);
            match self.process_citation_with_format::<crate::render::html::Html>(&citation) {
                Ok(rendered) => result.push_str(&placeholders.push_inline(rendered)),
                Err(_) => result.push_str(&content[parsed.start..parsed.end]),
            }
            last_idx = parsed.end;
        }

        result.push_str(&content[last_idx..]);
        result
    }

    fn process_note_document<F>(&self, content: &str, mut parsed: ParsedDocument) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let (generated_notes, rendered_notes) =
            self.prepare_note_citations::<F>(content, &mut parsed);
        let note_rule = self.note_rule();

        let mut result = String::new();
        let mut last_idx = 0;
        for (index, parsed_citation) in parsed.citations.iter().enumerate() {
            result.push_str(&content[last_idx..parsed_citation.start]);
            match &parsed_citation.placement {
                CitationPlacement::ManualFootnote { .. } => {
                    if let Some(rendered) = rendered_notes.get(&index) {
                        result.push_str(rendered);
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                    }
                    last_idx = parsed_citation.end;
                }
                CitationPlacement::InlineProse => {
                    if let Some(note) = generated_notes
                        .iter()
                        .find(|note| note.citation_index == index)
                    {
                        if matches!(
                            parsed_citation.citation.mode,
                            citum_schema::citation::CitationMode::Integral
                        ) && let Ok(anchor) = self
                            .render_note_integral_anchor_with_format::<F>(&parsed_citation.citation)
                        {
                            result.push_str(&anchor);
                        }
                        let consumed_right = render_note_reference_in_prose(
                            &mut result,
                            &content[parsed_citation.end..],
                            &format!("[^{}]", note.label),
                            note_rule,
                        );
                        last_idx = parsed_citation.end + consumed_right;
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                        last_idx = parsed_citation.end;
                    }
                }
            }
        }
        result.push_str(&content[last_idx..]);

        if !generated_notes.is_empty() {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');

            for note in &generated_notes {
                if let Some(rendered) = rendered_notes.get(&note.citation_index) {
                    result.push_str(&format!("[^{}]: {}\n", note.label, rendered));
                }
            }
        }

        result
    }

    fn prepare_note_citation_state(
        &self,
        parsed: &mut ParsedDocument,
    ) -> (Vec<GeneratedNote>, HashMap<String, Vec<usize>>) {
        let mut used_labels = parsed.manual_note_labels.clone();
        let mut manual_numbers: HashMap<String, u32> = HashMap::new();
        let mut manual_citations: HashMap<String, Vec<usize>> = HashMap::new();
        let mut note_occurrences: Vec<NoteOccurrence> = parsed
            .manual_note_references
            .iter()
            .map(|note| NoteOccurrence::Manual {
                label: note.label.clone(),
                start: note.start,
            })
            .collect();

        for (index, parsed_citation) in parsed.citations.iter().enumerate() {
            match &parsed_citation.placement {
                CitationPlacement::InlineProse => {
                    note_occurrences.push(NoteOccurrence::Generated {
                        citation_index: index,
                        start: parsed_citation.start,
                    })
                }
                CitationPlacement::ManualFootnote { label } => {
                    manual_citations
                        .entry(label.clone())
                        .or_default()
                        .push(index);
                }
            }
        }

        for indices in manual_citations.values_mut() {
            indices.sort_by_key(|index| parsed.citations[*index].start);
        }

        note_occurrences.sort_by_key(NoteOccurrence::start);

        let mut next_note = 1_u32;
        let mut generated_notes = Vec::new();
        for occurrence in &note_occurrences {
            match occurrence {
                NoteOccurrence::Manual { label, .. } => {
                    manual_numbers.entry(label.clone()).or_insert_with(|| {
                        let current = next_note;
                        next_note = next_note.saturating_add(1);
                        current
                    });
                }
                NoteOccurrence::Generated { citation_index, .. } => {
                    let note_number = next_note;
                    next_note = next_note.saturating_add(1);
                    parsed.citations[*citation_index].citation.note_number = Some(note_number);
                    generated_notes.push(GeneratedNote {
                        citation_index: *citation_index,
                        label: next_generated_note_label(&mut used_labels, note_number),
                        note_number,
                    });
                }
            }
        }

        // Definitions without a matching in-body reference still need stable note context.
        let mut orphan_labels: Vec<_> = manual_citations
            .keys()
            .filter(|label| !manual_numbers.contains_key(*label))
            .cloned()
            .collect();
        orphan_labels.sort_by_key(|label| {
            manual_citations
                .get(label)
                .and_then(|indices| indices.first())
                .map(|index| parsed.citations[*index].start)
                .unwrap_or(usize::MAX)
        });
        for label in orphan_labels {
            manual_numbers.insert(label, {
                let current = next_note;
                next_note = next_note.saturating_add(1);
                current
            });
        }

        for (label, indices) in &manual_citations {
            if let Some(note_number) = manual_numbers.get(label).copied() {
                for index in indices {
                    parsed.citations[*index].citation.note_number = Some(note_number);
                }
            }
        }

        let ordered_indices = build_note_order_indices(&note_occurrences, &manual_citations);
        let mut ordered_citations: Vec<Citation> = ordered_indices
            .iter()
            .map(|index| parsed.citations[*index].citation.clone())
            .collect();
        let ordered_contexts: Vec<_> = ordered_indices
            .iter()
            .map(|index| IntegralNameContext {
                placement: parsed.citations[*index].placement.clone(),
                structure: parsed.citations[*index].structure.clone(),
            })
            .collect();
        self.annotate_integral_name_states(&mut ordered_citations, &ordered_contexts);
        ordered_citations = self.normalize_note_context(&ordered_citations);
        self.annotate_positions(&mut ordered_citations);

        for (ordered, index) in ordered_citations
            .into_iter()
            .zip(ordered_indices.into_iter())
        {
            parsed.citations[index].citation = ordered;
        }

        generated_notes.sort_by_key(|note| note.note_number);
        (generated_notes, manual_citations)
    }

    fn prepare_note_citations<F>(
        &self,
        content: &str,
        parsed: &mut ParsedDocument,
    ) -> (Vec<GeneratedNote>, HashMap<usize, String>)
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let (generated_notes, manual_citations) = self.prepare_note_citation_state(parsed);
        let mut rendered_notes: HashMap<usize, String> = HashMap::new();

        for generated in &generated_notes {
            let note_citation = self.note_render_citation_for_generated(
                &parsed.citations[generated.citation_index].citation,
            );
            let rendered = self
                .process_citation_with_format::<F>(&note_citation)
                .unwrap_or_else(|_| {
                    content[parsed.citations[generated.citation_index].start
                        ..parsed.citations[generated.citation_index].end]
                        .to_string()
                });
            rendered_notes.insert(generated.citation_index, rendered);
        }

        for (label, indices) in manual_citations {
            let _ = label;
            for index in indices {
                let rendered = self
                    .process_citation_with_format::<F>(&parsed.citations[index].citation)
                    .unwrap_or_else(|_| {
                        content[parsed.citations[index].start..parsed.citations[index].end]
                            .to_string()
                    });
                rendered_notes.insert(
                    index,
                    adjust_manual_note_citation_rendering(
                        &rendered,
                        &content[parsed.citations[index].end..],
                    ),
                );
            }
        }

        (generated_notes, rendered_notes)
    }

    fn prepare_note_citations_html(
        &self,
        content: &str,
        parsed: &mut ParsedDocument,
        placeholders: &mut HtmlPlaceholderRegistry,
    ) -> (Vec<GeneratedNote>, HashMap<usize, String>) {
        let (generated_notes, manual_citations) = self.prepare_note_citation_state(parsed);
        let mut rendered_notes = HashMap::new();

        for generated in &generated_notes {
            let note_citation = self.note_render_citation_for_generated(
                &parsed.citations[generated.citation_index].citation,
            );
            let rendered = self
                .process_citation_with_format::<crate::render::html::Html>(&note_citation)
                .unwrap_or_else(|_| {
                    content[parsed.citations[generated.citation_index].start
                        ..parsed.citations[generated.citation_index].end]
                        .to_string()
                });
            rendered_notes.insert(generated.citation_index, placeholders.push_inline(rendered));
        }

        for indices in manual_citations.values() {
            for index in indices {
                let rendered = self
                    .process_citation_with_format::<crate::render::html::Html>(
                        &parsed.citations[*index].citation,
                    )
                    .unwrap_or_else(|_| {
                        content[parsed.citations[*index].start..parsed.citations[*index].end]
                            .to_string()
                    });
                rendered_notes.insert(
                    *index,
                    placeholders.push_inline(adjust_manual_note_citation_rendering(
                        &rendered,
                        &content[parsed.citations[*index].end..],
                    )),
                );
            }
        }

        (generated_notes, rendered_notes)
    }

    fn process_note_document_html(
        &self,
        content: &str,
        mut parsed: ParsedDocument,
        placeholders: &mut HtmlPlaceholderRegistry,
    ) -> String {
        let (generated_notes, rendered_notes) =
            self.prepare_note_citations_html(content, &mut parsed, placeholders);
        let note_rule = self.note_rule();

        let mut result = String::new();
        let mut last_idx = 0;
        for (index, parsed_citation) in parsed.citations.iter().enumerate() {
            result.push_str(&content[last_idx..parsed_citation.start]);
            match &parsed_citation.placement {
                CitationPlacement::ManualFootnote { .. } => {
                    if let Some(rendered) = rendered_notes.get(&index) {
                        result.push_str(rendered);
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                    }
                    last_idx = parsed_citation.end;
                }
                CitationPlacement::InlineProse => {
                    if let Some(note) = generated_notes
                        .iter()
                        .find(|note| note.citation_index == index)
                    {
                        if matches!(
                            parsed_citation.citation.mode,
                            citum_schema::citation::CitationMode::Integral
                        ) && let Ok(anchor) = self
                            .render_note_integral_anchor_with_format::<crate::render::html::Html>(
                                &parsed_citation.citation,
                            )
                        {
                            result.push_str(&placeholders.push_inline(anchor));
                        }
                        let consumed_right = render_note_reference_in_prose(
                            &mut result,
                            &content[parsed_citation.end..],
                            &format!("[^{}]", note.label),
                            note_rule,
                        );
                        last_idx = parsed_citation.end + consumed_right;
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                        last_idx = parsed_citation.end;
                    }
                }
            }
        }
        result.push_str(&content[last_idx..]);

        if !generated_notes.is_empty() {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');

            for note in &generated_notes {
                if let Some(rendered) = rendered_notes.get(&note.citation_index) {
                    result.push_str(&format!("[^{}]: {}\n", note.label, rendered));
                }
            }
        }

        result
    }
}

impl Processor {
    fn note_render_citation_for_generated(&self, citation: &Citation) -> Citation {
        let mut note_citation = citation.clone();
        if matches!(
            note_citation.mode,
            citum_schema::citation::CitationMode::Integral
        ) {
            note_citation.mode = citum_schema::citation::CitationMode::NonIntegral;
        }
        note_citation
    }

    fn render_note_integral_anchor_with_format<F>(
        &self,
        citation: &Citation,
    ) -> Result<String, crate::error::ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let default_spec = citum_schema::CitationSpec::default();
        let effective_spec = self.style.citation.as_ref().map_or_else(
            || std::borrow::Cow::Borrowed(&default_spec),
            |cs| {
                let position_resolved = cs.resolve_for_position(citation.position.as_ref());
                let spec_for_mode = position_resolved.into_owned();
                std::borrow::Cow::Owned(
                    spec_for_mode
                        .resolve_for_mode(&citum_schema::citation::CitationMode::Integral)
                        .into_owned(),
                )
            },
        );

        let sorted_items = self.sort_citation_items(citation.items.clone(), &effective_spec);
        let inter_delimiter = effective_spec
            .multi_cite_delimiter
            .as_deref()
            .unwrap_or("; ");

        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            self.get_config(),
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );

        renderer.render_integral_anchor_with_format::<F>(
            &sorted_items,
            &effective_spec,
            inter_delimiter,
            citation.suppress_author,
            citation.position.as_ref(),
        )
    }

    fn note_rule(&self) -> NoteRule {
        if let Some(notes) = self.get_config().notes.as_ref() {
            return merge_note_rule(self.locale_note_rule(), notes);
        }

        self.locale_note_rule()
    }

    fn locale_note_rule(&self) -> NoteRule {
        let locale = self
            .style
            .info
            .default_locale
            .as_deref()
            .unwrap_or(self.locale.locale.as_str())
            .to_ascii_lowercase();
        match locale.as_str() {
            "en-us" => NoteRule {
                punctuation: PunctuationRule::Inside,
                number: NumberRule::Outside,
                order: NoteOrder::After,
            },
            tag if language_tag(tag) == "fr" => NoteRule {
                punctuation: PunctuationRule::Adaptive,
                number: NumberRule::Same,
                order: NoteOrder::Before,
            },
            _ => NoteRule {
                punctuation: PunctuationRule::Adaptive,
                number: NumberRule::Outside,
                order: NoteOrder::After,
            },
        }
    }
}

fn merge_note_rule(default: NoteRule, config: &StyleNoteConfig) -> NoteRule {
    NoteRule {
        punctuation: config
            .punctuation
            .map(map_quote_placement)
            .unwrap_or(default.punctuation),
        number: config
            .number
            .map(map_number_placement)
            .unwrap_or(default.number),
        order: config.order.map(map_note_order).unwrap_or(default.order),
    }
}

fn map_quote_placement(value: NoteQuotePlacement) -> PunctuationRule {
    match value {
        NoteQuotePlacement::Inside => PunctuationRule::Inside,
        NoteQuotePlacement::Outside => PunctuationRule::Outside,
        NoteQuotePlacement::Adaptive => PunctuationRule::Adaptive,
    }
}

fn map_number_placement(value: NoteNumberPlacement) -> NumberRule {
    match value {
        NoteNumberPlacement::Inside => NumberRule::Inside,
        NoteNumberPlacement::Outside => NumberRule::Outside,
        NoteNumberPlacement::Same => NumberRule::Same,
    }
}

fn map_note_order(value: NoteMarkerOrder) -> NoteOrder {
    match value {
        NoteMarkerOrder::Before => NoteOrder::Before,
        NoteMarkerOrder::After => NoteOrder::After,
    }
}

fn language_tag(locale: &str) -> &str {
    locale.split('-').next().unwrap_or(locale)
}

fn rewrite_group_headings_for_document(rendered: String, format: DocumentFormat) -> String {
    match format {
        DocumentFormat::Typst => rendered
            .lines()
            .map(|line| {
                if let Some(rest) = line.strip_prefix("# ") {
                    format!("== {rest}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => rendered,
    }
}

fn rewrite_document_markup_for_typst(rendered: String, format: DocumentFormat) -> String {
    match format {
        DocumentFormat::Typst => {
            let mut seen_labels = HashSet::new();
            rendered
                .lines()
                .map(|line| {
                    let hashes = line.chars().take_while(|ch| *ch == '#').count();
                    let normalized = if hashes > 0 && line.chars().nth(hashes) == Some(' ') {
                        format!("{}{}", "=".repeat(hashes), &line[hashes..])
                    } else {
                        line.to_string()
                    };

                    if let Some(idx) = normalized.rfind(" <ref-")
                        && normalized.ends_with('>')
                    {
                        let label = &normalized[idx + 2..normalized.len() - 1];
                        if !seen_labels.insert(label.to_string()) {
                            return normalized[..idx].to_string();
                        }
                    }

                    normalized
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => rendered,
    }
}

fn render_note_reference_in_prose(
    result: &mut String,
    right: &str,
    note_ref: &str,
    rule: NoteRule,
) -> usize {
    let left = pop_left_context(result);
    let right_ctx = inspect_right_context(right);

    let quote = left.quote.or(right_ctx.quote);
    if let Some(quote_char) = quote {
        let mut inside_punctuation = if left.quote.is_some() {
            left.punctuation
        } else {
            None
        };
        let mut outside_punctuation = if right_ctx.quote.is_some() || left.quote.is_some() {
            right_ctx.punctuation
        } else {
            None
        };

        if inside_punctuation.is_some() ^ outside_punctuation.is_some() {
            let punctuation = inside_punctuation.take().or(outside_punctuation.take());
            match desired_punctuation_side(rule, left.punctuation.is_some() && left.quote.is_some())
            {
                QuoteSide::Inside => inside_punctuation = punctuation,
                QuoteSide::Outside => outside_punctuation = punctuation,
            }
        }

        let note_side = desired_note_side(rule, inside_punctuation, outside_punctuation);
        let inside = side_content(
            note_side == QuoteSide::Inside,
            inside_punctuation,
            rule.order,
            note_ref,
        );
        let outside = side_content(
            note_side == QuoteSide::Outside,
            outside_punctuation,
            rule.order,
            note_ref,
        );

        result.push_str(&inside);
        result.push(quote_char);
        result.push_str(&outside);
        right_ctx.consumed_len
    } else {
        let punctuation = right_ctx.punctuation.or(left.punctuation);
        result.push_str(&side_content(true, punctuation, rule.order, note_ref));
        right_ctx.consumed_len
    }
}

fn desired_punctuation_side(rule: NoteRule, punctuation_inside_quote: bool) -> QuoteSide {
    match rule.punctuation {
        PunctuationRule::Inside => QuoteSide::Inside,
        PunctuationRule::Outside => QuoteSide::Outside,
        PunctuationRule::Adaptive => {
            if punctuation_inside_quote {
                QuoteSide::Inside
            } else {
                QuoteSide::Outside
            }
        }
    }
}

fn desired_note_side(
    rule: NoteRule,
    inside_punctuation: Option<char>,
    outside_punctuation: Option<char>,
) -> QuoteSide {
    match rule.number {
        NumberRule::Inside => QuoteSide::Inside,
        NumberRule::Outside => QuoteSide::Outside,
        NumberRule::Same => match (inside_punctuation.is_some(), outside_punctuation.is_some()) {
            (true, false) => QuoteSide::Inside,
            (false, true) => QuoteSide::Outside,
            _ => QuoteSide::Outside,
        },
    }
}

fn side_content(
    include_note: bool,
    punctuation: Option<char>,
    order: NoteOrder,
    note_ref: &str,
) -> String {
    match (include_note, punctuation) {
        (true, Some(punctuation)) => match order {
            NoteOrder::Before => format!("{note_ref}{punctuation}"),
            NoteOrder::After => format!("{punctuation}{note_ref}"),
        },
        (true, None) => note_ref.to_string(),
        (false, Some(punctuation)) => punctuation.to_string(),
        (false, None) => String::new(),
    }
}

fn pop_left_context(result: &mut String) -> LeftContext {
    while result.ends_with(char::is_whitespace) {
        result.pop();
    }

    let mut context = LeftContext::default();
    if let Some(last) = result.chars().last()
        && is_quote(last)
    {
        result.pop();
        context.quote = Some(last);
    }
    if let Some(last) = result.chars().last()
        && is_movable_punctuation(last)
    {
        result.pop();
        context.punctuation = Some(last);
    }
    context
}

fn inspect_right_context(right: &str) -> RightContext {
    let mut chars = right.char_indices();
    let mut context = RightContext::default();

    if let Some((idx, ch)) = chars.next() {
        if is_movable_punctuation(ch) {
            context.punctuation = Some(ch);
            context.consumed_len = idx + ch.len_utf8();
            if let Some((next_idx, next)) = chars.next()
                && is_quote(next)
            {
                context.quote = Some(next);
                context.consumed_len = next_idx + next.len_utf8();
            }
            return context;
        }
        if is_quote(ch) {
            context.quote = Some(ch);
            context.consumed_len = idx + ch.len_utf8();
            if let Some((next_idx, next)) = chars.next()
                && is_movable_punctuation(next)
            {
                context.punctuation = Some(next);
                context.consumed_len = next_idx + next.len_utf8();
            }
        }
    }
    context
}

fn is_movable_punctuation(ch: char) -> bool {
    MOVABLE_PUNCTUATION.contains(&ch)
}

fn is_quote(ch: char) -> bool {
    matches!(ch, '"' | '\'' | '”' | '’' | '»')
}

fn build_note_order_indices(
    note_occurrences: &[NoteOccurrence],
    manual_citations: &HashMap<String, Vec<usize>>,
) -> Vec<usize> {
    let mut ordered = Vec::new();
    let mut seen_manual = HashSet::new();

    for occurrence in note_occurrences {
        match occurrence {
            NoteOccurrence::Manual { label, .. } => {
                if seen_manual.insert(label.clone())
                    && let Some(indices) = manual_citations.get(label)
                {
                    ordered.extend(indices.iter().copied());
                }
            }
            NoteOccurrence::Generated { citation_index, .. } => ordered.push(*citation_index),
        }
    }

    let mut orphan_manual: Vec<_> = manual_citations
        .iter()
        .filter(|(label, _)| !seen_manual.contains(*label))
        .collect();
    orphan_manual.sort_by_key(|(_, indices)| indices.first().copied().unwrap_or(usize::MAX));
    for (_, indices) in orphan_manual {
        ordered.extend(indices.iter().copied());
    }

    ordered
}

fn build_integral_name_order_indices(parsed: &ParsedDocument) -> Vec<usize> {
    let mut manual_citations: HashMap<String, Vec<usize>> = HashMap::new();
    let mut note_occurrences: Vec<NoteOccurrence> = parsed
        .manual_note_references
        .iter()
        .map(|note| NoteOccurrence::Manual {
            label: note.label.clone(),
            start: note.start,
        })
        .collect();

    for (index, parsed_citation) in parsed.citations.iter().enumerate() {
        match &parsed_citation.placement {
            CitationPlacement::InlineProse => note_occurrences.push(NoteOccurrence::Generated {
                citation_index: index,
                start: parsed_citation.start,
            }),
            CitationPlacement::ManualFootnote { label } => {
                manual_citations
                    .entry(label.clone())
                    .or_default()
                    .push(index);
            }
        }
    }

    for indices in manual_citations.values_mut() {
        indices.sort_by_key(|index| parsed.citations[*index].start);
    }

    note_occurrences.sort_by_key(NoteOccurrence::start);
    build_note_order_indices(&note_occurrences, &manual_citations)
}

fn next_generated_note_label(used_labels: &mut HashSet<String>, note_number: u32) -> String {
    let mut candidate = note_number;
    loop {
        let label = format!("{GENERATED_NOTE_LABEL_PREFIX}{candidate}");
        if used_labels.insert(label.clone()) {
            return label;
        }
        candidate = candidate.saturating_add(1);
    }
}

fn adjust_manual_note_citation_rendering(rendered: &str, right: &str) -> String {
    let trimmed_right = right.trim_start_matches([' ', '\t']);
    if trimmed_right.is_empty() || trimmed_right.starts_with('\n') {
        return rendered.to_string();
    }

    trim_visible_terminal_period(rendered)
}

fn trim_visible_terminal_period(rendered: &str) -> String {
    let mut cut = rendered.len();
    loop {
        cut = rendered[..cut].trim_end().len();

        if !rendered[..cut].ends_with('>') {
            break;
        }

        let Some(tag_start) = rendered[..cut].rfind("</") else {
            break;
        };
        let tag = &rendered[tag_start..cut];
        if tag.ends_with('>') && !tag[2..tag.len() - 1].contains('<') {
            cut = tag_start;
            continue;
        }
        break;
    }

    let Some((period_idx, '.')) = rendered[..cut].char_indices().next_back() else {
        return rendered.to_string();
    };

    format!("{}{}", &rendered[..period_idx], &rendered[cut..])
}
