/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Markdown markup to OutputFormat adapter.
//!
//! Converts pulldown-cmark block+inline events to the frame-stack renderer.

use super::renderer::{FrameKind, MarkupRenderer};
use crate::render::format::OutputFormat;
use pulldown_cmark::{Event, HeadingLevel, Options, Tag, TagEnd};

/// Render a Markdown document body to the target output format.
///
/// Parses `body` using pulldown-cmark with strikethrough enabled and routes
/// events through the shared frame-stack renderer, preserving both block and
/// inline structure. Markdown `*x*` → emphasis, `**x**` → strong.
pub(crate) fn render_markdown_body<F: OutputFormat<Output = String>>(
    body: &str,
    fmt: &F,
) -> String {
    let mut renderer = MarkupRenderer::new(fmt);
    let parser = pulldown_cmark::Parser::new_ext(body, Options::ENABLE_STRIKETHROUGH);

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    renderer.start_container(FrameKind::Paragraph);
                }
                Tag::BlockQuote(_) => {
                    renderer.start_container(FrameKind::BlockQuote);
                }
                Tag::List(None) => {
                    renderer.start_container(FrameKind::BulletList);
                }
                Tag::List(Some(_)) => {
                    renderer.start_container(FrameKind::OrderedList);
                }
                Tag::Item => {
                    renderer.start_container(FrameKind::ListItem);
                }
                Tag::Heading { level, .. } => {
                    let lvl = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    renderer.start_container(FrameKind::Heading { level: lvl });
                }
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) if !lang.is_empty() => {
                            Some(lang.to_string())
                        }
                        _ => None,
                    };
                    renderer.start_container(FrameKind::CodeBlock { lang });
                }
                Tag::Emphasis => {
                    renderer.start_container(FrameKind::Emph);
                }
                Tag::Strong => {
                    renderer.start_container(FrameKind::Strong);
                }
                Tag::Strikethrough => {
                    renderer.start_container(FrameKind::Strikeout);
                }
                Tag::Link { dest_url, .. } => {
                    renderer.start_container(FrameKind::Link {
                        url: dest_url.to_string(),
                    });
                }
                // Other tags (Table, Image, FootnoteDefinition, …) collapse.
                _ => {
                    renderer.start_container(FrameKind::Transparent);
                }
            },
            Event::End(tag) => match tag {
                // Only pop if we actually pushed a real frame for these tags.
                TagEnd::Paragraph
                | TagEnd::BlockQuote(_)
                | TagEnd::List(_)
                | TagEnd::Item
                | TagEnd::Heading(_)
                | TagEnd::CodeBlock
                | TagEnd::Emphasis
                | TagEnd::Strong
                | TagEnd::Strikethrough
                | TagEnd::Link => {
                    renderer.end_container();
                }
                // Transparent wrappers also need their frame popped.
                _ => {
                    renderer.end_container();
                }
            },
            Event::Text(text) => {
                if renderer.in_code_block() {
                    renderer.push_raw_text(text.to_string());
                } else {
                    renderer.push_text(text.to_string());
                }
            }
            Event::Code(code) => {
                // Inline code: render via the format's inline_code method.
                let escaped = fmt.text(&code);
                let output = fmt.inline_code(escaped);
                renderer.push_output(output);
            }
            Event::SoftBreak => {
                renderer.push_soft_break();
            }
            Event::HardBreak => {
                renderer.push_hard_break();
            }
            // Other events (Html, FootnoteReference, Rule, …) are skipped.
            _ => {}
        }
    }

    renderer.finish()
}
