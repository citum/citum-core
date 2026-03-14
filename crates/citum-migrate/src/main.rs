#![allow(missing_docs)]

use citum_migrate::{
    Compressor, MacroInliner, OptionsExtractor, TemplateCompiler, Upsampler, analysis,
    debug_output::DebugOutputFormatter, fixups::{
        citation_template_has_citation_number, citation_template_is_author_year_only,
        ensure_inferred_media_type_templates, ensure_inferred_patent_type_template,
        ensure_numeric_locator_citation_component, ensure_personal_communication_omitted,
        move_group_wrap_to_citation_items, normalize_author_date_inferred_contributors,
        normalize_author_date_locator_citation_component, normalize_contributor_form_to_short,
        normalize_legal_case_type_template, normalize_wrapped_numeric_locator_citation_component,
        note_citation_template_is_underfit,
        scrub_inferred_literal_artifacts, selector_matches_any, should_merge_inferred_type_template,
    }, passes, preset_detector, provenance::ProvenanceTracker,
    template_resolver,
};
use citum_schema::{
    BibliographySpec, CitationSpec, Style, StyleInfo,
    template::{
        DateVariable, DelimiterPunctuation, Rendering, SimpleVariable, TemplateComponent,
        TemplateList, TemplateVariable, TitleType, TypeSelector, WrapPunctuation,
    },
};
use csl_legacy::{
    model::{CslNode, Layout},
    parser::parse_style,
};
use roxmltree::Document;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let program_name = args
        .first()
        .and_then(|arg| std::path::Path::new(arg).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("citum-migrate");

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help(program_name);
        return Ok(());
    }

    // Parse command-line arguments
    let mut path = "styles-legacy/apa.csl";
    let mut debug_variable: Option<String> = None;
    let mut template_mode = template_resolver::TemplateMode::Auto;
    let mut template_dir: Option<PathBuf> = None;
    let mut min_template_confidence = 0.70_f64;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--debug-variable" => {
                if i + 1 < args.len() {
                    debug_variable = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --debug-variable requires an argument");
                    std::process::exit(1);
                }
            }
            "--template-source" => {
                if i + 1 < args.len() {
                    template_mode = match args[i + 1].parse::<template_resolver::TemplateMode>() {
                        Ok(mode) => mode,
                        Err(msg) => {
                            eprintln!("Error: {}", msg);
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!(
                        "Error: --template-source requires an argument (auto|hand|inferred|xml)"
                    );
                    std::process::exit(1);
                }
            }
            "--min-template-confidence" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(val) if (0.0..=1.0).contains(&val) => {
                            min_template_confidence = val;
                            i += 2;
                        }
                        _ => {
                            eprintln!(
                                "Error: --min-template-confidence requires a number in [0.0, 1.0]"
                            );
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: --min-template-confidence requires a numeric argument");
                    std::process::exit(1);
                }
            }
            "--template-dir" => {
                if i + 1 < args.len() {
                    template_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --template-dir requires a path argument");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with('-') => {
                path = &args[i];
                i += 1;
            }
            _ => {
                eprintln!("Error: unknown argument '{}'", args[i]);
                eprintln!();
                print_help(program_name);
                std::process::exit(1);
            }
        }
    }

    // Initialize provenance tracking if debug variable is specified
    let enable_provenance = debug_variable.is_some();
    let tracker = ProvenanceTracker::new(enable_provenance);

    eprintln!("Migrating {} to Citum...", path);

    let text = fs::read_to_string(path)?;
    let doc = Document::parse(&text)?;
    let legacy_style = parse_style(doc.root_element())?;

    // 0. Extract global options (new Citum Config)
    let mut options = OptionsExtractor::extract(&legacy_style);
    apply_preset_extractions(&mut options);
    let citation_contributor_overrides =
        citum_migrate::options_extractor::contributors::extract_citation_contributor_overrides(
            &legacy_style,
        );
    let bibliography_contributor_overrides =
        citum_migrate::options_extractor::contributors::extract_bibliography_contributor_overrides(
            &legacy_style,
        );
    let citation_has_scope_shorten = citation_contributor_overrides
        .as_ref()
        .and_then(|contributors| contributors.shorten.as_ref())
        .is_some();

    // Resolve template: try hand-authored, cached inferred, or live inference
    // before falling back to the XML compiler pipeline.
    let style_name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Determine workspace root by finding the Cargo workspace directory.
    // For relative paths like "styles-legacy/foo.csl", this is the current directory.
    // For absolute paths, walk up from the style file to find the workspace.
    let workspace_root = {
        let style_path = std::path::Path::new(path);
        if style_path.is_absolute() {
            // Walk up to find Cargo.toml
            style_path
                .ancestors()
                .find(|p| p.join("Cargo.toml").exists())
                .unwrap_or(style_path.parent().unwrap_or(std::path::Path::new(".")))
                .to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    };

    let mut resolved = template_resolver::resolve_templates(
        path,
        style_name,
        template_dir.as_deref(),
        &workspace_root,
        template_mode,
        min_template_confidence,
    );

    // Guardrails for inferred citation templates:
    // - Empty citation templates regress fidelity heavily.
    // - Numeric styles require citation-number in citation templates.
    let mut reject_inferred_citation_reason: Option<&str> = None;
    if let Some(resolved_cit) = resolved.citation.as_ref() {
        let is_inferred_source = matches!(
            resolved_cit.source,
            template_resolver::TemplateSource::InferredCached(_)
                | template_resolver::TemplateSource::InferredLive
        );
        if is_inferred_source {
            if resolved_cit.template.is_empty() {
                reject_inferred_citation_reason = Some("empty citation template");
            } else if matches!(
                options.processing,
                Some(citum_schema::options::Processing::Numeric)
            ) && !citation_template_has_citation_number(&resolved_cit.template)
            {
                reject_inferred_citation_reason =
                    Some("numeric style citation template missing citation-number");
            } else if legacy_style.class == "note"
                && note_citation_template_is_underfit(&resolved_cit.template)
            {
                reject_inferred_citation_reason =
                    Some("note style citation template is contributor-only underfit");
            }
        }
    }
    if let Some(reason) = reject_inferred_citation_reason {
        eprintln!(
            "Rejecting inferred citation template for {}: {}. Falling back to XML citation template.",
            style_name, reason
        );
        resolved.citation = None;
    }

    // Heuristic normalization for note styles:
    // If inferred citation template is a simple author-year shape, prefer short
    // contributor form to align with typical note citation behavior.
    let should_normalize_author_year_citations = legacy_style.class == "note"
        || matches!(
            options.processing,
            Some(citum_schema::options::Processing::AuthorDate)
        );

    if should_normalize_author_year_citations && let Some(resolved_cit) = resolved.citation.as_mut()
    {
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
                "Normalized citation contributor form to short for {} (author-year inferred citation template).",
                style_name
            );
        }
    }

    // For inferred in-text author-year citations, normalize contributor rendering
    // toward family-only short forms and defer et-al thresholds to citation-scope
    // options when present. Some styles are extracted as `processing: custom`
    // despite using author-year in-text behavior, so detect by template shape.
    let is_in_text_class = legacy_style.class == "in-text";
    if is_in_text_class && let Some(resolved_cit) = resolved.citation.as_mut() {
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
                "Normalized inferred author-date citation contributors for {} (family-short + scoped shorten).",
                style_name
            );
        }
    }

    let xml_fallback = Some(compile_from_xml(
        &legacy_style,
        &mut options,
        enable_provenance,
        &tracker,
    ));

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

    let (mut new_bib, mut type_templates, inferred_bib_source) =
        if let Some(ref resolved_bib) = resolved.bibliography {
            let inferred_bib = matches!(
                resolved_bib.source,
                template_resolver::TemplateSource::InferredCached(_)
                    | template_resolver::TemplateSource::InferredLive
            );

            // When bibliography comes from inferred output, merge selective
            // branch-derived type templates from the XML fallback path. This keeps
            // inferred global ordering while restoring high-value type branches
            // (e.g., patent/webpage/entry-encyclopedia/legal-case) that frequently
            // need full template specialization.
            let merged_type_templates = if inferred_bib {
                xml_fallback
                    .as_ref()
                    .and_then(|(_, type_templates, _, _)| type_templates.clone())
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
                            .collect::<std::collections::HashMap<_, _>>()
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
            let (new_bib, type_templates, _, _) = xml_fallback
                .as_ref()
                .expect("XML fallback must exist when bibliography is unresolved");
            (new_bib.clone(), type_templates.clone(), false)
        };

    if inferred_bib_source {
        // Output-driven inference can leak literal sample years into prefixes
        // (e.g., " 2023 " in titles, "; 2006; " in page prefixes).
        // Strip those artifacts while keeping component structure intact.
        for component in &mut new_bib {
            scrub_inferred_literal_artifacts(component);
        }
        relax_inferred_bibliography_date_suppression(&mut new_bib);
        if let Some(type_templates) = type_templates.as_mut() {
            for template in type_templates.values_mut() {
                for component in template.iter_mut() {
                    scrub_inferred_literal_artifacts(component);
                }
                relax_inferred_bibliography_date_suppression(template);
            }
        }
        normalize_legal_case_type_template(&legacy_style, &mut type_templates);
        ensure_inferred_media_type_templates(&legacy_style, &mut type_templates, &new_bib);
        ensure_inferred_patent_type_template(&legacy_style, &mut type_templates, &new_bib);
    }

    let mut citation_subsequent_override: Option<Vec<TemplateComponent>> = None;
    let mut citation_ibid_override: Option<Vec<TemplateComponent>> = None;
    let mut new_cit = if let Some(ref resolved_cit) = resolved.citation {
        resolved_cit.template.clone()
    } else {
        let (_, _, new_cit, citation_overrides) = xml_fallback
            .as_ref()
            .expect("XML fallback must exist when citation is unresolved");
        citation_subsequent_override = citation_overrides.subsequent.clone();
        citation_ibid_override = citation_overrides.ibid.clone();
        new_cit.clone()
    };

    if inferred_bib_source {
        ensure_personal_communication_omitted(&legacy_style, &new_cit, &mut type_templates);
    }

    // Override bibliography options with inferred values when available.
    // The XML options extractor often gets the wrong delimiter because it reads group
    // delimiters rather than rendered output.
    if let Some(ref resolved_bib) = resolved.bibliography {
        let is_inferred_source = matches!(
            resolved_bib.source,
            template_resolver::TemplateSource::InferredCached(_)
                | template_resolver::TemplateSource::InferredLive
        );
        let allow_bib_punctuation_override = !(legacy_style.class == "note" && is_inferred_source);

        if allow_bib_punctuation_override {
            if let Some(ref delim) = resolved_bib.delimiter {
                eprintln!("  Overriding bibliography separator: {:?}", delim);
                let bib_cfg = options.bibliography.get_or_insert_with(Default::default);
                bib_cfg.separator = Some(delim.clone());
            }

            if let Some(ref suffix) = resolved_bib.entry_suffix {
                eprintln!("  Overriding bibliography entry suffix: {:?}", suffix);
                let bib_cfg = options.bibliography.get_or_insert_with(Default::default);
                bib_cfg.entry_suffix = Some(suffix.clone());
            }
        } else {
            eprintln!(
                "  Skipping inferred bibliography separator/entry-suffix override for note style."
            );
        }
    }

    let (mut citation_wrap, mut citation_prefix, mut citation_suffix) =
        analysis::citation::infer_citation_wrapping(&legacy_style.citation.layout);
    let mut citation_delimiter = analysis::citation::extract_citation_delimiter(
        &legacy_style.citation.layout,
        &legacy_style.macros,
    );

    // Output-driven citation metadata is higher fidelity than XML analysis when available.
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

    // Numeric citation fixups informed by migration quality runs:
    // - Keep locator labels when legacy style has a citation-locator macro.
    // - Preserve per-item wrapping for grouped numeric layouts (e.g., IEEE).
    if matches!(
        options.processing,
        Some(citum_schema::options::Processing::Numeric)
    ) {
        ensure_numeric_locator_citation_component(&legacy_style.citation.layout, &mut new_cit);
        normalize_wrapped_numeric_locator_citation_component(
            &legacy_style.citation.layout,
            &mut new_cit,
            &mut citation_delimiter,
        );
        move_group_wrap_to_citation_items(
            &legacy_style.citation.layout,
            &mut new_cit,
            &mut citation_wrap,
        );
    } else if legacy_style.class == "in-text" {
        normalize_author_date_locator_citation_component(
            &legacy_style.citation.layout,
            &legacy_style.macros,
            &mut new_cit,
        );
    }

    // 5. Build Style in correct format for citum_engine
    let citation_scope_options =
        citation_contributor_overrides.map(|contributors| citum_schema::options::Config {
            contributors: Some(contributors),
            ..Default::default()
        });

    let bibliography_scope_options =
        bibliography_contributor_overrides.map(|contributors| citum_schema::options::Config {
            contributors: Some(contributors),
            ..Default::default()
        });

    // Preserve legacy bibliography sort semantics at the Citum bibliography spec level.
    // This is required for numeric alphabetical variants where citation numbers
    // follow bibliography order rather than reference registry order.
    let bibliography_sort = resolve_migrated_bibliography_sort(
        options.processing.as_ref(),
        legacy_style
            .bibliography
            .as_ref()
            .and_then(|bib| bib.sort.as_ref()),
    );

    let style = Style {
        info: StyleInfo {
            title: Some(legacy_style.info.title.clone()),
            id: Some(legacy_style.info.id.clone()),
            default_locale: legacy_style.default_locale.clone(),
            ..Default::default()
        },
        templates: None,
        options: Some(options.clone()),
        citation: Some({
            CitationSpec {
                options: citation_scope_options,
                use_preset: None,
                template: Some(new_cit),
                wrap: citation_wrap,
                prefix: citation_prefix,
                suffix: citation_suffix,
                delimiter: citation_delimiter,
                multi_cite_delimiter: legacy_style.citation.layout.delimiter.clone(),
                subsequent: citation_subsequent_override.map(|template| {
                    Box::new(CitationSpec {
                        template: Some(template),
                        ..Default::default()
                    })
                }),
                ibid: citation_ibid_override.map(|template| {
                    Box::new(CitationSpec {
                        template: Some(template),
                        ..Default::default()
                    })
                }),
                ..Default::default()
            }
        }),
        bibliography: Some(BibliographySpec {
            options: bibliography_scope_options,
            use_preset: None,
            template: Some(new_bib),
            type_templates,
            sort: bibliography_sort,
            ..Default::default()
        }),
        ..Default::default()
    };

    // Output YAML to stdout
    let yaml = serde_yaml::to_string(&style)?;
    println!("{}", yaml);

    // Output debug information if requested
    if let Some(var_name) = debug_variable {
        eprintln!("\n");
        eprintln!("=== PROVENANCE DEBUG ===\n");
        let debug_output = DebugOutputFormatter::format_variable(&tracker, &var_name);
        eprint!("{}", debug_output);
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
        .and_then(|processing| processing.default_bibliography_sort())
        .is_some_and(|preset| preset.group_sort() == *sort)
}

#[derive(Debug, Clone, Default)]
struct CitationPositionOverrides {
    subsequent: Option<Vec<TemplateComponent>>,
    ibid: Option<Vec<TemplateComponent>>,
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

/// Run the full XML compilation pipeline for bibliography and citation templates.
/// This is the fallback when no hand-authored or inferred template is available.
#[allow(clippy::type_complexity)]
fn compile_from_xml(
    legacy_style: &csl_legacy::model::Style,
    options: &mut citum_schema::options::Config,
    enable_provenance: bool,
    tracker: &citum_migrate::provenance::ProvenanceTracker,
) -> (
    Vec<TemplateComponent>,
    Option<std::collections::HashMap<citum_schema::template::TypeSelector, Vec<TemplateComponent>>>,
    Vec<TemplateComponent>,
    CitationPositionOverrides,
) {
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
    if citation_position_nodes.unsupported_mixed_conditions {
        eprintln!(
            "Warning: citation position branches could not be migrated cleanly for style {}. Falling back to base citation template only.",
            legacy_style.info.id
        );
    }
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

    // Record template placements if provenance tracking is enabled
    if enable_provenance {
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

    // Apply author suffix extracted from original CSL (lost during macro inlining)
    analysis::bibliography::apply_author_suffix(&mut new_bib, author_suffix);

    // Apply bibliography-specific 'and' setting (may differ from citation)
    analysis::bibliography::apply_bibliography_and(&mut new_bib, bib_and);

    // For author-date styles with in-text class, apply standard formatting.
    // Note styles (class="note") should NOT have these transformations applied.
    let is_in_text_class = legacy_style.class == "in-text";
    let is_author_date_processing = matches!(
        options.processing,
        Some(citum_schema::options::Processing::AuthorDate)
    );

    // Apply to all in-text styles (both author-date and numeric)
    if is_in_text_class {
        // Add space prefix to volume when it follows parent-serial directly.
        // This handles numeric styles where journal and volume are siblings, not in a List.
        passes::reorder::add_volume_prefix_after_serial(&mut new_bib);
    }

    // Detect holistic style preset for semantic fixups
    let style_preset = preset_detector::detect_style_preset(options);
    if let Some(preset) = style_preset {
        eprintln!("Detected style preset: {:?}", preset);
    }

    if is_in_text_class && is_author_date_processing {
        // Detect if the style uses space prefix for volume (Elsevier pattern)
        let volume_list_has_space_prefix = new_bib.iter().any(|c| {
            if let TemplateComponent::List(list) = c {
                let has_volume = list.items.iter().any(|item| {
                    matches!(item, TemplateComponent::Number(n) if n.number == citum_schema::template::NumberVariable::Volume)
                });
                if has_volume {
                    // Check if the List has a space-only prefix
                    return list.rendering.prefix.as_deref() == Some(" ");
                }
            }
            false
        });

        // Add type-specific overrides (recursively to handle nested Lists)
        // Pass the extracted volume-pages delimiter for journal article pages
        let vol_pages_delim = options.volume_pages_delimiter.clone();
        for component in &mut new_bib {
            apply_type_overrides(
                component,
                vol_pages_delim.clone(),
                volume_list_has_space_prefix,
                style_preset,
            );
        }

        // Move DOI/URL to the end of the bibliography template.
        passes::reorder::move_access_components_to_end(&mut new_bib);

        // Ensure publisher and publisher-place are unsuppressed for chapters
        passes::reorder::unsuppress_for_type(&mut new_bib, "chapter");
        passes::reorder::unsuppress_for_type(&mut new_bib, "paper-conference");
        passes::reorder::unsuppress_for_type(&mut new_bib, "thesis");
        passes::reorder::unsuppress_for_type(&mut new_bib, "document");

        // Remove duplicate titles from Lists that already appear at top level.
        passes::deduplicate::deduplicate_titles_in_lists(&mut new_bib);

        // Suppress variables that appear in multiple sibling lists (enforce variable-once rule).
        passes::deduplicate::deduplicate_variables_cross_lists(&mut new_bib);

        // Propagate type-specific overrides within Lists.
        passes::reorder::propagate_list_overrides(&mut new_bib);

        // Remove duplicate nested Lists that have identical contents.
        passes::deduplicate::deduplicate_nested_lists(&mut new_bib);

        // Reorder serial components: container-title before volume.
        passes::reorder::reorder_serial_components(&mut new_bib);

        // Combine volume and issue into a grouped structure: volume(issue)
        passes::grouping::group_volume_and_issue(&mut new_bib, options, style_preset);

        // Move pages to after the container-title/volume List for serial types.
        passes::reorder::reorder_pages_for_serials(&mut new_bib);

        // Reorder publisher-place for Chicago journal articles.
        passes::reorder::reorder_publisher_place_for_chicago(&mut new_bib, style_preset);

        // Reorder chapters for APA: "In " prefix + editors before book title
        passes::reorder::reorder_chapters_for_apa(&mut new_bib, style_preset);

        // Reorder chapters for Chicago: "In" prefix + book title before editors
        passes::reorder::reorder_chapters_for_chicago(&mut new_bib, style_preset);

        // Fix Chicago issue placement
        passes::deduplicate::suppress_duplicate_issue_for_journals(&mut new_bib, style_preset);
    }

    let type_templates_opt = if type_templates.is_empty() {
        None
    } else {
        Some(type_templates)
    };

    (
        new_bib,
        type_templates_opt,
        new_cit,
        citation_position_overrides,
    )
}

fn apply_type_overrides(
    component: &mut TemplateComponent,
    volume_pages_delimiter: Option<citum_schema::template::DelimiterPunctuation>,
    volume_list_has_space_prefix: bool,
    style_preset: Option<preset_detector::StylePreset>,
) {
    use preset_detector::StylePreset;
    match component {
        // Primary title: style-specific suffix for articles
        TemplateComponent::Title(t) if t.title == citum_schema::template::TitleType::Primary => {
            if matches!(style_preset, Some(StylePreset::Apa)) {
                let overrides = t.overrides.get_or_insert_with(Default::default);
                merge_type_rendering(
                    overrides,
                    "article-journal",
                    citum_schema::template::Rendering {
                        suffix: Some(". ".to_string()),
                        ..Default::default()
                    },
                );
            }
        }
        // Container-title (parent-monograph): style-specific unsuppression
        TemplateComponent::Title(t)
            if t.title == citum_schema::template::TitleType::ParentMonograph =>
        {
            if matches!(style_preset, Some(StylePreset::Apa)) {
                let overrides = t.overrides.get_or_insert_with(Default::default);
                merge_type_rendering(
                    overrides,
                    "paper-conference",
                    citum_schema::template::Rendering {
                        suppress: Some(true),
                        ..Default::default()
                    },
                );
            }
        }
        // Container-title (parent-serial): style-specific suffix and unsuppression
        // - APA: comma suffix, no prefix
        // - Chicago: space suffix (prevents default period separator)
        // - Elsevier: space prefix (handled by List), no suffix needed
        TemplateComponent::Title(t)
            if t.title == citum_schema::template::TitleType::ParentSerial =>
        {
            let is_chicago = matches!(style_preset, Some(StylePreset::Chicago));

            // Always unsuppress article-journal (journal title must show)
            let suffix = if volume_list_has_space_prefix {
                // Elsevier: no suffix, spacing handled by List prefix
                None
            } else if is_chicago {
                Some(" ".to_string())
            } else {
                // APA: comma suffix
                Some(",".to_string())
            };

            let overrides = t.overrides.get_or_insert_with(Default::default);
            merge_type_rendering(
                overrides,
                "article-journal",
                citum_schema::template::Rendering {
                    suffix,
                    suppress: Some(false),
                    ..Default::default()
                },
            );
            // Ensure paper-conference shows container title (proceedings name)
            merge_type_rendering(
                overrides,
                "paper-conference",
                citum_schema::template::Rendering {
                    suffix: Some(",".to_string()),
                    suppress: Some(false),
                    ..Default::default()
                },
            );
        }
        // Publisher: suppress for journal articles (journals don't have publishers in bib)
        TemplateComponent::Variable(v)
            if v.variable == citum_schema::template::SimpleVariable::Publisher =>
        {
            let overrides = v.overrides.get_or_insert_with(Default::default);
            merge_type_rendering(
                overrides,
                "article-journal",
                citum_schema::template::Rendering {
                    suppress: Some(true),
                    ..Default::default()
                },
            );
        }
        // Publisher-place: suppress for journal articles
        TemplateComponent::Variable(v)
            if v.variable == citum_schema::template::SimpleVariable::PublisherPlace =>
        {
            let overrides = v.overrides.get_or_insert_with(Default::default);
            merge_type_rendering(
                overrides,
                "article-journal",
                citum_schema::template::Rendering {
                    suppress: Some(true),
                    ..Default::default()
                },
            );
        }
        // Pages: apply volume-pages delimiter for journal articles
        TemplateComponent::Number(n)
            if n.number == citum_schema::template::NumberVariable::Pages =>
        {
            if let Some(delim) = volume_pages_delimiter {
                let overrides = n.overrides.get_or_insert_with(Default::default);
                merge_type_rendering(
                    overrides,
                    "article-journal",
                    citum_schema::template::Rendering {
                        prefix: Some(match delim {
                            citum_schema::template::DelimiterPunctuation::Comma => ", ".to_string(),
                            citum_schema::template::DelimiterPunctuation::Colon => ":".to_string(),
                            citum_schema::template::DelimiterPunctuation::Space => " ".to_string(),
                            _ => "".to_string(),
                        }),
                        ..Default::default()
                    },
                );
            }
        }
        TemplateComponent::List(list) => {
            for item in &mut list.items {
                apply_type_overrides(
                    item,
                    volume_pages_delimiter.clone(),
                    volume_list_has_space_prefix,
                    style_preset,
                );
            }
        }
        _ => {}
    }
}

fn relax_inferred_bibliography_date_suppression(template: &mut [TemplateComponent]) {
    for component in template {
        match component {
            TemplateComponent::Date(date_component) => {
                if date_component.date != DateVariable::Issued {
                    continue;
                }
                if let Some(overrides) = date_component.overrides.as_mut() {
                    for (selector, override_value) in overrides.iter_mut() {
                        if !selector_matches_any(
                            selector,
                            &[
                                "patent",
                                "broadcast",
                                "interview",
                                "motion_picture",
                                "webpage",
                                "legal_case",
                                "legal-case",
                            ],
                        ) {
                            continue;
                        }
                        if let citum_schema::template::ComponentOverride::Rendering(rendering) =
                            override_value
                            && rendering.suppress == Some(true)
                        {
                            rendering.suppress = Some(false);
                        }
                    }
                }
            }
            TemplateComponent::List(list) => {
                relax_inferred_bibliography_date_suppression(&mut list.items)
            }
            _ => {}
        }
    }
}

/// Insert a single `Rendering` override keyed by `type_name` into an overrides map.
fn merge_type_rendering(
    overrides: &mut std::collections::HashMap<
        citum_schema::template::TypeSelector,
        citum_schema::template::ComponentOverride,
    >,
    type_name: &str,
    rendering: citum_schema::template::Rendering,
) {
    use citum_schema::template::ComponentOverride;
    overrides.insert(
        citum_schema::template::TypeSelector::Single(type_name.to_string()),
        ComponentOverride::Rendering(rendering),
    );
}

fn apply_preset_extractions(options: &mut citum_schema::options::Config) {
    if let Some(contributors) = options.contributors.clone()
        && let Some(preset) = preset_detector::detect_contributor_preset(&contributors)
    {
        options.contributors = Some(preset.config());
    }

    if let Some(titles) = options.titles.clone()
        && let Some(preset) = preset_detector::detect_title_preset(&titles)
    {
        options.titles = Some(preset.config());
    }

    if let Some(dates) = options.dates.clone()
        && let Some(preset) = preset_detector::detect_date_preset(&dates)
    {
        options.dates = Some(preset.config());
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
mod tests {
    use super::*;
    use csl_legacy::model::{
        CslNode, Formatting, Group, Sort as LegacySort, SortKey as LegacySortKey, Text,
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
                show_label: Some(true),
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
        assert_eq!(locator.show_label, Some(true));
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
        let (_, _, citation_template, citation_overrides) =
            compile_from_xml(&legacy_style, &mut options, false, &tracker);

        assert!(
            citation_template
                .iter()
                .any(|component| matches!(component, TemplateComponent::Title(_))),
            "explicit first-position branch should become part of the base citation template"
        );
        assert!(
            citation_template
                .iter()
                .any(|component| matches!(component, TemplateComponent::Date(_))),
            "fallback content from sibling chooses should remain in the base citation template"
        );

        let subsequent_template = citation_overrides
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

        let ibid_template = citation_overrides
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
        let (_, _, citation_template, citation_overrides) =
            compile_from_xml(&legacy_style, &mut options, false, &tracker);

        assert!(
            citation_template
                .iter()
                .any(|component| matches!(component, TemplateComponent::Title(_))),
            "base citation template should still contain the first-citation title"
        );
        assert!(
            citation_overrides.subsequent.is_some(),
            "mixed note trees should now emit a subsequent override"
        );
        assert!(
            citation_overrides
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
        let (_, _, citation_template, citation_overrides) =
            compile_from_xml(&legacy_style, &mut options, false, &tracker);

        assert!(
            !citation_template.is_empty(),
            "unsupported trees must still compile a base citation template"
        );
        assert!(
            citation_overrides.subsequent.is_none() && citation_overrides.ibid.is_none(),
            "unsupported trees should not emit partial position overrides"
        );
    }
}
