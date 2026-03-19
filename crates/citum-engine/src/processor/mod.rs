/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! The Citum processor for rendering citations and bibliographies.
//!
//! ## Architecture
//!
//! `Processor` is intentionally a thin facade over a small set of focused
//! implementation modules:
//! - `setup`: construction, configuration resolution, and numbering setup
//! - `note_context`: note-number normalization and citation position inference
//! - `citation`: citation rendering orchestration
//! - `bibliography`: bibliography rendering, grouping, and document-facing helpers
//!
//! The processor remains intentionally "dumb": it applies the style as written
//! without implicit logic. Style-specific behavior (for example, suppressing a
//! publisher for journals) should be expressed in the style YAML via
//! `overrides`, not hardcoded here.
//!
//! ## CSL 1.0 Compatibility
//!
//! The processor implements the CSL 1.0 "variable-once" rule:
//! > "Substituted variables are suppressed in the rest of the output to
//! > prevent duplication."
//!
//! This is tracked via `rendered_vars` in `process_template()`.

mod bibliography;
mod citation;
mod note_context;
mod setup;

/// Author/date disambiguation and year-suffix assignment.
pub mod disambiguation;
pub mod document;
pub mod labels;
/// Matching helpers for substitution and repeated-contributor detection.
pub mod matching;
/// Template rendering orchestration and per-component state handling.
pub mod rendering;
/// Citation and bibliography sorting helpers.
pub mod sorting;

#[cfg(test)]
mod tests;

use crate::reference::Bibliography;
use crate::render::ProcEntry;
use crate::values::ProcHints;
use citum_schema::Style;
use citum_schema::locale::Locale;
use citum_schema::options::Config;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// The Citum processor facade.
///
/// Takes a style, bibliography, and locale context, then delegates citation
/// and bibliography work to the processor submodules.
#[derive(Debug)]
pub struct Processor {
    /// The style definition.
    pub style: Style,
    /// The bibliography (references keyed by ID).
    pub bibliography: Bibliography,
    /// The locale for terms and formatting.
    pub locale: Locale,
    /// Default configuration.
    pub default_config: Config,
    /// Pre-calculated processing hints.
    pub hints: HashMap<String, ProcHints>,
    /// Citation numbers assigned to references (for numeric styles).
    pub citation_numbers: RefCell<HashMap<String, usize>>,
    /// IDs of items that were cited in a visible way.
    pub cited_ids: RefCell<HashSet<String>>,
    /// Compound sets keyed by set ID.
    pub compound_sets: IndexMap<String, Vec<String>>,
    /// Reverse lookup for set membership by reference ID.
    pub compound_set_by_ref: HashMap<String, String>,
    /// Position within a set (0-based) for each reference ID.
    pub compound_member_index: HashMap<String, usize>,
    /// Compound numeric groups: citation number → ordered ref IDs in the group.
    pub compound_groups: RefCell<IndexMap<usize, Vec<String>>>,
    /// Whether to output semantic markup (HTML spans, Djot attributes).
    /// Defaults to true; set to false to suppress class attributes (e.g. `--no-semantics`).
    pub show_semantics: bool,
    /// Whether to annotate semantic HTML wrappers with source template indices.
    pub inject_ast_indices: bool,
}

/// Processed output containing citations and bibliography.
#[derive(Debug, Default)]
pub struct ProcessedReferences {
    /// Rendered bibliography entries with metadata.
    pub bibliography: Vec<ProcEntry>,
    /// Rendered citations as formatted strings.
    ///
    /// None if no citations were processed; Some(vec) otherwise.
    pub citations: Option<Vec<String>>,
}
