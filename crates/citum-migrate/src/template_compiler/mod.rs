/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Compiles legacy CslnNode trees into Citum TemplateComponents.
//!
//! This is the final step in migration: converting the upsampled node tree
//! into the clean, declarative TemplateComponent format.

use citum_schema::{
    CslnNode, FormattingOptions, ItemType, Variable,
    template::{
        ContributorForm, ContributorRole, DateForm, DateVariable, DelimiterPunctuation,
        NumberVariable, Rendering, SimpleVariable, TemplateComponent, TemplateContributor,
        TemplateDate, TemplateList, TemplateNumber, TemplateTitle, TemplateVariable, TitleType,
    },
};
use indexmap::IndexMap;
use std::collections::HashMap;
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
    /// Type-specific branch (THEN/ELSE_IF with type conditions).
    /// Components here should be shown ONLY for these types.
    TypeSpecific(Vec<ItemType>),
    /// Default branch (ELSE or no condition).
    /// Components here should be shown for ALL types except overridden.
    Default,
}

/// Records a component's occurrence in a specific branch context.
#[derive(Debug, Clone)]
struct ComponentOccurrence {
    component: TemplateComponent,
    context: BranchContext,
    source_order: Option<usize>,
}

/// Compiles CslnNode trees into TemplateComponents.
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
    /// Compile a list of CslnNodes into TemplateComponents.
    ///
    /// Uses occurrence-based compilation to properly handle mutually exclusive
    /// conditional branches. Components are collected with their branch context,
    /// then merged with correct suppress semantics.
    pub fn compile(&self, nodes: &[CslnNode]) -> Vec<TemplateComponent> {
        let no_wrap = (None, None, None);
        let mut occurrences = Vec::new();
        self.collect_occurrences(nodes, &no_wrap, &BranchContext::Default, &mut occurrences);
        self.merge_occurrences(occurrences)
    }

    /// Compile and sort for citation output (author first, then date).
    /// Uses simplified compile that skips else branches to avoid extra fields.
    pub fn compile_citation(&self, nodes: &[CslnNode]) -> Vec<TemplateComponent> {
        let mut components = self.compile_simple(nodes);
        self.sort_citation_components(&mut components);
        components
    }
}
