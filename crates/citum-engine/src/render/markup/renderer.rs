/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Block-aware markup renderer using frame-stack accumulation.
//!
//! Provides a format-neutral renderer that maps markup events from Djot or
//! Markdown parsers to `OutputFormat` method calls via a frame stack.

use crate::render::format::OutputFormat;

/// Kind of frame currently open on the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FrameKind {
    /// Document root; the final join of all top-level blocks.
    Root,
    /// A text paragraph.
    Paragraph,
    /// A block quotation.
    BlockQuote,
    /// An unordered (bullet) list; children are pre-rendered item strings.
    BulletList,
    /// An ordered (numbered) list; children are pre-rendered item strings.
    OrderedList,
    /// A single list item.
    ListItem,
    /// A heading at the given nesting level (1 = top).
    Heading { level: u8 },
    /// A fenced or indented code block with an optional language tag.
    CodeBlock { lang: Option<String> },
    /// Emphasis (typically italics).
    Emph,
    /// Strong emphasis (typically bold).
    Strong,
    /// Strikethrough text.
    Strikeout,
    /// A hyperlink to the given URL.
    Link { url: String },
    /// Inline code span.
    InlineCode,
    /// Unknown or unsupported container; collapses transparently to its text.
    Transparent,
}

/// A single frame on the rendering stack.
struct Frame {
    kind: FrameKind,
    parts: Vec<String>,
}

impl Frame {
    fn new(kind: FrameKind) -> Self {
        Self {
            kind,
            parts: Vec::new(),
        }
    }
}

/// Stack-based renderer mapping markup events to `OutputFormat` calls.
pub(crate) struct MarkupRenderer<'a, F: OutputFormat> {
    fmt: &'a F,
    stack: Vec<Frame>,
}

impl<'a, F: OutputFormat<Output = String>> MarkupRenderer<'a, F> {
    /// Create a new renderer with a root frame pre-pushed.
    pub(crate) fn new(fmt: &'a F) -> Self {
        Self {
            fmt,
            stack: vec![Frame::new(FrameKind::Root)],
        }
    }

    /// Open a new container frame.
    ///
    /// A `Root` kind pushed by an adapter (e.g. jotdown's `Document`) is
    /// silently dropped — the renderer already manages its own root frame.
    pub(crate) fn start_container(&mut self, kind: FrameKind) {
        if kind == FrameKind::Root {
            return;
        }
        self.stack.push(Frame::new(kind));
    }

    /// Close the current container, apply formatting, and push into parent.
    pub(crate) fn end_container(&mut self) {
        if self.stack.len() <= 1 {
            return;
        }
        if let Some(frame) = self.stack.pop() {
            let content = self.apply_frame(frame);
            if let Some(parent) = self.stack.last_mut() {
                parent.parts.push(content);
            }
        }
    }

    /// Produce the formatted string for a completed frame.
    fn apply_frame(&self, frame: Frame) -> String {
        match frame.kind {
            FrameKind::Root | FrameKind::Transparent => frame.parts.join(""),
            FrameKind::Paragraph => self.fmt.paragraph(frame.parts.join("")),
            FrameKind::BlockQuote => self.fmt.block_quote(frame.parts.join("")),
            FrameKind::BulletList => self.fmt.bullet_list(frame.parts),
            FrameKind::OrderedList => self.fmt.ordered_list(frame.parts),
            FrameKind::ListItem => self.fmt.list_item(frame.parts.join("")),
            FrameKind::Heading { level } => self.fmt.heading(level, frame.parts.join("")),
            FrameKind::CodeBlock { ref lang } => {
                self.fmt.code_block(lang.as_deref(), frame.parts.join(""))
            }
            FrameKind::Emph => self.fmt.emph(frame.parts.join("")),
            FrameKind::Strong => self.fmt.strong(frame.parts.join("")),
            FrameKind::Strikeout => self.fmt.strikeout(frame.parts.join("")),
            FrameKind::Link { ref url } => self.fmt.link(url, frame.parts.join("")),
            FrameKind::InlineCode => self.fmt.inline_code(frame.parts.join("")),
        }
    }

    /// Push a text leaf, routing through `fmt.text()` for escaping.
    pub(crate) fn push_text(&mut self, text: String) {
        let escaped = self.fmt.text(&text);
        if let Some(frame) = self.stack.last_mut() {
            frame.parts.push(escaped);
        }
    }

    /// Push raw (unescaped) text — used inside code blocks.
    pub(crate) fn push_raw_text(&mut self, text: String) {
        if let Some(frame) = self.stack.last_mut() {
            frame.parts.push(text);
        }
    }

    /// Push a pre-rendered output string directly (e.g. an inline code span).
    pub(crate) fn push_output(&mut self, output: String) {
        if let Some(frame) = self.stack.last_mut() {
            frame.parts.push(output);
        }
    }

    /// Push a soft break as a single space.
    pub(crate) fn push_soft_break(&mut self) {
        if let Some(frame) = self.stack.last_mut() {
            frame.parts.push(" ".to_string());
        }
    }

    /// Push a hard line break via the format's `hard_break` method.
    pub(crate) fn push_hard_break(&mut self) {
        let br = self.fmt.hard_break();
        if let Some(frame) = self.stack.last_mut() {
            frame.parts.push(br);
        }
    }

    /// Finalize: collapse any unclosed frames and return the rendered output.
    pub(crate) fn finish(mut self) -> String {
        while self.stack.len() > 1 {
            if let Some(frame) = self.stack.pop() {
                let content = self.apply_frame(frame);
                if let Some(parent) = self.stack.last_mut() {
                    parent.parts.push(content);
                }
            }
        }
        self.stack
            .pop()
            .map(|f| f.parts.join(""))
            .unwrap_or_default()
    }

    /// Return whether the current (innermost) frame is a raw context.
    ///
    /// Both `CodeBlock` and `InlineCode` hold literal content that must not
    /// flow through the format's text-escaping path.
    pub(crate) fn in_raw_context(&self) -> bool {
        self.stack
            .last()
            .is_some_and(|f| matches!(f.kind, FrameKind::CodeBlock { .. } | FrameKind::InlineCode))
    }
}
