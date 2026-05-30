/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Djot markup to OutputFormat adapter.
//!
//! Converts jotdown block+inline events to the frame-stack renderer.

use super::renderer::{FrameKind, MarkupRenderer};
use crate::render::format::OutputFormat;

/// Render a Djot document body to the target output format.
///
/// Parses `body` using jotdown in block mode and routes events through the
/// shared frame-stack renderer, preserving both block and inline structure.
pub(crate) fn render_djot_body<F: OutputFormat<Output = String>>(body: &str, fmt: &F) -> String {
    let mut renderer = MarkupRenderer::new(fmt);

    for event in jotdown::Parser::new(body) {
        match event {
            jotdown::Event::Start(container, _attrs) => {
                let kind = match container {
                    // Top-level document wrapper — the renderer already has a root frame.
                    jotdown::Container::Document => FrameKind::Root,
                    jotdown::Container::Paragraph => FrameKind::Paragraph,
                    jotdown::Container::Blockquote => FrameKind::BlockQuote,
                    jotdown::Container::List { kind, .. } => match kind {
                        jotdown::ListKind::Ordered { .. } => FrameKind::OrderedList,
                        // Task lists are rendered as bullet lists.
                        jotdown::ListKind::Unordered(_) | jotdown::ListKind::Task(_) => {
                            FrameKind::BulletList
                        }
                    },
                    jotdown::Container::ListItem
                    | jotdown::Container::TaskListItem { .. }
                    | jotdown::Container::DescriptionDetails => FrameKind::ListItem,
                    jotdown::Container::Heading { level, .. } => {
                        FrameKind::Heading { level: level as u8 }
                    }
                    jotdown::Container::CodeBlock { language } => {
                        let lang = if language.is_empty() {
                            None
                        } else {
                            Some(language.to_string())
                        };
                        FrameKind::CodeBlock { lang }
                    }
                    jotdown::Container::Emphasis => FrameKind::Emph,
                    jotdown::Container::Strong => FrameKind::Strong,
                    jotdown::Container::Delete => FrameKind::Strikeout,
                    jotdown::Container::Link(url, _) => FrameKind::Link {
                        url: url.to_string(),
                    },
                    jotdown::Container::Verbatim => FrameKind::InlineCode,
                    // All other containers (Div, Span, Table, …) collapse to text.
                    _ => FrameKind::Transparent,
                };
                renderer.start_container(kind);
            }
            jotdown::Event::End(_) => {
                renderer.end_container();
            }
            jotdown::Event::Str(s) => {
                if renderer.in_code_block() {
                    renderer.push_raw_text(s.to_string());
                } else {
                    renderer.push_text(s.to_string());
                }
            }
            jotdown::Event::Softbreak => {
                renderer.push_soft_break();
            }
            jotdown::Event::Hardbreak => {
                renderer.push_hard_break();
            }
            // Other leaf events (ThematicBreak, Blankline, …) are skipped.
            _ => {}
        }
    }

    renderer.finish()
}
