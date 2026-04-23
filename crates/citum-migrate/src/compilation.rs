/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Unified compilation pipeline from CSL 1.0 to Citum templates.

use crate::{
    Compressor, MacroInliner, TemplateCompiler, Upsampler, analysis, base_detector, passes,
    provenance::ProvenanceTracker,
};
use citum_schema::template::{TemplateComponent, TypeSelector};

/// Shorthand for the per-type template map used throughout the migration pipeline.
pub type TypeTemplateMap = indexmap::IndexMap<TypeSelector, Vec<TemplateComponent>>;

/// Output from the XML compilation pipeline.
#[derive(Debug, Clone)]
pub struct XmlCompilationOutput {
    /// Base bibliography template.
    pub bibliography: Vec<TemplateComponent>,
    /// Type-specific bibliography variants.
    pub type_templates: Option<TypeTemplateMap>,
    /// Base citation template.
    pub citation: Vec<TemplateComponent>,
    /// Position-specific citation overrides.
    pub citation_overrides: CitationPositionOverrides,
    /// Whether citation position branches could not be migrated cleanly.
    pub unsupported_mixed_conditions: bool,
}

/// Position-specific citation overrides.
#[derive(Debug, Clone, Default)]
pub struct CitationPositionOverrides {
    /// Subsequent citation override.
    pub subsequent: Option<Vec<TemplateComponent>>,
    /// Ibid citation override.
    pub ibid: Option<Vec<TemplateComponent>>,
}

/// Run the full XML compilation pipeline for bibliography and citation templates.
/// This is the fallback when no hand-authored or inferred template is available.
pub fn compile_from_xml(
    legacy_style: &csl_legacy::model::Style,
    options: &mut citum_schema::options::Config,
    enable_provenance: bool,
    tracker: &ProvenanceTracker,
) -> XmlCompilationOutput {
    // Extract author suffix before macro inlining (will be lost during inlining)
    let author_suffix = if let Some(ref bib) = legacy_style.bibliography {
        analysis::bibliography::extract_author_suffix(&bib.layout)
    } else {
        None
    };

    // Extract bibliography-specific 'and' setting (may differ from citation)
    let bib_and = analysis::bibliography::extract_bibliography_and(legacy_style);

    // 1. Deconstruction
    let inliner = if enable_provenance {
        MacroInliner::with_provenance(legacy_style, tracker.clone())
    } else {
        MacroInliner::new(legacy_style)
    };
    let flattened_bib = inliner
        .inline_bibliography(legacy_style)
        .unwrap_or_default();
    let flattened_cit = inliner.inline_citation(legacy_style);

    // 2. Semantic Upsampling
    let mut upsampler = if enable_provenance {
        Upsampler::with_provenance(tracker.clone())
    } else {
        Upsampler::new()
    };

    // Set citation-specific thresholds for citation upsampling
    upsampler.et_al_min = legacy_style.citation.et_al_min;
    upsampler.et_al_use_first = legacy_style.citation.et_al_use_first;
    let citation_position_nodes = upsampler.extract_citation_position_templates(&flattened_cit);
    let unsupported_mixed_conditions = citation_position_nodes.unsupported_mixed_conditions;

    // We don't print to stderr here to keep the library quiet,
    // but the flag is available if the caller wants it.

    let raw_cit = citation_position_nodes
        .first
        .clone()
        .unwrap_or_else(|| upsampler.upsample_nodes(&flattened_cit));

    // Set bibliography-specific thresholds for bibliography upsampling
    if let Some(ref bib) = legacy_style.bibliography {
        upsampler.et_al_min = bib.et_al_min;
        upsampler.et_al_use_first = bib.et_al_use_first;
    }
    let raw_bib = upsampler.upsample_nodes(&flattened_bib);

    // 3. Compression (Pattern Recognition)
    let compressor = Compressor;
    let csln_bib = compressor.compress_nodes(raw_bib.clone());
    let csln_cit = compressor.compress_nodes(raw_cit.clone());

    // 4. Template Compilation
    let template_compiler = TemplateCompiler;

    // Detect if this is a numeric style
    let is_numeric = matches!(
        options.processing,
        Some(citum_schema::options::Processing::Numeric)
    );

    let (mut new_bib, type_templates) =
        template_compiler.compile_bibliography_with_types(&csln_bib, is_numeric);
    let new_cit = template_compiler.compile_citation(&csln_cit);

    let citation_position_overrides = CitationPositionOverrides {
        subsequent: compile_citation_position_override(
            &template_compiler,
            &compressor,
            citation_position_nodes.subsequent,
        ),
        ibid: compile_citation_position_override(
            &template_compiler,
            &compressor,
            citation_position_nodes.ibid,
        ),
    };

    record_template_placements_if_enabled(&new_bib, enable_provenance, tracker);
    // Apply author suffix extracted from original CSL (lost during macro inlining)
    analysis::bibliography::apply_author_suffix(&mut new_bib, author_suffix);

    // Apply bibliography-specific 'and' setting (may differ from citation)
    analysis::bibliography::apply_bibliography_and(&mut new_bib, bib_and);

    // For author-date styles with in-text class, apply standard formatting.
    let is_in_text_class = legacy_style.class == "in-text";
    let is_author_date_processing = matches!(
        options.processing,
        Some(citum_schema::options::Processing::AuthorDate)
    );

    // Apply to all in-text styles (both author-date and numeric)
    if is_in_text_class {
        passes::reorder::add_volume_prefix_after_serial(&mut new_bib);
    }

    // Detect the narrow formatting family needed by migration fixups.
    let fixup_family = base_detector::detect_fixup_family(options);

    if is_in_text_class && is_author_date_processing {
        apply_author_date_bibliography_passes(&mut new_bib, options, fixup_family);
    }

    let type_templates_opt = if type_templates.is_empty() {
        None
    } else {
        Some(type_templates)
    };

    XmlCompilationOutput {
        bibliography: new_bib,
        type_templates: type_templates_opt,
        citation: new_cit,
        citation_overrides: citation_position_overrides,
        unsupported_mixed_conditions,
    }
}

fn compile_citation_position_override(
    compiler: &TemplateCompiler,
    compressor: &Compressor,
    nodes: Option<Vec<citum_schema::CslnNode>>,
) -> Option<Vec<TemplateComponent>> {
    let nodes = nodes?;
    let compressed = compressor.compress_nodes(nodes);
    let compiled = compiler.compile_citation(&compressed);
    if compiled.is_empty() {
        None
    } else {
        Some(compiled)
    }
}

fn record_template_placements_if_enabled(
    new_bib: &[TemplateComponent],
    enable_provenance: bool,
    tracker: &ProvenanceTracker,
) {
    if !enable_provenance {
        return;
    }
    for (index, component) in new_bib.iter().enumerate() {
        match component {
            TemplateComponent::Variable(v) => {
                let var_name = format!("{:?}", v.variable).to_lowercase();
                tracker.record_template_placement(
                    &var_name,
                    index,
                    "bibliography.template",
                    "Variable",
                );
            }
            TemplateComponent::Number(n) => {
                let var_name = format!("{:?}", n.number).to_lowercase();
                tracker.record_template_placement(
                    &var_name,
                    index,
                    "bibliography.template",
                    "Number",
                );
            }
            TemplateComponent::Date(d) => {
                let var_name = format!("{:?}", d.date).to_lowercase();
                tracker.record_template_placement(
                    &var_name,
                    index,
                    "bibliography.template",
                    "Date",
                );
            }
            TemplateComponent::Title(t) => {
                let var_name = format!("{:?}", t.title).to_lowercase();
                tracker.record_template_placement(
                    &var_name,
                    index,
                    "bibliography.template",
                    "Title",
                );
            }
            TemplateComponent::Contributor(_) => {
                tracker.record_template_placement(
                    "contributor",
                    index,
                    "bibliography.template",
                    "Contributor",
                );
            }
            _ => {}
        }
    }
}

fn apply_author_date_bibliography_passes(
    new_bib: &mut Vec<TemplateComponent>,
    options: &mut citum_schema::options::Config,
    fixup_family: Option<base_detector::FixupFamily>,
) {
    // Detect if the style uses space prefix for volume (Elsevier pattern)
    let volume_list_has_space_prefix = new_bib.iter().any(|c| {
        if let TemplateComponent::Group(list) = c {
            let has_volume = list.group.iter().any(|item| {
                matches!(item, TemplateComponent::Number(n) if n.number == citum_schema::template::NumberVariable::Volume)
            });
            if has_volume {
                return list.rendering.prefix.as_deref() == Some(" ");
            }
        }
        false
    });

    let vol_pages_delim = options.volume_pages_delimiter.clone();
    for component in &mut *new_bib {
        apply_type_overrides(
            component,
            vol_pages_delim.clone(),
            volume_list_has_space_prefix,
            fixup_family,
        );
    }

    passes::reorder::move_access_components_to_end(new_bib);
    passes::reorder::unsuppress_for_type(new_bib, "chapter");
    passes::reorder::unsuppress_for_type(new_bib, "paper-conference");
    passes::reorder::unsuppress_for_type(new_bib, "thesis");
    passes::reorder::unsuppress_for_type(new_bib, "document");
    passes::deduplicate::deduplicate_titles_in_lists(new_bib);
    passes::deduplicate::deduplicate_variables_cross_lists(new_bib);
    passes::reorder::propagate_list_overrides(new_bib);
    passes::deduplicate::deduplicate_nested_lists(new_bib);
    passes::reorder::reorder_serial_components(new_bib);
    passes::grouping::group_volume_and_issue(new_bib, options, fixup_family);
    passes::reorder::reorder_pages_for_serials(new_bib);
    passes::reorder::reorder_publisher_place_for_chicago(new_bib, fixup_family);
    passes::reorder::reorder_chapters_for_apa(new_bib, fixup_family);
    passes::reorder::reorder_chapters_for_chicago(new_bib, fixup_family);
    passes::deduplicate::suppress_duplicate_issue_for_journals(new_bib, fixup_family);
}

#[allow(
    clippy::only_used_in_recursion,
    reason = "params propagated for future type-override logic"
)]
fn apply_type_overrides(
    component: &mut TemplateComponent,
    volume_pages_delimiter: Option<citum_schema::template::DelimiterPunctuation>,
    volume_list_has_space_prefix: bool,
    fixup_family: Option<base_detector::FixupFamily>,
) {
    if let TemplateComponent::Group(list) = component {
        for item in &mut list.group {
            apply_type_overrides(
                item,
                volume_pages_delimiter.clone(),
                volume_list_has_space_prefix,
                fixup_family,
            );
        }
    }
}
