/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
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
//! This is tracked by `TemplateComponentTracker` during template rendering.
//! Suppressed components do not claim variables; see
//! `docs/specs/TEMPLATE_RENDERING_SEMANTICS.md`.

mod bibliography;
mod citation;
mod note_context;
mod run_state;
mod setup;

/// Author/date disambiguation and year-suffix assignment.
pub mod disambiguation;
pub mod document;
pub mod labels;
/// Matching helpers for substitution and repeated-contributor detection.
pub mod matching;
/// Template rendering orchestration and per-component state handling.
pub mod rendering;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests;

use crate::reference::Bibliography;
use crate::render::ProcEntry;
use crate::values::ProcHints;
use citum_schema::Style;
use citum_schema::locale::Locale;
use citum_schema::options::Config;
use indexmap::IndexMap;
use run_state::RunState;
use std::collections::HashMap;

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
    /// Compound sets keyed by set ID.
    pub compound_sets: IndexMap<String, Vec<String>>,
    /// Reverse lookup for set membership by reference ID.
    pub compound_set_by_ref: HashMap<String, String>,
    /// Position within a set (0-based) for each reference ID.
    pub compound_member_index: HashMap<String, usize>,
    /// Whether to output semantic markup (HTML spans, Djot attributes).
    /// Defaults to true; set to false to suppress class attributes (e.g. `--no-semantics`).
    pub show_semantics: bool,
    /// Whether to annotate semantic HTML wrappers with source template indices.
    pub inject_ast_indices: bool,
    /// Document-level abbreviation map for post-render substitution.
    pub abbreviation_map: Option<crate::api::AbbreviationMap>,
    /// Mutable per-render-run state (citation numbers, cite-order tracking,
    /// dynamic compound groups, first-note tracking).
    ///
    /// See `docs/specs/EXPLICIT_RENDER_RUN_STATE.md`. This is a transitional
    /// single-run-per-processor shape; a later migration step will thread
    /// `RunState` explicitly through registration/render calls instead of
    /// storing it on `Processor`.
    pub(crate) run_state: RunState,
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

/// Validate optional compound sets against the loaded bibliography.
///
/// Validation rules:
/// - Every member ID must exist in `bibliography`.
/// - A member ID must not appear more than once in a single set.
/// - A member ID must not appear across multiple sets.
///
/// # Errors
///
/// Returns an error when a compound set references an unknown ID or reuses the
/// same member within or across sets.
pub fn validate_compound_sets(
    sets: Option<IndexMap<String, Vec<String>>>,
    bibliography: &Bibliography,
) -> Result<Option<IndexMap<String, Vec<String>>>, crate::error::ProcessorError> {
    let Some(sets) = sets else {
        return Ok(None);
    };

    let mut member_owner: HashMap<String, String> = HashMap::new();
    for (set_id, members) in &sets {
        let mut seen_in_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for member in members {
            if !seen_in_set.insert(member.clone()) {
                return Err(crate::error::ProcessorError::CompoundSetValidation(
                    format!(
                        "reference '{member}' appears more than once in compound set '{set_id}'"
                    ),
                ));
            }
            if !bibliography.contains_key(member) {
                return Err(crate::error::ProcessorError::CompoundSetValidation(
                    format!("compound set '{set_id}' references unknown id '{member}'"),
                ));
            }
            if let Some(existing) = member_owner.insert(member.clone(), set_id.clone()) {
                return Err(crate::error::ProcessorError::CompoundSetValidation(
                    format!(
                        "reference '{member}' appears in both compound sets '{existing}' and '{set_id}'"
                    ),
                ));
            }
        }
    }

    Ok(Some(sets))
}
