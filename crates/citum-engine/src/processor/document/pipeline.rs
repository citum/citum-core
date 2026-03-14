/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! High-level document-processing orchestration.

use super::output::{
    HtmlPlaceholderRegistry, RenderedDocumentBody, append_document_bibliography,
    bibliography_block_placeholder, render_document_bibliography_block_replacement,
    rewrite_document_markup_for_typst, rewrite_group_headings_for_document,
    stage_document_bibliography_blocks,
};
use super::{CitationParser, DocumentFormat, ParsedDocument, djot};
use crate::processor::Processor;

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
        let mut parsed = parser.parse_document(content, &self.locale);
        let owned_processor = self.processor_with_document_integral_name_override(
            parsed.frontmatter_integral_names.as_ref(),
        );
        let processor = owned_processor.as_ref().unwrap_or(self);
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

    fn process_document_with_bibliography_blocks<P, F>(
        &self,
        body: &str,
        blocks: Vec<djot::BibliographyBlock>,
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
        self.finalize_document_output(parser, format, rendered)
    }

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
            |processor| processor.render_grouped_bibliography_with_format::<F>(),
        )
    }

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
        self.finalize_document_output(parser, format, rendered)
    }

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

    fn replace_document_bibliography_blocks<F>(
        &self,
        rendered: &mut RenderedDocumentBody,
        blocks: &[djot::BibliographyBlock],
        format: DocumentFormat,
    ) where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        for (index, block) in blocks.iter().enumerate() {
            let placeholder = bibliography_block_placeholder(index);
            let rendered_group = self.render_document_bibliography_block::<F>(&block.group);
            let replacement = render_document_bibliography_block_replacement(
                rendered.placeholders.as_mut(),
                format,
                rendered_group.heading,
                rendered_group.body,
            );
            rendered.content = rendered.content.replace(&placeholder, &replacement);
        }
    }

    fn finalize_document_output<P>(
        &self,
        parser: &P,
        format: DocumentFormat,
        rendered: RenderedDocumentBody,
    ) -> String
    where
        P: CitationParser,
    {
        let result = rewrite_document_markup_for_typst(rendered.content, format);
        match rendered.placeholders {
            Some(placeholders) => placeholders.apply(parser.finalize_html_output(&result)),
            None => match format {
                DocumentFormat::Html => parser.finalize_html_output(&result),
                DocumentFormat::Djot
                | DocumentFormat::Plain
                | DocumentFormat::Latex
                | DocumentFormat::Typst => result,
            },
        }
    }
}
