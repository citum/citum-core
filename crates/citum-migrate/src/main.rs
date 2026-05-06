#![allow(missing_docs, reason = "bin")]

use citum_migrate::{
    OptionsExtractor, analysis,
    compilation::{self, XmlCompilationOutput as XmlFallback},
    debug_output::DebugOutputFormatter,
    fixups::{
        citation_template_has_citation_number, citation_template_is_author_year_only,
        ensure_inferred_media_type_templates, ensure_inferred_patent_type_template,
        ensure_numeric_locator_citation_component, ensure_personal_communication_omitted,
        move_group_wrap_to_citation_items, normalize_author_date_inferred_contributors,
        normalize_author_date_locator_citation_component, normalize_contributor_form_to_short,
        normalize_legal_case_type_template, normalize_wrapped_numeric_locator_citation_component,
        note_citation_template_is_underfit, scrub_inferred_literal_artifacts,
        should_merge_inferred_type_template,
    },
    lineage::StyleLineage,
    options_extractor::MigrationOptions,
    provenance::ProvenanceTracker,
    template_resolver,
};
use citum_schema::{
    BibliographySpec, CitationCollapse, CitationSpec, Style, StyleInfo,
    template::{
        Rendering, TemplateAddOperation, TemplateComponent, TemplateComponentSelector,
        TemplateModifyOperation, TemplateRemoveOperation, TemplateVariant, TemplateVariantDiff,
        TypeSelector, WrapPunctuation,
    },
};
use csl_legacy::parser::parse_style;
use roxmltree::Document;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Shorthand for the per-type template map used throughout the migration pipeline.
type TypeTemplateMap = indexmap::IndexMap<TypeSelector, Vec<TemplateComponent>>;

type TypeVariantMap = indexmap::IndexMap<TypeSelector, TemplateVariant>;

fn build_type_variants(
    default_template: &[TemplateComponent],
    type_templates: TypeTemplateMap,
) -> TypeVariantMap {
    let mut variants = TypeVariantMap::new();
    let mut candidate_parents: Vec<(TypeSelector, Vec<TemplateComponent>)> = Vec::new();

    for (selector, template) in type_templates {
        let variant = template_variant_from_full_template(
            default_template,
            &candidate_parents,
            &selector,
            template.clone(),
        );
        variants.insert(selector.clone(), variant);
        candidate_parents.push((selector, template));
    }

    variants
}

fn template_variant_from_full_template(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    selector: &TypeSelector,
    target_template: Vec<TemplateComponent>,
) -> TemplateVariant {
    let Some(diff) =
        derive_best_template_variant_diff(default_template, candidate_parents, &target_template)
    else {
        return TemplateVariant::Full(target_template);
    };

    if diff_resolves_to_template(
        default_template,
        candidate_parents,
        selector,
        &diff,
        &target_template,
    ) {
        TemplateVariant::Diff(diff)
    } else {
        TemplateVariant::Full(target_template)
    }
}

fn derive_best_template_variant_diff(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    target_template: &[TemplateComponent],
) -> Option<TemplateVariantDiff> {
    let mut best_diff = derive_template_variant_diff(default_template, target_template);
    let mut best_weight = best_diff
        .as_ref()
        .map(diff_operation_weight)
        .unwrap_or(usize::MAX);

    for (parent_selector, parent_template) in candidate_parents {
        let Some(mut parent_diff) = derive_template_variant_diff(parent_template, target_template)
        else {
            continue;
        };
        let weight = diff_operation_weight(&parent_diff);
        if weight >= best_weight {
            continue;
        }
        parent_diff.extends = Some(parent_selector.clone());
        best_diff = Some(parent_diff);
        best_weight = weight;
    }

    best_diff
}

fn diff_operation_weight(diff: &TemplateVariantDiff) -> usize {
    diff.modify.len() + diff.remove.len() + diff.add.len()
}

fn diff_resolves_to_template(
    default_template: &[TemplateComponent],
    candidate_parents: &[(TypeSelector, Vec<TemplateComponent>)],
    selector: &TypeSelector,
    diff: &TemplateVariantDiff,
    expected_template: &[TemplateComponent],
) -> bool {
    let mut variants = TypeVariantMap::new();
    for (parent_selector, parent_template) in candidate_parents {
        variants.insert(
            parent_selector.clone(),
            TemplateVariant::Full(parent_template.clone()),
        );
    }
    variants.insert(selector.clone(), TemplateVariant::Diff(diff.clone()));
    let style = Style {
        bibliography: Some(BibliographySpec {
            template: Some(default_template.to_vec()),
            type_variants: Some(variants),
            ..Default::default()
        }),
        ..Default::default()
    };

    style
        .try_into_resolved()
        .ok()
        .and_then(|style| style.bibliography)
        .and_then(|bibliography| bibliography.type_variants)
        .and_then(|variants| variants.get(selector).cloned())
        .and_then(TemplateVariant::into_template)
        .is_some_and(|resolved| resolved == expected_template)
}

fn derive_template_variant_diff(
    default_template: &[TemplateComponent],
    target_template: &[TemplateComponent],
) -> Option<TemplateVariantDiff> {
    if default_template.is_empty() {
        return None;
    }

    let default_keys = component_keys(default_template)?;
    let target_keys = component_keys(target_template)?;
    let common_pairs = lcs_pairs(&default_keys, &target_keys);
    let mut diff = TemplateVariantDiff::default();

    for (index, component) in default_template.iter().enumerate() {
        if !common_pairs
            .iter()
            .any(|(base_index, _)| *base_index == index)
        {
            diff.remove.push(TemplateRemoveOperation {
                match_selector: component_selector(component)?,
            });
        }
    }

    for (base_index, target_index) in &common_pairs {
        let base_component = default_template.get(*base_index)?;
        let target_component = target_template.get(*target_index)?;
        if base_component != target_component {
            if !is_rendering_only_change(base_component, target_component) {
                return None;
            }
            diff.modify.push(TemplateModifyOperation {
                match_selector: component_selector(base_component)?,
                label_form: modified_number_label_form(base_component, target_component),
                rendering: target_component.rendering().clone(),
            });
        }
    }

    let mut last_anchor: Option<TemplateComponentSelector> = None;
    for (target_index, component) in target_template.iter().enumerate() {
        if let Some((base_index, _)) = common_pairs
            .iter()
            .find(|(_, common_target_index)| *common_target_index == target_index)
        {
            last_anchor = default_template
                .get(*base_index)
                .and_then(component_selector);
            continue;
        }

        let next_anchor = common_pairs
            .iter()
            .find(|(_, common_target_index)| *common_target_index > target_index)
            .and_then(|(base_index, _)| default_template.get(*base_index))
            .and_then(component_selector);

        let component_selector = component_selector(component)?;
        let add = if let Some(before) = next_anchor {
            TemplateAddOperation {
                before: Some(before),
                after: None,
                component: component.clone(),
            }
        } else if let Some(after) = last_anchor.clone() {
            TemplateAddOperation {
                before: None,
                after: Some(after),
                component: component.clone(),
            }
        } else {
            return None;
        };
        diff.add.push(add);
        last_anchor = Some(component_selector);
    }

    Some(diff)
}

fn component_keys(template: &[TemplateComponent]) -> Option<Vec<String>> {
    template
        .iter()
        .map(|component| serde_json::to_string(&component_selector(component)?.fields).ok())
        .collect()
}

fn component_selector(component: &TemplateComponent) -> Option<TemplateComponentSelector> {
    let serde_json::Value::Object(fields) = serde_json::to_value(component).ok()? else {
        return None;
    };
    for key in [
        "contributor",
        "date",
        "title",
        "number",
        "variable",
        "term",
        "group",
    ] {
        if let Some(value) = fields.get(key) {
            let mut selector = BTreeMap::new();
            selector.insert(key.to_string(), value.clone());
            return Some(TemplateComponentSelector { fields: selector });
        }
    }
    None
}

fn modified_number_label_form(
    base: &TemplateComponent,
    target: &TemplateComponent,
) -> Option<citum_schema::template::LabelForm> {
    match (base, target) {
        (TemplateComponent::Number(base_number), TemplateComponent::Number(target_number))
            if base_number.label_form != target_number.label_form =>
        {
            target_number.label_form.clone()
        }
        _ => None,
    }
}

fn is_rendering_only_change(base: &TemplateComponent, target: &TemplateComponent) -> bool {
    let mut normalized_base = base.clone();
    let mut normalized_target = target.clone();
    *normalized_base.rendering_mut() = Rendering::default();
    *normalized_target.rendering_mut() = Rendering::default();

    match (&mut normalized_base, &mut normalized_target) {
        (TemplateComponent::Number(base_number), TemplateComponent::Number(target_number))
            if base_number.label_form != target_number.label_form =>
        {
            if target_number.label_form.is_none() {
                return false;
            }
            base_number.label_form = None;
            target_number.label_form = None;
        }
        _ => {}
    }

    normalized_base == normalized_target
}

fn lcs_pairs(left: &[String], right: &[String]) -> Vec<(usize, usize)> {
    let mut lengths = vec![vec![0usize; right.len() + 1]; left.len() + 1];

    for i in (0..left.len()).rev() {
        for j in (0..right.len()).rev() {
            let diagonal = lengths
                .get(i + 1)
                .and_then(|row| row.get(j + 1))
                .copied()
                .unwrap_or(0);
            let down = lengths
                .get(i + 1)
                .and_then(|row| row.get(j))
                .copied()
                .unwrap_or(0);
            let right_value = lengths
                .get(i)
                .and_then(|row| row.get(j + 1))
                .copied()
                .unwrap_or(0);
            let value = if left.get(i) == right.get(j) {
                diagonal + 1
            } else {
                down.max(right_value)
            };
            if let Some(cell) = lengths.get_mut(i).and_then(|row| row.get_mut(j)) {
                *cell = value;
            }
        }
    }

    let mut pairs = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < left.len() && j < right.len() {
        if left.get(i) == right.get(j) {
            pairs.push((i, j));
            i += 1;
            j += 1;
            continue;
        }

        let down = lengths
            .get(i + 1)
            .and_then(|row| row.get(j))
            .copied()
            .unwrap_or(0);
        let right_value = lengths
            .get(i)
            .and_then(|row| row.get(j + 1))
            .copied()
            .unwrap_or(0);
        if down >= right_value {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}

struct CliArgs {
    path: String,
    debug_variable: Option<String>,
    template_mode: template_resolver::TemplateMode,
    live_infer_backend: template_resolver::LiveInferBackend,
    template_dir: Option<PathBuf>,
    min_template_confidence: f64,
}

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

fn parse_template_mode_arg(value: &str) -> template_resolver::TemplateMode {
    match value.parse::<template_resolver::TemplateMode>() {
        Ok(mode) => mode,
        Err(msg) => {
            eprintln!("Error: {msg}");
            std::process::exit(1);
        }
    }
}

fn parse_live_infer_backend_arg(value: &str) -> template_resolver::LiveInferBackend {
    match value.parse::<template_resolver::LiveInferBackend>() {
        Ok(backend) => backend,
        Err(msg) => {
            eprintln!("Error: {msg}");
            std::process::exit(1);
        }
    }
}

fn parse_min_template_confidence_arg(value: &str) -> f64 {
    match value.parse::<f64>() {
        Ok(parsed) if (0.0..=1.0).contains(&parsed) => parsed,
        _ => {
            eprintln!("Error: --min-template-confidence requires a number in [0.0, 1.0]");
            std::process::exit(1);
        }
    }
}

fn parse_cli_args(args: &[String]) -> CliArgs {
    let program_name = args
        .first()
        .and_then(|arg| std::path::Path::new(arg).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("citum-migrate");

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help(program_name);
        std::process::exit(0);
    }

    let mut path = "styles-legacy/apa.csl".to_string();
    let mut debug_variable: Option<String> = None;
    let mut template_mode = template_resolver::TemplateMode::Auto;
    let mut live_infer_backend = template_resolver::LiveInferBackend::Auto;
    let mut template_dir: Option<PathBuf> = None;
    let mut min_template_confidence = 0.70_f64;

    let mut iter = args.iter().skip(1).peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--debug-variable" => {
                if let Some(val) = iter.next() {
                    debug_variable = Some(val.clone());
                } else {
                    eprintln!("Error: --debug-variable requires an argument");
                    std::process::exit(1);
                }
            }
            "--template-source" => {
                if let Some(val) = iter.next() {
                    template_mode = parse_template_mode_arg(val);
                } else {
                    eprintln!(
                        "Error: --template-source requires an argument (auto|hand|inferred|xml)"
                    );
                    std::process::exit(1);
                }
            }
            "--live-infer-backend" => {
                if let Some(val) = iter.next() {
                    live_infer_backend = parse_live_infer_backend_arg(val);
                } else {
                    eprintln!(
                        "Error: --live-infer-backend requires an argument (auto|embedded|node)"
                    );
                    std::process::exit(1);
                }
            }
            "--min-template-confidence" => {
                if let Some(val) = iter.next() {
                    min_template_confidence = parse_min_template_confidence_arg(val);
                } else {
                    eprintln!("Error: --min-template-confidence requires a numeric argument");
                    std::process::exit(1);
                }
            }
            "--template-dir" => {
                if let Some(val) = iter.next() {
                    template_dir = Some(PathBuf::from(val));
                } else {
                    eprintln!("Error: --template-dir requires a path argument");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with('-') => {
                path = arg.to_string();
            }
            _ => {
                eprintln!("Error: unknown argument '{}'", arg);
                eprintln!();
                print_help(program_name);
                std::process::exit(1);
            }
        }
    }

    CliArgs {
        path,
        debug_variable,
        template_mode,
        live_infer_backend,
        template_dir,
        min_template_confidence,
    }
}

/// Extracts style name from path and resolves templates.
fn resolve_style_name_and_templates(
    path: &str,
    cli: &CliArgs,
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
    if style_path.is_absolute() {
        style_path
            .ancestors()
            .find(|p| p.join("Cargo.toml").exists())
            .unwrap_or(style_path.parent().unwrap_or(Path::new(".")))
            .to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

fn extract_citation_collapse(citation: &csl_legacy::model::Citation) -> Option<CitationCollapse> {
    match citation.collapse.as_deref() {
        Some("citation-number") => Some(CitationCollapse::CitationNumber),
        _ => None,
    }
}

/// Assemble the final Citum Style from compiled output and legacy metadata.
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

    // [PRUNING] Remove bibliography type-variants that are identical to the primary template.
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
    let args: Vec<String> = std::env::args().collect();
    let cli = parse_cli_args(&args);
    let path = &cli.path;

    // Initialize provenance tracking if debug variable is specified
    let enable_provenance = cli.debug_variable.is_some();
    let tracker = ProvenanceTracker::new(enable_provenance);
    let workspace_root = workspace_root_for_style_path(path);
    let lineage = StyleLineage::resolve(path, &workspace_root)?;

    eprintln!("Migrating {path} to Citum...");
    eprintln!(
        "Resolved lineage: semantic={:?}, form={:?}, parent={}",
        lineage.semantic_class,
        lineage.implementation_form,
        lineage.parent_style_id.as_deref().unwrap_or("none")
    );

    let text = fs::read_to_string(path)?;
    let doc = Document::parse(&text)?;
    let legacy_style = parse_style(doc.root_element())?;

    // 0. Extract global options (new Citum Config)
    let MigrationOptions {
        mut options,
        mut bibliography_options,
        citation_contributor_overrides,
        bibliography_contributor_overrides,
        citation_has_scope_shorten,
    } = OptionsExtractor::extract_migration_options(&legacy_style);

    // Resolve template: try hand-authored, cached inferred, or live inference
    // before falling back to the XML compiler pipeline.
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
        eprintln!(
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

    // 5. Build Style in correct format for citum_engine
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
    let style = lineage.apply_to_migrated_style(standalone_style)?;

    output_style_and_debug(&style, cli.debug_variable.as_deref(), &tracker)?;
    Ok(())
}

fn output_style_and_debug(
    style: &Style,
    debug_variable: Option<&str>,
    tracker: &ProvenanceTracker,
) -> Result<(), Box<dyn std::error::Error>> {
    // Output YAML to stdout
    let yaml = serde_yaml::to_string(style)?;
    println!("{yaml}");

    // Output debug information if requested
    if let Some(var_name) = debug_variable {
        eprintln!("\n");
        eprintln!("=== PROVENANCE DEBUG ===\n");
        let debug_output = DebugOutputFormatter::format_variable(tracker, var_name);
        eprint!("{debug_output}");
    }

    Ok(())
}

fn print_help(program_name: &str) {
    eprintln!("Citum style migration tool");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  {program_name} [STYLE.csl] [options]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  STYLE.csl                       Input CSL 1.0 style path");
    eprintln!("                                  (default: styles-legacy/apa.csl)");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help                      Show this help text");
    eprintln!("  --debug-variable <name>         Print provenance details for one variable");
    eprintln!("  --template-source <mode>        Template source: auto|hand|inferred|xml");
    eprintln!("  --live-infer-backend <mode>     Live inference backend: auto|embedded|node");
    eprintln!("  --template-dir <path>           Override directory for hand-authored templates");
    eprintln!("  --min-template-confidence <n>   Minimum inferred confidence [0.0, 1.0]");
}

fn resolve_migrated_bibliography_sort(
    processing: Option<&citum_schema::options::Processing>,
    legacy_sort: Option<&csl_legacy::model::Sort>,
) -> Option<citum_schema::grouping::GroupSortEntry> {
    let extracted = legacy_sort.and_then(
        citum_migrate::options_extractor::bibliography::extract_group_sort_from_bibliography,
    )?;

    if bibliography_sort_matches_processing_default(processing, &extracted) {
        None
    } else {
        Some(citum_schema::grouping::GroupSortEntry::Explicit(extracted))
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

fn log_template_sources(resolved: &template_resolver::ResolvedTemplates) {
    if let Some(ref resolved_bib) = resolved.bibliography {
        eprintln!("Using {} bibliography template", resolved_bib.source);
        if let Some(conf) = resolved_bib.confidence {
            eprintln!("  bibliography confidence: {:.0}%", conf * 100.0);
        }
    } else {
        eprintln!(
            "Using {} bibliography template",
            template_resolver::TemplateSource::XmlCompiled
        );
    }

    if let Some(ref resolved_cit) = resolved.citation {
        eprintln!("Using {} citation template", resolved_cit.source);
        if let Some(conf) = resolved_cit.confidence {
            eprintln!("  citation confidence: {:.0}%", conf * 100.0);
        }
    } else {
        eprintln!(
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
            let inferred_bib = matches!(
                resolved_bib.source,
                template_resolver::TemplateSource::InferredCached(_)
                    | template_resolver::TemplateSource::InferredLive
            );

            let merged_type_templates = if inferred_bib {
                xml_fallback
                    .as_ref()
                    .and_then(|out| out.type_templates.clone())
                    .map(|type_templates| {
                        type_templates
                            .into_iter()
                            .filter(|(selector, type_template)| {
                                selector.type_names().iter().any(|type_name| {
                                    should_merge_inferred_type_template(
                                        type_name,
                                        &resolved_bib.template,
                                        type_template,
                                    )
                                })
                            })
                            .collect::<indexmap::IndexMap<_, _>>()
                    })
                    .filter(|m| !m.is_empty())
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

fn override_bibliography_options_if_inferred(
    resolved: &template_resolver::ResolvedTemplates,
    legacy_style: &csl_legacy::model::Style,
    options: &mut Option<citum_schema::BibliographyOptions>,
) {
    if let Some(ref resolved_bib) = resolved.bibliography {
        let is_inferred_source = matches!(
            resolved_bib.source,
            template_resolver::TemplateSource::InferredCached(_)
                | template_resolver::TemplateSource::InferredLive
        );
        let allow_bib_punctuation_override = !(legacy_style.class == "note" && is_inferred_source);

        if allow_bib_punctuation_override {
            if let Some(ref delim) = resolved_bib.delimiter {
                eprintln!("  Overriding bibliography separator: {delim:?}");
                let bib_cfg = options.get_or_insert_with(Default::default);
                bib_cfg.separator = Some(delim.clone());
            }

            if let Some(ref suffix) = resolved_bib.entry_suffix {
                eprintln!("  Overriding bibliography entry suffix: {suffix:?}");
                let bib_cfg = options.get_or_insert_with(Default::default);
                bib_cfg.entry_suffix = Some(suffix.clone());
            }
        } else {
            eprintln!(
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

fn postprocess_inferred_bibliography(
    new_bib: &mut Vec<TemplateComponent>,
    type_templates: &mut Option<TypeTemplateMap>,
    legacy_style: &csl_legacy::model::Style,
) {
    for component in &mut *new_bib {
        scrub_inferred_literal_artifacts(component);
    }
    relax_inferred_bibliography_date_suppression(new_bib);
    if let Some(type_templates) = type_templates.as_mut() {
        for template in type_templates.values_mut() {
            for component in template.iter_mut() {
                scrub_inferred_literal_artifacts(component);
            }
            relax_inferred_bibliography_date_suppression(template);
        }
        repair_inferred_bibliography_type_templates(new_bib, type_templates);
    }
    normalize_legal_case_type_template(legacy_style, type_templates);
    ensure_inferred_media_type_templates(legacy_style, type_templates, new_bib);
    ensure_inferred_patent_type_template(legacy_style, type_templates, new_bib);
}

fn repair_inferred_bibliography_type_templates(
    default_template: &[TemplateComponent],
    type_templates: &mut TypeTemplateMap,
) {
    let base_title = default_template
        .iter()
        .find(|component| component_is_primary_title(component))
        .cloned();
    let base_publisher = default_template
        .iter()
        .find(|component| component_is_publisher(component))
        .cloned();

    for (selector, template) in type_templates.iter_mut() {
        let type_names = selector.type_names();

        if should_inherit_primary_title(&type_names)
            && !template.iter().any(component_is_primary_title)
            && let Some(base_title) = base_title.clone()
        {
            let insert_at = template
                .iter()
                .position(|component| {
                    component_is_container_title(component) || component_is_in_term(component)
                })
                .or_else(|| {
                    template
                        .iter()
                        .rposition(|component| {
                            component_is_author(component) || component_is_issued_date(component)
                        })
                        .map(|index| index + 1)
                })
                .unwrap_or(0);
            template.insert(insert_at, base_title);
        }

        if should_inherit_publisher(&type_names)
            && !template.iter().any(component_is_publisher)
            && let Some(base_publisher) = base_publisher.clone()
        {
            let insert_at = template
                .iter()
                .rposition(component_is_publisher_place)
                .map(|index| index + 1)
                .or_else(|| {
                    template
                        .iter()
                        .rposition(component_is_primary_title)
                        .map(|index| index + 1)
                })
                .unwrap_or(template.len());
            template.insert(insert_at, base_publisher);
        }
    }
}

fn should_inherit_primary_title(type_names: &[String]) -> bool {
    type_names.iter().any(|type_name| {
        matches!(
            type_name.as_str(),
            "article-magazine"
                | "article-newspaper"
                | "book"
                | "broadcast"
                | "motion_picture"
                | "motion-picture"
                | "report"
        )
    })
}

fn should_inherit_publisher(type_names: &[String]) -> bool {
    type_names.iter().any(|type_name| {
        matches!(
            type_name.as_str(),
            "book" | "motion_picture" | "motion-picture" | "report" | "thesis"
        )
    })
}

fn component_is_primary_title(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if title.title == citum_schema::template::TitleType::Primary
    )
}

fn component_is_container_title(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if matches!(
                title.title,
                citum_schema::template::TitleType::ParentMonograph
                    | citum_schema::template::TitleType::ParentSerial
            )
    )
}

fn component_is_author(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Contributor(contributor)
            if contributor.contributor == citum_schema::template::ContributorRole::Author
    )
}

fn component_is_issued_date(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(date)
            if date.date == citum_schema::template::DateVariable::Issued
    )
}

fn component_is_in_term(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Term(term)
            if term.term == citum_schema::locale::GeneralTerm::In
    )
}

fn component_is_publisher(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable)
            if variable.variable == citum_schema::template::SimpleVariable::Publisher
    )
}

fn component_is_publisher_place(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable)
            if variable.variable == citum_schema::template::SimpleVariable::PublisherPlace
    )
}

fn validate_and_normalize_inferred_citations(
    resolved: &mut template_resolver::ResolvedTemplates,
    options: &citum_schema::options::Config,
    legacy_style: &csl_legacy::model::Style,
    style_name: &str,
    citation_has_scope_shorten: bool,
) {
    if let Some(resolved_cit) = resolved.citation.as_ref() {
        let is_inferred_source = matches!(
            resolved_cit.source,
            template_resolver::TemplateSource::InferredCached(_)
                | template_resolver::TemplateSource::InferredLive
        );
        if is_inferred_source {
            let reject_reason = if resolved_cit.template.is_empty() {
                Some("empty citation template")
            } else if matches!(
                options.processing,
                Some(citum_schema::options::Processing::Numeric)
            ) && !citation_template_has_citation_number(&resolved_cit.template)
            {
                Some("numeric style citation template missing citation-number")
            } else if legacy_style.class == "note"
                && note_citation_template_is_underfit(&resolved_cit.template)
            {
                Some("note style citation template is contributor-only underfit")
            } else {
                None
            };
            if let Some(reason) = reject_reason {
                eprintln!(
                    "Rejecting inferred citation template for {style_name}: {reason}. Falling back to XML citation template."
                );
                resolved.citation = None;
            }
        }
    }

    let should_normalize = legacy_style.class == "note"
        || matches!(
            options.processing,
            Some(citum_schema::options::Processing::AuthorDate)
        );

    if should_normalize && let Some(resolved_cit) = resolved.citation.as_mut() {
        let is_inferred_source = matches!(
            resolved_cit.source,
            template_resolver::TemplateSource::InferredCached(_)
                | template_resolver::TemplateSource::InferredLive
        );
        if is_inferred_source
            && citation_template_is_author_year_only(&resolved_cit.template)
            && normalize_contributor_form_to_short(&mut resolved_cit.template)
        {
            eprintln!(
                "Normalized citation contributor form to short for {style_name} (author-year inferred citation template)."
            );
        }
    }

    if legacy_style.class == "in-text"
        && let Some(resolved_cit) = resolved.citation.as_mut()
    {
        let is_inferred_source = matches!(
            resolved_cit.source,
            template_resolver::TemplateSource::InferredCached(_)
                | template_resolver::TemplateSource::InferredLive
        );
        let is_author_year_shape = citation_template_is_author_year_only(&resolved_cit.template)
            && !citation_template_has_citation_number(&resolved_cit.template);
        if is_inferred_source
            && is_author_year_shape
            && normalize_author_date_inferred_contributors(
                &mut resolved_cit.template,
                citation_has_scope_shorten,
            )
        {
            eprintln!(
                "Normalized inferred author-date citation contributors for {style_name} (family-short + scoped shorten)."
            );
        }
    }
}

fn relax_inferred_bibliography_date_suppression(template: &mut [TemplateComponent]) {
    for component in template {
        if let TemplateComponent::Group(list) = component {
            relax_inferred_bibliography_date_suppression(&mut list.group);
        }
    }
}

trait TypeSelectorNames {
    fn type_names(&self) -> Vec<String>;
}

impl TypeSelectorNames for TypeSelector {
    fn type_names(&self) -> Vec<String> {
        match self {
            TypeSelector::Single(name) => vec![name.clone()],
            TypeSelector::Multiple(names) => names.clone(),
        }
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
    use citum_schema::locale::GeneralTerm;
    use citum_schema::template::{
        ContributorRole, DateVariable, Rendering, SimpleVariable, TemplateComponent,
        TemplateContributor, TemplateDate, TemplateTerm, TemplateTitle, TemplateVariable,
        TitleType, TypeSelector,
    };
    use csl_legacy::model::{
        Citation, CslNode, Formatting, Group, Info, Layout, Sort as LegacySort,
        SortKey as LegacySortKey, Style as LegacyStyle, Text,
    };

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
            Some(citum_schema::grouping::GroupSortEntry::Explicit(
                citum_schema::grouping::GroupSort {
                    template: vec![
                        citum_schema::grouping::GroupSortKey {
                            key: citum_schema::grouping::SortKey::Author,
                            ascending: true,
                            order: None,
                            sort_order: None,
                        },
                        citum_schema::grouping::GroupSortKey {
                            key: citum_schema::grouping::SortKey::Issued,
                            ascending: true,
                            order: None,
                            sort_order: None,
                        },
                        citum_schema::grouping::GroupSortKey {
                            key: citum_schema::grouping::SortKey::Title,
                            ascending: true,
                            order: None,
                            sort_order: None,
                        },
                    ],
                }
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
            out.citation
                .iter()
                .any(|component| matches!(component, TemplateComponent::Title(_))),
            "explicit first-position branch should become part of the base citation template"
        );
        assert!(
            out.citation
                .iter()
                .any(|component| matches!(component, TemplateComponent::Date(_))),
            "fallback content from sibling chooses should remain in the base citation template"
        );

        let subsequent_template = out
            .citation_overrides
            .subsequent
            .expect("subsequent branch should be migrated");
        assert!(
            subsequent_template
                .iter()
                .any(|component| matches!(component, TemplateComponent::Contributor(_))),
            "subsequent override should preserve author short-form branch"
        );
        assert!(
            subsequent_template
                .iter()
                .any(|component| matches!(component, TemplateComponent::Date(_))),
            "sibling choose fallback content should remain in the subsequent override"
        );

        let ibid_template = out
            .citation_overrides
            .ibid
            .expect("ibid branch should be migrated");
        assert!(
            ibid_template.iter().any(|component| matches!(
                component,
                TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Locator
            )),
            "merged ibid override should preserve locator-aware content"
        );
        assert!(
            ibid_template.iter().any(|component| matches!(
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
