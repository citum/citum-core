/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use roxmltree::Document;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::util::short_name_from_identifier;

use citum_migrate::{OptionsExtractor, compilation, fixups, provenance::ProvenanceTracker};
use citum_schema::StyleBase;
use citum_schema::locale::GeneralTerm;
use citum_schema::template::{
    ContributorRole, DateVariable, NumberVariable, SimpleVariable, TemplateComponent, TitleType,
};
use csl_legacy::parser::parse_style;

const AUDITED_CANDIDATES: &[AuditedCandidateSpec] = &[
    AuditedCandidateSpec {
        requested_id: "pharmacoepidemiology-and-drug-safety",
        proposed_parent: "elsevier-with-titles",
        corrected_parent: Some("american-medical-association"),
        disposition: AuditDisposition::JournalProfile,
        note: "Corrected away from Elsevier. Current CSL metadata and alias evidence point to AMA, and the remaining journal delta reduces to scoped options only.",
    },
    AuditedCandidateSpec {
        requested_id: "disability-and-rehabilitation",
        proposed_parent: "elsevier-with-titles",
        corrected_parent: Some("elsevier-with-titles"),
        disposition: AuditDisposition::JournalProfile,
        note: "The current journal instructions do not contradict the Elsevier-like implementation family, and the remaining delta reduces to scoped options only.",
    },
    AuditedCandidateSpec {
        requested_id: "zoological-journal-of-the-linnean-society",
        proposed_parent: "springer-basic-author-date",
        corrected_parent: None,
        disposition: AuditDisposition::FalsePositive,
        note: "The current OUP/Linnean signal does not support the Springer family mapping.",
    },
    AuditedCandidateSpec {
        requested_id: "the-lichenologist",
        proposed_parent: "springer-basic-author-date",
        corrected_parent: None,
        disposition: AuditDisposition::FalsePositive,
        note: "The current Cambridge guide and weak output-match evidence do not justify a profile-style parent mapping.",
    },
    AuditedCandidateSpec {
        requested_id: "memorias-do-instituto-oswaldo-cruz",
        proposed_parent: "springer-basic-author-date",
        corrected_parent: None,
        disposition: AuditDisposition::FalsePositive,
        note: "Current journal guidance supports a journal-specific house style, but not a reusable Springer parent. The reduced Citum style remains standalone.",
    },
    AuditedCandidateSpec {
        requested_id: "techniques-et-culture",
        proposed_parent: "taylor-and-francis-council-of-science-editors-author-date",
        corrected_parent: None,
        disposition: AuditDisposition::FalsePositive,
        note: "Current OpenEdition and CSL-template evidence do not support the Taylor & Francis CSE family mapping.",
    },
    AuditedCandidateSpec {
        requested_id: "hawaii-int-conf-system-sciences",
        proposed_parent: "taylor-and-francis-national-library-of-medicine",
        corrected_parent: None,
        disposition: AuditDisposition::FalsePositive,
        note: "The shorthand candidate ID was normalized, but current HICSS author instructions now prefer APA 7th, so the legacy numeric style should not keep an inferred parent wrapper.",
    },
    AuditedCandidateSpec {
        requested_id: "cell-numeric",
        proposed_parent: "elsevier-with-titles",
        corrected_parent: Some("elsevier-with-titles"),
        disposition: AuditDisposition::JournalProfile,
        note: "The Cell-family numeric variant now reduces to metadata and scoped options only over Elsevier With Titles.",
    },
];

const SPECIAL_NORMALIZATIONS: &[(&str, &str)] = &[(
    "hawaii-int-conf-system-sciences",
    "hawaii-international-conference-on-system-sciences-proceedings",
)];

/// Runs the profile discovery audit.
pub fn run_profile_discovery(styles_dir: &str, json_output: bool) {
    let workspace_root = workspace_root();
    let bases = collect_base_semantic_sets();
    let tracker = ProvenanceTracker::new(false);

    let renamed_styles = load_renamed_styles(&workspace_root);
    let alias_report = load_alias_report(&workspace_root);
    let registry = load_registry_catalog(&workspace_root);
    let styles_root = Path::new(styles_dir);

    let mut audited_candidates = Vec::new();
    let mut parse_errors = Vec::new();

    for spec in AUDITED_CANDIDATES {
        match audit_candidate(
            styles_root,
            spec,
            &bases,
            &tracker,
            &renamed_styles,
            alias_report.as_ref(),
            registry.as_ref(),
        ) {
            Ok(candidate) => audited_candidates.push(candidate),
            Err(error) => parse_errors.push(format!("{}: {error}", spec.requested_id)),
        }
    }

    let report = ProfileDiscoveryReport {
        styles_dir: styles_dir.to_string(),
        summary: ProfileDiscoverySummary::from_candidates(&audited_candidates, &parse_errors),
        audited_candidates,
        parse_errors,
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print_profile_discovery_report(&report);
    }
}

#[derive(Debug, Clone, Copy)]
struct AuditedCandidateSpec {
    requested_id: &'static str,
    proposed_parent: &'static str,
    corrected_parent: Option<&'static str>,
    disposition: AuditDisposition,
    note: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "The current shortlist uses only part of the audit vocabulary, but the full set of dispositions remains intentional."
)]
#[serde(rename_all = "kebab-case")]
enum AuditDisposition {
    JournalProfile,
    JournalAlias,
    JournalStructural,
    FalsePositive,
}

#[derive(Debug, Serialize)]
struct ProfileDiscoveryReport {
    styles_dir: String,
    summary: ProfileDiscoverySummary,
    audited_candidates: Vec<AuditedProfileCandidate>,
    parse_errors: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
struct ProfileDiscoverySummary {
    total_audited: usize,
    journal_profile: usize,
    journal_alias: usize,
    journal_structural: usize,
    false_positive: usize,
    corrected_parent_changes: usize,
    parse_errors: usize,
}

impl ProfileDiscoverySummary {
    fn from_candidates(candidates: &[AuditedProfileCandidate], parse_errors: &[String]) -> Self {
        let mut summary = Self {
            total_audited: candidates.len() + parse_errors.len(),
            parse_errors: parse_errors.len(),
            ..Self::default()
        };

        for candidate in candidates {
            match candidate.disposition {
                AuditDisposition::JournalProfile => summary.journal_profile += 1,
                AuditDisposition::JournalAlias => summary.journal_alias += 1,
                AuditDisposition::JournalStructural => summary.journal_structural += 1,
                AuditDisposition::FalsePositive => summary.false_positive += 1,
            }

            if candidate
                .corrected_parent
                .as_deref()
                .is_some_and(|parent| parent != candidate.proposed_parent.as_str())
            {
                summary.corrected_parent_changes += 1;
            }
        }

        summary
    }
}

#[derive(Debug, Serialize)]
struct AuditedProfileCandidate {
    requested_id: String,
    normalized_id: String,
    legacy_path: String,
    title: String,
    proposed_parent: String,
    corrected_parent: Option<String>,
    disposition: AuditDisposition,
    profile_ready: bool,
    citation_format: Option<String>,
    fields: Vec<String>,
    template_target: Option<String>,
    documentation_links: Vec<String>,
    alias_evidence: Option<AliasEvidence>,
    structural_match: StructuralMatch,
    audit_note: String,
}

#[derive(Debug, Clone, Serialize)]
struct AliasEvidence {
    best_target: String,
    similarity: f32,
    citation_match: f32,
    bibliography_match: f32,
    evidence_url: Option<String>,
    confidence_note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct StructuralMatch {
    best_target: String,
    bibliography_similarity: f32,
    citation_similarity: f32,
    combined_similarity: f32,
}

#[derive(Debug, Default)]
struct RegistryCatalog {
    ids: HashSet<String>,
    alias_to_id: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    styles: Vec<RegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    id: String,
    aliases: Option<Vec<String>>,
}

#[derive(Debug)]
struct LegacyStyleMetadata {
    title: String,
    citation_format: Option<String>,
    fields: Vec<String>,
}

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

fn collect_base_semantic_sets()
-> Vec<(StyleBase, HashSet<SemanticItem>, Vec<HashSet<SemanticItem>>)> {
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
        .collect()
}

fn audit_candidate(
    styles_root: &Path,
    spec: &AuditedCandidateSpec,
    bases: &[(StyleBase, HashSet<SemanticItem>, Vec<HashSet<SemanticItem>>)],
    tracker: &ProvenanceTracker,
    renamed_styles: &HashMap<String, String>,
    alias_report: Option<&HashMap<String, AliasEvidence>>,
    registry: Option<&RegistryCatalog>,
) -> Result<AuditedProfileCandidate, String> {
    let normalized_id = normalize_style_id(spec.requested_id, renamed_styles);
    let file_path = styles_root.join(format!("{normalized_id}.csl"));
    let content = fs::read_to_string(&file_path).map_err(|e| format!("read error: {e}"))?;
    let doc = Document::parse(&content).map_err(|e| format!("parse error: {e}"))?;
    let legacy_style =
        parse_style(doc.root_element()).map_err(|e| format!("legacy parse error: {e}"))?;
    let metadata = parse_legacy_style_metadata(&doc);

    let template_target = first_template_target(&legacy_style.info.links)
        .map(|target| resolve_registry_target(&target, registry));
    let documentation_links = documentation_links(&legacy_style.info.links);
    let structural_match = resolve_structural_match(
        compute_structural_match(&legacy_style, bases, tracker),
        registry,
    );
    let alias_evidence = alias_report.and_then(|rows| rows.get(&normalized_id).cloned());

    Ok(AuditedProfileCandidate {
        requested_id: spec.requested_id.to_string(),
        normalized_id: normalized_id.clone(),
        legacy_path: file_path.display().to_string(),
        title: metadata.title,
        proposed_parent: spec.proposed_parent.to_string(),
        corrected_parent: spec.corrected_parent.map(str::to_string),
        disposition: spec.disposition,
        profile_ready: matches!(spec.disposition, AuditDisposition::JournalProfile),
        citation_format: metadata.citation_format,
        fields: metadata.fields,
        template_target,
        documentation_links,
        alias_evidence,
        structural_match,
        audit_note: spec.note.to_string(),
    })
}

fn parse_legacy_style_metadata(doc: &Document<'_>) -> LegacyStyleMetadata {
    let root = doc.root_element();
    let mut title = String::new();
    let mut citation_format = None;
    let mut fields = Vec::new();

    for child in root.children().filter(roxmltree::Node::is_element) {
        if child.tag_name().name() != "info" {
            continue;
        }

        for info_child in child.children().filter(roxmltree::Node::is_element) {
            match info_child.tag_name().name() {
                "title" if title.is_empty() => {
                    title = info_child.text().unwrap_or_default().trim().to_string();
                }
                "category" => {
                    if let Some(format) = info_child.attribute("citation-format") {
                        citation_format = Some(format.to_string());
                    }
                    if let Some(field) = info_child.attribute("field") {
                        fields.push(field.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    LegacyStyleMetadata {
        title,
        citation_format,
        fields,
    }
}

fn compute_structural_match(
    legacy_style: &csl_legacy::model::Style,
    bases: &[(StyleBase, HashSet<SemanticItem>, Vec<HashSet<SemanticItem>>)],
    tracker: &ProvenanceTracker,
) -> StructuralMatch {
    let migration_options = OptionsExtractor::extract_migration_options(legacy_style);
    let mut options = migration_options.options;
    let compiled = compilation::compile_from_xml(legacy_style, &mut options, false, tracker);

    let mut citation_template = compiled.citation.clone();
    if matches!(
        options.processing,
        Some(citum_schema::options::Processing::Numeric)
    ) {
        fixups::ensure_numeric_locator_citation_component(
            &legacy_style.citation.layout,
            &mut citation_template,
        );
    } else if legacy_style.class == "in-text" {
        fixups::normalize_author_date_locator_citation_component(
            &legacy_style.citation.layout,
            &legacy_style.macros,
            &mut citation_template,
        );
    }

    let bibliography_set = template_to_set(&compiled.bibliography);
    let citation_set = template_to_set(&citation_template);

    let mut best_match = StructuralMatch {
        best_target: String::new(),
        bibliography_similarity: 0.0,
        citation_similarity: 0.0,
        combined_similarity: 0.0,
    };

    for (base, base_bib_set, base_cit_sets) in bases {
        let bibliography_similarity = jaccard_similarity(&bibliography_set, base_bib_set);
        let citation_similarity = if base_cit_sets.is_empty() {
            1.0
        } else {
            base_cit_sets
                .iter()
                .map(|base_cit_set| jaccard_similarity(&citation_set, base_cit_set))
                .fold(0.0, f32::max)
        };
        let combined_similarity = f32::midpoint(bibliography_similarity, citation_similarity);

        if combined_similarity > best_match.combined_similarity {
            best_match = StructuralMatch {
                best_target: base.key().to_string(),
                bibliography_similarity,
                citation_similarity,
                combined_similarity,
            };
        }
    }

    best_match
}

fn first_template_target(links: &[csl_legacy::model::InfoLink]) -> Option<String> {
    links
        .iter()
        .find(|link| link.rel.as_deref() == Some("template"))
        .map(|link| short_name_from_identifier(&link.href).into_owned())
}

fn documentation_links(links: &[csl_legacy::model::InfoLink]) -> Vec<String> {
    links
        .iter()
        .filter(|link| link.rel.as_deref() == Some("documentation"))
        .map(|link| sanitize_url(&link.href))
        .collect()
}

fn load_alias_report(workspace_root: &Path) -> Option<HashMap<String, AliasEvidence>> {
    let path = workspace_root.join("scripts/report-data/alias-candidates-2026-04-19.tsv");
    let content = fs::read_to_string(path).ok()?;
    let mut rows = HashMap::new();

    for line in content.lines().skip(1) {
        let columns: Vec<_> = line.split('\t').collect();
        if columns.len() < 7 {
            continue;
        }

        let candidate_id = columns[0].to_string();
        rows.insert(
            candidate_id,
            AliasEvidence {
                best_target: columns[1].to_string(),
                similarity: columns[2].parse().unwrap_or_default(),
                citation_match: columns[3].parse().unwrap_or_default(),
                bibliography_match: columns[4].parse().unwrap_or_default(),
                evidence_url: non_empty(columns[5]),
                confidence_note: non_empty(columns[6]),
            },
        );
    }

    Some(rows)
}

fn load_renamed_styles(workspace_root: &Path) -> HashMap<String, String> {
    let path = workspace_root.join("styles-legacy/renamed-styles.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn load_registry_catalog(workspace_root: &Path) -> Option<RegistryCatalog> {
    let path = workspace_root.join("registry/default.yaml");
    let content = fs::read_to_string(path).ok()?;
    let registry: RegistryFile = serde_yaml::from_str(&content).ok()?;
    let mut catalog = RegistryCatalog::default();

    for entry in registry.styles {
        catalog.ids.insert(entry.id.clone());
        if let Some(aliases) = entry.aliases {
            for alias in aliases {
                catalog.alias_to_id.insert(alias, entry.id.clone());
            }
        }
    }

    Some(catalog)
}

fn resolve_registry_target(target: &str, registry: Option<&RegistryCatalog>) -> String {
    match registry {
        Some(registry) if registry.ids.contains(target) => target.to_string(),
        Some(registry) if target.ends_with("-core") => {
            let public_target = target.trim_end_matches("-core");
            if registry.ids.contains(public_target) {
                public_target.to_string()
            } else {
                target.to_string()
            }
        }
        Some(registry) => registry
            .alias_to_id
            .get(target)
            .cloned()
            .unwrap_or_else(|| target.to_string()),
        None => target.to_string(),
    }
}

fn resolve_structural_match(
    mut structural_match: StructuralMatch,
    registry: Option<&RegistryCatalog>,
) -> StructuralMatch {
    structural_match.best_target = resolve_registry_target(&structural_match.best_target, registry);
    structural_match
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn normalize_style_id(style_id: &str, renamed_styles: &HashMap<String, String>) -> String {
    let mut normalized = style_id.to_string();

    for (from, to) in SPECIAL_NORMALIZATIONS {
        if normalized == *from {
            normalized = (*to).to_string();
        }
    }

    renamed_styles
        .get(&normalized)
        .cloned()
        .unwrap_or(normalized)
}

fn jaccard_similarity(left: &HashSet<SemanticItem>, right: &HashSet<SemanticItem>) -> f32 {
    let intersection = left.intersection(right).count();
    let union = left.union(right).count();

    if union == 0 {
        1.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Maps a template component to semantic items; structural wrappers (Conditional, Substitute, etc.) are intentionally skipped.
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

fn sanitize_url(url: &str) -> String {
    if url.starts_with("hhttps://") {
        url[1..].to_string()
    } else {
        url.to_string()
    }
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn print_profile_discovery_report(report: &ProfileDiscoveryReport) {
    println!("=== Journal-Profile Candidate Audit ===\n");
    println!("Styles directory: {}", report.styles_dir);
    println!("Audited candidates: {}", report.summary.total_audited);
    println!(
        "Dispositions: {} journal-structural, {} false-positive, {} journal-profile, {} journal-alias",
        report.summary.journal_structural,
        report.summary.false_positive,
        report.summary.journal_profile,
        report.summary.journal_alias
    );
    println!(
        "Corrected parent mappings: {}",
        report.summary.corrected_parent_changes
    );
    println!();

    for candidate in &report.audited_candidates {
        println!(
            "{} [{}]",
            candidate.normalized_id,
            candidate.disposition_label()
        );
        println!("  proposed parent: {}", candidate.proposed_parent);
        if let Some(corrected_parent) = &candidate.corrected_parent {
            println!("  corrected parent: {corrected_parent}");
        }
        println!(
            "  structural match: {} (bib {:.2}, cit {:.2}, combined {:.2})",
            candidate.structural_match.best_target,
            candidate.structural_match.bibliography_similarity,
            candidate.structural_match.citation_similarity,
            candidate.structural_match.combined_similarity
        );
        if let Some(alias_evidence) = &candidate.alias_evidence {
            println!(
                "  alias evidence: {} (sim {:.2}, cite {:.2}, bib {:.2})",
                alias_evidence.best_target,
                alias_evidence.similarity,
                alias_evidence.citation_match,
                alias_evidence.bibliography_match
            );
        }
        if let Some(template_target) = &candidate.template_target {
            println!("  CSL template link: {template_target}");
        }
        println!("  note: {}", candidate.audit_note);
        println!();
    }

    if !report.parse_errors.is_empty() {
        println!("Parse errors:");
        for error in &report.parse_errors {
            println!("  - {error}");
        }
    }
}

impl AuditedProfileCandidate {
    fn disposition_label(&self) -> &'static str {
        match self.disposition {
            AuditDisposition::JournalProfile => "journal-profile",
            AuditDisposition::JournalAlias => "journal-alias",
            AuditDisposition::JournalStructural => "journal-structural",
            AuditDisposition::FalsePositive => "false-positive",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_special_hawaii_candidate_id() {
        let renamed = HashMap::new();
        assert_eq!(
            normalize_style_id("hawaii-int-conf-system-sciences", &renamed),
            "hawaii-international-conference-on-system-sciences-proceedings"
        );
    }

    #[test]
    fn applies_renamed_style_map_after_special_normalization() {
        let renamed = HashMap::from([(
            "hawaii-international-conference-on-system-sciences-proceedings".to_string(),
            "hawaii-proceedings".to_string(),
        )]);
        assert_eq!(
            normalize_style_id("hawaii-int-conf-system-sciences", &renamed),
            "hawaii-proceedings"
        );
    }

    #[test]
    fn parses_non_empty_values() {
        assert_eq!(non_empty(""), None);
        assert_eq!(non_empty("  "), None);
        assert_eq!(non_empty("value"), Some("value".to_string()));
    }
}
