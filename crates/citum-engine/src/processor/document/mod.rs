/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Document-level citation processing.

pub mod djot;

#[cfg(test)]
mod tests;

use crate::Citation;
use crate::processor::Processor;
use std::collections::{HashMap, HashSet};

const GENERATED_NOTE_LABEL_PREFIX: &str = "citum-auto-";

/// Describes where a parsed citation appears in the source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationPlacement {
    /// The citation marker appears in prose and should become a generated note
    /// reference for note styles.
    InlineProse,
    /// The citation marker appears inside a manually authored footnote
    /// definition and should render in place.
    ManualFootnote { label: String },
}

/// A citation marker parsed from a document.
#[derive(Debug, Clone)]
pub struct ParsedCitation {
    pub start: usize,
    pub end: usize,
    pub citation: Citation,
    pub placement: CitationPlacement,
}

#[derive(Debug, Clone)]
pub(crate) struct ManualNoteReference {
    pub label: String,
    pub start: usize,
}

/// Structured output from a document parser.
#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    pub citations: Vec<ParsedCitation>,
    pub manual_note_order: Vec<String>,
    pub(crate) manual_note_references: Vec<ManualNoteReference>,
    pub(crate) manual_note_labels: HashSet<String>,
}

/// A trait for document parsers that can identify citations.
pub trait CitationParser {
    /// Parse the document into citation placements and note metadata.
    fn parse_document(&self, content: &str) -> ParsedDocument;

    /// Find and extract citations from a document string.
    /// Returns a list of (start_index, end_index, citation_model) tuples.
    fn parse_citations(&self, content: &str) -> Vec<(usize, usize, Citation)> {
        self.parse_document(content)
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
}

#[derive(Debug, Clone)]
struct GeneratedNote {
    citation_index: usize,
    label: String,
    note_number: u32,
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

impl Processor {
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
        let parsed = parser.parse_document(content);

        let rendered = if self.is_note_style() {
            self.process_note_document::<F>(content, parsed)
        } else {
            self.process_inline_document::<F>(content, parsed)
        };

        let bib_heading = match format {
            DocumentFormat::Latex => "\n\n\\section*{Bibliography}\n\n",
            _ => "\n\n# Bibliography\n\n",
        };
        let mut result = rendered;
        result.push_str(bib_heading);
        result.push_str(&self.render_grouped_bibliography_with_format::<F>());

        match format {
            DocumentFormat::Html => self::djot::djot_to_html(&result),
            DocumentFormat::Djot | DocumentFormat::Plain | DocumentFormat::Latex => result,
        }
    }

    fn process_inline_document<F>(&self, content: &str, parsed: ParsedDocument) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut result = String::new();
        let mut last_idx = 0;
        let citation_models: Vec<Citation> = parsed
            .citations
            .iter()
            .map(|parsed| parsed.citation.clone())
            .collect();
        let normalized = self.normalize_note_context(&citation_models);

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

    fn process_note_document<F>(&self, content: &str, mut parsed: ParsedDocument) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let (generated_notes, rendered_notes) =
            self.prepare_note_citations::<F>(content, &mut parsed);

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
                }
                CitationPlacement::InlineProse => {
                    if let Some(note) = generated_notes
                        .iter()
                        .find(|note| note.citation_index == index)
                    {
                        result.push_str(&format!("[^{}]", note.label));
                    } else {
                        result.push_str(&content[parsed_citation.start..parsed_citation.end]);
                    }
                }
            }
            last_idx = parsed_citation.end;
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

    fn prepare_note_citations<F>(
        &self,
        content: &str,
        parsed: &mut ParsedDocument,
    ) -> (Vec<GeneratedNote>, HashMap<usize, String>)
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
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
        let mut rendered_notes = HashMap::new();
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
        ordered_citations = self.normalize_note_context(&ordered_citations);
        self.annotate_positions(&mut ordered_citations);

        for (ordered, index) in ordered_citations
            .into_iter()
            .zip(ordered_indices.into_iter())
        {
            parsed.citations[index].citation = ordered;
        }

        generated_notes.sort_by_key(|note| note.note_number);
        for generated in &generated_notes {
            let rendered = self
                .process_citation_with_format::<F>(
                    &parsed.citations[generated.citation_index].citation,
                )
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
                rendered_notes.insert(index, rendered);
            }
        }

        (generated_notes, rendered_notes)
    }
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
