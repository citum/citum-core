/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! citum-migrate

#![allow(missing_docs, reason = "lib/crate")]

use csl_legacy::model::{Choose, ChooseBranch, CslNode, Group, Names, Style, Substitute};
use std::collections::HashMap;

/// CSL 1.0 analysis utilities.
pub mod analysis;
/// Base detector for style classification.
pub mod base_detector;
/// Unified compilation pipeline.
pub mod compilation;
/// YAML/template compressor for reducing style size.
pub mod compressor;
/// Debug output formatting.
pub mod debug_output;
/// Crate-level error type for the measured-selection pipeline.
pub mod error;
/// Machine-readable evidence describing migration lineage decisions.
pub mod evidence;
/// Post-migration template fixup helpers.
pub mod fixups;
/// Style metadata extraction.
pub mod info_extractor;
/// Intermediate representation produced from CSL 1.0 XML, consumed by the
/// template compiler when emitting modern [`citum_schema`] types.
pub mod ir;
mod js_runtime;
/// Migration-time lineage and wrapper classification.
pub mod lineage;
/// Measured inferred-vs-XML citation template selection.
pub mod measured_citation;
/// Options extraction from CSL 1.0.
pub mod options_extractor;
/// Multi-pass processing pipeline.
pub mod passes;
/// Provenance tracking for style migration.
pub mod provenance;
/// Output-driven template synthesis loop.
pub mod synthesis;
/// CSL 1.0 to Citum template compilation.
pub mod template_compiler;
pub(crate) mod template_diff;
/// Template resolution and preprocessing.
pub mod template_resolver;
/// Upsamples flattened legacy CSL 1.0 nodes into migration IR ([`ir::Node`])
/// and citation-position variants ([`upsampler::CitationPositionTemplates`]).
pub mod upsampler;

pub use base_detector::{detect_contributor_preset, detect_date_preset, detect_title_preset};
pub use compressor::Compressor;
pub use debug_output::DebugOutputFormatter;
pub use info_extractor::InfoExtractor;
pub use options_extractor::OptionsExtractor;
pub use provenance::{ProvenanceTracker, SourceLocation};
pub use template_compiler::TemplateCompiler;
pub use upsampler::Upsampler;

/// Recursively expands CSL 1.0 macro references into their definitions.
///
/// Handles nested macros and preserves rendering order across macro boundaries.
pub struct MacroInliner<'a> {
    macros: HashMap<&'a str, &'a [CslNode]>,
    provenance: Option<ProvenanceTracker>,
}

impl<'a> MacroInliner<'a> {
    /// Create a new macro inliner from a CSL 1.0 style.
    #[must_use]
    pub fn new(style: &'a Style) -> Self {
        let mut macros = HashMap::new();
        for m in &style.macros {
            macros.insert(m.name.as_str(), m.children.as_slice());
        }
        Self {
            macros,
            provenance: None,
        }
    }

    /// Create a new macro inliner with provenance tracking.
    ///
    /// Tracks source locations to help debug migration issues.
    #[must_use]
    pub fn with_provenance(style: &'a Style, provenance: ProvenanceTracker) -> Self {
        let mut macros = HashMap::new();
        for m in &style.macros {
            macros.insert(m.name.as_str(), m.children.as_slice());
        }
        Self {
            macros,
            provenance: Some(provenance),
        }
    }

    /// Return the optional provenance tracker.
    #[must_use]
    pub fn provenance(&self) -> Option<&ProvenanceTracker> {
        self.provenance.as_ref()
    }

    /// Expand all macro calls in a node list.
    ///
    /// Recursively replaces macro references with their definitions.
    #[must_use]
    pub fn expand_nodes(&self, nodes: &[CslNode]) -> Vec<CslNode> {
        let mut order_counter = 0;
        self.expand_nodes_with_order(nodes, &mut order_counter)
    }

    /// Expand macros starting from a specific order counter.
    ///
    /// Used when layout macros have pre-assigned orders and nested macros
    /// should continue numbering from where layout left off.
    fn expand_nodes_from_order(&self, nodes: &[CslNode], initial_order: usize) -> Vec<CslNode> {
        let mut order_counter = initial_order;
        self.expand_nodes_with_order(nodes, &mut order_counter)
    }

    /// Expand macros without incrementing order counter.
    ///
    /// Nested macros inherit their parent's order.
    fn expand_macros_no_increment(&self, nodes: &[CslNode]) -> Vec<CslNode> {
        let mut expanded = Vec::with_capacity(nodes.len());
        for node in nodes {
            match node {
                CslNode::Text(text) if text.macro_name.is_some() => {
                    if let Some(name) = &text.macro_name {
                        if let Some(macro_children) = self.macros.get(name.as_str()) {
                            // Recursively expand nested macros without incrementing
                            let expanded_children = self.expand_macros_no_increment(macro_children);
                            expanded.extend(expanded_children);
                        } else {
                            expanded.push(node.clone());
                        }
                    }
                }
                CslNode::Group(group) => {
                    let children = self.expand_macros_no_increment(&group.children);
                    expanded.push(CslNode::Group(clone_group_with_children(group, children)));
                }
                CslNode::Names(names) => {
                    let children = self.expand_macros_no_increment(&names.children);
                    expanded.push(CslNode::Names(clone_names_with_children(names, children)));
                }
                CslNode::Choose(choose) => {
                    expanded.push(CslNode::Choose(self.expand_choose_no_increment(choose)));
                }
                CslNode::Substitute(sub) => {
                    let children = self.expand_macros_no_increment(&sub.children);
                    expanded.push(CslNode::Substitute(Substitute { children }));
                }
                _ => {
                    expanded.push(node.clone());
                }
            }
        }
        expanded
    }

    /// Internal method that tracks macro call order during expansion.
    /// The `order_counter` is incremented each time a TOP-LEVEL macro is expanded,
    /// and all nodes within that macro inherit the same order value.
    fn expand_nodes_with_order(
        &self,
        nodes: &[CslNode],
        order_counter: &mut usize,
    ) -> Vec<CslNode> {
        let mut expanded = Vec::with_capacity(nodes.len());
        for node in nodes {
            match node {
                CslNode::Text(text) if text.macro_name.is_some() => {
                    if let Some(name) = &text.macro_name {
                        if let Some(macro_children) = self.macros.get(name.as_str()) {
                            // Check if this macro call has a pre-assigned order from the layout
                            let is_layout_macro = text.macro_call_order.is_some();

                            let current_order = if let Some(order) = text.macro_call_order {
                                // This is a layout-level macro call - use its pre-assigned order
                                order
                            } else {
                                // This is a nested macro - assign it the current counter value
                                let order = *order_counter;
                                *order_counter += 1;
                                order
                            };

                            // If this is a layout-level macro, expand its children WITH order tracking
                            // so that nested macro calls get their own order numbers, then fill the
                            // macro's own order into direct children so they sort at the call site.
                            // Otherwise, expand without tracking to inherit the parent's order.
                            let expanded_children = if is_layout_macro {
                                let mut children =
                                    self.expand_nodes_with_order(macro_children, order_counter);
                                for child in &mut children {
                                    Self::assign_macro_order_if_none(child, current_order);
                                }
                                children
                            } else {
                                let children = self.expand_macros_no_increment(macro_children);
                                // For non-layout macros, assign the parent's order only to nodes that don't already have one
                                // This preserves orders from nested macros while filling in orders for non-macro nodes
                                children
                                    .into_iter()
                                    .map(|mut child| {
                                        Self::assign_macro_order_if_none(&mut child, current_order);
                                        child
                                    })
                                    .collect()
                            };

                            // For layout macros, children already have their orders from nested expansion
                            // For non-layout macros, children have been assigned the parent's order above
                            expanded.extend(expanded_children);
                        } else {
                            // If macro not found, keep the original node (might be an error in the style)
                            expanded.push(node.clone());
                        }
                    }
                }
                // For other nodes that have children, we must recurse into them
                CslNode::Group(group) => {
                    let children = self.expand_nodes_with_order(&group.children, order_counter);
                    expanded.push(CslNode::Group(clone_group_with_children(group, children)));
                }
                CslNode::Names(names) => {
                    let children = self.expand_nodes_with_order(&names.children, order_counter);
                    expanded.push(CslNode::Names(clone_names_with_children(names, children)));
                }
                CslNode::Choose(choose) => {
                    expanded.push(CslNode::Choose(
                        self.expand_choose_with_order(choose, order_counter),
                    ));
                }
                CslNode::Substitute(sub) => {
                    let children = self.expand_nodes_with_order(&sub.children, order_counter);
                    expanded.push(CslNode::Substitute(Substitute { children }));
                }
                // Nodes with no children or that don't call macros directly
                _ => expanded.push(node.clone()),
            }
        }
        expanded
    }

    fn expand_choose_no_increment(&self, choose: &Choose) -> Choose {
        let if_children = self.expand_macros_no_increment(&choose.if_branch.children);
        let else_if_branches = choose
            .else_if_branches
            .iter()
            .map(|branch| {
                let children = self.expand_macros_no_increment(&branch.children);
                clone_choose_branch_with_children(branch, children)
            })
            .collect();
        let else_branch = choose
            .else_branch
            .as_ref()
            .map(|children| self.expand_macros_no_increment(children));

        Choose {
            if_branch: clone_choose_branch_with_children(&choose.if_branch, if_children),
            else_if_branches,
            else_branch,
        }
    }

    fn expand_choose_with_order(&self, choose: &Choose, order_counter: &mut usize) -> Choose {
        // For macro order tracking, we want sequential orders across ALL branches.
        // This tracks the SOURCE order of macro calls, not runtime execution order.
        // At runtime only one branch executes, but we need to track where each
        // macro appeared in the CSL 1.0 source.
        let if_children = self.expand_nodes_with_order(&choose.if_branch.children, order_counter);
        let else_if_branches = choose
            .else_if_branches
            .iter()
            .map(|branch| {
                let children = self.expand_nodes_with_order(&branch.children, order_counter);
                clone_choose_branch_with_children(branch, children)
            })
            .collect();
        let else_branch = choose
            .else_branch
            .as_ref()
            .map(|children| self.expand_nodes_with_order(children, order_counter));

        Choose {
            if_branch: clone_choose_branch_with_children(&choose.if_branch, if_children),
            else_if_branches,
            else_branch,
        }
    }

    /// Assigns `macro_call_order` only if not already set.
    /// This ensures nested macro nodes keep their own order while non-macro nodes get the parent order.
    fn assign_macro_order_if_none(node: &mut CslNode, order: usize) {
        match node {
            CslNode::Text(text) if text.macro_call_order.is_none() => {
                text.macro_call_order = Some(order);
            }
            CslNode::Date(date) if date.macro_call_order.is_none() => {
                date.macro_call_order = Some(order);
            }
            CslNode::Label(label) if label.macro_call_order.is_none() => {
                label.macro_call_order = Some(order);
            }
            CslNode::Names(names) => {
                if names.macro_call_order.is_none() {
                    names.macro_call_order = Some(order);
                }
                // Recursively assign to children
                for child in &mut names.children {
                    Self::assign_macro_order_if_none(child, order);
                }
            }
            CslNode::Group(group) => {
                if group.macro_call_order.is_none() {
                    group.macro_call_order = Some(order);
                }
                // Recursively assign to children
                for child in &mut group.children {
                    Self::assign_macro_order_if_none(child, order);
                }
            }
            CslNode::Number(number) if number.macro_call_order.is_none() => {
                number.macro_call_order = Some(order);
            }
            CslNode::Choose(choose) => {
                // Recursively assign to all branches
                for child in &mut choose.if_branch.children {
                    Self::assign_macro_order_if_none(child, order);
                }
                for branch in &mut choose.else_if_branches {
                    for child in &mut branch.children {
                        Self::assign_macro_order_if_none(child, order);
                    }
                }
                if let Some(ref mut else_children) = choose.else_branch {
                    for child in else_children {
                        Self::assign_macro_order_if_none(child, order);
                    }
                }
            }
            CslNode::Substitute(sub) => {
                // Recursively assign to children
                for child in &mut sub.children {
                    Self::assign_macro_order_if_none(child, order);
                }
            }
            _ => {}
        }
    }

    /// Assigns `macro_call_order` to a node and all its descendants.
    /// This ensures all nodes within an expanded macro inherit the macro's order.
    #[allow(
        dead_code,
        reason = "reference implementation for assign_macro_order_if_none; kept for potential future use"
    )]
    fn assign_macro_order(node: &mut CslNode, order: usize) {
        match node {
            CslNode::Text(text) => {
                text.macro_call_order = Some(order);
            }
            CslNode::Date(date) => {
                date.macro_call_order = Some(order);
            }
            CslNode::Label(label) => {
                label.macro_call_order = Some(order);
            }
            CslNode::Names(names) => {
                names.macro_call_order = Some(order);
                // Recursively assign to children
                for child in &mut names.children {
                    Self::assign_macro_order(child, order);
                }
            }
            CslNode::Group(group) => {
                group.macro_call_order = Some(order);
                // Recursively assign to children
                for child in &mut group.children {
                    Self::assign_macro_order(child, order);
                }
            }
            CslNode::Number(number) => {
                number.macro_call_order = Some(order);
            }
            CslNode::Choose(choose) => {
                // Recursively assign to all branches
                for child in &mut choose.if_branch.children {
                    Self::assign_macro_order(child, order);
                }
                for branch in &mut choose.else_if_branches {
                    for child in &mut branch.children {
                        Self::assign_macro_order(child, order);
                    }
                }
                if let Some(else_children) = &mut choose.else_branch {
                    for child in else_children {
                        Self::assign_macro_order(child, order);
                    }
                }
            }
            CslNode::Substitute(sub) => {
                // Recursively assign to children
                for child in &mut sub.children {
                    Self::assign_macro_order(child, order);
                }
            }
            _ => {}
        }
    }

    /// Assign layout order to every renderable node before expansion.
    /// Macro calls and direct nodes (`<text variable>`, `<date>`, `<names>`,
    /// `<number>`, `<label>`) all receive sequential orders based on their
    /// position in the layout, which is preserved during macro expansion.
    /// Without orders on direct nodes, they sort after every macro-derived
    /// component and the compiled template loses the authored layout order.
    fn assign_layout_order(nodes: &mut [CslNode], order_counter: &mut usize) {
        for node in nodes {
            match node {
                CslNode::Text(text) => {
                    // Macro call or direct text node - assign it the current order
                    text.macro_call_order = Some(*order_counter);
                    *order_counter += 1;
                }
                CslNode::Date(date) => {
                    date.macro_call_order = Some(*order_counter);
                    *order_counter += 1;
                }
                CslNode::Number(number) => {
                    number.macro_call_order = Some(*order_counter);
                    *order_counter += 1;
                }
                CslNode::Label(label) => {
                    label.macro_call_order = Some(*order_counter);
                    *order_counter += 1;
                }
                CslNode::Group(group) => {
                    // Order the group itself, then recurse so children keep
                    // their relative positions if the group is flattened.
                    group.macro_call_order = Some(*order_counter);
                    *order_counter += 1;
                    Self::assign_layout_order(&mut group.children, order_counter);
                }
                CslNode::Choose(choose) => {
                    // For layout order assignment, we want ALL macro calls to get unique sequential orders
                    // even across Choose branches, because we're tracking SOURCE order, not runtime order.
                    // At runtime only one branch executes, but we need to track where each macro appeared
                    // in the CSL 1.0 source.
                    Self::assign_layout_order(&mut choose.if_branch.children, order_counter);

                    for branch in &mut choose.else_if_branches {
                        Self::assign_layout_order(&mut branch.children, order_counter);
                    }

                    if let Some(else_children) = &mut choose.else_branch {
                        Self::assign_layout_order(else_children, order_counter);
                    }
                }
                CslNode::Names(names) => {
                    names.macro_call_order = Some(*order_counter);
                    *order_counter += 1;
                    Self::assign_layout_order(&mut names.children, order_counter);
                }
                CslNode::Substitute(sub) => {
                    Self::assign_layout_order(&mut sub.children, order_counter);
                }
                _ => {}
            }
        }
    }

    /// Returns a version of the bibliography layout with all macros inlined.
    #[must_use]
    pub fn inline_bibliography(&self, style: &Style) -> Option<Vec<CslNode>> {
        style.bibliography.as_ref().map(|bib| {
            // Clone the layout children so we can mutate them
            let mut layout_nodes = bib.layout.children.clone();

            // Assign order to layout macro calls before expansion
            let mut order_counter = 0;
            Self::assign_layout_order(&mut layout_nodes, &mut order_counter);

            // Expand macros, starting nested macro numbering from where layout assignment left off.
            // This prevents collisions between layout macro orders and nested macro orders.
            self.expand_nodes_from_order(&layout_nodes, order_counter)
        })
    }

    /// Returns a version of the citation layout with all macros inlined.
    #[must_use]
    pub fn inline_citation(&self, style: &Style) -> Vec<CslNode> {
        self.expand_nodes(&style.citation.layout.children)
    }
}

/// Clones a group component with its children, used during macro inlining to avoid mutating the original.
fn clone_group_with_children(group: &Group, children: Vec<CslNode>) -> Group {
    Group {
        delimiter: group.delimiter.clone(),
        prefix: group.prefix.clone(),
        suffix: group.suffix.clone(),
        children,
        macro_call_order: group.macro_call_order,
        formatting: group.formatting.clone(),
    }
}

/// Clones a names component with its children for inline substitution.
fn clone_names_with_children(names: &Names, children: Vec<CslNode>) -> Names {
    Names {
        variable: names.variable.clone(),
        delimiter: names.delimiter.clone(),
        delimiter_precedes_et_al: names.delimiter_precedes_et_al.clone(),
        et_al_min: names.et_al_min,
        et_al_use_first: names.et_al_use_first,
        et_al_subsequent_min: names.et_al_subsequent_min,
        et_al_subsequent_use_first: names.et_al_subsequent_use_first,
        prefix: names.prefix.clone(),
        suffix: names.suffix.clone(),
        children,
        macro_call_order: names.macro_call_order,
        formatting: names.formatting.clone(),
    }
}

/// Clones a choose branch with its child components for inline substitution.
fn clone_choose_branch_with_children(
    branch: &ChooseBranch,
    children: Vec<CslNode>,
) -> ChooseBranch {
    ChooseBranch {
        match_mode: branch.match_mode.clone(),
        type_: branch.type_.clone(),
        variable: branch.variable.clone(),
        is_numeric: branch.is_numeric.clone(),
        is_uncertain_date: branch.is_uncertain_date.clone(),
        locator: branch.locator.clone(),
        position: branch.position.clone(),
        children,
    }
}
