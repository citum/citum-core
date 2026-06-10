/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Compiles legacy Node trees into Citum TemplateComponents.
//!
//! This is the final step in migration: converting the upsampled node tree
//! into the clean, declarative TemplateComponent format.

use crate::ir::{FormattingOptions, ItemType, Node, Variable};
use citum_schema::template::{
    ContributorForm, ContributorRole, DateForm, DateVariable, DelimiterPunctuation, NumberVariable,
    Rendering, SimpleVariable, TemplateComponent, TemplateContributor, TemplateDate, TemplateGroup,
    TemplateNumber, TemplateTitle, TemplateVariable, TitleType,
};
use indexmap::IndexMap;
use std::sync::OnceLock;

mod bibliography;
mod compilation;
mod deduplication;
mod formatting;
mod node_compiler;
mod sorting;
mod types;

/// Context for a conditional branch, distinguishing between type-specific
/// and default branches. This is critical for correct suppress semantics.
#[derive(Debug, Clone)]
enum BranchContext {
    /// Conditional branch (e.g. `THEN/ELSE_IF` targeting specific types).
    /// Components here are only active when the branch condition is met.
    Conditional,
    /// Default branch (ELSE or no condition).
    /// Components here should be shown for ALL types.
    Default,
}

/// Records a component's occurrence in a specific branch context.
#[derive(Debug, Clone)]
struct ComponentOccurrence {
    component: TemplateComponent,
    context: BranchContext,
    source_order: Option<usize>,
}

/// Compiles `Node` trees into `TemplateComponents`.
pub struct TemplateCompiler;

fn migrate_debug_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("CITUM_MIGRATE_DEBUG")
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    })
}

impl TemplateCompiler {
    /// Compile a list of ``ir::Node`s` into `TemplateComponents`.
    ///
    /// Uses occurrence-based compilation to properly handle mutually exclusive
    /// conditional branches. Components are collected with their branch context,
    /// then merged with correct suppress semantics.
    #[must_use]
    pub fn compile(&self, nodes: &[Node]) -> Vec<TemplateComponent> {
        let no_wrap = (None, None, None);
        let mut occurrences = Vec::new();
        self.collect_occurrences(nodes, &no_wrap, &BranchContext::Default, &mut occurrences);
        self.merge_occurrences(occurrences)
    }

    /// Compile and sort for citation output (author first, then date).
    /// Uses simplified compile that skips else branches to avoid extra fields.
    #[must_use]
    pub fn compile_citation(&self, nodes: &[Node]) -> Vec<TemplateComponent> {
        let mut components = self.compile_simple(nodes);
        self.sort_citation_components(&mut components);
        components
    }

    /// Compile a note-class citation template.
    ///
    /// Note citations are full bibliographic entries, not author-date or
    /// numeric markers: the simplified citation path (skip else branches,
    /// author-first sort) discards the type-conditional structure and the
    /// affix order the note layout depends on. Route through the same
    /// occurrence-based compilation the bibliography uses and preserve the
    /// authored component order.
    #[must_use]
    pub fn compile_citation_note(&self, nodes: &[Node]) -> Vec<TemplateComponent> {
        let mut components = self.compile(nodes);
        crate::passes::deduplicate::deduplicate_numbers_in_lists(&mut components);
        crate::passes::deduplicate::deduplicate_dates_in_lists(&mut components);
        crate::passes::deduplicate::remove_redundant_no_date_terms(&mut components);
        self.fix_duplicate_variables(&mut components);
        crate::passes::suppression::strip_suppressed_variable_poison(&mut components);
        components
    }
}
