/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Machine-readable evidence describing migration lineage decisions.
//!
//! Emitted as an optional sidecar (`<style>.evidence.json`) when the CLI is
//! invoked with `--emit-evidence`. Captures the registry alias status,
//! parent/template links discovered in the source CSL, the canonical target
//! that the migration routed against (if any), the form actually emitted, the
//! template-bearing paths preserved or discarded by the diff machinery, and
//! the resulting output line count.
//!
//! Downstream tooling (the SQI scorecard, future hub UX) consumes this record
//! to surface compression candidates and reason about converter decisions
//! without re-parsing the YAML.

use serde::Serialize;

/// How the legacy CSL ID relates to the embedded style registry.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RegistryAliasStatus {
    /// The legacy ID is itself a canonical embedded style.
    ExactMatch,
    /// The legacy ID is a registered alias of a canonical embedded style.
    Alias { target: String },
    /// The legacy ID is not present in the registry.
    None,
}

/// Origin of a discovered candidate parent for wrapper routing.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParentDiscoverySource {
    /// Parent declared by `<info><link rel="independent-parent">` in the
    /// source CSL.
    IndependentParentLink,
    /// Parent declared by `<info><link rel="template">` in the source CSL.
    TemplateLink,
    /// Parent inferred from a reverse `<info><link rel="template">` in an
    /// embedded canonical style pointing at the legacy ID. The embedded
    /// style declares the legacy ID as the historical template source, so
    /// the embedded style is a candidate ancestor for output-driven
    /// compression.
    ReverseTemplateLink,
    /// Registry alias resolution selected this parent.
    RegistryAlias,
    /// A pre-existing local `extends:` declaration in the checked-in style.
    LocalExtends,
    /// Caller-forced parent selection via `--family-candidate <id>`. Distinct
    /// from `ReverseTemplateLink` because the override may target a parent
    /// the resolver did not auto-discover.
    ExplicitOverride,
}

/// A discovered candidate parent that was not (yet) acted on.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FamilyCandidate {
    /// Canonical id of the candidate parent style.
    pub canonical_id: String,
    /// How the candidate was discovered.
    pub source: ParentDiscoverySource,
}

/// The artifact form the migration actually emitted.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EmittedForm {
    /// A single standalone style with no inherited parent.
    Standalone,
    /// A wrapper over an established parent style.
    ExistingWrapper {
        /// Parent style ID actually emitted in `extends:`.
        parent_style_id: String,
        /// Whether the wrapper retained template-bearing diffs.
        preserve_template_deltas: bool,
        /// Whether the wrapper was minimized to info + extends only
        /// (evidence-driven minimal form, no template diffs).
        #[serde(default)]
        minimized: bool,
    },
}

/// Source of the minimization decision for this invocation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MinimizationDecisionSource {
    /// The caller explicitly disabled family-candidate routing.
    ExplicitOff,
    /// The caller explicitly requested family-candidate routing.
    ExplicitFlags,
    /// No checked or explicit decision was available.
    None,
}

/// Outcome of the minimization decision for this invocation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MinimizationDecisionOutcome {
    /// The decision source selected a candidate parent for family routing.
    ///
    /// Check `emitted_form.minimized` to distinguish a minimized wrapper from
    /// an explicitly promoted wrapper that still preserves migrated deltas.
    Accepted,
    /// The candidate was rejected and standalone output was preserved.
    Rejected,
    /// The decision source did not select any candidate.
    NotSelected,
}

/// Audit trail for explicit wrapper minimization routing.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MinimizationDecisionAudit {
    /// Source that controlled minimization routing.
    pub source: MinimizationDecisionSource,
    /// Decision outcome.
    pub outcome: MinimizationDecisionOutcome,
    /// Parent style id selected or rejected by the decision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_style_id: Option<String>,
    /// Human-readable decision reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl MinimizationDecisionAudit {
    /// Build an audit entry for an invocation with no applicable decision.
    #[must_use]
    pub fn none() -> Self {
        Self {
            source: MinimizationDecisionSource::None,
            outcome: MinimizationDecisionOutcome::NotSelected,
            parent_style_id: None,
            reason: Some("no minimization decision selected this style".to_string()),
        }
    }
}

/// Full evidence record for a single migration invocation.
#[derive(Debug, Clone, Serialize)]
pub struct MigrationEvidence {
    /// Legacy CSL ID being migrated (derived from filename).
    pub style_id: String,
    /// How the legacy ID relates to the embedded registry.
    pub registry_alias_status: RegistryAliasStatus,
    /// Parent candidates discovered during lineage resolution, ordered by
    /// preference. The first entry is what `output_plan` consulted.
    pub discovered_parents: Vec<FamilyCandidate>,
    /// The form the migration actually emitted.
    pub emitted_form: EmittedForm,
    /// Audit trail for explicit minimization decisions.
    pub minimization_decision: MinimizationDecisionAudit,
    /// Template-bearing paths whose diffs were preserved in the wrapper.
    /// Empty for standalone output or when `preserve_template_deltas=false`.
    pub preserved_template_paths: Vec<String>,
    /// Template-bearing paths whose diffs were discarded by the wrapper.
    /// Populated for `ExistingWrapper { preserve_template_deltas: false }`.
    pub discarded_template_paths: Vec<String>,
    /// Output size of the standalone form, in lines. Reference point for
    /// downstream compression analysis.
    pub standalone_output_lines: usize,
    /// Output size of the emitted form, in lines.
    pub emitted_output_lines: usize,
}

impl MigrationEvidence {
    /// Whether the emitted form is smaller than the standalone reference.
    #[must_use]
    pub fn compressed(&self) -> bool {
        self.emitted_output_lines < self.standalone_output_lines
    }

    /// Percentage reduction in output lines, rounded to two decimals.
    /// Returns `0.0` when the standalone reference is zero. Uses `u64` for
    /// the lossless `usize -> f64` cast path so large line counts do not
    /// silently clamp.
    #[must_use]
    pub fn reduction_pct(&self) -> f64 {
        if self.standalone_output_lines == 0 {
            return 0.0;
        }
        // `usize` is at most 64 bits on supported targets, so `u64::try_from`
        // is infallible in practice. The cast to f64 is lossy for counts above
        // 2^53 but that's astronomically larger than any plausible YAML.
        let standalone = u64::try_from(self.standalone_output_lines).unwrap_or(u64::MAX) as f64;
        let emitted = u64::try_from(self.emitted_output_lines).unwrap_or(u64::MAX) as f64;
        let pct = ((standalone - emitted) / standalone) * 100.0;
        (pct * 100.0).round() / 100.0
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Panicking is acceptable in tests."
)]
mod tests {
    use super::*;

    fn fixture() -> MigrationEvidence {
        MigrationEvidence {
            style_id: "apa-6th-edition".to_string(),
            registry_alias_status: RegistryAliasStatus::None,
            discovered_parents: vec![FamilyCandidate {
                canonical_id: "apa-7th".to_string(),
                source: ParentDiscoverySource::ReverseTemplateLink,
            }],
            emitted_form: EmittedForm::Standalone,
            minimization_decision: MinimizationDecisionAudit::none(),
            preserved_template_paths: Vec::new(),
            discarded_template_paths: Vec::new(),
            standalone_output_lines: 5662,
            emitted_output_lines: 5662,
        }
    }

    #[test]
    fn standalone_form_is_not_compressed_against_itself() {
        let evidence = fixture();
        assert!(!evidence.compressed());
        assert!((evidence.reduction_pct() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn wrapper_form_reports_reduction() {
        let mut evidence = fixture();
        evidence.emitted_form = EmittedForm::ExistingWrapper {
            parent_style_id: "apa-7th".to_string(),
            preserve_template_deltas: true,
            minimized: false,
        };
        evidence.emitted_output_lines = 1200;
        assert!(evidence.compressed());
        assert!((evidence.reduction_pct() - 78.81).abs() < 0.05);
    }

    #[test]
    fn reverse_template_link_serializes_as_kebab_case() {
        let evidence = fixture();
        let json = serde_json::to_string(&evidence).expect("evidence should serialize");
        assert!(
            json.contains("\"source\":\"reverse-template-link\""),
            "expected kebab-case source tag, got: {json}"
        );
        assert!(
            json.contains("\"registry-alias-status\":\"none\"")
                || json.contains("\"registry_alias_status\":\"none\""),
        );
    }
}
