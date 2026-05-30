/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Block-aware markup renderer for terminal formats (Typst, LaTeX).
//!
//! Provides a format-neutral event model plus source adapters
//! for Djot (via `jotdown`) and Markdown (via `pulldown_cmark`), and a renderer
//! that maps events to [`OutputFormat`] methods.

mod djot;
mod markdown;
mod renderer;

pub(crate) use djot::render_djot_body;
pub(crate) use markdown::render_markdown_body;
