/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "bin")]

mod bib_postprocess;
mod citation_validate;
mod cli;
mod template_diff;

use bib_postprocess::{
    is_inferred_bib_source, merge_inferred_type_templates, postprocess_inferred_bibliography,
};
use citation_validate::validate_and_normalize_inferred_citations;
use clap::Parser;
use cli::{Args, FamilyCandidateMode};
use template_diff::{TypeTemplateMap, build_type_variants};

use citum_migrate::{
    OptionsExtractor, analysis,
    compilation::{self, XmlCompilationOutput as XmlFallback},
    debug_output::DebugOutputFormatter,
    evidence::{
        EmittedForm, MinimizationDecisionAudit, MinimizationDecisionOutcome,
        MinimizationDecisionSource,
    },
    fixups::{
        ensure_numeric_locator_citation_component, ensure_personal_communication_omitted,
        move_group_wrap_to_citation_items, normalize_author_date_locator_citation_component,
        normalize_wrapped_numeric_locator_citation_component,
    },
    lineage::{MigrationOutputPlan, StyleLineage},
    options_extractor::MigrationOptions,
    provenance::ProvenanceTracker,
    template_resolver,
};
use citum_schema::{
    BibliographySpec, CitationCollapse, CitationSpec, Style, StyleInfo,
    template::{TemplateComponent, WrapPunctuation},
};
use csl_legacy::parser::parse_style;
use roxmltree::Document;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// All compiled template and option data needed to build the final Style.
struct CompiledOutput {
    options: citum_schema::options::Config,
    citation_contributor_overrides: Option<citum_schema::options::ContributorConfig>,
    bibliography_options: Option<citum_schema::BibliographyOptions>,
    bibliography_contributor_overrides: Option<citum_schema::options::ContributorConfig>,
    new_cit: Vec<TemplateComponent>,
    new_bib: Vec<TemplateComponent>,
    type_templates: Option<TypeTemplateMap>,
    citation_wrap: Option<WrapPunctuation>,
    citation_prefix: Option<String>,
    citation_suffix: Option<String>,
    citation_delimiter: Option<String>,
    citation_subsequent_override: Option<Vec<TemplateComponent>>,
    citation_ibid_override: Option<Vec<TemplateComponent>>,
}

fn extract_citation_collapse(citation: &csl_legacy::model::Citation) -> Option<CitationCollapse> {
    match citation.collapse.as_deref() {
        Some("citation-number") => Some(CitationCollapse::CitationNumber),
        _ => None,
    }
}

/// Assembles the final Citum Style from compiled output and legacy metadata.
fn build_final_style(legacy_style: &csl_legacy::model::Style, mut c: CompiledOutput) -> Style {
    let citation_scope_options =
        c.citation_contributor_overrides
            .map(|contributors| citum_schema::CitationOptions {
                contributors: Some(contributors),
                ..Default::default()
            });
    let mut bibliography_scope_options = c.bibliography_options.take().unwrap_or_default();
    if let Some(contributors) = c.bibliography_contributor_overrides.take() {
        bibliography_scope_options.contributors = Some(contributors);
    }
    let bibliography_scope_options = (bibliography_scope_options
        != citum_schema::BibliographyOptions::default())
    .then_some(bibliography_scope_options);
    let bibliography_sort = resolve_migrated_bibliography_sort(
        c.options.processing.as_ref(),
        legacy_style
            .bibliography
            .as_ref()
            .and_then(|bib| bib.sort.as_ref()),
    );

    // [PRUNING] Remove bibliography type-variants identical to the primary template.
    if let Some(type_templates) = c.type_templates.as_mut() {
        type_templates.retain(|_, template| template != &c.new_bib);
    }
    let type_variants = c
        .type_templates
        .take()
        .map(|type_templates| build_type_variants(&c.new_bib, type_templates));

    // [PRUNING] Prune redundant citation modes (e.g. ibid/subsequent if they match base).
    let subsequent = c
        .citation_subsequent_override
        .filter(|t| t != &c.new_cit)
        .map(|t| {
            Box::new(CitationSpec {
                template: Some(t),
                ..Default::default()
            })
        });

    let ibid = c
        .citation_ibid_override
        .filter(|t| t != &c.new_cit)
        .map(|t| {
            Box::new(CitationSpec {
                template: Some(t),
                ..Default::default()
            })
        });

    Style {
        info: StyleInfo {
            title: Some(legacy_style.info.title.clone()),
            id: Some(legacy_style.info.id.clone()),
            default_locale: legacy_style.default_locale.clone(),
            ..Default::default()
        },
        templates: None,
        options: Some(c.options),
        citation: Some(CitationSpec {
            options: citation_scope_options,
            template_ref: None,
            template: Some(c.new_cit),
            collapse: extract_citation_collapse(&legacy_style.citation),
            wrap: c.citation_wrap.map(Into::into),
            prefix: c.citation_prefix,
            suffix: c.citation_suffix,
            delimiter: c.citation_delimiter,
            multi_cite_delimiter: legacy_style.citation.layout.delimiter.clone(),
            subsequent,
            ibid,
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            options: bibliography_scope_options,
            template_ref: None,
            template: Some(c.new_bib),
            type_variants,
            sort: bibliography_sort,
            ..Default::default()
        }),
        ..Default::default()
    }
}

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

    let mut lineage = StyleLineage::resolve(path, &workspace_root, &legacy_style.info.links)?;
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
        mut bibliography_options,
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

    let (new_bib, mut type_templates, inferred_bib_source) =
        select_and_process_bibliography_template(&resolved, &xml_fallback, &legacy_style);

    let (mut new_cit, citation_subsequent_override, citation_ibid_override) =
        select_citation_template(
            &resolved,
            &xml_fallback,
            inferred_bib_source,
            &legacy_style,
            &mut type_templates,
        );

    override_bibliography_options_if_inferred(&resolved, &legacy_style, &mut bibliography_options);

    let (citation_wrap, citation_prefix, citation_suffix, citation_delimiter) =
        resolve_citation_metadata(&resolved, &legacy_style, &options, &mut new_cit);

    let standalone_style = build_final_style(
        &legacy_style,
        CompiledOutput {
            options,
            citation_contributor_overrides,
            bibliography_options,
            bibliography_contributor_overrides,
            new_cit,
            new_bib,
            type_templates,
            citation_wrap,
            citation_prefix,
            citation_suffix,
            citation_delimiter,
            citation_subsequent_override,
            citation_ibid_override,
        },
    );
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
    )?;

    output_style_and_debug(&style, cli.debug_variable.as_deref(), &tracker)?;
    Ok(())
}

fn write_optional_evidence(
    cli: &Args,
    lineage: &StyleLineage,
    standalone_lines: usize,
    emitted_lines: usize,
    minimized: bool,
    minimization_decision: MinimizationDecisionAudit,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(evidence_path) = cli.emit_evidence.as_deref() {
        write_evidence_sidecar(
            evidence_path,
            lineage,
            standalone_lines,
            emitted_lines,
            minimized,
            minimization_decision,
        )?;
    }
    Ok(())
}

/// Effective family-candidate routing after applying explicit flags.
struct FamilyCandidateRouting {
    audit: MinimizationDecisionAudit,
}

/// Promote a discovered family-candidate parent into the lineage's active
/// routing slot when explicit flags request it.
fn apply_family_candidate_routing(
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

/// Build the migration evidence record and write it as a JSON sidecar at the
/// CLI-supplied path. Centralizes the sidecar policy so `main` stays compact.
fn write_evidence_sidecar(
    evidence_path: &Path,
    lineage: &StyleLineage,
    standalone_lines: usize,
    emitted_lines: usize,
    minimized: bool,
    minimization_decision: MinimizationDecisionAudit,
) -> Result<(), Box<dyn std::error::Error>> {
    let emitted_form = describe_emitted_form(lineage, minimized);
    // Classify which template-bearing scopes the wrapper retained vs
    // inherited from the parent. The minimize path drops every
    // template-bearing scope; the regular wrapper path either preserves
    // them (`preserve_template_deltas: true`) or discards them
    // (`preserve_template_deltas: false`). Standalone output has no
    // template-bearing inheritance to report.
    let (preserved, discarded) = classify_template_paths(&emitted_form);
    let evidence = lineage.build_evidence(
        standalone_lines,
        emitted_form,
        emitted_lines,
        minimization_decision,
        preserved,
        discarded,
    );
    let json = serde_json::to_string_pretty(&evidence)?;
    fs::write(evidence_path, json)?;
    Ok(())
}

/// Snapshot of the dotted-path representation of the template-bearing scopes
/// the migrator tracks. Mirrors `lineage::TEMPLATE_BEARING_PATHS` so the
/// evidence record stays meaningful even when callers don't have direct
/// access to that internal constant.
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

/// Count the lines of a style's YAML serialization. Used as a cheap reference
/// for the migration evidence record; counting on the in-memory value avoids
/// re-reading the final emitted file.
fn count_yaml_lines(style: &Style) -> Result<usize, Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(style)?;
    Ok(yaml.lines().count())
}

/// Translate the lineage's effective `MigrationOutputPlan` into the
/// reviewer-facing `EmittedForm` enum used by the evidence record.
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
        // Multi-artifact plans are not yet wired through `apply_to_migrated_style`;
        // they remain rare and fall back to the standalone description for
        // evidence reporting purposes.
        _ => EmittedForm::Standalone,
    }
}

/// Extracts style name from path and resolves templates.
fn resolve_style_name_and_templates(
    path: &str,
    cli: &Args,
) -> (String, template_resolver::ResolvedTemplates) {
    let style_name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let workspace_root = workspace_root_for_style_path(path);

    let resolved = template_resolver::resolve_templates(
        path,
        style_name.as_str(),
        cli.template_dir.as_deref(),
        &workspace_root,
        cli.template_mode,
        cli.min_template_confidence,
        cli.live_infer_backend,
    );

    (style_name, resolved)
}

fn workspace_root_for_style_path(path: &str) -> PathBuf {
    let style_path = Path::new(path);
    let rooted_style_path = if style_path.is_absolute() {
        style_path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(style_path)
    };

    let workspace_root = rooted_style_path
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists())
        .unwrap_or(rooted_style_path.parent().unwrap_or(Path::new(".")))
        .to_path_buf();
    fs::canonicalize(&workspace_root).unwrap_or(workspace_root)
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
fn log_migration_output_plan(lineage: &StyleLineage) {
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
        plan if plan.requires_multi_artifact_write() => {
            tracing::debug!("Migration output plan: multi-artifact {plan:?}");
        }
        plan => {
            tracing::debug!("Migration output plan: {plan:?}");
        }
    }
}

fn output_style_and_debug(
    style: &Style,
    debug_variable: Option<&str>,
    tracker: &ProvenanceTracker,
) -> Result<(), Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(style)?;
    writeln!(std::io::stdout(), "{yaml}")?;

    if let Some(var_name) = debug_variable {
        let debug_output = DebugOutputFormatter::format_variable(tracker, var_name);
        eprint!("{debug_output}");
    }

    Ok(())
}

fn resolve_migrated_bibliography_sort(
    processing: Option<&citum_schema::options::Processing>,
    legacy_sort: Option<&csl_legacy::model::Sort>,
) -> Option<citum_schema::grouping::GroupSortEntry> {
    let extracted_entry = legacy_sort.and_then(
        citum_migrate::options_extractor::bibliography::extract_group_sort_from_bibliography,
    )?;
    let extracted = extracted_entry.resolve();

    if bibliography_sort_matches_processing_default(processing, &extracted) {
        None
    } else {
        Some(extracted_entry)
    }
}

fn bibliography_sort_matches_processing_default(
    processing: Option<&citum_schema::options::Processing>,
    sort: &citum_schema::grouping::GroupSort,
) -> bool {
    processing
        .and_then(citum_schema::options::Processing::default_bibliography_sort)
        .is_some_and(|preset| preset.group_sort() == *sort)
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
fn log_template_sources(resolved: &template_resolver::ResolvedTemplates) {
    if let Some(ref resolved_bib) = resolved.bibliography {
        tracing::debug!("Using {} bibliography template", resolved_bib.source);
        if let Some(conf) = resolved_bib.confidence {
            tracing::debug!("  bibliography confidence: {:.0}%", conf * 100.0);
        }
    } else {
        tracing::debug!(
            "Using {} bibliography template",
            template_resolver::TemplateSource::XmlCompiled
        );
    }

    if let Some(ref resolved_cit) = resolved.citation {
        tracing::debug!("Using {} citation template", resolved_cit.source);
        if let Some(conf) = resolved_cit.confidence {
            tracing::debug!("  citation confidence: {:.0}%", conf * 100.0);
        }
    } else {
        tracing::debug!(
            "Using {} citation template",
            template_resolver::TemplateSource::XmlCompiled
        );
    }
}

fn select_and_process_bibliography_template(
    resolved: &template_resolver::ResolvedTemplates,
    xml_fallback: &Option<XmlFallback>,
    legacy_style: &csl_legacy::model::Style,
) -> (Vec<TemplateComponent>, Option<TypeTemplateMap>, bool) {
    let (mut new_bib, mut type_templates, inferred_bib_source) =
        if let Some(ref resolved_bib) = resolved.bibliography {
            let inferred_bib = is_inferred_bib_source(&resolved_bib.source);
            let merged_type_templates = if inferred_bib {
                xml_fallback
                    .as_ref()
                    .and_then(|out| merge_inferred_type_templates(out, &resolved_bib.template))
            } else {
                None
            };
            (
                resolved_bib.template.clone(),
                merged_type_templates,
                inferred_bib,
            )
        } else {
            #[allow(clippy::expect_used, reason = "fatal bootstrap error")]
            let out = xml_fallback
                .as_ref()
                .expect("XML fallback must exist when bibliography is unresolved");
            (out.bibliography.clone(), out.type_templates.clone(), false)
        };

    if inferred_bib_source {
        postprocess_inferred_bibliography(&mut new_bib, &mut type_templates, legacy_style);
    }

    (new_bib, type_templates, inferred_bib_source)
}

fn select_citation_template(
    resolved: &template_resolver::ResolvedTemplates,
    xml_fallback: &Option<XmlFallback>,
    inferred_bib_source: bool,
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<TypeTemplateMap>,
) -> (
    Vec<TemplateComponent>,
    Option<Vec<TemplateComponent>>,
    Option<Vec<TemplateComponent>>,
) {
    let mut citation_subsequent_override: Option<Vec<TemplateComponent>> = None;
    let mut citation_ibid_override: Option<Vec<TemplateComponent>> = None;
    let new_cit = if let Some(ref resolved_cit) = resolved.citation {
        // Inferred templates only capture the first-position form. For note
        // styles, the subsequent/ibid short forms exist solely in the XML
        // pipeline's position extraction — attach them so the inferred first
        // form keeps its repeat behavior. Hand-authored templates are left
        // alone: their repeat shape is a deliberate authoring decision.
        if legacy_style.class == "note"
            && citation_validate::is_inferred_source(&resolved_cit.source)
            && let Some(out) = xml_fallback.as_ref()
        {
            citation_subsequent_override = out.citation_overrides.subsequent.clone();
            citation_ibid_override = out.citation_overrides.ibid.clone();
        }
        resolved_cit.template.clone()
    } else {
        #[allow(clippy::expect_used, reason = "fatal bootstrap error")]
        let out = xml_fallback
            .as_ref()
            .expect("XML fallback must exist when citation is unresolved");
        citation_subsequent_override = out.citation_overrides.subsequent.clone();
        citation_ibid_override = out.citation_overrides.ibid.clone();
        out.citation.clone()
    };

    if inferred_bib_source {
        ensure_personal_communication_omitted(legacy_style, &new_cit, type_templates);
    }

    (
        new_cit,
        citation_subsequent_override,
        citation_ibid_override,
    )
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
fn override_bibliography_options_if_inferred(
    resolved: &template_resolver::ResolvedTemplates,
    legacy_style: &csl_legacy::model::Style,
    options: &mut Option<citum_schema::BibliographyOptions>,
) {
    if let Some(ref resolved_bib) = resolved.bibliography {
        let allow_bib_punctuation_override =
            !(legacy_style.class == "note" && is_inferred_bib_source(&resolved_bib.source));

        if allow_bib_punctuation_override {
            if let Some(ref delim) = resolved_bib.delimiter {
                tracing::debug!("  Overriding bibliography separator: {delim:?}");
                let bib_cfg = options.get_or_insert_with(Default::default);
                bib_cfg.separator = Some(delim.clone());
            }

            if let Some(ref suffix) = resolved_bib.entry_suffix {
                tracing::debug!("  Overriding bibliography entry suffix: {suffix:?}");
                let bib_cfg = options.get_or_insert_with(Default::default);
                bib_cfg.entry_suffix = Some(suffix.clone());
            }
        } else {
            tracing::debug!(
                "  Skipping inferred bibliography separator/entry-suffix override for note style."
            );
        }
    }
}

fn resolve_citation_metadata(
    resolved: &template_resolver::ResolvedTemplates,
    legacy_style: &csl_legacy::model::Style,
    options: &citum_schema::options::Config,
    new_cit: &mut Vec<TemplateComponent>,
) -> (
    Option<citum_schema::template::WrapPunctuation>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let (mut citation_wrap, mut citation_prefix, mut citation_suffix) =
        analysis::citation::infer_citation_wrapping(&legacy_style.citation.layout);
    let mut citation_delimiter = analysis::citation::extract_citation_delimiter(
        &legacy_style.citation.layout,
        &legacy_style.macros,
    );

    if let Some(ref resolved_cit) = resolved.citation {
        if let Some(ref wrap) = resolved_cit.wrap {
            citation_wrap = Some(wrap.clone());
            citation_prefix = None;
            citation_suffix = None;
        }
        if let Some(ref delim) = resolved_cit.delimiter {
            citation_delimiter = Some(delim.clone());
        }
    }

    if matches!(
        options.processing,
        Some(citum_schema::options::Processing::Numeric)
    ) {
        ensure_numeric_locator_citation_component(&legacy_style.citation.layout, new_cit);
        normalize_wrapped_numeric_locator_citation_component(
            &legacy_style.citation.layout,
            new_cit,
            &mut citation_delimiter,
        );
        move_group_wrap_to_citation_items(
            &legacy_style.citation.layout,
            new_cit,
            &mut citation_wrap,
        );
    } else if legacy_style.class == "in-text" {
        normalize_author_date_locator_citation_component(
            &legacy_style.citation.layout,
            &legacy_style.macros,
            new_cit,
        );
    }

    (
        citation_wrap,
        citation_prefix,
        citation_suffix,
        citation_delimiter,
    )
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
    use super::bib_postprocess::{
        component_is_in_term, component_is_primary_title, component_is_publisher,
        component_is_publisher_place, repair_inferred_bibliography_type_templates,
    };
    use super::template_diff::template_variant_from_full_template;
    use super::*;
    use citum_schema::locale::GeneralTerm;
    use citum_schema::template::{
        ContributorRole, DateVariable, Rendering, SimpleVariable, TemplateComponent,
        TemplateContributor, TemplateDate, TemplateTerm, TemplateTitle, TemplateVariable,
        TemplateVariant, TitleType, TypeSelector,
    };
    use csl_legacy::model::{
        Citation, CslNode, Formatting, Group, Info, Layout, Sort as LegacySort,
        SortKey as LegacySortKey, Style as LegacyStyle, Text,
    };
    use std::sync::{Mutex, OnceLock};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate should live under crates/citum-migrate")
            .to_path_buf()
    }

    fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("cwd lock should not be poisoned")
    }

    #[test]
    fn workspace_root_resolves_relative_paths_from_subdirectories() {
        let _guard = cwd_lock();
        let original_cwd = std::env::current_dir().expect("current dir should be available");
        std::env::set_current_dir(repo_root().join("crates"))
            .expect("test should enter repo subdirectory");

        let workspace_root = workspace_root_for_style_path("../styles-legacy/apa-6th-edition.csl");

        std::env::set_current_dir(original_cwd).expect("test should restore cwd");
        assert_eq!(workspace_root, repo_root());
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

    fn legacy_sort(keys: &[&str]) -> LegacySort {
        LegacySort {
            keys: keys
                .iter()
                .map(|key| LegacySortKey {
                    variable: Some((*key).to_string()),
                    macro_name: None,
                    sort: None,
                })
                .collect(),
        }
    }

    #[test]
    fn template_v3_diff_generator_emits_rendering_modify() {
        let default_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                ..Default::default()
            }),
        ];
        let target_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                rendering: Rendering {
                    suffix: Some(".".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ];

        let variant = template_variant_from_full_template(
            &default_template,
            &[],
            &TypeSelector::Single("book".to_string()),
            target_template,
        );

        let TemplateVariant::Diff(diff) = variant else {
            panic!("rendering-only template changes should emit Template V3 diffs");
        };
        assert_eq!(diff.modify.len(), 1);
        assert!(diff.remove.is_empty());
        assert!(diff.add.is_empty());
    }

    #[test]
    fn template_v3_diff_generator_emits_structural_remove_and_add() {
        let default_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                ..Default::default()
            }),
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                ..Default::default()
            }),
        ];
        let target_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Date(TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                ..Default::default()
            }),
        ];

        let variant = template_variant_from_full_template(
            &default_template,
            &[],
            &TypeSelector::Single("book".to_string()),
            target_template,
        );

        let TemplateVariant::Diff(diff) = variant else {
            panic!("safe structural template changes should emit Template V3 diffs");
        };
        assert!(diff.modify.is_empty());
        assert_eq!(diff.remove.len(), 1);
        assert_eq!(diff.add.len(), 1);
    }

    #[test]
    fn template_v3_diff_generator_falls_back_for_non_rendering_changes() {
        let default_template = vec![TemplateComponent::Title(TemplateTitle {
            title: TitleType::Primary,
            ..Default::default()
        })];
        let target_template = vec![TemplateComponent::Title(TemplateTitle {
            title: TitleType::Primary,
            form: Some(citum_schema::template::TitleForm::Short),
            ..Default::default()
        })];

        let variant = template_variant_from_full_template(
            &default_template,
            &[],
            &TypeSelector::Single("book".to_string()),
            target_template,
        );

        assert!(matches!(variant, TemplateVariant::Full(_)));
    }

    #[test]
    fn template_v3_diff_generator_can_extend_prior_variant() {
        let default_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                ..Default::default()
            }),
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                ..Default::default()
            }),
        ];
        let book_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                rendering: Rendering {
                    suffix: Some(".".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ];
        let chapter_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                rendering: Rendering {
                    suffix: Some("!".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ];
        let parent_selector = TypeSelector::Single("book".to_string());
        let parents = vec![(parent_selector.clone(), book_template)];

        let variant = template_variant_from_full_template(
            &default_template,
            &parents,
            &TypeSelector::Single("chapter".to_string()),
            chapter_template,
        );

        let TemplateVariant::Diff(diff) = variant else {
            panic!("variant should extend prior variant when it is more concise");
        };
        assert_eq!(diff.extends, Some(parent_selector));
        assert_eq!(diff.modify.len(), 1);
        assert!(diff.remove.is_empty());
    }

    #[test]
    fn inferred_type_variants_recover_missing_primary_title() {
        let default_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Date(TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                ..Default::default()
            }),
        ];

        let mut type_templates = indexmap::IndexMap::from([(
            TypeSelector::Single("article-newspaper".to_string()),
            vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    ..Default::default()
                }),
                TemplateComponent::Date(TemplateDate {
                    date: DateVariable::Issued,
                    ..Default::default()
                }),
                TemplateComponent::Term(TemplateTerm {
                    term: GeneralTerm::In,
                    ..Default::default()
                }),
            ],
        )]);

        repair_inferred_bibliography_type_templates(&default_template, &mut type_templates);

        let variant = type_templates
            .get(&TypeSelector::Single("article-newspaper".to_string()))
            .expect("article-newspaper variant should exist");

        assert!(
            variant.iter().any(component_is_primary_title),
            "underfit inferred type variants should inherit the base primary title"
        );
        let title_index = variant
            .iter()
            .position(component_is_primary_title)
            .expect("title should be present");
        let in_index = variant
            .iter()
            .position(component_is_in_term)
            .expect("in term should be present");
        assert!(
            title_index < in_index,
            "recovered title should appear before container-introducing terms"
        );
    }

    #[test]
    fn inferred_type_variants_recover_missing_publisher() {
        let default_template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            }),
            TemplateComponent::Date(TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            }),
            TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                ..Default::default()
            }),
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::PublisherPlace,
                ..Default::default()
            }),
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                ..Default::default()
            }),
        ];

        let mut type_templates = indexmap::IndexMap::from([(
            TypeSelector::Single("book".to_string()),
            vec![
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    ..Default::default()
                }),
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    ..Default::default()
                }),
                TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::PublisherPlace,
                    ..Default::default()
                }),
            ],
        )]);

        repair_inferred_bibliography_type_templates(&default_template, &mut type_templates);

        let variant = type_templates
            .get(&TypeSelector::Single("book".to_string()))
            .expect("book variant should exist");

        assert!(
            variant.iter().any(component_is_publisher),
            "monographic inferred type variants should inherit the base publisher"
        );
        let publisher_place_index = variant
            .iter()
            .position(component_is_publisher_place)
            .expect("publisher-place should be present");
        let publisher_index = variant
            .iter()
            .position(component_is_publisher)
            .expect("publisher should be present");
        assert_eq!(
            publisher_index,
            publisher_place_index + 1,
            "publisher should follow publisher-place after repair"
        );
    }

    #[test]
    fn suppresses_author_date_default_bibliography_sort() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::AuthorDate),
            Some(&legacy_sort(&["author", "issued", "title"])),
        );

        assert_eq!(sort, None);
    }

    #[test]
    fn suppresses_note_default_bibliography_sort() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::Note),
            Some(&legacy_sort(&["author", "title", "issued"])),
        );

        assert_eq!(sort, None);
    }

    #[test]
    fn preserves_numeric_bibliography_sort() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::Numeric),
            Some(&legacy_sort(&["author", "issued", "title"])),
        );

        assert_eq!(
            sort,
            Some(citum_schema::grouping::GroupSortEntry::Preset(
                citum_schema::presets::SortPreset::AuthorDateTitle
            ))
        );
    }

    #[test]
    fn preserves_note_family_exceptions() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::Note),
            Some(&legacy_sort(&["author", "issued", "title"])),
        );

        assert!(sort.is_some());
    }

    fn minimal_legacy_style() -> LegacyStyle {
        LegacyStyle {
            version: "1.0".to_string(),
            xmlns: "http://purl.org/net/xbiblio/csl".to_string(),
            class: "in-text".to_string(),
            default_locale: None,
            initialize_with: None,
            initialize_with_hyphen: None,
            names_delimiter: None,
            name_as_sort_order: None,
            sort_separator: None,
            delimiter_precedes_last: None,
            delimiter_precedes_et_al: None,
            demote_non_dropping_particle: None,
            and: None,
            page_range_format: None,
            info: Info::default(),
            locale: vec![],
            macros: vec![],
            citation: Citation {
                layout: Layout {
                    prefix: None,
                    suffix: None,
                    delimiter: None,
                    children: vec![],
                },
                sort: None,
                collapse: None,
                et_al_min: None,
                et_al_use_first: None,
                disambiguate_add_year_suffix: None,
                disambiguate_add_names: None,
                disambiguate_add_givenname: None,
                disambiguate_givenname_rule: None,
            },
            bibliography: None,
        }
    }

    #[test]
    fn maps_legacy_citation_number_collapse() {
        let mut style = minimal_legacy_style();
        style.citation.collapse = Some("citation-number".to_string());

        let migrated = build_final_style(
            &style,
            CompiledOutput {
                options: citum_schema::options::Config::default(),
                bibliography_options: None,
                citation_contributor_overrides: None,
                bibliography_contributor_overrides: None,
                new_cit: vec![],
                new_bib: vec![],
                type_templates: None,
                citation_wrap: None,
                citation_prefix: None,
                citation_suffix: None,
                citation_delimiter: None,
                citation_subsequent_override: None,
                citation_ibid_override: None,
            },
        );

        assert_eq!(
            migrated
                .citation
                .as_ref()
                .and_then(|citation| citation.collapse.clone()),
            Some(CitationCollapse::CitationNumber)
        );
    }

    #[test]
    fn author_date_locator_prefers_group_delimiter() {
        let layout = Layout {
            prefix: None,
            suffix: None,
            delimiter: None,
            children: vec![CslNode::Group(Group {
                delimiter: Some(", ".to_string()),
                prefix: None,
                suffix: None,
                children: vec![
                    CslNode::Text(Text {
                        value: None,
                        variable: None,
                        macro_name: Some("author-short".to_string()),
                        term: None,
                        form: None,
                        prefix: None,
                        suffix: None,
                        quotes: None,
                        text_case: None,
                        strip_periods: None,
                        plural: None,
                        macro_call_order: None,
                        formatting: Formatting::default(),
                    }),
                    CslNode::Text(Text {
                        value: None,
                        variable: None,
                        macro_name: Some("issued-year".to_string()),
                        term: None,
                        form: None,
                        prefix: None,
                        suffix: None,
                        quotes: None,
                        text_case: None,
                        strip_periods: None,
                        plural: None,
                        macro_call_order: None,
                        formatting: Formatting::default(),
                    }),
                    CslNode::Text(Text {
                        value: None,
                        variable: None,
                        macro_name: Some("citation-locator".to_string()),
                        term: None,
                        form: None,
                        prefix: None,
                        suffix: None,
                        quotes: None,
                        text_case: None,
                        strip_periods: None,
                        plural: None,
                        macro_call_order: None,
                        formatting: Formatting::default(),
                    }),
                ],
                macro_call_order: None,
                formatting: Formatting::default(),
            })],
        };
        let mut template = vec![
            TemplateComponent::Contributor(citum_schema::template::TemplateContributor {
                contributor: citum_schema::template::ContributorRole::Author,
                form: citum_schema::template::ContributorForm::Short,
                name_order: Some(citum_schema::template::NameOrder::FamilyFirst),
                ..Default::default()
            }),
            TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: DateVariable::Issued,
                form: citum_schema::template::DateForm::Year,
                rendering: Rendering {
                    prefix: Some(", ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Locator,
                rendering: Rendering {
                    prefix: Some(" ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ];

        normalize_author_date_locator_citation_component(&layout, &[], &mut template);

        let locator = template
            .iter()
            .find_map(|component| match component {
                TemplateComponent::Variable(variable)
                    if variable.variable == SimpleVariable::Locator =>
                {
                    Some(variable)
                }
                _ => None,
            })
            .expect("locator component should exist");

        assert_eq!(locator.rendering.prefix.as_deref(), Some(", "));
    }

    fn parse_legacy_style(xml: &str) -> csl_legacy::model::Style {
        let doc = Document::parse(xml).expect("test style XML should parse");
        parse_style(doc.root_element()).expect("legacy style parsing should succeed")
    }

    // The note-class citation path preserves authored group structure, so
    // template assertions must search nested groups, not just the top level.
    fn template_contains(
        components: &[TemplateComponent],
        predicate: &dyn Fn(&TemplateComponent) -> bool,
    ) -> bool {
        components.iter().any(|component| {
            predicate(component)
                || matches!(
                    component,
                    TemplateComponent::Group(group)
                        if template_contains(&group.group, predicate)
                )
        })
    }

    #[test]
    fn compile_from_xml_maps_nested_position_chooses_into_citation_overrides() {
        let legacy_style = parse_legacy_style(
            r#"
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <title>position-test</title>
    <id>https://example.org/position-test</id>
  </info>
  <citation>
    <layout>
      <group delimiter=" ">
        <text value="prefix"/>
        <choose>
          <if position="subsequent">
            <text variable="author"/>
          </if>
          <else-if position="first">
            <text variable="title"/>
          </else-if>
          <else>
            <date variable="issued">
              <date-part name="year"/>
            </date>
          </else>
        </choose>
        <choose>
          <if position="ibid-with-locator">
            <group delimiter=" ">
              <text term="ibid"/>
              <text variable="locator"/>
            </group>
          </if>
          <else-if position="ibid">
            <text term="ibid"/>
          </else-if>
          <else>
            <date variable="issued">
              <date-part name="year"/>
            </date>
          </else>
        </choose>
        <text value="suffix"/>
      </group>
    </layout>
  </citation>
</style>
"#,
        );

        let mut options = citum_schema::options::Config::default();
        let tracker = ProvenanceTracker::new(false);
        let out = compilation::compile_from_xml(&legacy_style, &mut options, false, &tracker);

        assert!(
            template_contains(&out.citation, &|component| matches!(
                component,
                TemplateComponent::Title(_)
            )),
            "explicit first-position branch should become part of the base citation template"
        );
        assert!(
            template_contains(&out.citation, &|component| matches!(
                component,
                TemplateComponent::Date(_)
            )),
            "fallback content from sibling chooses should remain in the base citation template"
        );

        assert_position_override_shapes(&out);
    }

    fn assert_position_override_shapes(out: &compilation::XmlCompilationOutput) {
        let subsequent_template = out
            .citation_overrides
            .subsequent
            .as_ref()
            .expect("subsequent branch should be migrated");
        assert!(
            template_contains(subsequent_template, &|component| matches!(
                component,
                TemplateComponent::Contributor(_)
            )),
            "subsequent override should preserve author short-form branch"
        );
        assert!(
            template_contains(subsequent_template, &|component| matches!(
                component,
                TemplateComponent::Date(_)
            )),
            "sibling choose fallback content should remain in the subsequent override"
        );

        let ibid_template = out
            .citation_overrides
            .ibid
            .as_ref()
            .expect("ibid branch should be migrated");
        assert!(
            template_contains(ibid_template, &|component| matches!(
                component,
                TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Locator
            )),
            "merged ibid override should preserve locator-aware content"
        );
        assert!(
            template_contains(ibid_template, &|component| matches!(
                component,
                TemplateComponent::Term(term)
                    if term.term == citum_schema::locale::GeneralTerm::Ibid
            )),
            "merged ibid override should still contain the ibid term"
        );
    }

    #[test]
    fn compile_from_xml_maps_mixed_note_position_tree_into_citation_overrides() {
        let legacy_style = parse_legacy_style(
            r#"
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <title>mixed-note-position-test</title>
    <id>https://example.org/mixed-note-position-test</id>
  </info>
  <citation>
    <layout>
      <choose>
        <if position="subsequent">
          <group delimiter=", ">
            <text variable="author"/>
            <choose>
              <if match="any" variable="archive archive-place container-title DOI number publisher references URL"/>
              <else-if position="first" type="interview">
                <date variable="issued">
                  <date-part name="year"/>
                </date>
              </else-if>
              <else-if position="first" type="personal_communication">
                <text variable="publisher"/>
              </else-if>
            </choose>
          </group>
        </if>
        <else>
          <text variable="title"/>
        </else>
      </choose>
    </layout>
  </citation>
</style>
"#,
        );

        let mut options = citum_schema::options::Config::default();
        let tracker = ProvenanceTracker::new(false);
        let out = compilation::compile_from_xml(&legacy_style, &mut options, false, &tracker);

        assert!(
            out.citation
                .iter()
                .any(|component| matches!(component, TemplateComponent::Title(_))),
            "base citation template should still contain the first-citation title"
        );
        assert!(
            out.citation_overrides.subsequent.is_some(),
            "mixed note trees should now emit a subsequent override"
        );
        assert!(
            out.citation_overrides
                .subsequent
                .as_ref()
                .is_some_and(|template| template
                    .iter()
                    .any(|component| matches!(component, TemplateComponent::Contributor(_)))),
            "subsequent override should preserve the shortened-note contributor content"
        );
    }

    #[test]
    fn compile_from_xml_truly_unsupported_mixed_position_tree_falls_back_without_overrides() {
        let legacy_style = parse_legacy_style(
            r#"
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <title>unsupported-mixed-position-test</title>
    <id>https://example.org/unsupported-mixed-position-test</id>
  </info>
  <citation>
    <layout>
      <choose>
        <if variable="title">
          <text variable="title"/>
        </if>
        <else-if position="subsequent">
          <text variable="author"/>
        </else-if>
        <else>
          <text variable="publisher"/>
        </else>
      </choose>
    </layout>
  </citation>
</style>
"#,
        );

        let mut options = citum_schema::options::Config::default();
        let tracker = ProvenanceTracker::new(false);
        let out = compilation::compile_from_xml(&legacy_style, &mut options, false, &tracker);

        assert!(
            !out.citation.is_empty(),
            "unsupported trees must still compile a base citation template"
        );
        assert!(
            out.citation_overrides.subsequent.is_none() && out.citation_overrides.ibid.is_none(),
            "unsupported trees should not emit partial position overrides"
        );
    }
}
