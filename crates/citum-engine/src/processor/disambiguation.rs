use crate::reference::{Bibliography, Reference};
use crate::values::ProcHints;
use citum_schema::options::Config;
use std::collections::{HashMap, HashSet};

use crate::grouping::GroupSorter;
use citum_schema::grouping::GroupSort;
use citum_schema::locale::Locale;

/// Handles disambiguation logic for author-date citations.
///
/// Disambiguation resolves ambiguities when multiple references produce
/// identical rendered strings. The processor applies strategies in cascade:
///
/// 1. **Name expansion** (`disambiguate-add-names`): If et-al is triggered
///    in the base citation, try expanding the author list to differentiate
///    references with same first author and year.
///
/// 2. **Given name expansion** (`disambiguate-add-givenname`): Add initials
///    or full given names to author list to resolve remaining collisions
///    (e.g., "Smith, John" vs "Smith, Jane").
///
/// 3. **Combined expansion**: Try showing both more names AND given names
///    to maximize differentiation before falling back to year suffix.
///
/// 4. **Year suffix fallback** (`disambiguate-add-year-suffix`): If above
///    strategies fail, append letters (a, b, c, ..., z, aa, ab, ...) to
///    the year. Ordering follows the resolved per-group sort when one is
///    configured, otherwise lowercase reference title order.
///
/// ## Algorithm Overview
///
/// - References are grouped by their base collision key
///   (for example, `smith:2020` or a label key)
/// - For each group with 2+ collisions, strategies are applied in order
/// - Once a strategy resolves ambiguity, higher-priority strategies skip
/// - Year suffix assignment is deterministic from the resolved per-group sort
///
/// ## Output
///
/// Returns `ProcHints` for each reference containing:
/// - `group_index`: Position within collision group (1-indexed)
/// - `group_length`: Total references in collision group
/// - `group_key`: Author-year key used for grouping
/// - `disamb_condition`: Whether year suffix should be applied
/// - `expand_given_names`: Whether to show given names/initials
/// - `min_names_to_show`: Minimum author count for name expansion
pub struct Disambiguator<'a> {
    bibliography: &'a Bibliography,
    config: &'a Config,
    locale: &'a Locale,
    group_sort: Option<&'a GroupSort>,
}

#[derive(Clone, Copy)]
struct DisambiguationFlags {
    add_names: bool,
    add_givenname: bool,
    year_suffix: bool,
    is_label_mode: bool,
}

struct GroupDisambiguationContext<'a> {
    key: &'a str,
    group: &'a [&'a Reference],
    flags: DisambiguationFlags,
    author_group_lengths: &'a HashMap<String, usize>,
}

#[derive(Clone, Copy)]
struct HintPlan<'a> {
    key: &'a str,
    expand_given_names: bool,
    min_names_to_show: Option<usize>,
    disamb_condition: bool,
}

#[derive(Clone, Copy)]
enum HintOrder {
    Encountered,
    GroupSorted,
}

impl<'a> Disambiguator<'a> {
    /// Creates a disambiguator that uses the default title-based fallback order.
    pub fn new(bibliography: &'a Bibliography, config: &'a Config, locale: &'a Locale) -> Self {
        Self {
            bibliography,
            config,
            locale,
            group_sort: None,
        }
    }

    /// Creates a disambiguator with an explicit per-group sort specification.
    pub fn with_group_sort(
        bibliography: &'a Bibliography,
        config: &'a Config,
        locale: &'a Locale,
        group_sort: &'a GroupSort,
    ) -> Self {
        Self {
            bibliography,
            config,
            locale,
            group_sort: Some(group_sort),
        }
    }

    /// Calculate processing hints for disambiguation across all references.
    ///
    /// This is a single-pass algorithm that:
    /// 1. Groups references by their base collision key
    /// 2. For each group with multiple references, applies disambiguation
    ///    strategies in cascade order
    /// 3. Returns pre-calculated hints for the renderer
    ///
    /// ## Cascade Order
    ///
    /// For each collision group:
    /// - Try expanding author list (et-al → full names)
    /// - Try adding given names/initials
    /// - Try combined approach (more names + given names)
    /// - Fall back to year suffix (a, b, c, ...)
    ///
    /// ## Performance
    ///
    /// - O(n) for grouping, where n = number of references
    /// - O(g²) for collision detection within each group g
    /// - Total: O(n + Σ(g²)) where typical g << n
    ///
    /// ## Example
    ///
    /// Input bibliography:
    /// - Smith, John (2020) - "Article A"
    /// - Smith, Jane (2020) - "Article B"
    /// - Brown, Tom (2020) - "Article C"
    ///
    /// Output hints:
    /// - "item-1": { group_key: "smith:2020", expand_given_names: true, group_length: 2 }
    /// - "item-2": { group_key: "smith:2020", expand_given_names: true, group_length: 2 }
    /// - "item-3": { group_key: "brown:2020" } (no collision)
    pub fn calculate_hints(&self) -> HashMap<String, ProcHints> {
        let mut hints = HashMap::new();
        let refs: Vec<&Reference> = self.bibliography.values().collect();
        let grouped = self.group_references(&refs);
        let author_group_lengths = self.author_group_lengths(&refs);
        let flags = self.disambiguation_flags();

        for (key, group) in grouped {
            self.apply_group_hints(
                &mut hints,
                GroupDisambiguationContext {
                    key: &key,
                    group: &group,
                    flags,
                    author_group_lengths: &author_group_lengths,
                },
            );
        }

        hints
    }

    fn disambiguation_flags(&self) -> DisambiguationFlags {
        let disamb_config = match self.config.processing.as_ref() {
            Some(processing) => processing.config().disambiguate,
            None => {
                citum_schema::options::Processing::AuthorDate
                    .config()
                    .disambiguate
            }
        };

        DisambiguationFlags {
            add_names: disamb_config.as_ref().map(|d| d.names).unwrap_or(false),
            add_givenname: disamb_config
                .as_ref()
                .map(|d| d.add_givenname)
                .unwrap_or(false),
            year_suffix: disamb_config
                .as_ref()
                .map(|d| d.year_suffix)
                .unwrap_or(false),
            is_label_mode: self
                .config
                .processing
                .as_ref()
                .is_some_and(|p| matches!(p, citum_schema::options::Processing::Label(_))),
        }
    }

    fn author_group_lengths(&self, refs: &[&Reference]) -> HashMap<String, usize> {
        let mut author_group_lengths = HashMap::new();
        for reference in refs {
            let author_key = self.make_author_key(reference);
            if !author_key.is_empty() {
                *author_group_lengths.entry(author_key).or_insert(0) += 1;
            }
        }
        author_group_lengths
    }

    fn apply_group_hints(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: GroupDisambiguationContext<'_>,
    ) {
        if self.try_apply_singleton_hint(hints, &context) {
            return;
        }

        if self.try_apply_label_mode_year_suffix(hints, &context) {
            return;
        }

        if self.try_apply_name_partitions(hints, &context) {
            return;
        }

        if self.try_apply_givenname_resolution(hints, &context) {
            return;
        }

        if self.try_apply_combined_resolution(hints, &context) {
            return;
        }

        self.apply_year_suffix(hints, &context, false, None);
    }

    fn try_apply_singleton_hint(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: &GroupDisambiguationContext<'_>,
    ) -> bool {
        if context.group.len() != 1 {
            return false;
        }

        self.insert_hint(
            hints,
            context.group[0],
            context.author_group_lengths,
            ProcHints::default(),
        );
        true
    }

    fn try_apply_label_mode_year_suffix(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: &GroupDisambiguationContext<'_>,
    ) -> bool {
        if !(context.flags.is_label_mode && context.flags.year_suffix) {
            return false;
        }

        self.apply_year_suffix(hints, context, false, None);
        true
    }

    fn try_apply_name_partitions(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: &GroupDisambiguationContext<'_>,
    ) -> bool {
        if !context.flags.add_names {
            return false;
        }

        let Some((min_names_to_show, partitions)) = self.partition_by_name_expansion(context.group)
        else {
            return false;
        };

        for subgroup in partitions.values() {
            if subgroup.len() == 1 {
                self.apply_resolution(hints, subgroup, context, false, Some(min_names_to_show));
                continue;
            }

            if context.flags.add_givenname
                && self.check_givenname_resolution(subgroup, Some(min_names_to_show))
            {
                self.apply_resolution(hints, subgroup, context, true, Some(min_names_to_show));
                continue;
            }

            self.apply_year_suffix_for_group(
                hints,
                subgroup,
                context,
                false,
                Some(min_names_to_show),
            );
        }

        true
    }

    fn try_apply_givenname_resolution(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: &GroupDisambiguationContext<'_>,
    ) -> bool {
        if !(context.flags.add_givenname && self.check_givenname_resolution(context.group, None)) {
            return false;
        }

        self.apply_resolution(hints, context.group, context, true, None);
        true
    }

    fn try_apply_combined_resolution(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: &GroupDisambiguationContext<'_>,
    ) -> bool {
        if !context.flags.add_names || !context.flags.add_givenname {
            return false;
        }

        let Some(min_names_to_show) = self.find_combined_resolution(context.group) else {
            return false;
        };

        self.apply_resolution(hints, context.group, context, true, Some(min_names_to_show));
        true
    }

    fn find_combined_resolution(&self, group: &[&Reference]) -> Option<usize> {
        let max_authors = group
            .iter()
            .map(|r| r.author().map(|a| a.to_names_vec().len()).unwrap_or(0))
            .max()
            .unwrap_or(0);

        (2..=max_authors).find(|&n| self.check_givenname_resolution(group, Some(n)))
    }

    fn apply_resolution(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        group: &[&Reference],
        context: &GroupDisambiguationContext<'_>,
        expand_given_names: bool,
        min_names_to_show: Option<usize>,
    ) {
        self.insert_group_hints(
            hints,
            group,
            context.author_group_lengths,
            HintPlan {
                key: context.key,
                expand_given_names,
                min_names_to_show,
                disamb_condition: false,
            },
            HintOrder::Encountered,
        );
    }

    fn insert_hint(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        reference: &Reference,
        author_group_lengths: &HashMap<String, usize>,
        mut hint: ProcHints,
    ) {
        hint.group_length = self
            .author_group_length(reference, author_group_lengths)
            .unwrap_or(1);
        hints.insert(reference.id().unwrap_or_default(), hint);
    }

    fn author_group_length(
        &self,
        reference: &Reference,
        author_group_lengths: &HashMap<String, usize>,
    ) -> Option<usize> {
        let author_key = self.make_author_key(reference);
        author_group_lengths.get(&author_key).copied()
    }

    fn apply_year_suffix(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: &GroupDisambiguationContext<'_>,
        expand_given_names: bool,
        min_names_to_show: Option<usize>,
    ) {
        self.apply_year_suffix_for_group(
            hints,
            context.group,
            context,
            expand_given_names,
            min_names_to_show,
        );
    }

    fn apply_year_suffix_for_group(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        group: &[&Reference],
        context: &GroupDisambiguationContext<'_>,
        expand_given_names: bool,
        min_names_to_show: Option<usize>,
    ) {
        self.insert_group_hints(
            hints,
            group,
            context.author_group_lengths,
            HintPlan {
                key: context.key,
                expand_given_names,
                min_names_to_show,
                disamb_condition: true,
            },
            HintOrder::GroupSorted,
        );
    }

    fn insert_group_hints(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        group: &[&Reference],
        author_group_lengths: &HashMap<String, usize>,
        plan: HintPlan<'_>,
        order: HintOrder,
    ) {
        for (idx, reference) in self.ordered_group(group, order).iter().enumerate() {
            self.insert_hint(
                hints,
                reference,
                author_group_lengths,
                ProcHints {
                    disamb_condition: plan.disamb_condition,
                    group_index: idx + 1,
                    group_key: plan.key.to_string(),
                    expand_given_names: plan.expand_given_names,
                    min_names_to_show: plan.min_names_to_show,
                    ..Default::default()
                },
            );
        }
    }

    fn ordered_group<'b>(&self, group: &[&'b Reference], order: HintOrder) -> Vec<&'b Reference> {
        match order {
            HintOrder::Encountered => group.to_vec(),
            HintOrder::GroupSorted => self.sort_group_for_year_suffix(group),
        }
    }

    fn sort_group_for_year_suffix<'b>(&self, group: &[&'b Reference]) -> Vec<&'b Reference> {
        if let Some(sort_spec) = self.group_sort {
            let sorter = GroupSorter::new(self.locale);
            sorter.sort_references(group.to_vec(), sort_spec)
        } else {
            let mut sorted: Vec<&Reference> = group.to_vec();
            sorted.sort_by(|a, b| {
                let a_title = a
                    .title()
                    .map(|t| t.to_string())
                    .unwrap_or_default()
                    .to_lowercase();
                let b_title = b
                    .title()
                    .map(|t| t.to_string())
                    .unwrap_or_default()
                    .to_lowercase();
                a_title.cmp(&b_title)
            });
            sorted
        }
    }

    /// Partition a collision group by showing more names, preserving `et al.`
    /// distinction when some references still have hidden trailing names.
    fn partition_by_name_expansion<'b>(
        &self,
        group: &[&'b Reference],
    ) -> Option<(usize, HashMap<String, Vec<&'b Reference>>)> {
        let max_authors = group
            .iter()
            .map(|r| r.author().map(|a| a.to_names_vec().len()).unwrap_or(0))
            .max()
            .unwrap_or(0);

        for n in 2..=max_authors {
            let mut partitions: HashMap<String, Vec<&Reference>> = HashMap::new();
            for reference in group {
                partitions
                    .entry(self.make_name_expansion_key(reference, n))
                    .or_default()
                    .push(*reference);
            }

            if partitions.len() > 1 {
                return Some((n, partitions));
            }
        }

        None
    }

    fn make_name_expansion_key(&self, reference: &Reference, n: usize) -> String {
        if let Some(authors) = reference.author() {
            let names = authors.to_names_vec();
            let mut parts = names
                .iter()
                .take(n)
                .map(|name| name.family_or_literal().to_lowercase())
                .collect::<Vec<_>>();

            if names.len() > n {
                parts.push("et-al".to_string());
            }

            parts.join("|")
        } else {
            String::new()
        }
    }

    /// Check if expanding to full names resolves ambiguity in the group.
    /// If `min_names` is Some(n), it checks resolution when showing n names.
    fn check_givenname_resolution(&self, group: &[&Reference], min_names: Option<usize>) -> bool {
        let mut seen = HashSet::new();
        let mut collision = false;
        for reference in group {
            if let Some(authors) = reference.author() {
                let n = min_names.unwrap_or(1);
                // Create a key for the first n authors with full names
                let key = authors
                    .to_names_vec()
                    .iter()
                    .take(n)
                    .map(|n| {
                        format!(
                            "{:?}|{:?}|{:?}|{:?}",
                            n.family, n.given, n.non_dropping_particle, n.dropping_particle
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("||");

                if !seen.insert(key) {
                    collision = true;
                    break;
                }
            } else if !seen.insert("".to_string()) {
                collision = true;
                break;
            }
        }
        !collision
    }

    /// Group references by their base collision key for disambiguation.
    fn group_references<'b>(
        &self,
        references: &[&'b Reference],
    ) -> HashMap<String, Vec<&'b Reference>> {
        let mut groups: HashMap<String, Vec<&'b Reference>> = HashMap::new();

        for reference in references {
            let key = self.make_group_key(reference);
            groups.entry(key).or_default().push(*reference);
        }

        groups
    }

    /// Create a grouping key for a reference based on its author field.
    fn make_author_key(&self, reference: &Reference) -> String {
        let shorten = self
            .config
            .contributors
            .as_ref()
            .and_then(|c| c.shorten.as_ref());

        if let Some(authors) = reference.author() {
            let names_vec = authors.to_names_vec();
            if let Some(opts) = shorten {
                if names_vec.len() >= opts.min as usize {
                    // Show 'use_first' names in the base citation
                    names_vec
                        .iter()
                        .take(opts.use_first as usize)
                        .map(|n| n.family_or_literal().to_lowercase())
                        .collect::<Vec<_>>()
                        .join(",")
                        + ",et-al"
                } else {
                    names_vec
                        .iter()
                        .map(|n| n.family_or_literal().to_lowercase())
                        .collect::<Vec<_>>()
                        .join(",")
                }
            } else {
                names_vec
                    .iter()
                    .map(|n| n.family_or_literal().to_lowercase())
                    .collect::<Vec<_>>()
                    .join(",")
            }
        } else {
            "".to_string()
        }
    }

    /// Create a grouping key for a reference based on its base citation form.
    fn make_group_key(&self, reference: &Reference) -> String {
        // In label mode, group by base label string rather than author-year.
        // This ensures disambiguation happens at the label level (Knu84a/Knu84b)
        // rather than the author-year level.
        if let Some(citum_schema::options::Processing::Label(config)) = &self.config.processing {
            let params = config.effective_params();
            return crate::processor::labels::generate_base_label(reference, &params);
        }

        let author_key = self.make_author_key(reference);

        let year = reference
            .issued()
            .and_then(|d| d.year().parse::<i32>().ok())
            .map(|y| y.to_string())
            .unwrap_or_default();

        format!("{}:{}", author_key, year)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Processor;
    use citum_schema::citation::Citation;
    use citum_schema::grouping::{GroupSort, GroupSortKey, SortKey};
    use citum_schema::options::{Config, ContributorConfig, DisplayAsSort};
    use citum_schema::reference::{
        Contributor, EdtfString, InputReference as Reference, Monograph, MonographType,
        MultilingualString, StructuredName, Title,
    };
    use citum_schema::template::{TemplateComponent, WrapPunctuation};
    use citum_schema::{BibliographySpec, CitationSpec, Style, StyleInfo};

    fn make_ref(id: &str, family: &str, given: &str, year: i32) -> Reference {
        let title = format!("Title {}", id);
        Reference::Monograph(Box::new(Monograph {
            id: Some(id.to_string()),
            r#type: MonographType::Book,
            title: Some(Title::Single(title.to_string())),
            container_title: None,
            author: Some(Contributor::StructuredName(StructuredName {
                family: MultilingualString::Simple(family.to_string()),
                given: MultilingualString::Simple(given.to_string()),
                suffix: None,
                dropping_particle: None,
                non_dropping_particle: None,
            })),
            editor: None,
            translator: None,
            recipient: None,
            interviewer: None,
            issued: EdtfString(year.to_string()),
            publisher: None,
            url: None,
            accessed: None,
            language: None,
            field_languages: Default::default(),
            note: None,
            isbn: None,
            doi: None,
            edition: None,
            report_number: None,
            collection_number: None,
            genre: None,
            medium: None,
            archive: None,
            archive_location: None,
            keywords: None,
            original_date: None,
            original_title: None,
            ads_bibcode: None,
        }))
    }

    fn make_multi_author_ref(id: &str, authors: &[(&str, &str)], year: i32) -> Reference {
        let title = format!("Title {}", id);
        Reference::Monograph(Box::new(Monograph {
            id: Some(id.to_string()),
            r#type: MonographType::Book,
            title: Some(Title::Single(title)),
            container_title: None,
            author: Some(Contributor::ContributorList(
                citum_schema::reference::ContributorList(
                    authors
                        .iter()
                        .map(|(family, given)| {
                            Contributor::StructuredName(StructuredName {
                                family: MultilingualString::Simple((*family).to_string()),
                                given: MultilingualString::Simple((*given).to_string()),
                                suffix: None,
                                dropping_particle: None,
                                non_dropping_particle: None,
                            })
                        })
                        .collect(),
                ),
            )),
            editor: None,
            translator: None,
            recipient: None,
            interviewer: None,
            issued: EdtfString(year.to_string()),
            publisher: None,
            url: None,
            accessed: None,
            language: None,
            field_languages: Default::default(),
            note: None,
            isbn: None,
            doi: None,
            edition: None,
            report_number: None,
            collection_number: None,
            genre: None,
            medium: None,
            archive: None,
            archive_location: None,
            keywords: None,
            original_date: None,
            original_title: None,
            ads_bibcode: None,
        }))
    }

    fn make_author_date_style(config: Config, bibliography_sort: Option<GroupSort>) -> Style {
        Style {
            info: StyleInfo {
                title: Some("Disambiguation Test".to_string()),
                id: Some("disambiguation-test".to_string()),
                ..Default::default()
            },
            options: Some(config),
            citation: Some(CitationSpec {
                template: Some(vec![
                    citum_schema::tc_contributor!(Author, Short),
                    citum_schema::tc_date!(Issued, Year, prefix = ", "),
                ]),
                wrap: Some(WrapPunctuation::Parentheses),
                ..Default::default()
            }),
            bibliography: Some(BibliographySpec {
                sort: bibliography_sort.map(citum_schema::grouping::GroupSortEntry::Explicit),
                template: Some(vec![TemplateComponent::Title(
                    citum_schema::template::TemplateTitle {
                        title: citum_schema::template::TitleType::Primary,
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_group_aware_year_suffix_sort() {
        use citum_schema::options::{Disambiguation, Processing, ProcessingCustom};

        let r1 = make_ref("r1", "Smith", "Same", 2020);
        let r2 = make_ref("r2", "Smith", "Same", 2020);

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), r1);
        bib.insert("r2".to_string(), r2);

        let config = Config::default();
        let locale = Locale::en_us();

        // 1. Default sorting (by title): r1 should be 'a', r2 should be 'b'.
        // Title r1 < Title r2 alphabetically, so r1 gets group_index 1.
        let disamb_default = Disambiguator::new(&bib, &config, &locale);
        let hints_default = disamb_default.calculate_hints();

        assert_eq!(hints_default.get("r1").unwrap().group_index, 1);
        assert_eq!(hints_default.get("r2").unwrap().group_index, 2);

        // 2. Custom group sort: Sort by title descending -> r2 should be 'a', r1 should be 'b'
        let sort_spec = GroupSort {
            template: vec![GroupSortKey {
                key: SortKey::Title,
                ascending: false,
                order: None,
                sort_order: None,
            }],
        };

        let disamb_custom = Disambiguator::with_group_sort(&bib, &config, &locale, &sort_spec);
        let hints_custom = disamb_custom.calculate_hints();

        assert_eq!(hints_custom.get("r2").unwrap().group_index, 1);
        assert_eq!(hints_custom.get("r1").unwrap().group_index, 2);

        let style = make_author_date_style(
            Config {
                processing: Some(Processing::Custom(ProcessingCustom {
                    disambiguate: Some(Disambiguation {
                        names: false,
                        add_givenname: false,
                        year_suffix: true,
                    }),
                    ..Default::default()
                })),
                contributors: Some(ContributorConfig {
                    display_as_sort: Some(DisplayAsSort::First),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Some(sort_spec),
        );
        let processor = Processor::new(style, bib);

        let rendered_r1 = processor.process_citation(&Citation::simple("r1")).unwrap();
        let rendered_r2 = processor.process_citation(&Citation::simple("r2")).unwrap();

        assert!(
            rendered_r1.contains("2020b"),
            "expected r1 to sort second: {rendered_r1}"
        );
        assert!(
            rendered_r2.contains("2020a"),
            "expected r2 to sort first: {rendered_r2}"
        );
    }

    #[test]
    fn test_disambiguate_given_names() {
        use citum_schema::options::{Disambiguation, Processing, ProcessingCustom};

        // Use different given names to test if expansion resolves the collision
        let r1 = make_ref("r1", "Smith", "John", 2020);
        let r2 = make_ref("r2", "Smith", "Alice", 2020);

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), r1);
        bib.insert("r2".to_string(), r2);

        let config = Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                disambiguate: Some(Disambiguation {
                    names: false,
                    add_givenname: true,
                    year_suffix: false,
                }),
                ..Default::default()
            })),
            ..Default::default()
        };
        let locale = Locale::en_us();

        let disamb = Disambiguator::new(&bib, &config, &locale);
        let hints = disamb.calculate_hints();

        // Both should have expand_given_names set to true to resolve the Smith (2020) collision
        assert!(hints.get("r1").unwrap().expand_given_names);
        assert!(hints.get("r2").unwrap().expand_given_names);

        // Should NOT have year suffix since it's disabled in config (and given names resolve it)
        assert!(!hints.get("r1").unwrap().disamb_condition);
        assert!(!hints.get("r2").unwrap().disamb_condition);

        // Collision resolved: entries occupy distinct positions
        assert_ne!(
            hints.get("r1").unwrap().group_index,
            hints.get("r2").unwrap().group_index
        );

        let style = make_author_date_style(
            Config {
                processing: Some(Processing::Custom(ProcessingCustom {
                    disambiguate: Some(Disambiguation {
                        names: false,
                        add_givenname: true,
                        year_suffix: false,
                    }),
                    ..Default::default()
                })),
                contributors: Some(ContributorConfig {
                    initialize_with: Some(". ".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
        );
        let processor = Processor::new(style, bib);

        let rendered_r1 = processor.process_citation(&Citation::simple("r1")).unwrap();
        let rendered_r2 = processor.process_citation(&Citation::simple("r2")).unwrap();

        assert!(
            rendered_r1.contains("J. Smith"),
            "expected expanded given name for r1: {rendered_r1}"
        );
        assert!(
            rendered_r2.contains("A. Smith"),
            "expected expanded given name for r2: {rendered_r2}"
        );
    }

    #[test]
    fn test_partitioned_name_expansion_keeps_unique_items_and_suffixes_remainders() {
        use citum_schema::options::{
            ContributorConfig, Disambiguation, Processing, ProcessingCustom, ShortenListOptions,
        };

        let mut bib = Bibliography::new();
        bib.insert(
            "r1".to_string(),
            make_multi_author_ref("r1", &[("Smith", "John"), ("Jones", "Peter")], 2020),
        );
        bib.insert(
            "r2".to_string(),
            make_multi_author_ref("r2", &[("Smith", "John"), ("Brown", "Alice")], 2020),
        );
        bib.insert(
            "r3".to_string(),
            make_multi_author_ref("r3", &[("Smith", "John"), ("Brown", "Adam")], 2020),
        );

        let config = Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                disambiguate: Some(Disambiguation {
                    names: true,
                    add_givenname: false,
                    year_suffix: true,
                }),
                ..Default::default()
            })),
            contributors: Some(ContributorConfig {
                shorten: Some(ShortenListOptions {
                    min: 2,
                    use_first: 1,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::en_us();

        let hints = Disambiguator::new(&bib, &config, &locale).calculate_hints();

        let unique = hints.get("r1").unwrap();
        assert!(!unique.disamb_condition);
        assert_eq!(unique.group_index, 1);
        assert_eq!(unique.min_names_to_show, Some(2));
        assert_eq!(unique.group_length, 3);

        let remaining_a = hints.get("r2").unwrap();
        let remaining_b = hints.get("r3").unwrap();
        assert!(remaining_a.disamb_condition);
        assert!(remaining_b.disamb_condition);
        assert_eq!(remaining_a.min_names_to_show, Some(2));
        assert_eq!(remaining_b.min_names_to_show, Some(2));
        assert_eq!(remaining_a.group_length, 3);
        assert_eq!(remaining_b.group_length, 3);
        assert_ne!(remaining_a.group_index, remaining_b.group_index);
    }

    #[test]
    fn test_label_mode_skips_name_strategies_and_suffixes_by_label_group() {
        use citum_schema::options::{LabelConfig, LabelPreset, Processing};

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), make_ref("r1", "Kuhn", "Thomas", 1962));
        bib.insert("r2".to_string(), make_ref("r2", "Kuhn", "Thomas", 1962));

        let config = Config {
            processing: Some(Processing::Label(LabelConfig {
                preset: LabelPreset::Din,
                ..Default::default()
            })),
            ..Default::default()
        };
        let locale = Locale::en_us();

        let hints = Disambiguator::new(&bib, &config, &locale).calculate_hints();
        let first = hints.get("r1").unwrap();
        let second = hints.get("r2").unwrap();

        assert!(first.disamb_condition);
        assert!(second.disamb_condition);
        assert!(!first.expand_given_names);
        assert!(!second.expand_given_names);
        assert_eq!(first.min_names_to_show, None);
        assert_eq!(second.min_names_to_show, None);
        assert_eq!(first.group_key, second.group_key);
        assert!(!first.group_key.contains(':'));
        assert_ne!(first.group_index, second.group_index);
    }
}
