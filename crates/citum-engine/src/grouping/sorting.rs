/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Group-specific sorting for bibliography grouping.
//!
//! This module implements per-group sorting with support for:
//! - Type-order sorting (explicit sequence like [legal-case, statute, treaty])
//! - Name-order sorting (family-given vs given-family for multilingual bibliographies)
//! - Integration with standard sort keys (author, title, issued)

use std::collections::HashMap;

use citum_schema::grouping::{GroupSort, GroupSortKey, NameSortOrder, SortKey as GroupSortKeyType};
use citum_schema::locale::Locale;

use crate::reference::Reference;
use crate::sort_support::{TextCollator, author_sort_key_opt, normalize_sort_text, title_sort_key};

fn compare_optional_years(a_year: Option<i32>, b_year: Option<i32>) -> std::cmp::Ordering {
    match (a_year, b_year) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

/// Sorts grouped bibliography entries using group-specific sort rules.
pub struct GroupSorter<'a> {
    locale: &'a Locale,
    text_collator: TextCollator,
}

struct CachedReference<'a> {
    reference: &'a Reference,
    sort_values: Vec<CachedSortValue>,
}

enum CachedSortValue {
    RefType { name: String, rank: Option<usize> },
    OptionalText(Option<String>),
    Text(String),
    Issued(Option<i32>),
}

enum CompiledSortKey<'a> {
    RefType {
        ascending: bool,
        rank_by_type: Option<HashMap<String, usize>>,
    },
    Author {
        ascending: bool,
        name_order: NameSortOrder,
    },
    Title {
        ascending: bool,
    },
    Issued {
        ascending: bool,
    },
    Field {
        ascending: bool,
        field_name: &'a str,
    },
}

impl<'a> GroupSorter<'a> {
    /// Create a sorter that uses `locale` for locale-sensitive comparisons.
    #[must_use]
    pub fn new(locale: &'a Locale) -> Self {
        Self {
            locale,
            text_collator: TextCollator::new(locale),
        }
    }

    /// Sort references according to a group sort specification.
    ///
    /// Applies sort keys in order, with later keys acting as tiebreakers.
    ///
    /// # Arguments
    ///
    /// * `references` - References to sort
    /// * `sort_spec` - Group sort specification
    #[must_use]
    pub fn sort_references<'b>(
        &self,
        mut references: Vec<&'b Reference>,
        sort_spec: &GroupSort,
    ) -> Vec<&'b Reference> {
        let compiled_keys = self.compile_sort_keys(sort_spec);
        let mut cached_references = references
            .drain(..)
            .map(|reference| CachedReference {
                reference,
                sort_values: compiled_keys
                    .iter()
                    .map(|sort_key| self.cache_sort_value(reference, sort_key))
                    .collect(),
            })
            .collect::<Vec<_>>();

        cached_references.sort_by(|a, b| self.compare_cached_references(a, b, &compiled_keys));
        cached_references
            .into_iter()
            .map(|entry| entry.reference)
            .collect()
    }

    /// Compare two references by a single sort key.
    #[must_use]
    pub fn compare_by_key(
        &self,
        a: &Reference,
        b: &Reference,
        sort_key: &GroupSortKey,
    ) -> std::cmp::Ordering {
        self.compare_by_key_with_context(a, b, sort_key)
    }

    fn compare_by_key_with_context(
        &self,
        a: &Reference,
        b: &Reference,
        sort_key: &GroupSortKey,
    ) -> std::cmp::Ordering {
        let cmp = match &sort_key.key {
            GroupSortKeyType::RefType => sort_key.order.as_ref().map_or_else(
                || a.ref_type().cmp(&b.ref_type()),
                |order| Self::compare_by_type_order(a, b, order),
            ),
            GroupSortKeyType::Author => sort_key.sort_order.as_ref().map_or_else(
                || self.compare_by_author_with_order(a, b, NameSortOrder::FamilyGiven),
                |name_order| self.compare_by_author_with_order(a, b, *name_order),
            ),
            GroupSortKeyType::Title => self.compare_by_title(a, b),
            GroupSortKeyType::Issued => Self::compare_by_issued(a, b),
            GroupSortKeyType::Field(field_name) => Self::compare_by_field(a, b, field_name),
        };

        if sort_key.ascending {
            cmp
        } else {
            cmp.reverse()
        }
    }

    /// Compare by type using explicit order sequence.
    ///
    /// Types appear in the order specified, regardless of alphabetical content.
    /// Types not in the order list sort after those in the list, alphabetically.
    fn compare_by_type_order(a: &Reference, b: &Reference, order: &[String]) -> std::cmp::Ordering {
        let a_type = a.ref_type();
        let b_type = b.ref_type();

        let a_pos = order.iter().position(|t| t == &a_type);
        let b_pos = order.iter().position(|t| t == &b_type);

        match (a_pos, b_pos) {
            (Some(a_idx), Some(b_idx)) => a_idx.cmp(&b_idx),
            (Some(_), None) => std::cmp::Ordering::Less, // a in order, b not
            (None, Some(_)) => std::cmp::Ordering::Greater, // b in order, a not
            (None, None) => a_type.cmp(&b_type),         // both not in order, alphabetical
        }
    }

    fn compile_sort_keys<'b>(&self, sort_spec: &'b GroupSort) -> Vec<CompiledSortKey<'b>> {
        sort_spec
            .template
            .iter()
            .map(|sort_key| match &sort_key.key {
                GroupSortKeyType::RefType => CompiledSortKey::RefType {
                    ascending: sort_key.ascending,
                    rank_by_type: sort_key.order.as_ref().map(|order| {
                        order
                            .iter()
                            .enumerate()
                            .map(|(index, ref_type)| (ref_type.clone(), index))
                            .collect()
                    }),
                },
                GroupSortKeyType::Author => CompiledSortKey::Author {
                    ascending: sort_key.ascending,
                    name_order: sort_key.sort_order.unwrap_or(NameSortOrder::FamilyGiven),
                },
                GroupSortKeyType::Title => CompiledSortKey::Title {
                    ascending: sort_key.ascending,
                },
                GroupSortKeyType::Issued => CompiledSortKey::Issued {
                    ascending: sort_key.ascending,
                },
                GroupSortKeyType::Field(field_name) => CompiledSortKey::Field {
                    ascending: sort_key.ascending,
                    field_name,
                },
            })
            .collect()
    }

    fn cache_sort_value(
        &self,
        reference: &Reference,
        sort_key: &CompiledSortKey<'_>,
    ) -> CachedSortValue {
        match sort_key {
            CompiledSortKey::RefType { rank_by_type, .. } => {
                let ref_type = reference.ref_type();
                CachedSortValue::RefType {
                    name: ref_type.clone(),
                    rank: rank_by_type
                        .as_ref()
                        .and_then(|ranks| ranks.get(&ref_type).copied()),
                }
            }
            CompiledSortKey::Author { name_order, .. } => CachedSortValue::OptionalText(
                self.extract_author_sort_key_opt(reference, *name_order),
            ),
            CompiledSortKey::Title { .. } => CachedSortValue::Text(self.title_sort_key(reference)),
            CompiledSortKey::Issued { .. } => CachedSortValue::Issued(Self::issued_year(reference)),
            CompiledSortKey::Field { field_name, .. } => {
                CachedSortValue::Text(Self::field_sort_value(reference, field_name))
            }
        }
    }

    fn compare_cached_references(
        &self,
        a: &CachedReference<'_>,
        b: &CachedReference<'_>,
        compiled_keys: &[CompiledSortKey<'_>],
    ) -> std::cmp::Ordering {
        for (index, sort_key) in compiled_keys.iter().enumerate() {
            let cmp = self.compare_cached_value(&a.sort_values[index], &b.sort_values[index]);
            let cmp = if Self::is_ascending(sort_key) {
                cmp
            } else {
                cmp.reverse()
            };

            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }
        }

        std::cmp::Ordering::Equal
    }

    fn compare_cached_value(&self, a: &CachedSortValue, b: &CachedSortValue) -> std::cmp::Ordering {
        match (a, b) {
            (
                CachedSortValue::RefType {
                    name: a_name,
                    rank: a_rank,
                },
                CachedSortValue::RefType {
                    name: b_name,
                    rank: b_rank,
                },
            ) => match (a_rank, b_rank) {
                (Some(a_idx), Some(b_idx)) => a_idx.cmp(b_idx),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a_name.cmp(b_name),
            },
            (CachedSortValue::OptionalText(a_text), CachedSortValue::OptionalText(b_text)) => {
                match (a_text, b_text) {
                    (Some(a_value), Some(b_value)) => self.text_collator.compare(a_value, b_value),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            }
            (CachedSortValue::Text(a_text), CachedSortValue::Text(b_text)) => {
                self.text_collator.compare(a_text, b_text)
            }
            (CachedSortValue::Issued(a_year), CachedSortValue::Issued(b_year)) => {
                compare_optional_years(*a_year, *b_year)
            }
            _ => std::cmp::Ordering::Equal,
        }
    }

    fn is_ascending(sort_key: &CompiledSortKey<'_>) -> bool {
        match sort_key {
            CompiledSortKey::RefType { ascending, .. }
            | CompiledSortKey::Author { ascending, .. }
            | CompiledSortKey::Title { ascending }
            | CompiledSortKey::Issued { ascending }
            | CompiledSortKey::Field { ascending, .. } => *ascending,
        }
    }

    /// Compare by author with culturally appropriate name ordering.
    fn compare_by_author_with_order(
        &self,
        a: &Reference,
        b: &Reference,
        name_order: NameSortOrder,
    ) -> std::cmp::Ordering {
        let a_key = self.extract_author_sort_key_opt(a, name_order);
        let b_key = self.extract_author_sort_key_opt(b, name_order);
        match (a_key, b_key) {
            (Some(a), Some(b)) => self.text_collator.compare(&a, &b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    }

    /// Extract author sort key with specified name ordering.
    ///
    /// Unlike generic bibliography sorting, author-key sorting follows CSL
    /// semantics for name keys: items without author/editor names are treated
    /// as missing-name entries and sort after named entries.
    fn extract_author_sort_key_opt(
        &self,
        reference: &Reference,
        name_order: NameSortOrder,
    ) -> Option<String> {
        author_sort_key_opt(reference, name_order, self.locale, true)
    }

    /// Public helper retained for tests/debugging.
    #[must_use]
    pub fn extract_author_sort_key(
        &self,
        reference: &Reference,
        name_order: NameSortOrder,
    ) -> String {
        self.extract_author_sort_key_opt(reference, name_order)
            .unwrap_or_default()
    }

    /// Compare by title (with article stripping).
    fn compare_by_title(&self, a: &Reference, b: &Reference) -> std::cmp::Ordering {
        let a_title = self.title_sort_key(a);
        let b_title = self.title_sort_key(b);
        self.text_collator.compare(&a_title, &b_title)
    }

    /// Compare by issued date.
    fn compare_by_issued(a: &Reference, b: &Reference) -> std::cmp::Ordering {
        let a_year = Self::issued_year(a);
        let b_year = Self::issued_year(b);
        compare_optional_years(a_year, b_year)
    }

    /// Compare by custom field.
    fn compare_by_field(a: &Reference, b: &Reference, field_name: &str) -> std::cmp::Ordering {
        Self::field_sort_value(a, field_name).cmp(&Self::field_sort_value(b, field_name))
    }

    fn title_sort_key(&self, reference: &Reference) -> String {
        title_sort_key(reference, self.locale)
    }

    fn issued_year(reference: &Reference) -> Option<i32> {
        reference
            .csl_issued_date()
            .and_then(|d| d.year().parse::<i32>().ok())
            .filter(|year| *year != 0)
    }

    fn field_sort_value(reference: &Reference, field_name: &str) -> String {
        match field_name {
            "language" => normalize_sort_text(reference.language().unwrap_or_default().as_ref()),
            // Future: support for keywords, custom metadata
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::grouping::GroupSortKey;

    fn make_locale() -> Locale {
        Locale::en_us()
    }

    fn make_reference(
        id: &str,
        ref_type: &str,
        author_family: &str,
        title: &str,
        year: i32,
    ) -> Reference {
        let json = serde_json::json!({
            "id": id,
            "type": ref_type,
            "author": [{"family": author_family, "given": "Test"}],
            "issued": {"date-parts": [[year]]},
            "title": title,
            "container-title": "Test Container",
        });
        let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(json).unwrap();
        legacy.into()
    }

    fn make_reference_no_author(id: &str, ref_type: &str, title: &str, year: i32) -> Reference {
        let json = serde_json::json!({
            "id": id,
            "type": ref_type,
            "issued": {"date-parts": [[year]]},
            "title": title,
            "container-title": "Test Container",
        });
        let legacy: csl_legacy::csl_json::Reference = serde_json::from_value(json).unwrap();
        legacy.into()
    }

    #[test]
    fn test_type_order_sorting() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        // Use standard CSL JSON types for testing
        let journal = make_reference("r1", "article-journal", "Smith", "Title J", 1990);
        let magazine = make_reference("r2", "article-magazine", "Jones", "Title M", 2000);
        let newspaper = make_reference("r3", "article-newspaper", "Brown", "Title N", 1985);
        let book = make_reference("r4", "book", "Davis", "Title B", 1995);

        let mut refs = vec![&book, &newspaper, &journal, &magazine];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::RefType,
                ascending: true,
                order: Some(vec![
                    "article-journal".to_string(),
                    "article-magazine".to_string(),
                    "article-newspaper".to_string(),
                ]),
                sort_order: None,
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        // Should be: article-journal, article-magazine, article-newspaper, then book (alphabetically after)
        assert_eq!(refs[0].id().unwrap(), "r1"); // article-journal
        assert_eq!(refs[1].id().unwrap(), "r2"); // article-magazine
        assert_eq!(refs[2].id().unwrap(), "r3"); // article-newspaper
        assert_eq!(refs[3].id().unwrap(), "r4"); // book
    }

    #[test]
    fn test_author_family_given_order() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let smith = make_reference("r1", "book", "Smith", "Title", 2000);
        let jones = make_reference("r2", "book", "Jones", "Title", 2000);
        let brown = make_reference("r3", "book", "Brown", "Title", 2000);

        let mut refs = vec![&smith, &jones, &brown];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::Author,
                ascending: true,
                order: None,
                sort_order: Some(NameSortOrder::FamilyGiven),
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        // Should be alphabetical by family name
        assert_eq!(refs[0].id().unwrap(), "r3"); // Brown
        assert_eq!(refs[1].id().unwrap(), "r2"); // Jones
        assert_eq!(refs[2].id().unwrap(), "r1"); // Smith
    }

    #[test]
    fn test_author_sort_uses_unicode_collation_for_accented_names() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let celik = make_reference("r1", "book", "Çelik", "Title", 2000);
        let zimring = make_reference("r2", "book", "Zimring", "Title", 2000);
        let o_tuathail = make_reference("r3", "book", "Ó Tuathail", "Title", 2000);

        let mut refs = vec![&o_tuathail, &zimring, &celik];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::Author,
                ascending: true,
                order: None,
                sort_order: Some(NameSortOrder::FamilyGiven),
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        assert_eq!(refs[0].id().unwrap(), "r1");
        assert_eq!(refs[1].id().unwrap(), "r3");
        assert_eq!(refs[2].id().unwrap(), "r2");
    }

    #[test]
    fn test_title_sort_uses_unicode_collation_for_accented_titles() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let accent = make_reference_no_author("r1", "book", "Órbitas del sur", 2000);
        let plain = make_reference_no_author("r2", "book", "Origins of Theory", 2000);
        let zeta = make_reference_no_author("r3", "book", "Zebra Studies", 2000);

        let mut refs = vec![&zeta, &plain, &accent];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::Title,
                ascending: true,
                order: None,
                sort_order: None,
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        assert_eq!(refs[0].id().unwrap(), "r1");
        assert_eq!(refs[1].id().unwrap(), "r2");
        assert_eq!(refs[2].id().unwrap(), "r3");
    }

    #[test]
    fn test_issued_descending() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let old = make_reference("r1", "book", "Smith", "Title", 1990);
        let new = make_reference("r2", "book", "Jones", "Title", 2020);
        let mid = make_reference("r3", "book", "Brown", "Title", 2005);

        let mut refs = vec![&old, &new, &mid];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::Issued,
                ascending: false, // Descending
                order: None,
                sort_order: None,
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        // Should be newest first
        assert_eq!(refs[0].id().unwrap(), "r2"); // 2020
        assert_eq!(refs[1].id().unwrap(), "r3"); // 2005
        assert_eq!(refs[2].id().unwrap(), "r1"); // 1990
    }

    #[test]
    fn test_issued_ascending_places_undated_last() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let dated_early = make_reference("r1", "book", "Smith", "Book D", 1999);
        let dated_late = make_reference("r2", "book", "Jones", "Book B", 2000);
        let mut undated = make_reference("r3", "book", "Brown", "Book A", 2000);
        if let Reference::Monograph(monograph) = &mut undated {
            monograph.issued = citum_schema::reference::EdtfString(String::new());
        }

        let mut refs = vec![&undated, &dated_late, &dated_early];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::Issued,
                ascending: true,
                order: None,
                sort_order: None,
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        assert_eq!(refs[0].id().unwrap(), "r1");
        assert_eq!(refs[1].id().unwrap(), "r2");
        assert_eq!(refs[2].id().unwrap(), "r3");
    }

    #[test]
    fn test_issued_sort_uses_created_when_issued_is_missing() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let dated = make_reference("r1", "book", "Smith", "Book D", 1999);
        let mut created_only = make_reference("r2", "book", "Jones", "Book C", 2000);
        if let Reference::Monograph(monograph) = &mut created_only {
            monograph.created = citum_schema::reference::EdtfString("1985".to_string());
            monograph.issued = citum_schema::reference::EdtfString(String::new());
        }

        let mut refs = vec![&dated, &created_only];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::Issued,
                ascending: true,
                order: None,
                sort_order: None,
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        assert_eq!(refs[0].id().unwrap(), "r2");
        assert_eq!(refs[1].id().unwrap(), "r1");
    }

    #[test]
    fn test_composite_sort() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let smith2020 = make_reference("r1", "book", "Smith", "Title", 2020);
        let smith2010 = make_reference("r2", "book", "Smith", "Title", 2010);
        let jones2020 = make_reference("r3", "book", "Jones", "Title", 2020);

        let mut refs = vec![&smith2020, &jones2020, &smith2010];

        let sort_spec = GroupSort {
            template: vec![
                GroupSortKey {
                    key: GroupSortKeyType::Author,
                    ascending: true,
                    order: None,
                    sort_order: Some(NameSortOrder::FamilyGiven),
                },
                GroupSortKey {
                    key: GroupSortKeyType::Issued,
                    ascending: false, // Descending within author
                    order: None,
                    sort_order: None,
                },
            ],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        // Should be: Jones 2020, then Smith 2020, then Smith 2010
        assert_eq!(refs[0].id().unwrap(), "r3"); // Jones 2020
        assert_eq!(refs[1].id().unwrap(), "r1"); // Smith 2020
        assert_eq!(refs[2].id().unwrap(), "r2"); // Smith 2010
    }

    #[test]
    fn test_author_sort_falls_back_to_title_for_missing_names() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let no_author = make_reference_no_author("r1", "legal-case", "Brown v. Board", 1954);
        let brown = make_reference("r2", "book", "Brown", "Title", 2000);
        let smith = make_reference("r3", "book", "Smith", "Title", 2000);

        let mut refs = vec![&no_author, &smith, &brown];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::Author,
                ascending: true,
                order: None,
                sort_order: Some(NameSortOrder::FamilyGiven),
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        assert_eq!(refs[0].id().unwrap(), "r2"); // Brown
        assert_eq!(refs[1].id().unwrap(), "r1"); // Brown v. Board
        assert_eq!(refs[2].id().unwrap(), "r3"); // Smith
    }

    #[test]
    fn test_legal_citation_sort() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let case_a = make_reference("r1", "legal-case", "", "Doe v. Smith", 1990);
        let case_b = make_reference("r2", "legal-case", "", "Brown v. Board", 1954);

        let mut refs = vec![&case_a, &case_b];

        let sort_spec = GroupSort {
            template: vec![
                GroupSortKey {
                    key: GroupSortKeyType::Title, // Case name
                    ascending: true,
                    order: None,
                    sort_order: None,
                },
                GroupSortKey {
                    key: GroupSortKeyType::Issued,
                    ascending: true,
                    order: None,
                    sort_order: None,
                },
            ],
        };

        refs = sorter.sort_references(refs, &sort_spec);
        assert_eq!(refs[0].id().unwrap(), "r2"); // Brown v. Board
    }

    #[test]
    fn test_legal_hierarchy_sort() {
        let locale = make_locale();
        let sorter = GroupSorter::new(&locale);

        let statute = make_reference("r1", "statute", "", "Clean Air Act", 1970);
        let case = make_reference("r2", "legal-case", "", "Roe v. Wade", 1973);
        let treaty = make_reference("r3", "treaty", "", "Paris Agreement", 2015);

        let mut refs = vec![&treaty, &case, &statute];

        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: GroupSortKeyType::RefType,
                ascending: true,
                order: Some(vec![
                    "legal-case".to_string(),
                    "statute".to_string(),
                    "treaty".to_string(),
                ]),
                sort_order: None,
            }],
        };

        refs = sorter.sort_references(refs, &sort_spec);

        // Hierarchy: case, statute, treaty
        assert_eq!(refs[0].id().unwrap(), "r2");
        assert_eq!(refs[1].id().unwrap(), "r1");
        assert_eq!(refs[2].id().unwrap(), "r3");
    }
}
