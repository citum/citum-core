/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Lineage routing and migration evidence helpers for the CLI.

use crate::cli::{Args, FamilyCandidateMode};
use citum_migrate::{
    evidence::{
        EmittedForm, MeasuredSelectionEvidence, MigrationDiagnostic, MinimizationDecisionAudit,
        MinimizationDecisionOutcome, MinimizationDecisionSource,
    },
    lineage::{MigrationEvidenceParts, MigrationOutputPlan, StyleLineage},
};
use citum_schema::Style;
use std::{fs, path::Path};

/// Effective family-candidate routing after applying explicit flags.
pub(crate) struct FamilyCandidateRouting {
    /// Evidence audit for the family-candidate decision.
    pub(crate) audit: MinimizationDecisionAudit,
}

/// Optional measured-selection and diagnostic details for an evidence record.
pub(crate) struct EvidenceDetails {
    /// Output-driven template selection summaries.
    pub(crate) measured_selection: Option<MeasuredSelectionEvidence>,
    /// Recoverable migration warnings with stable codes.
    pub(crate) diagnostics: Vec<MigrationDiagnostic>,
}

/// Promote a discovered family-candidate parent into the lineage's active
/// routing slot when explicit flags request it.
pub(crate) fn apply_family_candidate_routing(
    lineage: &mut StyleLineage,
    workspace_root: &Path,
    mode: &FamilyCandidateMode,
    path: &str,
) -> Result<FamilyCandidateRouting, Box<dyn std::error::Error>> {
    match mode {
        FamilyCandidateMode::Default => Ok(FamilyCandidateRouting {
            audit: MinimizationDecisionAudit::none(),
        }),
        FamilyCandidateMode::Off => Ok(FamilyCandidateRouting {
            audit: MinimizationDecisionAudit {
                source: MinimizationDecisionSource::ExplicitOff,
                outcome: MinimizationDecisionOutcome::NotSelected,
                parent_style_id: None,
                reason: Some("caller disabled family-candidate routing".to_string()),
            },
        }),
        FamilyCandidateMode::Auto => {
            let promoted = lineage.promote_family_candidate(workspace_root, None)?;
            if !promoted {
                tracing::debug!(
                    "No family-candidate parent discovered for {path}; staying standalone."
                );
            }
            Ok(FamilyCandidateRouting {
                audit: MinimizationDecisionAudit {
                    source: MinimizationDecisionSource::ExplicitFlags,
                    outcome: if promoted {
                        MinimizationDecisionOutcome::Accepted
                    } else {
                        MinimizationDecisionOutcome::NotSelected
                    },
                    parent_style_id: lineage.parent_style_id.clone(),
                    reason: Some("caller requested --family-candidate auto".to_string()),
                },
            })
        }
        FamilyCandidateMode::Explicit(id) => {
            lineage.promote_family_candidate(workspace_root, Some(id))?;
            Ok(FamilyCandidateRouting {
                audit: MinimizationDecisionAudit {
                    source: MinimizationDecisionSource::ExplicitFlags,
                    outcome: MinimizationDecisionOutcome::Accepted,
                    parent_style_id: Some(id.clone()),
                    reason: Some("caller forced a family-candidate parent".to_string()),
                },
            })
        }
    }
}

/// Write the optional migration evidence sidecar requested by the CLI.
pub(crate) fn write_optional_evidence(
    cli: &Args,
    lineage: &StyleLineage,
    standalone_lines: usize,
    emitted_lines: usize,
    minimized: bool,
    minimization_decision: MinimizationDecisionAudit,
    details: EvidenceDetails,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(evidence_path) = cli.emit_evidence.as_deref() {
        write_evidence_sidecar(
            evidence_path,
            lineage,
            standalone_lines,
            emitted_lines,
            minimized,
            minimization_decision,
            details,
        )?;
    }
    Ok(())
}

/// Count the lines of a style's YAML serialization.
pub(crate) fn count_yaml_lines(style: &Style) -> Result<usize, Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(style)?;
    Ok(yaml.lines().count())
}

/// Log the selected migration output plan.
pub(crate) fn log_migration_output_plan(lineage: &StyleLineage) {
    match lineage.output_plan() {
        MigrationOutputPlan::Standalone => {
            tracing::debug!("Migration output plan: standalone");
        }
        MigrationOutputPlan::ExistingWrapper {
            parent_style_id,
            implementation_form,
            preserve_template_deltas,
        } => {
            tracing::debug!(
                "Migration output plan: existing-wrapper parent={parent_style_id} form={implementation_form:?} preserve-template-deltas={preserve_template_deltas}"
            );
        }
    }
}

fn write_evidence_sidecar(
    evidence_path: &Path,
    lineage: &StyleLineage,
    standalone_lines: usize,
    emitted_lines: usize,
    minimized: bool,
    minimization_decision: MinimizationDecisionAudit,
    details: EvidenceDetails,
) -> Result<(), Box<dyn std::error::Error>> {
    let emitted_form = describe_emitted_form(lineage, minimized);
    let (preserved, discarded) = classify_template_paths(&emitted_form);
    let evidence = lineage.build_evidence(MigrationEvidenceParts {
        standalone_lines,
        emitted_form,
        emitted_lines,
        minimization_decision,
        preserved_template_paths: preserved,
        discarded_template_paths: discarded,
        measured_selection: details.measured_selection,
        diagnostics: details.diagnostics,
    });
    let json = serde_json::to_string_pretty(&evidence)?;
    fs::write(evidence_path, json)?;
    Ok(())
}

const TEMPLATE_BEARING_PATH_LABELS: &[&str] = &[
    "templates",
    "citation.template",
    "citation.type-variants",
    "citation.integral.template",
    "citation.integral.type-variants",
    "citation.non-integral.template",
    "citation.non-integral.type-variants",
    "bibliography.template",
    "bibliography.type-variants",
];

fn classify_template_paths(emitted: &EmittedForm) -> (Vec<String>, Vec<String>) {
    let labels: Vec<String> = TEMPLATE_BEARING_PATH_LABELS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    match emitted {
        EmittedForm::Standalone => (Vec::new(), Vec::new()),
        EmittedForm::ExistingWrapper {
            preserve_template_deltas,
            minimized,
            ..
        } => {
            if *minimized || !*preserve_template_deltas {
                (Vec::new(), labels)
            } else {
                (labels, Vec::new())
            }
        }
    }
}

fn describe_emitted_form(lineage: &StyleLineage, minimized: bool) -> EmittedForm {
    match lineage.output_plan() {
        MigrationOutputPlan::Standalone => EmittedForm::Standalone,
        MigrationOutputPlan::ExistingWrapper {
            parent_style_id,
            preserve_template_deltas,
            ..
        } => EmittedForm::ExistingWrapper {
            parent_style_id,
            preserve_template_deltas,
            minimized,
        },
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "Panicking is acceptable in tests.")]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate should live under crates/citum-migrate")
            .to_path_buf()
    }

    #[test]
    fn explicit_off_preserves_standalone_output() {
        let mut lineage =
            StyleLineage::resolve("styles-legacy/apa-6th-edition.csl", &repo_root(), &[])
                .expect("lineage should resolve");
        let routing = apply_family_candidate_routing(
            &mut lineage,
            &repo_root(),
            &FamilyCandidateMode::Off,
            "styles-legacy/apa-6th-edition.csl",
        )
        .expect("explicit off should apply");

        assert!(lineage.parent_style_id.is_none());
        assert_eq!(
            routing.audit.source,
            MinimizationDecisionSource::ExplicitOff
        );
    }

    #[test]
    fn default_family_candidate_mode_preserves_standalone_output() {
        let mut lineage =
            StyleLineage::resolve("styles-legacy/apa-6th-edition.csl", &repo_root(), &[])
                .expect("lineage should resolve");
        let routing = apply_family_candidate_routing(
            &mut lineage,
            &repo_root(),
            &FamilyCandidateMode::Default,
            "styles-legacy/apa-6th-edition.csl",
        )
        .expect("default routing should apply");

        assert!(lineage.parent_style_id.is_none());
        assert_eq!(routing.audit.source, MinimizationDecisionSource::None);
        assert_eq!(
            routing.audit.outcome,
            MinimizationDecisionOutcome::NotSelected
        );
    }
}
