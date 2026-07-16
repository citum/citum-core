/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Unified compilation pipeline from CSL 1.0 to Citum templates.

use crate::{
    Compressor, MacroInliner, TemplateCompiler, Upsampler, analysis, base_detector, passes,
    provenance::ProvenanceTracker,
};
use citum_schema::{
    LocalizedTemplateSpec,
    template::{TemplateComponent, TypeSelector},
};
use std::collections::{BTreeSet, HashSet};

/// Shorthand for the per-type template map used throughout the migration pipeline.
pub type TypeTemplateMap = indexmap::IndexMap<TypeSelector, Vec<TemplateComponent>>;

/// Output from the XML compilation pipeline.
#[derive(Debug, Clone)]
pub struct XmlCompilationOutput {
    /// Base bibliography template.
    pub bibliography: Vec<TemplateComponent>,
    /// Ordered locale-specific bibliography templates, including the default branch.
    pub bibliography_locales: Option<Vec<LocalizedTemplateSpec>>,
    /// Type-specific bibliography variants.
    pub type_templates: Option<TypeTemplateMap>,
    /// Base citation template.
    pub citation: Vec<TemplateComponent>,
    /// Ordered locale-specific citation templates, including the default branch.
    pub citation_locales: Option<Vec<LocalizedTemplateSpec>>,
    /// Position-specific citation overrides.
    pub citation_overrides: CitationPositionOverrides,
    /// Whether citation position branches could not be migrated cleanly.
    pub unsupported_mixed_conditions: bool,
    /// Whether localized layouts contain shapes the Citum schema cannot preserve.
    pub unsupported_localized_layouts: bool,
}

/// Position-specific citation overrides.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CitationPositionOverrides {
    /// Subsequent citation override.
    pub subsequent: Option<Vec<TemplateComponent>>,
    /// Ibid citation override.
    pub ibid: Option<Vec<TemplateComponent>>,
}

/// Run the full XML layout-compilation pipeline for bibliography and citation templates.
///
/// Its output is a *transitional seed candidate* for the synthesis loop, not the template
/// authority. The synthesis default path (`synthesis::synthesize_citation` /
/// `synthesize_bibliography`) scores it against citeproc-js output and may select it, and it
/// also supplies type-variant templates and note-position overrides merged into the
/// inferred/synthesized result. It stays in-tree until the removal gate in bean `csl26-hxhx`
/// holds (the `xml` seed wins ≈0 selections); see
/// `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`. Fixing its output quality is valid work
/// until then. XML *attribute/options* extraction is permanent and separate from this.
pub fn compile_from_xml(
    legacy_style: &csl_legacy::model::Style,
    options: &mut citum_schema::options::Config,
    enable_provenance: bool,
    tracker: &ProvenanceTracker,
) -> XmlCompilationOutput {
    let mut output = compile_single_layouts(legacy_style, options, enable_provenance, tracker);
    let (bibliography_locales, citation_locales, unsupported_localized_layouts) =
        compile_localized_layouts(legacy_style, options, enable_provenance, tracker, &output);
    output.bibliography_locales = bibliography_locales;
    output.citation_locales = citation_locales;
    output.unsupported_localized_layouts = unsupported_localized_layouts;
    output
}

fn compile_single_layouts(
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
    let bib_ir = compressor.compress_nodes(raw_bib.clone());
    let cit_ir = compressor.compress_nodes(raw_cit.clone());

    // 4. Template Compilation
    let template_compiler = TemplateCompiler;

    // Detect if this is a numeric style
    let is_numeric = matches!(
        options.processing,
        Some(citum_schema::options::Processing::Numeric)
    );

    let (mut new_bib, mut type_templates) =
        template_compiler.compile_bibliography_with_types(&bib_ir, is_numeric);
    let is_note_class = legacy_style.class == "note";
    let mut new_cit = if is_note_class {
        template_compiler.compile_citation_note(&cit_ir)
    } else {
        template_compiler.compile_citation(&cit_ir)
    };

    let mut citation_position_overrides = CitationPositionOverrides {
        subsequent: compile_citation_position_override(
            &template_compiler,
            &compressor,
            citation_position_nodes.subsequent,
            is_note_class,
        ),
        ibid: compile_citation_position_override(
            &template_compiler,
            &compressor,
            citation_position_nodes.ibid,
            is_note_class,
        ),
    };

    record_template_placements_if_enabled(&new_bib, enable_provenance, tracker);
    // Apply author suffix extracted from original CSL (lost during macro inlining)
    analysis::bibliography::apply_author_suffix(&mut new_bib, author_suffix);

    // Apply bibliography-specific 'and' setting (may differ from citation)
    analysis::bibliography::apply_bibliography_and(&mut new_bib, bib_and);

    // For author-date styles with in-text class, apply standard formatting.
    let is_in_text_class = legacy_style.class == "in-text";
    let is_author_date_processing = options
        .processing
        .as_ref()
        .is_some_and(citum_schema::options::Processing::is_author_date_family);

    // Apply to all in-text styles (both author-date and numeric)
    if is_in_text_class {
        passes::reorder::add_volume_prefix_after_serial(&mut new_bib);
    }

    // Detect the narrow formatting family needed by migration fixups.
    let fixup_family = base_detector::detect_fixup_family(options);

    if is_in_text_class && is_author_date_processing {
        apply_author_date_bibliography_passes(&mut new_bib, options, fixup_family);
    }

    gate_leaked_in_terms(
        &mut new_bib,
        &mut new_cit,
        &mut type_templates,
        &mut citation_position_overrides,
    );

    let type_templates_opt = if type_templates.is_empty() {
        None
    } else {
        Some(type_templates)
    };

    XmlCompilationOutput {
        bibliography: new_bib,
        bibliography_locales: None,
        type_templates: type_templates_opt,
        citation: new_cit,
        citation_locales: None,
        citation_overrides: citation_position_overrides,
        unsupported_mixed_conditions,
        unsupported_localized_layouts: false,
    }
}

fn compile_localized_layouts(
    legacy_style: &csl_legacy::model::Style,
    options: &citum_schema::options::Config,
    enable_provenance: bool,
    tracker: &ProvenanceTracker,
    fallback: &XmlCompilationOutput,
) -> (
    Option<Vec<LocalizedTemplateSpec>>,
    Option<Vec<LocalizedTemplateSpec>>,
    bool,
) {
    let mut unsupported = false;
    let bibliography_locales = legacy_style.bibliography.as_ref().and_then(|bibliography| {
        (!bibliography.localized_layouts.is_empty()).then(|| {
            bibliography
                .localized_layouts
                .iter()
                .map(|localized| {
                    if localized.locales.is_empty() {
                        return localized_spec(localized, fallback.bibliography.clone());
                    }

                    let mut branch_style = legacy_style.clone();
                    if let Some(branch_bibliography) = branch_style.bibliography.as_mut() {
                        branch_bibliography.layout = localized.layout.clone();
                        branch_bibliography.localized_layouts.clear();
                    }
                    branch_style.citation.localized_layouts.clear();
                    let mut branch_options = options.clone();
                    let branch = compile_single_layouts(
                        &branch_style,
                        &mut branch_options,
                        enable_provenance,
                        tracker,
                    );
                    unsupported |=
                        !layout_metadata_matches(&localized.layout, &bibliography.layout)
                            || explicit_type_templates_differ(
                                legacy_style,
                                &localized.layout,
                                &bibliography.layout,
                                branch.type_templates.as_ref(),
                                fallback.type_templates.as_ref(),
                            );
                    localized_spec(localized, branch.bibliography)
                })
                .collect()
        })
    });

    let citation_locales = (!legacy_style.citation.localized_layouts.is_empty()).then(|| {
        legacy_style
            .citation
            .localized_layouts
            .iter()
            .map(|localized| {
                if localized.locales.is_empty() {
                    return localized_spec(localized, fallback.citation.clone());
                }

                let mut branch_style = legacy_style.clone();
                branch_style.citation.layout = localized.layout.clone();
                branch_style.citation.localized_layouts.clear();
                if let Some(branch_bibliography) = branch_style.bibliography.as_mut() {
                    branch_bibliography.localized_layouts.clear();
                }
                let mut branch_options = options.clone();
                let branch = compile_single_layouts(
                    &branch_style,
                    &mut branch_options,
                    enable_provenance,
                    tracker,
                );
                unsupported |=
                    !layout_metadata_matches(&localized.layout, &legacy_style.citation.layout)
                        || branch.citation_overrides != fallback.citation_overrides
                        || branch.unsupported_mixed_conditions;
                localized_spec(localized, branch.citation)
            })
            .collect()
    });

    (bibliography_locales, citation_locales, unsupported)
}

fn localized_spec(
    localized: &csl_legacy::model::LocalizedLayout,
    template: Vec<TemplateComponent>,
) -> LocalizedTemplateSpec {
    let is_default = localized.locales.is_empty();
    LocalizedTemplateSpec {
        locale: (!is_default).then(|| localized.locales.clone()),
        default: is_default.then_some(true),
        template,
        unknown_fields: std::collections::BTreeMap::new(),
    }
}

fn layout_metadata_matches(
    left: &csl_legacy::model::Layout,
    right: &csl_legacy::model::Layout,
) -> bool {
    left.prefix == right.prefix && left.suffix == right.suffix && left.delimiter == right.delimiter
}

fn explicit_type_templates_differ(
    style: &csl_legacy::model::Style,
    localized: &csl_legacy::model::Layout,
    fallback: &csl_legacy::model::Layout,
    localized_templates: Option<&TypeTemplateMap>,
    fallback_templates: Option<&TypeTemplateMap>,
) -> bool {
    let mut types = explicit_layout_types(style, localized);
    types.extend(explicit_layout_types(style, fallback));
    types.into_iter().any(|item_type| {
        matching_type_template(localized_templates, &item_type)
            != matching_type_template(fallback_templates, &item_type)
    })
}

fn matching_type_template<'a>(
    templates: Option<&'a TypeTemplateMap>,
    item_type: &str,
) -> Option<&'a Vec<TemplateComponent>> {
    templates.and_then(|templates| {
        templates
            .iter()
            .find_map(|(selector, template)| selector.matches(item_type).then_some(template))
    })
}

fn explicit_layout_types(
    style: &csl_legacy::model::Style,
    layout: &csl_legacy::model::Layout,
) -> BTreeSet<String> {
    let mut types = BTreeSet::new();
    let mut visited_macros = HashSet::new();
    collect_explicit_types(style, &layout.children, &mut visited_macros, &mut types);
    types
}

fn collect_explicit_types(
    style: &csl_legacy::model::Style,
    nodes: &[csl_legacy::model::CslNode],
    visited_macros: &mut HashSet<String>,
    types: &mut BTreeSet<String>,
) {
    use csl_legacy::model::CslNode;

    for node in nodes {
        match node {
            CslNode::Text(text) => {
                if let Some(macro_name) = text.macro_name.as_ref()
                    && visited_macros.insert(macro_name.clone())
                    && let Some(macro_definition) = style
                        .macros
                        .iter()
                        .find(|candidate| candidate.name == *macro_name)
                {
                    collect_explicit_types(
                        style,
                        &macro_definition.children,
                        visited_macros,
                        types,
                    );
                }
            }
            CslNode::Group(group) => {
                collect_explicit_types(style, &group.children, visited_macros, types);
            }
            CslNode::Names(names) => {
                collect_explicit_types(style, &names.children, visited_macros, types);
            }
            CslNode::Substitute(substitute) => {
                collect_explicit_types(style, &substitute.children, visited_macros, types);
            }
            CslNode::Choose(choose) => {
                for branch in
                    std::iter::once(&choose.if_branch).chain(choose.else_if_branches.iter())
                {
                    if let Some(type_names) = branch.type_.as_deref() {
                        types.extend(type_names.split_whitespace().map(str::to_owned));
                    }
                    collect_explicit_types(style, &branch.children, visited_macros, types);
                }
                if let Some(else_branch) = choose.else_branch.as_deref() {
                    collect_explicit_types(style, else_branch, visited_macros, types);
                }
            }
            CslNode::Date(_)
            | CslNode::Label(_)
            | CslNode::Number(_)
            | CslNode::Name(_)
            | CslNode::EtAl(_) => {}
        }
    }
}

/// Re-bind or drop the leaked root-level `in` term that lost its enclosing CSL
/// group during template specialization, restoring the engine's term-only
/// group suppression across every compiled template.
fn gate_leaked_in_terms(
    new_bib: &mut Vec<TemplateComponent>,
    new_cit: &mut Vec<TemplateComponent>,
    type_templates: &mut TypeTemplateMap,
    overrides: &mut CitationPositionOverrides,
) {
    crate::fixups::gate_leaked_in_term(new_bib);
    crate::fixups::gate_leaked_in_term(new_cit);
    for template in type_templates.values_mut() {
        crate::fixups::gate_leaked_in_term(template);
    }
    if let Some(template) = overrides.subsequent.as_mut() {
        crate::fixups::gate_leaked_in_term(template);
    }
    if let Some(template) = overrides.ibid.as_mut() {
        crate::fixups::gate_leaked_in_term(template);
    }
}

fn compile_citation_position_override(
    compiler: &TemplateCompiler,
    compressor: &Compressor,
    nodes: Option<Vec<crate::ir::Node>>,
    is_note_class: bool,
) -> Option<Vec<TemplateComponent>> {
    let nodes = nodes?;
    let compressed = compressor.compress_nodes(nodes);
    let compiled = if is_note_class {
        compiler.compile_citation_note(&compressed)
    } else {
        compiler.compile_citation(&compressed)
    };
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
