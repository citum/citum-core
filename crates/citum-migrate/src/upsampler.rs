/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use citum_schema as citum;
use csl_legacy::model::CslNode as LNode;
use std::sync::OnceLock;

mod mapping;
mod position;

#[cfg(test)]
mod tests;

pub use position::CitationPositionTemplates;

/// Citum-facing alias for schema nodes emitted by the upsampler.
pub type CitumNode = citum::CslnNode;

/// Returns true when verbose migration debug logging is enabled.
pub(super) fn migrate_debug_enabled() -> bool {
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

#[derive(Default)]
/// Convert flattened legacy CSL nodes into Citum nodes and citation position variants.
pub struct Upsampler {
    provenance: Option<crate::ProvenanceTracker>,
    pub et_al_min: Option<usize>,
    pub et_al_use_first: Option<usize>,
}

impl Upsampler {
    /// Create an upsampler without provenance tracking.
    #[must_use]
    pub fn new() -> Self {
        Self {
            provenance: None,
            et_al_min: None,
            et_al_use_first: None,
        }
    }

    /// Create an upsampler that records provenance during XML migration.
    #[must_use]
    pub fn with_provenance(provenance: crate::ProvenanceTracker) -> Self {
        Self {
            provenance: Some(provenance),
            et_al_min: None,
            et_al_use_first: None,
        }
    }

    /// The entry point for converting a flattened legacy tree into Citum nodes.
    #[must_use]
    pub fn upsample_nodes(&self, legacy_nodes: &[LNode]) -> Vec<CitumNode> {
        let mut citum_nodes = Vec::new();
        let mut i = 0;

        while i < legacy_nodes.len() {
            #[allow(clippy::indexing_slicing, reason = "i < legacy_nodes.len()")]
            let node = &legacy_nodes[i];

            if let LNode::Group(group) = node
                && let Some(collapsed) = self.try_collapse_label_variable(group)
            {
                citum_nodes.push(collapsed);
                i += 1;
                continue;
            }

            if let Some(mapped) = self.map_node(node) {
                citum_nodes.push(mapped);
            }

            i += 1;
        }

        citum_nodes
    }
}
