/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::util::{short_name_from_identifier, truncate};

/// Runs the corpus-savings report for preset and locale-override opportunities.
pub fn run_savings_report(styles_dir: &str, json_output: bool) {
    match analyze_savings(Path::new(styles_dir)) {
        Ok(report) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print_savings_report(&report);
            }
        }
        Err(error) => {
            eprintln!("Failed to analyze corpus savings: {error}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Default, serde::Serialize)]
struct SavingsReport {
    total_independent_styles: u32,
    total_dependent_styles: u32,
    unique_parent_styles: u32,
    dependent_alias_savings: u32,
    locale_override_savings_high_confidence: u32,
    locale_override_savings_possible: u32,
    preset_wrapper_opportunity: u32,
    avoided_conversion_estimate_lower_bound: u32,
    avoided_conversion_estimate_upper_bound: u32,
    top_parent_families: Vec<ParentSavings>,
    parse_errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
struct ParentSavings {
    parent_short_name: String,
    dependent_alias_count: u32,
    preset_wrapper_count: u32,
    combined_opportunity_count: u32,
}

#[derive(Debug, Clone)]
struct IndependentStyleInfo {
    slug: String,
    citation_format: Option<String>,
    default_locale: Option<String>,
    template_target: Option<String>,
}

#[derive(Debug, Clone)]
struct DependentStyleInfo {
    parent_id: String,
}

fn analyze_savings(styles_dir: &Path) -> Result<SavingsReport, String> {
    let independent_styles = collect_independent_styles(styles_dir)?;
    let dependent_dir = styles_dir.join("dependent");
    let dependent_styles = if dependent_dir.exists() {
        collect_dependent_styles(&dependent_dir)?
    } else {
        Vec::new()
    };

    let mut report = SavingsReport {
        total_independent_styles: independent_styles.len() as u32,
        total_dependent_styles: dependent_styles.len() as u32,
        dependent_alias_savings: dependent_styles.len() as u32,
        ..SavingsReport::default()
    };

    let slug_to_style: HashMap<&str, &IndependentStyleInfo> = independent_styles
        .iter()
        .map(|style| (style.slug.as_str(), style))
        .collect();

    let high_confidence_locale_variants =
        collect_high_confidence_locale_variants(&independent_styles, &slug_to_style);
    let possible_locale_variants = independent_styles
        .iter()
        .filter(|style| style.default_locale.is_some())
        .count() as u32;
    let preset_wrapper_candidates =
        collect_preset_wrapper_candidates(&independent_styles, &high_confidence_locale_variants);

    let dependent_parent_counts = collect_dependent_parent_counts(&dependent_styles);
    let wrapper_parent_counts = collect_wrapper_parent_counts(&independent_styles);

    report.unique_parent_styles = dependent_parent_counts.len() as u32;
    report.locale_override_savings_high_confidence = high_confidence_locale_variants.len() as u32;
    report.locale_override_savings_possible = possible_locale_variants;
    report.preset_wrapper_opportunity = preset_wrapper_candidates as u32;
    report.avoided_conversion_estimate_lower_bound =
        report.dependent_alias_savings + report.locale_override_savings_high_confidence;
    report.avoided_conversion_estimate_upper_bound =
        report.avoided_conversion_estimate_lower_bound + report.preset_wrapper_opportunity;
    report.top_parent_families =
        build_parent_savings(&dependent_parent_counts, &wrapper_parent_counts);

    Ok(report)
}

fn collect_independent_styles(styles_dir: &Path) -> Result<Vec<IndependentStyleInfo>, String> {
    let mut styles = Vec::new();

    for entry in WalkDir::new(styles_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "csl"))
    {
        styles.push(extract_independent_style_info(entry.path())?);
    }

    Ok(styles)
}

fn collect_dependent_styles(dependent_dir: &Path) -> Result<Vec<DependentStyleInfo>, String> {
    let mut styles = Vec::new();

    for entry in WalkDir::new(dependent_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "csl"))
    {
        if let Some(style) = extract_dependent_style_info(entry.path())? {
            styles.push(style);
        }
    }

    Ok(styles)
}

fn extract_independent_style_info(path: &Path) -> Result<IndependentStyleInfo, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read error: {e}"))?;
    let doc = roxmltree::Document::parse(&content).map_err(|e| format!("parse error: {e}"))?;
    let root = doc.root_element();

    let slug = path
        .file_stem()
        .ok_or_else(|| format!("missing file stem for {}", path.display()))?
        .to_string_lossy()
        .to_string();

    let mut citation_format = None;
    let mut template_target = None;

    for child in root.children().filter(roxmltree::Node::is_element) {
        if child.tag_name().name() != "info" {
            continue;
        }

        for info_child in child.children().filter(roxmltree::Node::is_element) {
            match info_child.tag_name().name() {
                "category" => {
                    if let Some(format) = info_child.attribute("citation-format") {
                        citation_format = Some(format.to_string());
                    }
                }
                "link" if info_child.attribute("rel") == Some("template") => {
                    template_target = info_child
                        .attribute("href")
                        .map(short_name_from_identifier)
                        .map(std::borrow::Cow::into_owned);
                }
                _ => {}
            }
        }
    }

    Ok(IndependentStyleInfo {
        slug,
        citation_format,
        default_locale: root
            .attribute("default-locale")
            .map(std::string::ToString::to_string),
        template_target,
    })
}

fn extract_dependent_style_info(path: &Path) -> Result<Option<DependentStyleInfo>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read error: {e}"))?;
    let doc = roxmltree::Document::parse(&content).map_err(|e| format!("parse error: {e}"))?;
    let root = doc.root_element();

    for child in root.children().filter(roxmltree::Node::is_element) {
        if child.tag_name().name() != "info" {
            continue;
        }

        for info_child in child.children().filter(roxmltree::Node::is_element) {
            if info_child.tag_name().name() == "link"
                && info_child.attribute("rel") == Some("independent-parent")
                && let Some(parent_id) = info_child.attribute("href")
            {
                return Ok(Some(DependentStyleInfo {
                    parent_id: parent_id.to_string(),
                }));
            }
        }
    }

    Ok(None)
}

fn collect_high_confidence_locale_variants(
    independent_styles: &[IndependentStyleInfo],
    slug_to_style: &HashMap<&str, &IndependentStyleInfo>,
) -> HashSet<String> {
    independent_styles
        .iter()
        .filter_map(|style| {
            let locale = style.default_locale.as_deref()?;
            let base_slug = locale_base_slug(&style.slug, locale)?;
            let base_style = slug_to_style.get(base_slug.as_str())?;

            if citation_formats_compatible(
                style.citation_format.as_deref(),
                base_style.citation_format.as_deref(),
            ) {
                Some(style.slug.clone())
            } else {
                None
            }
        })
        .collect()
}

fn collect_preset_wrapper_candidates(
    independent_styles: &[IndependentStyleInfo],
    high_confidence_locale_variants: &HashSet<String>,
) -> usize {
    independent_styles
        .iter()
        .filter(|style| style.template_target.is_some())
        .filter(|style| !high_confidence_locale_variants.contains(&style.slug))
        .count()
}

fn collect_dependent_parent_counts(
    dependent_styles: &[DependentStyleInfo],
) -> HashMap<String, u32> {
    let mut counts = HashMap::new();
    for style in dependent_styles {
        *counts
            .entry(short_name_from_identifier(&style.parent_id).into_owned())
            .or_insert(0) += 1;
    }
    counts
}

fn collect_wrapper_parent_counts(
    independent_styles: &[IndependentStyleInfo],
) -> HashMap<String, u32> {
    let mut counts = HashMap::new();
    for style in independent_styles {
        if let Some(parent) = &style.template_target {
            *counts.entry(parent.clone()).or_insert(0) += 1;
        }
    }
    counts
}

fn build_parent_savings(
    dependent_parent_counts: &HashMap<String, u32>,
    wrapper_parent_counts: &HashMap<String, u32>,
) -> Vec<ParentSavings> {
    let all_keys: HashSet<String> = dependent_parent_counts
        .keys()
        .chain(wrapper_parent_counts.keys())
        .cloned()
        .collect();

    let mut savings: Vec<_> = all_keys
        .into_iter()
        .map(|parent_short_name| {
            let dependent_alias_count = dependent_parent_counts
                .get(&parent_short_name)
                .copied()
                .unwrap_or(0);
            let preset_wrapper_count = wrapper_parent_counts
                .get(&parent_short_name)
                .copied()
                .unwrap_or(0);
            ParentSavings {
                parent_short_name,
                dependent_alias_count,
                preset_wrapper_count,
                combined_opportunity_count: dependent_alias_count + preset_wrapper_count,
            }
        })
        .collect();

    savings.sort_by(|left, right| {
        right
            .combined_opportunity_count
            .cmp(&left.combined_opportunity_count)
            .then_with(|| right.dependent_alias_count.cmp(&left.dependent_alias_count))
            .then_with(|| left.parent_short_name.cmp(&right.parent_short_name))
    });
    savings.truncate(15);
    savings
}

fn citation_formats_compatible(left: Option<&str>, right: Option<&str>) -> bool {
    left.is_none() || right.is_none() || left == right
}

fn locale_base_slug(slug: &str, locale: &str) -> Option<String> {
    let locale_suffix = locale_suffix_from_locale(locale)?;
    let explicit_suffix = format!("-{locale_suffix}");
    if let Some(base) = slug.strip_suffix(&explicit_suffix) {
        return Some(base.to_string());
    }

    let locale_parts: Vec<_> = locale.split('-').collect();
    if locale_parts.len() > 1 {
        let language_suffix = format!("-{}", locale_parts[0].to_lowercase());
        if let Some(base) = slug.strip_suffix(&language_suffix) {
            return Some(base.to_string());
        }
    }

    None
}

fn locale_suffix_from_locale(locale: &str) -> Option<String> {
    let mut parts = locale.split('-');
    let language = parts.next()?;
    let region = parts.next();

    match region {
        Some(region) if region.len() == 2 => Some(format!(
            "{}-{}",
            language.to_lowercase(),
            region.to_uppercase()
        )),
        _ => Some(language.to_lowercase()),
    }
}

fn print_savings_report(report: &SavingsReport) {
    println!("=== CSL Corpus Savings Report ===\n");
    println!("Independent styles: {}", report.total_independent_styles);
    println!("Dependent styles:   {}", report.total_dependent_styles);
    println!("Unique parents:     {}", report.unique_parent_styles);
    println!();
    println!("Certain savings:");
    println!(
        "  dependent alias savings: {}",
        report.dependent_alias_savings
    );
    println!(
        "  locale override savings (high confidence): {}",
        report.locale_override_savings_high_confidence
    );
    println!();
    println!("Heuristic opportunity:");
    println!(
        "  locale override savings (possible): {}",
        report.locale_override_savings_possible
    );
    println!(
        "  preset wrapper opportunity: {}",
        report.preset_wrapper_opportunity
    );
    println!();
    println!(
        "Lower-bound avoided conversions: {}",
        report.avoided_conversion_estimate_lower_bound
    );
    println!(
        "Upper-bound avoided conversions: {}",
        report.avoided_conversion_estimate_upper_bound
    );
    println!();
    println!("Top parent families by opportunity:\n");
    println!(
        "{:4}  {:40} {:>10} {:>10} {:>10}",
        "Rank", "Parent", "Aliases", "Wrappers", "Combined"
    );
    println!("{}", "-".repeat(84));
    for (index, family) in report.top_parent_families.iter().enumerate() {
        println!(
            "{:4}  {:40} {:>10} {:>10} {:>10}",
            index + 1,
            truncate(&family.parent_short_name, 40),
            family.dependent_alias_count,
            family.preset_wrapper_count,
            family.combined_opportunity_count
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ParentSavings, SavingsReport, analyze_savings, locale_base_slug, locale_suffix_from_locale,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn locale_suffix_normalizes_language_and_region() {
        assert_eq!(locale_suffix_from_locale("de-DE").as_deref(), Some("de-DE"));
        assert_eq!(locale_suffix_from_locale("fr").as_deref(), Some("fr"));
    }

    #[test]
    fn locale_base_slug_matches_language_only_variant() {
        assert_eq!(
            locale_base_slug("chicago-author-date-de", "de-DE").as_deref(),
            Some("chicago-author-date")
        );
        assert_eq!(locale_base_slug("apa-tr", "tr-TR").as_deref(), Some("apa"));
    }

    #[test]
    fn analyze_savings_counts_aliases_locale_variants_and_wrappers() {
        let fixture_dir = create_fixture_dir("savings-fixture");
        fs::create_dir_all(fixture_dir.join("dependent")).unwrap();

        write_style(
            &fixture_dir.join("chicago-author-date.csl"),
            independent_style_xml(None, None, Some("author-date")),
        );
        write_style(
            &fixture_dir.join("chicago-author-date-de.csl"),
            independent_style_xml(Some("de-DE"), None, Some("author-date")),
        );
        write_style(
            &fixture_dir.join("taylor-and-francis-chicago-author-date.csl"),
            independent_style_xml(
                None,
                Some("http://www.zotero.org/styles/chicago-author-date"),
                Some("author-date"),
            ),
        );
        write_style(
            &fixture_dir.join("plain-de.csl"),
            independent_style_xml(Some("de-DE"), None, Some("numeric")),
        );
        write_style(
            &fixture_dir.join("dependent/journal.csl"),
            dependent_style_xml("http://www.zotero.org/styles/chicago-author-date"),
        );

        let report = analyze_savings(&fixture_dir).unwrap();

        assert_eq!(report_summary(&report), (4, 1, 1, 1, 1, 2, 1, 2));
        assert_eq!(
            report.top_parent_families.first(),
            Some(&ParentSavings {
                parent_short_name: "chicago-author-date".to_string(),
                dependent_alias_count: 1,
                preset_wrapper_count: 1,
                combined_opportunity_count: 2,
            })
        );
    }

    #[test]
    fn corpus_totals_match_known_reference_counts() {
        let Some(styles_legacy_dir) = find_styles_legacy_dir() else {
            return;
        };
        let report = analyze_savings(&styles_legacy_dir).unwrap();
        if report.total_independent_styles == 0 {
            return;
        }

        assert_eq!(report.total_independent_styles, 2_844);
        assert_eq!(report.total_dependent_styles, 7_987);
        assert_eq!(report.unique_parent_styles, 298);
    }

    fn report_summary(report: &SavingsReport) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        (
            report.total_independent_styles,
            report.total_dependent_styles,
            report.unique_parent_styles,
            report.dependent_alias_savings,
            report.locale_override_savings_high_confidence,
            report.locale_override_savings_possible,
            report.preset_wrapper_opportunity,
            report.avoided_conversion_estimate_lower_bound,
        )
    }

    fn create_fixture_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn find_styles_legacy_dir() -> Option<PathBuf> {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find_map(|ancestor| {
                let candidate = ancestor.join("styles-legacy");
                if candidate.is_dir() {
                    Some(candidate)
                } else {
                    None
                }
            })
    }

    fn write_style(path: &Path, content: String) {
        fs::write(path, content).unwrap();
    }

    fn independent_style_xml(
        default_locale: Option<&str>,
        template_href: Option<&str>,
        citation_format: Option<&str>,
    ) -> String {
        let locale_attr = default_locale
            .map(|locale| format!(" default-locale=\"{locale}\""))
            .unwrap_or_default();
        let category = citation_format
            .map(|format| format!("<category citation-format=\"{format}\"/>"))
            .unwrap_or_default();
        let template = template_href
            .map(|href| format!("<link rel=\"template\" href=\"{href}\"/>"))
            .unwrap_or_default();

        format!(
            "<style xmlns=\"http://purl.org/net/xbiblio/csl\" version=\"1.0\" class=\"in-text\"{locale_attr}><info>{category}{template}</info></style>"
        )
    }

    fn dependent_style_xml(parent_href: &str) -> String {
        format!(
            "<style xmlns=\"http://purl.org/net/xbiblio/csl\" version=\"1.0\"><info><link rel=\"independent-parent\" href=\"{parent_href}\"/></info></style>"
        )
    }
}
