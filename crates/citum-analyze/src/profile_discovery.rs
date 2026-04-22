/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use roxmltree::Document;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use citum_migrate::{OptionsExtractor, compilation, fixups, provenance::ProvenanceTracker};
use citum_schema::StyleBase;
use citum_schema::locale::GeneralTerm;
use citum_schema::template::{
    ContributorRole, DateVariable, NumberVariable, SimpleVariable, TemplateComponent, TitleType,
};
use csl_legacy::parser::parse_style;

/// A simplified semantic representation of a template component.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum SemanticItem {
    Variable(SimpleVariable),
    Number(NumberVariable),
    Date(DateVariable),
    Contributor(ContributorRole),
    Title(TitleType),
    Term(GeneralTerm),
}

fn to_semantic_items(component: &TemplateComponent, items: &mut Vec<SemanticItem>) {
    match component {
        TemplateComponent::Variable(v) => items.push(SemanticItem::Variable(v.variable.clone())),
        TemplateComponent::Number(n) => items.push(SemanticItem::Number(n.number.clone())),
        TemplateComponent::Date(d) => items.push(SemanticItem::Date(d.date.clone())),
        TemplateComponent::Contributor(c) => {
            items.push(SemanticItem::Contributor(c.contributor.clone()));
        }
        TemplateComponent::Title(t) => items.push(SemanticItem::Title(t.title.clone())),
        TemplateComponent::Term(t) => items.push(SemanticItem::Term(t.term)),
        TemplateComponent::Group(g) => {
            for child in &g.group {
                to_semantic_items(child, items);
            }
        }
        _ => {}
    }
}

fn template_to_set(template: &[TemplateComponent]) -> HashSet<SemanticItem> {
    let mut items = Vec::new();
    for component in template {
        to_semantic_items(component, &mut items);
    }
    items.into_iter().collect()
}

/// Runs the profile discovery analysis.
pub fn run_profile_discovery(styles_dir: &str) {
    eprintln!(
        "Analyzing styles in {} to identify profile candidates...",
        styles_dir
    );

    // Pre-resolve all style bases and their semantic sets
    let bases: Vec<(StyleBase, HashSet<SemanticItem>, Vec<HashSet<SemanticItem>>)> =
        StyleBase::all()
            .iter()
            .map(|base| {
                let style = base.base().into_resolved();
                let bib_set = style
                    .bibliography
                    .as_ref()
                    .and_then(|b| b.template.as_ref())
                    .map(|t| template_to_set(t))
                    .unwrap_or_default();

                let mut cit_sets = Vec::new();
                if let Some(cit) = &style.citation {
                    if let Some(t) = &cit.template {
                        cit_sets.push(template_to_set(t));
                    }
                    if let Some(i) = &cit.integral
                        && let Some(t) = &i.template
                    {
                        cit_sets.push(template_to_set(t));
                    }
                    if let Some(ni) = &cit.non_integral
                        && let Some(t) = &ni.template
                    {
                        cit_sets.push(template_to_set(t));
                    }
                }

                (base.clone(), bib_set, cit_sets)
            })
            .collect();

    let tracker = ProvenanceTracker::new(false);
    let mut count = 0;

    for entry in WalkDir::new(styles_dir)
        .into_iter()
        .filter_entry(|e| e.file_name().to_str().is_none_or(|s| s != "dependent"))
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "csl"))
    {
        count += 1;
        if count % 100 == 0 {
            eprintln!("Processed {} styles...", count);
        }
        let path = entry.path();
        if let Err(_e) = identify_profile_match(path, &bases, &tracker) {
            // Silence errors for batch processing
        }
    }
}

fn identify_profile_match(
    path: &Path,
    bases: &[(StyleBase, HashSet<SemanticItem>, Vec<HashSet<SemanticItem>>)],
    tracker: &ProvenanceTracker,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let doc = Document::parse(&text)?;
    let legacy_style = parse_style(doc.root_element())?;

    // Skip dependent styles for this analysis
    if text.contains("rel=\"independent-parent\"") {
        return Ok(());
    }

    // 1. Extract migration options
    let migration_options = OptionsExtractor::extract_migration_options(&legacy_style);
    let mut options = migration_options.options;

    // 2. Compile templates from XML
    let compiled = compilation::compile_from_xml(&legacy_style, &mut options, false, tracker);

    let mut new_cit = compiled.citation.clone();
    if matches!(
        options.processing,
        Some(citum_schema::options::Processing::Numeric)
    ) {
        fixups::ensure_numeric_locator_citation_component(
            &legacy_style.citation.layout,
            &mut new_cit,
        );
    } else if legacy_style.class == "in-text" {
        fixups::normalize_author_date_locator_citation_component(
            &legacy_style.citation.layout,
            &legacy_style.macros,
            &mut new_cit,
        );
    }

    let bib_set = template_to_set(&compiled.bibliography);
    let cit_set = template_to_set(&new_cit);

    // 4. Compare semantic sets with each base
    for (base_enum, base_bib_set, base_cit_sets) in bases {
        // Use a similarity threshold instead of exact match for the set
        let bib_intersection = bib_set.intersection(base_bib_set).count();
        let bib_union = bib_set.union(base_bib_set).count();
        let bib_sim = if bib_union > 0 {
            bib_intersection as f32 / bib_union as f32
        } else {
            1.0
        };

        let cit_sim = if base_cit_sets.is_empty() {
            1.0
        } else {
            base_cit_sets
                .iter()
                .map(|base_cit_set| {
                    let cit_intersection = cit_set.intersection(base_cit_set).count();
                    let cit_union = cit_set.union(base_cit_set).count();
                    if cit_union > 0 {
                        cit_intersection as f32 / cit_union as f32
                    } else {
                        1.0
                    }
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0)
        };

        if bib_sim > 0.8 && cit_sim > 0.8 {
            println!(
                "Candidate found: {} (Similarity: bib={:.2}, cit={:.2}) could extend '{}'",
                path.display(),
                bib_sim,
                cit_sim,
                base_enum.key()
            );
        }
    }

    Ok(())
}
