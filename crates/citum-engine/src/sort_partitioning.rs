/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Multilingual bibliography partitioning helpers.

use std::cmp::Ordering;
use std::collections::HashMap;

use citum_schema::grouping::NameSortOrder;
use citum_schema::locale::Locale;
use citum_schema::options::{
    BibliographyPartitionKind, BibliographyPartitionMode, BibliographySortPartitioning,
};
use unicode_script::{Script, UnicodeScript};

use crate::reference::Reference;
use crate::sort_support::author_sort_key_opt;

/// Whether partitioning should affect flat bibliography sort order.
pub(crate) fn should_sort_flat(partitioning: &BibliographySortPartitioning) -> bool {
    matches!(
        partitioning.mode,
        BibliographyPartitionMode::SortOnly | BibliographyPartitionMode::SortAndSections
    )
}

/// Whether partitioning should render visible grouped bibliography sections.
pub(crate) fn should_render_sections(partitioning: &BibliographySortPartitioning) -> bool {
    matches!(
        partitioning.mode,
        BibliographyPartitionMode::Sections | BibliographyPartitionMode::SortAndSections
    )
}

/// Derive the bibliography partition key for one reference.
pub(crate) fn partition_key(
    reference: &Reference,
    locale: &Locale,
    partitioning: &BibliographySortPartitioning,
) -> Option<String> {
    match partitioning.by {
        BibliographyPartitionKind::Script => script_partition_key(reference, locale),
        BibliographyPartitionKind::Language => language_partition_key(reference),
    }
}

/// Sort references by partition while preserving existing order inside partitions.
pub(crate) fn sort_by_partition(
    references: &mut [&Reference],
    locale: &Locale,
    partitioning: &BibliographySortPartitioning,
) {
    let ordering = PartitionOrdering::new(partitioning);
    let mut cached = cache_partitioned_references(references.iter().copied(), locale, partitioning);
    cached.sort_by(|left, right| ordering.compare_keys(left.key.as_deref(), right.key.as_deref()));

    for (slot, entry) in references.iter_mut().zip(cached) {
        *slot = entry.reference;
    }
}

/// Partition references into section buckets ordered by configured partition rank.
pub(crate) fn partition_references<'a>(
    references: Vec<&'a Reference>,
    locale: &Locale,
    partitioning: &BibliographySortPartitioning,
) -> Vec<(Option<String>, Vec<&'a Reference>)> {
    let ordering = PartitionOrdering::new(partitioning);
    let mut grouped: HashMap<Option<String>, Vec<&'a Reference>> = HashMap::new();

    for entry in cache_partitioned_references(references, locale, partitioning) {
        grouped
            .entry(entry.key.clone())
            .or_default()
            .push(entry.reference);
    }

    let mut keys = grouped.keys().cloned().collect::<Vec<_>>();
    keys.sort_by(|left, right| ordering.compare_keys(left.as_deref(), right.as_deref()));

    keys.into_iter()
        .filter_map(|key| grouped.remove(&key).map(|entries| (key, entries)))
        .collect()
}

struct PartitionOrdering {
    configured_ranks: HashMap<String, usize>,
}

impl PartitionOrdering {
    fn new(partitioning: &BibliographySortPartitioning) -> Self {
        Self {
            configured_ranks: partitioning
                .order
                .iter()
                .cloned()
                .enumerate()
                .map(|(rank, key)| (key, rank))
                .collect(),
        }
    }

    fn compare_keys(&self, left: Option<&str>, right: Option<&str>) -> Ordering {
        self.partition_rank(left)
            .cmp(&self.partition_rank(right))
            .then_with(|| match (left, right) {
                (Some(left_key), Some(right_key)) => left_key.cmp(right_key),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            })
    }

    fn partition_rank(&self, partition_key: Option<&str>) -> usize {
        let Some(partition_key) = partition_key else {
            return usize::MAX;
        };

        self.configured_ranks
            .get(partition_key)
            .copied()
            .unwrap_or(usize::MAX - 1)
    }
}

struct PartitionedReference<'a> {
    reference: &'a Reference,
    key: Option<String>,
}

fn cache_partitioned_references<'a>(
    references: impl IntoIterator<Item = &'a Reference>,
    locale: &Locale,
    partitioning: &BibliographySortPartitioning,
) -> Vec<PartitionedReference<'a>> {
    references
        .into_iter()
        .map(|reference| PartitionedReference {
            key: partition_key(reference, locale, partitioning),
            reference,
        })
        .collect()
}

fn language_partition_key(reference: &Reference) -> Option<String> {
    crate::values::effective_item_language(reference).and_then(non_empty_key)
}

fn script_partition_key(reference: &Reference, locale: &Locale) -> Option<String> {
    let sort_key = author_sort_key_opt(reference, NameSortOrder::FamilyGiven, locale, true)?;
    sort_key.chars().find_map(script_code_for_char)
}

fn script_code_for_char(ch: char) -> Option<String> {
    let script = ch.script();
    if matches!(script, Script::Common | Script::Inherited | Script::Unknown) {
        return None;
    }

    let code = script.short_name();
    match code {
        "Hant" | "Hans" | "Jpan" | "Kore" => Some("Hani".to_string()),
        _ => Some(code.to_string()),
    }
}

fn non_empty_key(key: String) -> Option<String> {
    let trimmed = key.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
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
    use serde_json::json;

    fn partitioning(by: BibliographyPartitionKind) -> BibliographySortPartitioning {
        BibliographySortPartitioning {
            by,
            mode: BibliographyPartitionMode::SortOnly,
            order: Vec::new(),
            headings: std::collections::HashMap::new(),
        }
    }

    fn reference(id: &str, family: Option<&str>, title: &str, language: Option<&str>) -> Reference {
        let mut value = json!({
            "id": id,
            "type": "book",
            "title": title
        });
        if let Some(family) = family {
            value["author"] = json!([{"family": family, "given": "Test"}]);
        }
        if let Some(language) = language {
            value["language"] = json!(language);
        }

        let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(value).unwrap();
        legacy.into()
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "Comprehensive script detection test cases."
    )]
    fn detects_common_script_partition_keys() {
        let locale = Locale::en_us();
        let script = partitioning(BibliographyPartitionKind::Script);

        assert_eq!(
            partition_key(
                &reference("latn", Some("Smith"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Latn")
        );
        assert_eq!(
            partition_key(
                &reference("cyrl", Some("Пушкин"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Cyrl")
        );
        assert_eq!(
            partition_key(
                &reference("arab", Some("الغزالي"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Arab")
        );
        assert_eq!(
            partition_key(
                &reference("hani", Some("乌云"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Hani")
        );
        assert_eq!(
            partition_key(
                &reference("hira", Some("あおい"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Hira")
        );
        assert_eq!(
            partition_key(
                &reference("kana", Some("アオイ"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Kana")
        );
        assert_eq!(
            partition_key(
                &reference("hang", Some("김"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Hang")
        );

        // Normalized CJK scripts should all map to "Hani"
        assert_eq!(
            partition_key(
                &reference("hans", Some("张"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Hani")
        );
        assert_eq!(
            partition_key(
                &reference("hant", Some("張"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Hani")
        );
        assert_eq!(
            partition_key(
                &reference("jpan", Some("佐藤"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Hani")
        );
        assert_eq!(
            partition_key(
                &reference("kore", Some("김"), "Title", None),
                &locale,
                &script
            )
            .as_deref(),
            Some("Hang")
        );
    }

    #[test]
    fn skips_punctuation_before_script_detection() {
        let locale = Locale::en_us();
        let script = partitioning(BibliographyPartitionKind::Script);
        let reference = reference("punct", Some("...Пушкин"), "Title", None);

        assert_eq!(
            partition_key(&reference, &locale, &script).as_deref(),
            Some("Cyrl")
        );
    }

    #[test]
    fn falls_back_to_title_for_script_detection() {
        let locale = Locale::en_us();
        let script = partitioning(BibliographyPartitionKind::Script);
        let reference = reference("title", None, "東京", None);

        assert_eq!(
            partition_key(&reference, &locale, &script).as_deref(),
            Some("Hani")
        );
    }

    #[test]
    fn returns_none_for_unknown_script_key() {
        let locale = Locale::en_us();
        let script = partitioning(BibliographyPartitionKind::Script);
        let reference = reference("unknown", None, "1234", None);

        assert_eq!(partition_key(&reference, &locale, &script), None);
    }

    #[test]
    fn language_partitioning_uses_effective_item_language() {
        let locale = Locale::en_us();
        let language = partitioning(BibliographyPartitionKind::Language);
        let reference = reference("ru", Some("Пушкин"), "Title", Some("ru"));

        assert_eq!(
            partition_key(&reference, &locale, &language).as_deref(),
            Some("ru")
        );
    }

    #[test]
    fn partitions_references_in_configured_order() {
        let locale = Locale::en_us();
        let partitioning = BibliographySortPartitioning {
            by: BibliographyPartitionKind::Language,
            mode: BibliographyPartitionMode::Sections,
            order: vec!["ru".to_string(), "en".to_string()],
            headings: std::collections::HashMap::new(),
        };
        let ru = reference("ru", Some("Пушкин"), "Title", Some("ru"));
        let en = reference("en", Some("Smith"), "Title", Some("en"));
        let unknown = reference("unknown", Some("Doe"), "Title", None);

        let partitioned = partition_references(vec![&en, &unknown, &ru], &locale, &partitioning);

        assert_eq!(
            partitioned
                .iter()
                .map(|(key, refs)| (
                    key.as_deref(),
                    refs.iter()
                        .map(|reference| reference.id().expect("test fixture ids").to_string())
                        .collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>(),
            vec![
                (Some("ru"), vec!["ru".to_string()]),
                (Some("en"), vec!["en".to_string()]),
                (None, vec!["unknown".to_string()]),
            ]
        );
    }
}
