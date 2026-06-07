/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! High-level document-processing orchestration.

use super::output::{
    HtmlPlaceholderRegistry, RenderedDocumentBody, append_document_bibliography,
    bibliography_block_placeholder, render_document_bibliography_block_replacement,
    rewrite_document_markup_for_typst, rewrite_group_headings_for_document,
    stage_document_bibliography_blocks,
};
use super::{BibliographyBlock, CitationParser, DocumentFormat, ParsedDocument};
use crate::processor::Processor;

impl Processor {
    /// Process citations in a document and append a bibliography.
    ///
    /// This is the primary document-level entry point. It:
    /// 1. Parses the source document using the provided adapter.
    /// 2. Resolves frontmatter overrides (integral-name policy, bibliography options).
    /// 3. Chooses a bibliography orchestration path based on frontmatter and document blocks.
    #[allow(
        clippy::string_slice,
        reason = "parser-guaranteed boundaries and indices"
    )]
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
        let mut parsed = parser.parse_document(content, &self.locale);

        if let Some(err) = &parsed.frontmatter_error {
            eprintln!("citum: error: frontmatter parse error: {err}");
            std::process::exit(1);
        }

        // `options.*` fields take precedence over the legacy top-level fields.
        let effective_integral_override = parsed
            .frontmatter_options
            .as_ref()
            .and_then(|o| o.integral_name_memory.as_ref())
            .or(parsed.frontmatter_integral_name_memory.as_ref());
        let owned_integral =
            self.processor_with_document_integral_name_override(effective_integral_override);

        // `options.org-abbreviation-memory` takes precedence over the legacy top-level field.
        let effective_org_override = parsed
            .frontmatter_options
            .as_ref()
            .and_then(|o| o.org_abbreviation_memory.as_ref())
            .or(parsed.frontmatter_org_abbreviation_memory.as_ref());
        let owned_org = {
            let base = owned_integral.as_ref().unwrap_or(self);
            base.processor_with_document_org_abbreviation_override(effective_org_override)
        };

        // Apply bibliography overrides from the options block.
        let owned_bib = parsed
            .frontmatter_options
            .as_ref()
            .filter(|o| o.bibliography.is_some())
            .map(|options| {
                let base = owned_org
                    .as_ref()
                    .or(owned_integral.as_ref())
                    .unwrap_or(self);
                base.processor_with_bibliography_override(options)
            });

        let processor = owned_bib
            .as_ref()
            .or(owned_org.as_ref())
            .or(owned_integral.as_ref())
            .unwrap_or(self);
        let body = &content[parsed.body_start..];
        if let Some(groups) = parsed.frontmatter_groups.take() {
            return processor.process_document_with_frontmatter_groups::<P, F>(
                body, parsed, groups, parser, format,
            );
        }

        if !parsed.bibliography_blocks.is_empty() {
            return processor.process_document_with_bibliography_blocks::<P, F>(
                body,
                std::mem::take(&mut parsed.bibliography_blocks),
                parser,
                format,
            );
        }

        processor.process_document_with_default_bibliography::<P, F>(body, parsed, parser, format)
    }

    /// Orchestrate document processing with custom frontmatter bibliography groups.
    fn process_document_with_frontmatter_groups<P, F>(
        &self,
        body: &str,
        parsed: ParsedDocument,
        groups: Vec<citum_schema::grouping::BibliographyGroup>,
        parser: &P,
        format: DocumentFormat,
    ) -> String
    where
        P: CitationParser,
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.render_document_with_trailing_bibliography::<P, F, _>(
            body,
            parsed,
            parser,
            format,
            |processor| {
                rewrite_group_headings_for_document(
                    processor.render_document_bibliography_groups::<F>(&groups),
                    format,
                )
            },
        )
    }

    /// Orchestrate document processing with explicit bibliography blocks.
    fn process_document_with_bibliography_blocks<P, F>(
        &self,
        body: &str,
        blocks: Vec<BibliographyBlock>,
        parser: &P,
        format: DocumentFormat,
    ) -> String
    where
        P: CitationParser,
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let staged = stage_document_bibliography_blocks(body, &blocks);
        let parsed_staged = parser.parse_document(&staged, &self.locale);
        let mut rendered = self.render_document_body::<F>(&staged, parsed_staged, format);
        self.replace_document_bibliography_blocks::<F>(&mut rendered, &blocks, format);
        self.finalize_document_output::<P, F>(parser, format, rendered)
    }

    /// Orchestrate document processing with the default trailing bibliography.
    fn process_document_with_default_bibliography<P, F>(
        &self,
        body: &str,
        parsed: ParsedDocument,
        parser: &P,
        format: DocumentFormat,
    ) -> String
    where
        P: CitationParser,
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.render_document_with_trailing_bibliography::<P, F, _>(
            body,
            parsed,
            parser,
            format,
            super::super::Processor::render_grouped_document_bibliography_with_format::<F>,
        )
    }

    /// Generic helper for rendering document body + trailing bibliography.
    fn render_document_with_trailing_bibliography<P, F, B>(
        &self,
        body: &str,
        parsed: ParsedDocument,
        parser: &P,
        format: DocumentFormat,
        render_bibliography: B,
    ) -> String
    where
        P: CitationParser,
        F: crate::render::format::OutputFormat<Output = String>,
        B: FnOnce(&Self) -> String,
    {
        let mut rendered = self.render_document_body::<F>(body, parsed, format);
        let bibliography = render_bibliography(self);
        append_document_bibliography(&mut rendered, format, bibliography);
        self.finalize_document_output::<P, F>(parser, format, rendered)
    }

    /// Render the citation-annotated document body.
    ///
    /// Governs the choice between note-style and inline-style processing,
    /// and handles placeholder registration for format finalization.
    /// HTML and terminal formats (Typst, LaTeX) both use the placeholder path
    /// so that body markup can be converted after citations are spliced in.
    fn render_document_body<F>(
        &self,
        content: &str,
        parsed: ParsedDocument,
        format: DocumentFormat,
    ) -> RenderedDocumentBody
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        if matches!(format, DocumentFormat::Html) {
            let mut placeholders = HtmlPlaceholderRegistry::default();
            let content = if self.is_note_style() {
                self.process_note_document_html(content, parsed, &mut placeholders)
            } else {
                self.process_inline_document_html(content, parsed, &mut placeholders)
            };
            return RenderedDocumentBody {
                content,
                placeholders: Some(placeholders),
                trailing: None,
            };
        }

        // Terminal formats (Typst, LaTeX) need the same placeholder flow so
        // the body markup can be converted to the target format after citations
        // are replaced with NUL-token placeholders. This is a converted-output
        // path, not passthrough; passthrough is limited to Plain/Djot/Markdown.
        if matches!(format, DocumentFormat::Typst | DocumentFormat::Latex) {
            let mut placeholders = HtmlPlaceholderRegistry::default();
            // Note styles still emit source footnote syntax that the terminal
            // body renderer does not yet model, so keep that narrow legacy
            // exception isolated from author-date terminal conversion.
            let content = if self.is_note_style() {
                self.process_note_document::<F>(content, parsed)
            } else {
                self.process_inline_document_with_placeholders::<F>(
                    content,
                    parsed,
                    &mut placeholders,
                )
            };
            return RenderedDocumentBody {
                content,
                placeholders: if self.is_note_style() {
                    None
                } else {
                    Some(placeholders)
                },
                trailing: None,
            };
        }

        let content = if self.is_note_style() {
            self.process_note_document::<F>(content, parsed)
        } else {
            self.process_inline_document::<F>(content, parsed)
        };

        RenderedDocumentBody {
            content,
            placeholders: None,
            trailing: None,
        }
    }

    /// Splice `F`-rendered citations into document markup using NUL placeholders.
    ///
    /// Mirrors `process_inline_document_html` but renders citations using the
    /// generic format `F` (e.g. Typst or LaTeX) instead of HTML. The
    /// surrounding body markup still contains the source syntax at this point;
    /// `finalize_document_output` converts it after placeholder substitution.
    #[allow(
        clippy::string_slice,
        reason = "parser-guaranteed boundaries and indices"
    )]
    fn process_inline_document_with_placeholders<F>(
        &self,
        content: &str,
        parsed: ParsedDocument,
        placeholders: &mut HtmlPlaceholderRegistry,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut result = String::new();
        let mut last_idx = 0;
        let normalized = self.normalize_integral_name_citations(&parsed);

        for (parsed, citation) in parsed.citations.iter().zip(normalized) {
            result.push_str(&content[last_idx..parsed.start]);
            match self.process_citation_with_format::<F>(&citation) {
                Ok(rendered) => result.push_str(&placeholders.push_inline(rendered)),
                Err(_) => result.push_str(&content[parsed.start..parsed.end]),
            }
            last_idx = parsed.end;
        }

        result.push_str(&content[last_idx..]);
        result
    }

    /// Splice rendered citations into document markup for non-note styles.
    #[allow(
        clippy::string_slice,
        reason = "parser-guaranteed boundaries and indices"
    )]
    fn process_inline_document<F>(&self, content: &str, parsed: ParsedDocument) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut result = String::new();
        let mut last_idx = 0;
        let normalized = self.normalize_integral_name_citations(&parsed);

        for (parsed, citation) in parsed.citations.iter().zip(normalized) {
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

    /// Splice HTML-rendered citations into document markup using placeholders.
    #[allow(
        clippy::string_slice,
        reason = "parser-guaranteed boundaries and indices"
    )]
    fn process_inline_document_html(
        &self,
        content: &str,
        parsed: ParsedDocument,
        placeholders: &mut HtmlPlaceholderRegistry,
    ) -> String {
        let mut result = String::new();
        let mut last_idx = 0;
        let normalized = self.normalize_integral_name_citations(&parsed);

        for (parsed, citation) in parsed.citations.iter().zip(normalized) {
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

    /// Replace bibliography block placeholders with rendered content.
    fn replace_document_bibliography_blocks<F>(
        &self,
        rendered: &mut RenderedDocumentBody,
        blocks: &[BibliographyBlock],
        format: DocumentFormat,
    ) where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut assigned = std::collections::HashSet::<String>::new();
        for (index, block) in blocks.iter().enumerate() {
            let placeholder = bibliography_block_placeholder(index);
            let rendered_group =
                self.render_document_bibliography_block::<F>(&block.group, &mut assigned);
            let replacement = render_document_bibliography_block_replacement(
                rendered.placeholders.as_mut(),
                format,
                rendered_group.heading,
                rendered_group.body,
            );
            rendered.content = rendered.content.replace(&placeholder, &replacement);
        }
    }

    /// Perform final document rewrites and resolve placeholders.
    ///
    /// For HTML: converts body markup via `finalize_html_output` then
    /// substitutes citation placeholder tokens.
    /// For Typst/LaTeX: converts body markup via `render_body_markup::<F>`
    /// then substitutes citation placeholder tokens.
    /// For other formats: returns the spliced content as-is.
    fn finalize_document_output<P, F>(
        &self,
        parser: &P,
        format: DocumentFormat,
        rendered: RenderedDocumentBody,
    ) -> String
    where
        P: CitationParser,
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut result = if let Some(placeholders) = rendered.placeholders {
            let fmt = F::default();
            let converted = match format {
                DocumentFormat::Html => parser.finalize_html_output(&rendered.content),
                DocumentFormat::Typst | DocumentFormat::Latex => {
                    parser.render_body_markup(&rendered.content, &fmt)
                }
                _ => rendered.content,
            };
            placeholders.apply(converted)
        } else {
            // Passthrough path for Plain/Djot/Markdown, plus the isolated
            // note-style Typst/LaTeX exception documented in render_document_body.
            // Keep the heading-rewrite for Typst in case headings came from
            // bibliography group labels rather than body markup.
            let content = rewrite_document_markup_for_typst(rendered.content, format);
            match format {
                DocumentFormat::Html => parser.finalize_html_output(&content),
                _ => content,
            }
        };
        // Append any trailing content (e.g. Typst/LaTeX bibliography) that was
        // deferred so it would not pass through the body markup converter.
        // Trim the body's trailing whitespace first: the markup renderer may
        // have added paragraph-separator newlines that would otherwise double
        // the leading newlines of the bibliography heading.
        if let Some(tail) = rendered.trailing {
            let trimmed = result.trim_end_matches('\n');
            result = format!("{trimmed}{tail}");
        }
        result
    }
}
