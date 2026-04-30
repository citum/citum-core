/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Document-output helpers shared by the processing pipeline.

use super::{BibliographyBlock, DocumentFormat};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub(super) struct HtmlPlaceholderRegistry {
    next_index: usize,
    inline_replacements: Vec<(String, String)>,
    block_replacements: Vec<(String, String)>,
}

impl HtmlPlaceholderRegistry {
    pub(super) fn push_inline(&mut self, html: String) -> String {
        let token = self.next_token("INLINE");
        self.inline_replacements.push((token.clone(), html));
        token
    }

    pub(super) fn push_block(&mut self, html: String) -> String {
        let token = self.next_token("BLOCK");
        self.block_replacements.push((token.clone(), html));
        token
    }

    pub(super) fn apply(self, rendered: String) -> String {
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
        let token = format!("\x00CITUMHTML{kind}TOKEN{}\x00", self.next_index);
        self.next_index = self.next_index.saturating_add(1);
        token
    }
}

#[derive(Debug)]
pub(super) struct RenderedDocumentBody {
    pub(super) content: String,
    pub(super) placeholders: Option<HtmlPlaceholderRegistry>,
}

pub(super) fn stage_document_bibliography_blocks(
    body: &str,
    blocks: &[BibliographyBlock],
) -> String {
    let mut staged = body.to_string();
    for (index, block) in blocks.iter().enumerate().rev() {
        staged.replace_range(
            block.start..block.end,
            &bibliography_block_placeholder(index),
        );
    }
    staged
}

pub(super) fn bibliography_block_placeholder(index: usize) -> String {
    format!("\x00BIBBLOCK{index}\x00")
}

pub(super) fn append_document_bibliography(
    rendered: &mut RenderedDocumentBody,
    format: DocumentFormat,
    bibliography: String,
) {
    if bibliography.trim().is_empty() {
        return;
    }

    rendered
        .content
        .push_str(document_bibliography_heading(format));
    if let Some(placeholders) = rendered.placeholders.as_mut() {
        rendered
            .content
            .push_str(&placeholders.push_block(bibliography));
    } else {
        rendered.content.push_str(&bibliography);
    }
}

fn document_bibliography_heading(format: DocumentFormat) -> &'static str {
    match format {
        DocumentFormat::Latex => "\n\n\\section*{Bibliography}\n\n",
        DocumentFormat::Typst => "\n\n= Bibliography\n\n",
        DocumentFormat::Plain
        | DocumentFormat::Djot
        | DocumentFormat::Markdown
        | DocumentFormat::Html => "\n\n# Bibliography\n\n",
    }
}

pub(super) fn render_document_bibliography_block_replacement(
    placeholders: Option<&mut HtmlPlaceholderRegistry>,
    format: DocumentFormat,
    heading: Option<String>,
    body: String,
) -> String {
    if let Some(placeholders) = placeholders {
        let token = placeholders.push_block(body);
        return match heading {
            Some(heading) => format!("## {heading}\n\n{token}\n"),
            None => format!("{token}\n"),
        };
    }

    match heading {
        Some(heading) => {
            let prefix = match format {
                DocumentFormat::Latex => format!("\\subsection*{{{heading}}}\n\n"),
                DocumentFormat::Typst => format!("== {heading}\n\n"),
                DocumentFormat::Plain
                | DocumentFormat::Djot
                | DocumentFormat::Markdown
                | DocumentFormat::Html => {
                    format!("## {heading}\n\n")
                }
            };
            format!("{prefix}{body}\n")
        }
        None => format!("{body}\n"),
    }
}

pub(super) fn rewrite_group_headings_for_document(
    rendered: String,
    format: DocumentFormat,
) -> String {
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

#[allow(
    clippy::string_slice,
    clippy::indexing_slicing,
    reason = "parser-guaranteed boundaries and indices"
)]
pub(super) fn rewrite_document_markup_for_typst(
    rendered: String,
    format: DocumentFormat,
) -> String {
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
