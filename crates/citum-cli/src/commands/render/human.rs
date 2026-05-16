/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Human-readable reference and citation rendering helpers.

use super::RenderContext;
use citum_engine::{Citation, CitationItem, Processor};
use citum_schema::options::Processing;
use std::collections::HashSet;
use std::fmt::Write as _;

/// Panic-safe wrapper around [`print_human`].
///
/// Catches any Rust panics that escape the processor and converts them into an
/// `Err` with a user-friendly message, preventing the CLI from crashing.
pub(super) fn print_human_safe<F>(
    ctx: &RenderContext<'_>,
    show_cite: bool,
    show_bib: bool,
    citations: Option<Vec<Citation>>,
    show_keys: bool,
) -> Result<String, String>
where
    F: citum_engine::render::format::OutputFormat<Output = String> + Send + Sync + 'static,
{
    use std::panic;

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        print_human::<F>(ctx, show_cite, show_bib, citations, show_keys)
    }));

    match result {
        Ok(output) => Ok(output),
        Err(_) => Err(
            "The processor encountered a critical error during rendering. Please report this issue with the style and data used."
                .to_string(),
        ),
    }
}

/// Core human-readable renderer for references and citations.
///
/// Builds a formatted string containing citation clusters and/or bibliography
/// entries, optionally prefixed with reference IDs when `show_keys` is `true`.
fn print_human<F>(
    ctx: &RenderContext<'_>,
    show_cite: bool,
    show_bib: bool,
    citations: Option<Vec<Citation>>,
    show_keys: bool,
) -> String
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    let mut output = String::new();
    let _ = writeln!(output, "\n=== {} ===\n", ctx.style_name);

    if show_cite {
        render_citations_section::<F>(ctx, citations, show_keys, &mut output);
    }

    if show_bib {
        render_bibliography_section::<F>(ctx, show_keys, &mut output);
    }

    output
}

/// Render the citations section into output.
fn render_citations_section<F>(
    ctx: &RenderContext<'_>,
    citations: Option<Vec<Citation>>,
    show_keys: bool,
    output: &mut String,
) where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    if let Some(cite_list) = citations {
        let _ = writeln!(output, "CITATIONS (From file):");
        for (i, (citation, text)) in cite_list
            .iter()
            .zip(render_citation_file_entries::<F>(ctx.processor, &cite_list))
            .enumerate()
        {
            if show_keys {
                let _ = writeln!(
                    output,
                    "  [{}] {}",
                    citation.id.as_deref().unwrap_or(&format!("{i}")),
                    text
                );
            } else {
                let _ = writeln!(output, "  {text}");
            }
        }
    } else {
        let _ = writeln!(output, "CITATIONS (Non-Integral):");
        for id in ctx.item_ids {
            let citation = Citation {
                id: Some(id.clone()),
                items: vec![CitationItem {
                    id: id.clone(),
                    ..Default::default()
                }],
                mode: citum_schema::citation::CitationMode::NonIntegral,
                ..Default::default()
            };
            match ctx.processor.process_citation_with_format::<F>(&citation) {
                Ok(text) => {
                    if show_keys {
                        let _ = writeln!(output, "  [{id}] {text}");
                    } else {
                        let _ = writeln!(output, "  {text}");
                    }
                }
                Err(e) => {
                    let _ = writeln!(output, "  [{id}] ERROR: {e}");
                }
            }
        }
        let _ = writeln!(output);

        let _ = writeln!(output, "CITATIONS (Integral):");
        for id in ctx.item_ids {
            let citation = Citation {
                id: Some(id.clone()),
                items: vec![CitationItem {
                    id: id.clone(),
                    ..Default::default()
                }],
                mode: citum_schema::citation::CitationMode::Integral,
                ..Default::default()
            };
            match ctx.processor.process_citation_with_format::<F>(&citation) {
                Ok(text) => {
                    if show_keys {
                        let _ = writeln!(output, "  [{id}] {text}");
                    } else {
                        let _ = writeln!(output, "  {text}");
                    }
                }
                Err(e) => {
                    let _ = writeln!(output, "  [{id}] ERROR: {e}");
                }
            }
        }
    }
    let _ = writeln!(output);
}

/// Render the bibliography section into output.
fn render_bibliography_section<F>(ctx: &RenderContext<'_>, show_keys: bool, output: &mut String)
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    let _ = writeln!(output, "BIBLIOGRAPHY:");
    if show_keys {
        // When show_keys is requested, render each entry with its ID prefix so the
        // oracle parser can match entries by key. Group headings are omitted in this
        // mode because the oracle only looks for `[id] text` patterns.
        let filter: HashSet<&str> = ctx
            .item_ids
            .iter()
            .map(std::string::String::as_str)
            .collect();
        let processed = ctx.processor.process_references();
        for entry in processed.bibliography {
            if filter.contains(entry.id.as_str()) {
                let text = citum_engine::render::refs_to_string_with_format::<F>(
                    vec![entry.clone()],
                    ctx.annotations,
                    Some(ctx.annotation_style),
                );
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let _ = writeln!(output, "  [{}] {}", entry.id, trimmed);
                }
            }
        }
    } else {
        // Use engine's built-in bibliography renderer which handles grouping/partitioning.
        // We use render_selected_bibliography_with_format_and_annotations to respect the CLI's item_ids filter and propagate annotations.
        let rendered = ctx
            .processor
            .render_selected_bibliography_with_format_and_annotations::<F, _>(
                ctx.item_ids.to_vec(),
                ctx.annotations,
                Some(ctx.annotation_style),
            );
        output.push_str(&rendered);
        if !rendered.is_empty() && !rendered.ends_with('\n') {
            output.push('\n');
        }
    }
}

/// Render citation-file inputs in batch mode, falling back to per-citation rendering on error.
pub(super) fn render_citation_file_entries<F>(
    processor: &Processor,
    citations: &[Citation],
) -> Vec<String>
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    let is_numeric = processor
        .style
        .options
        .as_ref()
        .and_then(|config| config.processing.as_ref())
        .is_some_and(|processing| matches!(processing, Processing::Numeric));

    if is_numeric {
        processor
            .process_citations_with_format::<F>(citations)
            .unwrap_or_else(|_| render_citation_file_entries_one_by_one::<F>(processor, citations))
    } else {
        render_citation_file_entries_one_by_one::<F>(processor, citations)
    }
}

fn format_citation_file_render_error(error: impl std::fmt::Display) -> String {
    format!("ERROR: {error}")
}

/// Render citation-file inputs independently to avoid cross-citation context.
fn render_citation_file_entries_one_by_one<F>(
    processor: &Processor,
    citations: &[Citation],
) -> Vec<String>
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    citations
        .iter()
        .map(|citation| {
            processor
                .process_citation_with_format::<F>(citation)
                .unwrap_or_else(format_citation_file_render_error)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_citation_file_render_error_prefixes_message() {
        assert_eq!(format_citation_file_render_error("boom"), "ERROR: boom");
    }
}
