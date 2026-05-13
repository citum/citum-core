/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Formats provenance debug output for display.

use std::fmt::Write;

use crate::provenance::{ProvenanceTracker, TransformationEvent};

pub struct DebugOutputFormatter;

impl DebugOutputFormatter {
    /// Format debug output for a specific variable
    #[must_use]
    pub fn format_variable(tracker: &ProvenanceTracker, var_name: &str) -> String {
        match tracker.get_provenance(var_name) {
            Some(provenance) => {
                let mut output = String::new();
                let _ = writeln!(output, "Variable: {var_name}");
                output.push('\n');

                // Group events by category
                let mut source_nodes = Vec::new();
                let mut transformations = Vec::new();
                let mut placements = Vec::new();

                for event in &provenance.events {
                    match event {
                        TransformationEvent::SourceElement { .. } => source_nodes.push(event),
                        TransformationEvent::TemplatePlacement { .. } => placements.push(event),
                        _ => transformations.push(event),
                    }
                }

                // Source CSL nodes
                if !source_nodes.is_empty() {
                    output.push_str("Source CSL nodes:\n");
                    for (i, event) in source_nodes.iter().enumerate() {
                        let _ = writeln!(output, "  {}. {}", i + 1, event);
                    }
                    output.push('\n');
                }

                // Transformations
                if !transformations.is_empty() {
                    output.push_str("Transformations:\n");
                    for event in transformations {
                        let _ = writeln!(output, "  - {event}");
                    }
                    output.push('\n');
                }

                // Template placement
                if !placements.is_empty() {
                    let placements_count = placements.len();
                    output.push_str("Compiled to:\n");
                    for event in placements {
                        let _ = writeln!(output, "  - {event}");
                    }
                    output.push_str("\nSummary:\n");
                    let _ = writeln!(
                        output,
                        "  Total transformations: {}",
                        provenance.events.len()
                    );
                    let _ = writeln!(output, "  Source nodes found: {}", source_nodes.len());
                    let _ = writeln!(output, "  Template placements: {placements_count}");
                }

                output
            }
            None => {
                format!("Variable '{var_name}' not found in provenance.\n\nAvailable variables:\n")
                    + &Self::format_available_variables(tracker)
            }
        }
    }

    /// Format list of available variables
    #[must_use]
    pub fn format_available_variables(tracker: &ProvenanceTracker) -> String {
        let mut vars: Vec<_> = tracker.get_all_variables();
        vars.sort();

        if vars.is_empty() {
            "  (none tracked)\n".to_string()
        } else {
            let mut result = String::new();
            for (i, v) in vars.iter().enumerate() {
                let _ = writeln!(result, "  {}. {}", i + 1, v);
            }
            result
        }
    }

    /// Format full debug report for all tracked variables
    #[must_use]
    pub fn format_all_variables(tracker: &ProvenanceTracker) -> String {
        let mut vars: Vec<_> = tracker.get_all_variables();
        vars.sort();

        if vars.is_empty() {
            return "No variables tracked.\n".to_string();
        }

        let mut output = format!("Tracked {} variables:\n\n", vars.len());

        for (i, var) in vars.iter().enumerate() {
            if i > 0 {
                output.push('\n');
                output.push_str("---\n\n");
            }
            output.push_str(&Self::format_variable(tracker, var));
        }

        output
    }
}

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
mod tests {
    use super::*;
    use crate::provenance::SourceLocation;
    use std::collections::HashMap;

    #[test]
    fn test_format_variable() {
        let tracker = ProvenanceTracker::new(true);
        let loc = SourceLocation {
            line: 42,
            column: 10,
            context: "macro 'label-volume'".to_string(),
        };

        tracker.record_source_element("volume", loc, "text", HashMap::new());

        tracker.record_upsampling("volume", "Text", "Variable");
        tracker.record_template_placement("volume", 4, "bibliography.template", "Number");

        let output = DebugOutputFormatter::format_variable(&tracker, "volume");
        assert!(output.contains("Variable: volume"));
        assert!(output.contains("Source CSL nodes"));
        assert!(output.contains("Transformations"));
        assert!(output.contains("Compiled to"));
    }

    #[test]
    fn test_format_unknown_variable() {
        let tracker = ProvenanceTracker::new(true);
        let output = DebugOutputFormatter::format_variable(&tracker, "unknown");
        assert!(output.contains("Variable 'unknown' not found"));
    }
}
