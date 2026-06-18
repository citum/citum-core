/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "bin")]

mod assembly;
mod bib_postprocess;
mod citation_validate;
mod cli;
mod output_plan;
mod runtime;

use assembly::{
    StandaloneAssembly, TemplateSourceSelection, apply_measured_bibliography_selection,
    apply_measured_citation_selection,
};
use citation_validate::validate_and_normalize_inferred_citations;
use clap::Parser;
use cli::{Args, FamilyCandidateMode};
use output_plan::{
    apply_family_candidate_routing, count_yaml_lines, log_migration_output_plan,
    write_optional_evidence,
};
use runtime::{
    log_template_sources, output_style_and_debug, resolve_style_name_and_templates,
    workspace_root_for_style_path,
};

use citum_migrate::{
    OptionsExtractor, compilation,
    evidence::MeasuredSelectionEvidence,
    lineage::StyleLineage,
    options_extractor::{MigrationOptions, processing::detect_processing_mode},
    passes::sqi_refinement,
    provenance::ProvenanceTracker,
};
use csl_legacy::parser::parse_style;
use roxmltree::Document;
use std::{fs, path::Path};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let cli = Args::parse();
    let path = &cli.path;
    let family_candidate = FamilyCandidateMode::from_arg(cli.family_candidate.as_deref());

    let enable_provenance = cli.debug_variable.is_some();
    let tracker = ProvenanceTracker::new(enable_provenance);
    let workspace_root = workspace_root_for_style_path(path);

    let text = fs::read_to_string(path)?;
    let doc = Document::parse(&text)?;
    let legacy_style = parse_style(doc.root_element())?;

    let detected_regime = detect_processing_mode(&legacy_style);
    let mut lineage = StyleLineage::resolve(path, &workspace_root, &legacy_style.info.links)?
        .apply_regime_guard(detected_regime.as_ref());
    let routing =
        apply_family_candidate_routing(&mut lineage, &workspace_root, &family_candidate, path)?;

    tracing::debug!("Migrating {path} to Citum...");
    tracing::debug!(
        "Resolved lineage: semantic={:?}, form={:?}, parent={}",
        lineage.semantic_class,
        lineage.implementation_form,
        lineage.parent_style_id.as_deref().unwrap_or("none")
    );
    log_migration_output_plan(&lineage);

    let MigrationOptions {
        mut options,
        bibliography_options,
        citation_contributor_overrides,
        bibliography_contributor_overrides,
        citation_has_scope_shorten,
    } = OptionsExtractor::extract_migration_options(&legacy_style);

    let (style_name, mut resolved) = resolve_style_name_and_templates(path, &cli);

    validate_and_normalize_inferred_citations(
        &mut resolved,
        &options,
        &legacy_style,
        style_name.as_str(),
        citation_has_scope_shorten,
    );

    let xml_fallback = Some(compilation::compile_from_xml(
        &legacy_style,
        &mut options,
        enable_provenance,
        &tracker,
    ));

    if let Some(ref fallback) = xml_fallback
        && fallback.unsupported_mixed_conditions
    {
        tracing::debug!(
            "Warning: citation position branches could not be migrated cleanly for style {}. Falling back to base citation template only.",
            legacy_style.info.id
        );
    }

    log_template_sources(&resolved);

    let assembly = StandaloneAssembly {
        legacy_style: &legacy_style,
        resolved: &resolved,
        xml_fallback: &xml_fallback,
        options: &options,
        bibliography_options: &bibliography_options,
        citation_contributor_overrides: &citation_contributor_overrides,
        bibliography_contributor_overrides: &bibliography_contributor_overrides,
    };
    let (standalone_style, measured_selection) =
        apply_measured_selection_pipeline(&assembly, &style_name, &text, &workspace_root);
    let standalone_style = sqi_refinement::refine_style(standalone_style);
    // Measure the standalone form first so the evidence record can report
    // the compression delta without re-running the pipeline. Cheap: one YAML
    // serialization of an in-memory value.
    let standalone_lines = count_yaml_lines(&standalone_style)?;

    let style = if cli.minimize_wrapper {
        lineage.apply_to_migrated_style_minimized(standalone_style, true)?
    } else {
        lineage.apply_to_migrated_style(standalone_style)?
    };
    let emitted_lines = count_yaml_lines(&style)?;

    write_optional_evidence(
        &cli,
        &lineage,
        standalone_lines,
        emitted_lines,
        cli.minimize_wrapper,
        routing.audit,
        measured_selection,
    )?;

    output_style_and_debug(&style, cli.debug_variable.as_deref(), &tracker)?;
    Ok(())
}

fn apply_measured_selection_pipeline(
    assembly: &StandaloneAssembly<'_>,
    style_name: &str,
    style_xml: &str,
    workspace_root: &Path,
) -> (citum_schema::Style, Option<MeasuredSelectionEvidence>) {
    let mut source_selection = TemplateSourceSelection::default();
    let standalone_style = assembly.assemble_with_selection(source_selection);
    let (standalone_style, use_xml_citation, citation_selection) =
        apply_measured_citation_selection(
            standalone_style,
            assembly,
            source_selection,
            style_name,
            style_xml,
            workspace_root,
        );
    source_selection.suppress_inferred_citation = use_xml_citation;
    let (standalone_style, _, bibliography_selection) = apply_measured_bibliography_selection(
        standalone_style,
        assembly,
        source_selection,
        style_name,
        style_xml,
        workspace_root,
    );
    let measured_selection = MeasuredSelectionEvidence {
        citation: citation_selection,
        bibliography: bibliography_selection,
    };
    (
        standalone_style,
        (!measured_selection.is_empty()).then_some(measured_selection),
    )
}

#[cfg(test)]
mod main_tests;
