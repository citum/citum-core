/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use crate::reference::{Bibliography, Reference};
use crate::values::ProcHints;
use citum_schema::options::{Config, GivennameRule};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use crate::sorting::ReferenceSorter;
use citum_schema::grouping::GroupSort;
use citum_schema::locale::Locale;
use citum_schema::options::{Substitute, SubstituteKey};
use citum_schema::reference::{ClassExtension, Title};

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

#[derive(Clone, Copy, Default)]
struct DisambiguationFlags {
    add_names: bool,
    add_givenname: bool,
    year_suffix: bool,
    is_label_mode: bool,
    primary_givenname_only: bool,
}

struct GroupDisambiguationContext<'a> {
    key: &'a str,
    group: &'a [&'a CachedReference<'a>],
    flags: DisambiguationFlags,
    author_group_lengths: &'a HashMap<String, usize>,
}

#[derive(Clone, Copy)]
struct HintPlan<'a> {
    key: &'a str,
    expand_given_names: bool,
    expand_given_names_primary_only: bool,
    min_names_to_show: Option<usize>,
    disamb_condition: bool,
}

#[derive(Clone, Copy)]
enum HintOrder {
    Encountered,
    GroupSorted,
}

enum GroupHintAction<'a> {
    Singleton(&'a CachedReference<'a>),
    LabelYearSuffix,
    NamePartitions {
        min_names_to_show: usize,
        partitions: HashMap<String, Vec<&'a CachedReference<'a>>>,
    },
    GivennameResolution,
    CombinedResolution {
        min_names_to_show: usize,
        primary_only_requires_suffix: bool,
    },
    FallbackYearSuffix,
}

type ReferenceCache<'a> = Vec<CachedReference<'a>>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ReferenceCacheKey {
    Id(String),
    Index(usize),
}

struct CachedReference<'a> {
    reference: &'a Reference,
    #[allow(dead_code, reason = "Cache key policy is asserted in unit tests.")]
    key: ReferenceCacheKey,
    data: CachedReferenceData,
}

struct CachedReferenceData {
    author_key: String,
    group_key: String,
    names: Vec<crate::reference::FlatName>,
    title_key: Option<String>,
}

impl<'a> Disambiguator<'a> {
    /// Creates a disambiguator that uses the default title-based fallback order.
    #[must_use]
    pub fn new(bibliography: &'a Bibliography, config: &'a Config, locale: &'a Locale) -> Self {
        Self {
            bibliography,
            config,
            locale,
            group_sort: None,
        }
    }

    /// Creates a disambiguator with an explicit per-group sort specification.
    #[must_use]
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
    /// - "item-1": { `group_key`: "smith:2020", `expand_given_names`: true, `group_length`: 2 }
    /// - "item-2": { `group_key`: "smith:2020", `expand_given_names`: true, `group_length`: 2 }
    /// - "item-3": { `group_key`: "brown:2020" } (no collision)
    #[must_use]
    pub fn calculate_hints(&self) -> HashMap<String, ProcHints> {
        let mut hints = HashMap::new();
        let refs: Vec<&Reference> = self.bibliography.values().collect();
        let flags = self.disambiguation_flags();
        // Always populate title_key when year-suffix disambiguation is active so that
        // sort_group_for_year_suffix can use it as a stable tie-breaker regardless of
        // whether a group_sort is configured.
        let needs_title_key = flags.year_suffix;
        let cache = self.build_reference_cache(&refs, needs_title_key);
        let grouped = self.group_references(&cache);
        let author_group_lengths = self.author_group_lengths(&cache);

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

    /// Resolves disambiguation configuration from the processor config.
    fn disambiguation_flags(&self) -> DisambiguationFlags {
        let disamb_config = self.config.effective_processing().config().disambiguate;

        DisambiguationFlags {
            add_names: disamb_config.as_ref().is_some_and(|d| d.names),
            add_givenname: disamb_config.as_ref().is_some_and(|d| d.add_givenname),
            year_suffix: disamb_config.as_ref().is_some_and(|d| d.year_suffix),
            is_label_mode: self
                .config
                .processing
                .as_ref()
                .is_some_and(|p| matches!(p, citum_schema::options::Processing::Label(_))),
            primary_givenname_only: disamb_config.as_ref().is_some_and(|d| {
                matches!(
                    d.givenname_rule,
                    GivennameRule::PrimaryName | GivennameRule::PrimaryNameWithInitials
                )
            }),
        }
    }

    /// Builds an internal cache of reference data (author keys, group keys, titles)
    /// to avoid redundant string generation during disambiguation.
    fn build_reference_cache<'b>(
        &self,
        refs: &[&'b Reference],
        needs_title_key: bool,
    ) -> ReferenceCache<'b> {
        refs.iter()
            .enumerate()
            .map(|(index, reference)| {
                let names = reference.author().map_or_else(Vec::new, |authors| {
                    self.render_name_for_disambiguation(&authors)
                });
                let author_key = self.build_author_slot_key(reference, &names);
                let group_key = self.build_group_key(index, reference, &author_key);
                // Year-suffix letters (a, b, c…) must follow the effective bibliography
                // sort order. Reuse the bibliography title sort key (leading-article
                // stripping + locale collation) so suffix assignment cannot diverge from
                // the rendered order — a raw lowercased title sorts "An Ecology" before
                // "Biology", producing `2019b` before `2019a` (DISAMBIGUATION.md §3).
                let title_key = needs_title_key
                    .then(|| crate::sort_support::title_sort_key(reference, self.locale));

                CachedReference {
                    reference,
                    key: Self::reference_cache_key(index, reference),
                    data: CachedReferenceData {
                        author_key,
                        group_key,
                        names,
                        title_key,
                    },
                }
            })
            .collect()
    }

    fn build_author_slot_key(
        &self,
        reference: &Reference,
        author_names: &[crate::reference::FlatName],
    ) -> String {
        let author_key = self.build_author_key(author_names);
        if !author_key.is_empty() {
            return author_key;
        }

        let substitute = self
            .config
            .substitute
            .as_ref()
            .map(citum_schema::options::SubstituteConfig::resolve)
            .unwrap_or_default();

        self.build_substitute_author_key(reference, &substitute)
            .unwrap_or_default()
    }

    fn build_substitute_author_key(
        &self,
        reference: &Reference,
        substitute: &Substitute,
    ) -> Option<String> {
        for key in &substitute.template {
            let resolved = match key {
                SubstituteKey::CollectionEditor => reference
                    .contributor(citum_schema::reference::ContributorRole::Unknown(
                        "collection-editor".to_string(),
                    ))
                    .map(|names| {
                        self.build_author_key(&self.render_name_for_disambiguation(&names))
                    }),
                SubstituteKey::Editor => reference.editor().map(|names| {
                    self.build_author_key(&self.render_name_for_disambiguation(&names))
                }),
                SubstituteKey::Translator => reference.translator().map(|names| {
                    self.build_author_key(&self.render_name_for_disambiguation(&names))
                }),
                SubstituteKey::ParentSerial => {
                    resolve_parent_serial_title(reference).map(Self::title_substitute_key)
                }
                SubstituteKey::Title => reference.title().map(Self::title_substitute_key),
            };

            if let Some(key) = resolved.filter(|key| !key.is_empty()) {
                return Some(key);
            }
        }

        None
    }

    /// Calculates how many references in `refs` share the same `author_key`.
    /// The returned map is keyed only by `author_key` and is later used when
    /// populating `ProcHints::group_length`, rather than representing the size
    /// of a per-`group_key` collision group.
    fn author_group_lengths(&self, refs: &ReferenceCache<'_>) -> HashMap<String, usize> {
        let mut author_group_lengths = HashMap::new();
        for reference in refs {
            let author_key = &reference.data.author_key;
            if !author_key.is_empty() {
                *author_group_lengths.entry(author_key.clone()).or_insert(0) += 1;
            }
        }
        author_group_lengths
    }

    /// Orchestrates the disambiguation cascade for a single collision group.
    /// It attempts strategies in increasing order of disruptiveness (expansion -> year suffix).
    fn apply_group_hints(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: GroupDisambiguationContext<'_>,
    ) {
        match self.select_group_hint_action(&context) {
            GroupHintAction::Singleton(reference) => {
                self.insert_hint(
                    hints,
                    reference,
                    context.author_group_lengths,
                    ProcHints::default(),
                );
            }
            GroupHintAction::LabelYearSuffix => {
                self.apply_year_suffix(hints, &context, false, None);
            }
            GroupHintAction::NamePartitions {
                min_names_to_show,
                partitions,
            } => self.apply_name_partitions(hints, &context, min_names_to_show, &partitions),
            GroupHintAction::GivennameResolution => {
                self.apply_resolution(hints, context.group, &context, true, None);
            }
            GroupHintAction::CombinedResolution {
                min_names_to_show,
                primary_only_requires_suffix,
            } => {
                if primary_only_requires_suffix {
                    self.apply_year_suffix_for_group(
                        hints,
                        context.group,
                        &context,
                        true,
                        Some(min_names_to_show),
                    );
                } else {
                    self.apply_resolution(
                        hints,
                        context.group,
                        &context,
                        true,
                        Some(min_names_to_show),
                    );
                }
            }
            GroupHintAction::FallbackYearSuffix => {
                self.apply_year_suffix(hints, &context, false, None);
            }
        }
    }

    /// Selects the first applicable disambiguation action without mutating hint state.
    fn select_group_hint_action<'b>(
        &self,
        context: &GroupDisambiguationContext<'b>,
    ) -> GroupHintAction<'b> {
        if let Some(reference) = self.select_singleton_hint(context) {
            return GroupHintAction::Singleton(reference);
        }

        if self.select_label_mode_year_suffix(context) {
            return GroupHintAction::LabelYearSuffix;
        }

        if let Some((min_names_to_show, partitions)) = self.select_name_partitions(context) {
            return GroupHintAction::NamePartitions {
                min_names_to_show,
                partitions,
            };
        }

        if self.select_givenname_resolution(context) {
            return GroupHintAction::GivennameResolution;
        }

        if let Some((min_names_to_show, primary_only_requires_suffix)) =
            self.select_combined_resolution(context)
        {
            return GroupHintAction::CombinedResolution {
                min_names_to_show,
                primary_only_requires_suffix,
            };
        }

        GroupHintAction::FallbackYearSuffix
    }

    /// Selects singleton handling for groups with only one reference (no collision).
    fn select_singleton_hint<'b>(
        &self,
        context: &GroupDisambiguationContext<'b>,
    ) -> Option<&'b CachedReference<'b>> {
        if context.group.len() == 1 {
            #[allow(clippy::indexing_slicing, reason = "context.group.len() == 1")]
            return Some(context.group[0]);
        }

        None
    }

    /// Selects year-suffix disambiguation specifically for label-based styles (e.g. [Knu84a]).
    fn select_label_mode_year_suffix(&self, context: &GroupDisambiguationContext<'_>) -> bool {
        context.flags.is_label_mode && context.flags.year_suffix
    }

    /// Selects partitions produced by expanding the number of names shown (et al. expansion).
    fn select_name_partitions<'b>(
        &self,
        context: &GroupDisambiguationContext<'b>,
    ) -> Option<(usize, HashMap<String, Vec<&'b CachedReference<'b>>>)> {
        context
            .flags
            .add_names
            .then(|| self.partition_by_name_expansion(context.group))
            .flatten()
    }

    /// Selects collision resolution by adding given names or initials.
    fn select_givenname_resolution(&self, context: &GroupDisambiguationContext<'_>) -> bool {
        // Use full-expansion keys to determine whether givenname expansion can help at all.
        // (With n=1, the full and primary-only keys are equivalent — both inspect only the
        // primary author — so no separate primary-only check is needed here.)
        context.flags.add_givenname && self.check_givenname_resolution(context.group, None, false)
    }

    /// Selects collision resolution by using both more names AND given name expansion.
    ///
    /// When `primary_givenname_only` is active, the renderer only shows given names for
    /// the first author. `find_combined_resolution` uses full-expansion keys to find the
    /// minimum name count that would work in theory; this function then verifies whether
    /// that resolution also holds under the restricted primary-only rendering.
    fn select_combined_resolution(
        &self,
        context: &GroupDisambiguationContext<'_>,
    ) -> Option<(usize, bool)> {
        if !context.flags.add_names || !context.flags.add_givenname {
            return None;
        }

        let min_names_to_show = self.find_combined_resolution(context.group)?;
        let primary_only_requires_suffix = context.flags.primary_givenname_only
            && !self.check_givenname_resolution(context.group, Some(min_names_to_show), true);

        Some((min_names_to_show, primary_only_requires_suffix))
    }

    /// Applies a name-expansion partition plan, suffixing any unresolved subgroups.
    fn apply_name_partitions(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        context: &GroupDisambiguationContext<'_>,
        min_names_to_show: usize,
        partitions: &HashMap<String, Vec<&CachedReference<'_>>>,
    ) {
        for subgroup in partitions.values() {
            if subgroup.len() == 1 {
                self.apply_resolution(hints, subgroup, context, false, Some(min_names_to_show));
                continue;
            }

            if context.flags.add_givenname
                && self.check_givenname_resolution(subgroup, Some(min_names_to_show), false)
            {
                // Under primary-name rules, secondary given names are not rendered.
                // If the full-expansion check passes but primary-only does not, the
                // subgroup must fall back to year-suffix (with expansion retained).
                if context.flags.primary_givenname_only
                    && !self.check_givenname_resolution(subgroup, Some(min_names_to_show), true)
                {
                    self.apply_year_suffix_for_group(
                        hints,
                        subgroup,
                        context,
                        true,
                        Some(min_names_to_show),
                    );
                } else {
                    self.apply_resolution(hints, subgroup, context, true, Some(min_names_to_show));
                }
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
    }

    /// Searches for the minimum number of names that, when combined with given name expansion,
    /// resolves the collision group.
    fn find_combined_resolution(&self, group: &[&CachedReference<'_>]) -> Option<usize> {
        let max_authors = group
            .iter()
            .map(|reference| reference.data.names.len())
            .max()
            .unwrap_or(0);

        // Use full-expansion keys (primary_only: false) to find the minimum name count.
        // The caller is responsible for verifying the result under primary-only rendering
        // when primary_givenname_only is active.
        (2..=max_authors).find(|&n| self.check_givenname_resolution(group, Some(n), false))
    }

    /// Finalizes a successful disambiguation strategy by inserting the calculated hints into the map.
    fn apply_resolution(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        group: &[&CachedReference<'_>],
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
                expand_given_names_primary_only: context.flags.primary_givenname_only,
                min_names_to_show,
                disamb_condition: false,
            },
            HintOrder::Encountered,
        );
    }

    /// Inserts a single hint into the hints map, ensuring the author group length is correctly set.
    fn insert_hint(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        reference: &CachedReference<'_>,
        author_group_lengths: &HashMap<String, usize>,
        mut hint: ProcHints,
    ) {
        hint.group_length = self
            .author_group_length(reference, author_group_lengths)
            .unwrap_or(1);
        hints.insert(
            reference.reference.id().unwrap_or_default().to_string(),
            hint,
        );
    }

    /// Retrieves the number of references sharing the author key for a specific reference.
    fn author_group_length(
        &self,
        reference: &CachedReference<'_>,
        author_group_lengths: &HashMap<String, usize>,
    ) -> Option<usize> {
        let author_key = &reference.data.author_key;
        author_group_lengths.get(author_key).copied()
    }

    /// Applies year-suffix disambiguation to the entire group in the context.
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

    /// Applies year-suffix disambiguation to a specific (sub)group of references.
    fn apply_year_suffix_for_group(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        group: &[&CachedReference<'_>],
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
                expand_given_names_primary_only: context.flags.primary_givenname_only,
                min_names_to_show,
                disamb_condition: true,
            },
            HintOrder::GroupSorted,
        );
    }

    /// Iterates through a group of references and inserts hints according to the specified order.
    fn insert_group_hints(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        group: &[&CachedReference<'_>],
        author_group_lengths: &HashMap<String, usize>,
        plan: HintPlan<'_>,
        order: HintOrder,
    ) {
        match order {
            HintOrder::Encountered => {
                for (idx, reference) in group.iter().enumerate() {
                    self.insert_planned_hint(hints, reference, author_group_lengths, plan, idx + 1);
                }
            }
            HintOrder::GroupSorted => {
                for (idx, reference) in self.sort_group_for_year_suffix(group).iter().enumerate() {
                    self.insert_planned_hint(hints, reference, author_group_lengths, plan, idx + 1);
                }
            }
        }
    }

    /// Helper to insert a hint with common planned fields (key, expand flags, group index).
    fn insert_planned_hint(
        &self,
        hints: &mut HashMap<String, ProcHints>,
        reference: &CachedReference<'_>,
        author_group_lengths: &HashMap<String, usize>,
        plan: HintPlan<'_>,
        group_index: usize,
    ) {
        self.insert_hint(
            hints,
            reference,
            author_group_lengths,
            ProcHints {
                disamb_condition: plan.disamb_condition,
                group_index,
                group_key: plan.key.to_string(),
                expand_given_names: plan.expand_given_names,
                expand_given_names_primary_only: plan.expand_given_names_primary_only,
                min_names_to_show: plan.min_names_to_show,
                ..Default::default()
            },
        );
    }

    /// Sorts a collision group to determine the deterministic order for year-suffix assignment.
    /// It uses the provided group sort specification or falls back to title-based sorting.
    fn sort_group_for_year_suffix<'b>(
        &self,
        group: &[&'b CachedReference<'b>],
    ) -> Vec<&'b CachedReference<'b>> {
        if let Some(sort_spec) = self.group_sort {
            let sorter = ReferenceSorter::new(self.locale);
            // Pre-sort by title_key so that entries which compare equal under the primary
            // sort_spec retain a stable, deterministic order (title ascending as tiebreaker).
            // ReferenceSorter::sort_references uses sort_by (stable), so the pre-sort order is
            // preserved for entries that compare equal under the primary key.
            let mut pre_sorted: Vec<&CachedReference<'_>> = group.to_vec();
            pre_sorted.sort_by(|a, b| {
                let a_title = a.data.title_key.as_deref().unwrap_or_default();
                let b_title = b.data.title_key.as_deref().unwrap_or_default();
                a_title.cmp(b_title).then_with(|| {
                    year_suffix_date_key(a.reference).cmp(&year_suffix_date_key(b.reference))
                })
            });
            pre_sorted.sort_by(|a, b| {
                for sort_key in &sort_spec.template {
                    let cmp = sorter.compare_by_key(a.reference, b.reference, sort_key);
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }

                std::cmp::Ordering::Equal
            });
            pre_sorted
        } else {
            let mut sorted: Vec<&CachedReference<'_>> = group.to_vec();
            sorted.sort_by(|a, b| {
                let a_title = a.data.title_key.as_deref().unwrap_or_default();
                let b_title = b.data.title_key.as_deref().unwrap_or_default();
                a_title.cmp(b_title).then_with(|| {
                    year_suffix_date_key(a.reference).cmp(&year_suffix_date_key(b.reference))
                })
            });
            sorted
        }
    }

    /// Partition a collision group by showing more names, preserving `et al.`
    /// distinction when some references still have hidden trailing names.
    fn partition_by_name_expansion<'b>(
        &self,
        group: &[&'b CachedReference<'b>],
    ) -> Option<(usize, HashMap<String, Vec<&'b CachedReference<'b>>>)> {
        let max_authors = group
            .iter()
            .map(|reference| reference.data.names.len())
            .max()
            .unwrap_or(0);

        let mut buf = String::new();
        for n in 2..=max_authors {
            let mut partitions: HashMap<String, Vec<&'b CachedReference<'b>>> = HashMap::new();
            for reference in group {
                let names = &reference.data.names;
                buf.clear();
                self.append_name_expansion_key(&mut buf, names, n);
                if let Some(v) = partitions.get_mut(buf.as_str()) {
                    v.push(*reference);
                } else {
                    partitions.insert(buf.clone(), vec![*reference]);
                }
            }

            if partitions.len() > 1 {
                return Some((n, partitions));
            }
        }

        None
    }

    /// Check if expanding to full names resolves ambiguity in the group.
    ///
    /// If `min_names` is `Some(n)`, it checks resolution when showing `n` names.
    ///
    /// When `primary_only` is `true`, only the first author's given name is included
    /// in the resolution key — mirroring what `primary-name` and
    /// `primary-name-with-initials` actually render.  Use this to validate that a
    /// candidate expansion still works under restricted rendering before committing.
    fn check_givenname_resolution(
        &self,
        group: &[&CachedReference<'_>],
        min_names: Option<usize>,
        primary_only: bool,
    ) -> bool {
        let mut seen = HashSet::new();
        let mut buf = String::new();
        let n = min_names.unwrap_or(1);
        for reference in group {
            let names = &reference.data.names;
            buf.clear();
            self.append_givenname_resolution_key(&mut buf, names, n, primary_only);
            if !seen.insert(buf.clone()) {
                return false;
            }
        }
        true
    }

    /// Group references by their base collision key for disambiguation.
    fn group_references<'b>(
        &self,
        references: &'b ReferenceCache<'b>,
    ) -> HashMap<String, Vec<&'b CachedReference<'b>>> {
        let mut groups: HashMap<String, Vec<&'b CachedReference<'b>>> = HashMap::new();

        for reference in references {
            let key = reference.data.group_key.clone();
            groups.entry(key).or_default().push(reference);
        }

        groups
    }

    /// Flattens a contributor to names using the style's active multilingual display
    /// mode, so the collision key reflects the same surface form the style renders
    /// (DISAMBIGUATION.md §4). Monolingual contributors fall through to the original.
    fn render_name_for_disambiguation(
        &self,
        contributor: &citum_schema::reference::Contributor,
    ) -> Vec<crate::reference::FlatName> {
        let ml = self.config.multilingual.as_ref();
        crate::values::resolve_multilingual_name(
            contributor,
            ml.and_then(|m| m.name_mode.as_ref()),
            ml.and_then(|m| m.preferred_transliteration.as_deref()),
            ml.and_then(|m| m.preferred_script.as_ref()),
            &self.locale.locale,
        )
    }

    /// Generates a normalized author string used for grouping and et-al detection.
    fn build_author_key(&self, names: &[crate::reference::FlatName]) -> String {
        let shorten = self
            .config
            .contributors
            .as_ref()
            .and_then(|c| c.shorten.as_ref());

        if names.is_empty() {
            return String::new();
        }

        let mut key = String::new();
        if let Some(opts) = shorten
            && names.len() >= opts.min as usize
        {
            self.append_lowercased_families(&mut key, names, opts.use_first as usize, ',');
            if !key.is_empty() {
                key.push(',');
            }
            key.push_str("et-al");
            return key;
        }

        self.append_lowercased_families(&mut key, names, names.len(), ',');
        key
    }

    fn title_substitute_key(title: Title) -> String {
        let mut key = String::new();
        Self::push_lowercased(&mut key, title.to_string().trim());
        key
    }

    /// Create a grouping key for a reference based on its base citation form.
    fn build_group_key(&self, index: usize, reference: &Reference, author_key: &str) -> String {
        // In label mode, group by base label string rather than author-year.
        // This ensures disambiguation happens at the label level (Knu84a/Knu84b)
        // rather than the author-year level.
        if let Some(citum_schema::options::Processing::Label(config)) = &self.config.processing {
            let params = config.effective_params();
            return crate::processor::labels::generate_base_label(reference, &params);
        }

        // Anonymous entries (no author key) must not be grouped together for year-suffix
        // assignment. CSL year-suffix disambiguates entries with the same *author* —
        // anonymous entries are already distinguished by their title substitution.
        // Give each anonymous reference a unique key so it forms its own singleton group.
        if author_key.is_empty() {
            if let Some(ref_id) = reference.id().filter(|id| !id.is_empty()) {
                return format!("anon:{ref_id}");
            }
            return format!("anon:index:{index}");
        }

        let mut key = String::with_capacity(author_key.len() + 8);
        key.push_str(author_key);
        key.push(':');
        let Some(year) = reference
            .effective_issued_date()
            .and_then(|d| d.year().parse::<i32>().ok())
        else {
            return key;
        };
        let _ = write!(key, "{year}");
        key
    }

    /// Appends a sequence of family names to the key buffer, lowercased.
    fn append_lowercased_families(
        &self,
        key: &mut String,
        names: &[crate::reference::FlatName],
        take: usize,
        separator: char,
    ) {
        for (idx, name) in names.iter().take(take).enumerate() {
            if idx > 0 {
                key.push(separator);
            }
            Self::push_lowercased(key, name.family_or_literal());
        }
    }

    /// Creates a key representing the citation form when n names are shown.
    fn append_name_expansion_key(
        &self,
        key: &mut String,
        names: &[crate::reference::FlatName],
        n: usize,
    ) {
        self.append_lowercased_families(key, names, n, '|');
        if names.len() > n {
            if !key.is_empty() {
                key.push('|');
            }
            key.push_str("et-al");
        }
    }

    /// Creates a key including full name parts (given names, particles) for exact resolution.
    ///
    /// When `primary_only` is `true`, only the first author (index 0) receives full
    /// given-name/particle parts; subsequent authors contribute only their family name.
    /// This mirrors what `primary-name` and `primary-name-with-initials` actually render,
    /// allowing resolution checks to validate against the real rendered surface form.
    fn append_givenname_resolution_key(
        &self,
        key: &mut String,
        names: &[crate::reference::FlatName],
        n: usize,
        primary_only: bool,
    ) {
        for (idx, name) in names.iter().take(n).enumerate() {
            if idx > 0 {
                key.push_str("||");
            }
            Self::append_optional_part(key, name.family.as_deref());
            if primary_only && idx > 0 {
                // Secondary authors: family name only under primary-name rules.
                continue;
            }
            key.push('|');
            Self::append_optional_part(key, name.given.as_deref());
            key.push('|');
            Self::append_optional_part(key, name.non_dropping_particle.as_deref());
            key.push('|');
            Self::append_optional_part(key, name.dropping_particle.as_deref());
        }
    }

    /// Serializes an optional name part into the key buffer with its length.
    fn append_optional_part(key: &mut String, value: Option<&str>) {
        match value {
            Some(value) => {
                let _ = write!(key, "{}:", value.len());
                key.push_str(value);
            }
            None => key.push('-'),
        }
    }

    /// Pushes a lowercased version of the string to the buffer, optimized for ASCII.
    fn push_lowercased(key: &mut String, value: &str) {
        if value.is_ascii() {
            key.reserve(value.len());
            for byte in value.bytes() {
                key.push((byte as char).to_ascii_lowercase());
            }
        } else {
            key.push_str(&value.to_lowercase());
        }
    }

    /// Returns the stable per-run cache key used for disambiguation metadata.
    fn reference_cache_key(index: usize, reference: &Reference) -> ReferenceCacheKey {
        reference
            .id()
            .map_or(ReferenceCacheKey::Index(index), |id| {
                ReferenceCacheKey::Id(id.to_string())
            })
    }
}

fn resolve_parent_serial_title(reference: &Reference) -> Option<Title> {
    match reference.extension() {
        ClassExtension::SerialComponent(_)
        | ClassExtension::LegalCase(_)
        | ClassExtension::Treaty(_) => reference.container_title(),
        _ => None,
    }
}

fn year_suffix_date_key(reference: &Reference) -> String {
    reference
        .effective_issued_date()
        .map(|date| date.to_string())
        .unwrap_or_default()
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
    use crate::Processor;
    use citum_schema::citation::Citation;
    use citum_schema::grouping::{GroupSort, GroupSortKey, SortKey};
    use citum_schema::options::{Config, ContributorConfig, DisplayAsSort, NameForm};
    use citum_schema::reference::{
        Contributor, EdtfString, InputReference as Reference, Monograph, MonographType,
        MultilingualString, StructuredName, Title,
    };
    use citum_schema::template::{TemplateComponent, WrapPunctuation};
    use citum_schema::{BibliographySpec, CitationSpec, Style, StyleInfo};

    fn make_ref(id: &str, family: &str, given: &str, year: i32) -> Reference {
        let title = format!("Title {id}");
        Reference::Monograph(Box::new(Monograph {
            id: Some(id.into()),
            r#type: MonographType::Book,
            title: Some(Title::Single(title.clone())),
            short_title: None,
            container: None,
            author: Some(Contributor::StructuredName(StructuredName {
                family: MultilingualString::Simple(family.to_string()),
                given: MultilingualString::Simple(given.to_string()),
                suffix: None,
                dropping_particle: None,
                non_dropping_particle: None,
            })),
            editor: None,
            translator: None,
            issued: EdtfString(year.to_string()),
            ..Default::default()
        }))
    }

    fn make_ref_without_id(title_suffix: &str, family: &str, given: &str, year: i32) -> Reference {
        let title = format!("Title {title_suffix}");
        Reference::Monograph(Box::new(Monograph {
            id: None,
            r#type: MonographType::Book,
            title: Some(Title::Single(title)),
            short_title: None,
            container: None,
            author: Some(Contributor::StructuredName(StructuredName {
                family: MultilingualString::Simple(family.to_string()),
                given: MultilingualString::Simple(given.to_string()),
                suffix: None,
                dropping_particle: None,
                non_dropping_particle: None,
            })),
            editor: None,
            translator: None,
            issued: EdtfString(year.to_string()),
            ..Default::default()
        }))
    }

    fn make_multi_author_ref(id: &str, authors: &[(&str, &str)], year: i32) -> Reference {
        let title = format!("Title {id}");
        Reference::Monograph(Box::new(Monograph {
            id: Some(id.into()),
            r#type: MonographType::Book,
            title: Some(Title::Single(title)),
            short_title: None,
            container: None,
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
            issued: EdtfString(year.to_string()),
            ..Default::default()
        }))
    }

    fn make_author_date_style(config: Config, bibliography_sort: Option<GroupSort>) -> Style {
        Style {
            info: StyleInfo {
                title: Some("Disambiguation Test".to_string()),
                id: Some("disambiguation-test".into()),
                ..Default::default()
            },
            options: Some(config),
            citation: Some(CitationSpec {
                template: Some(vec![
                    citum_schema::tc_contributor!(Author, Short),
                    citum_schema::tc_date!(Issued, Year, prefix = ", "),
                ]),
                wrap: Some(WrapPunctuation::Parentheses.into()),
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
                    base: None,
                    disambiguate: Some(Disambiguation {
                        names: false,
                        add_givenname: false,
                        givenname_rule: GivennameRule::default(),
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
    fn test_author_date_default_uses_year_suffix_without_name_expansion() {
        use citum_schema::options::Processing;

        let r1 = make_ref("r1", "Smith", "John", 2020);
        let r2 = make_ref("r2", "Smith", "Alice", 2020);

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), r1);
        bib.insert("r2".to_string(), r2);

        let config = Config {
            processing: Some(Processing::AuthorDate),
            ..Default::default()
        };
        let locale = Locale::en_us();

        let disamb = Disambiguator::new(&bib, &config, &locale);
        let hints = disamb.calculate_hints();
        let r1_hints = hints.get("r1").unwrap();
        let r2_hints = hints.get("r2").unwrap();

        assert!(r1_hints.disamb_condition);
        assert!(r2_hints.disamb_condition);
        assert!(!r1_hints.expand_given_names);
        assert!(!r2_hints.expand_given_names);
        assert_eq!(r1_hints.min_names_to_show, None);
        assert_eq!(r2_hints.min_names_to_show, None);

        let style = make_author_date_style(config, None);
        let processor = Processor::new(style, bib);

        let rendered_r1 = processor.process_citation(&Citation::simple("r1")).unwrap();
        let rendered_r2 = processor.process_citation(&Citation::simple("r2")).unwrap();

        assert!(
            rendered_r1.contains("2020a") || rendered_r1.contains("2020b"),
            "expected r1 to receive a year suffix: {rendered_r1}"
        );
        assert!(
            rendered_r2.contains("2020a") || rendered_r2.contains("2020b"),
            "expected r2 to receive a year suffix: {rendered_r2}"
        );
        assert!(
            !rendered_r1.contains("John") && !rendered_r1.contains("J."),
            "expected r1 to avoid given-name expansion: {rendered_r1}"
        );
        assert!(
            !rendered_r2.contains("Alice") && !rendered_r2.contains("A."),
            "expected r2 to avoid given-name expansion: {rendered_r2}"
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
                base: None,
                disambiguate: Some(Disambiguation {
                    names: false,
                    add_givenname: true,
                    givenname_rule: GivennameRule::AllNames,
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
                    base: None,
                    disambiguate: Some(Disambiguation {
                        names: false,
                        add_givenname: true,
                        givenname_rule: GivennameRule::AllNames,
                        year_suffix: false,
                    }),
                    ..Default::default()
                })),
                contributors: Some(ContributorConfig {
                    initialize_with: Some(". ".to_string()),
                    name_form: Some(NameForm::Initials),
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

    /// When `primary-name` is active and expanding the first author's given name does
    /// not resolve the collision (both works share an identical primary author), the
    /// disambiguator must fall back to year-suffix while retaining the et-al expansion
    /// that was found.  Concretely: hints must have `expand_given_names: true`,
    /// `expand_given_names_primary_only: true`, `min_names_to_show: Some(2)`, and
    /// `disamb_condition: true` (year-suffix), with distinct `group_index` values.
    #[test]
    fn test_primary_name_identical_primary_falls_back_to_year_suffix() {
        use citum_schema::options::{
            Disambiguation, Processing, ProcessingCustom, ShortenListOptions,
        };

        // Primary author ("Asthma/Albert") is identical; secondary authors differ only
        // in given name ("Brandon" vs "Edward") — identical families.
        let r1 = make_multi_author_ref(
            "r1",
            &[
                ("Asthma", "Albert"),
                ("Bronchitis", "Brandon"),
                ("Cold", "Crispin"),
            ],
            1990,
        );
        let r2 = make_multi_author_ref(
            "r2",
            &[
                ("Asthma", "Albert"),
                ("Bronchitis", "Edward"),
                ("Cold", "Crispin"),
            ],
            1990,
        );

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), r1);
        bib.insert("r2".to_string(), r2);

        let config = Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                base: None,
                disambiguate: Some(Disambiguation {
                    names: true,
                    add_givenname: true,
                    givenname_rule: GivennameRule::PrimaryName,
                    year_suffix: true,
                }),
                ..Default::default()
            })),
            contributors: Some(ContributorConfig {
                shorten: Some(ShortenListOptions {
                    min: 3,
                    use_first: 1,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::en_us();

        let hints = Disambiguator::new(&bib, &config, &locale).calculate_hints();

        let h1 = hints.get("r1").expect("r1 must have a hint");
        let h2 = hints.get("r2").expect("r2 must have a hint");

        // Et-al expansion to two names must be retained.
        assert_eq!(
            h1.min_names_to_show,
            Some(2),
            "r1: expected min_names_to_show=2"
        );
        assert_eq!(
            h2.min_names_to_show,
            Some(2),
            "r2: expected min_names_to_show=2"
        );

        // Given-name expansion must be active (primary author initial shown).
        assert!(h1.expand_given_names, "r1: expected expand_given_names");
        assert!(h2.expand_given_names, "r2: expected expand_given_names");

        // Primary-only flag must be propagated.
        assert!(
            h1.expand_given_names_primary_only,
            "r1: expected primary-only"
        );
        assert!(
            h2.expand_given_names_primary_only,
            "r2: expected primary-only"
        );

        // Year-suffix must be assigned (disamb_condition true, distinct indices).
        assert!(
            h1.disamb_condition,
            "r1: expected disamb_condition (year-suffix)"
        );
        assert!(
            h2.disamb_condition,
            "r2: expected disamb_condition (year-suffix)"
        );
        assert_ne!(
            h1.group_index, h2.group_index,
            "r1 and r2 must receive distinct year-suffix positions"
        );
    }

    #[test]
    fn test_build_reference_cache_populates_title_keys_when_year_suffix_is_active() {
        // title_key must be populated whenever year-suffix is on (regardless of group_sort)
        // so that sort_group_for_year_suffix can use it as a stable tie-breaker.
        use citum_schema::options::{Disambiguation, Processing, ProcessingCustom};

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), make_ref("r1", "Smith", "John", 2020));
        let refs: Vec<&Reference> = bib.values().collect();
        let locale = Locale::en_us();

        let disabled_config = Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                base: None,
                disambiguate: Some(Disambiguation {
                    names: false,
                    add_givenname: true,
                    givenname_rule: GivennameRule::default(),
                    year_suffix: false,
                }),
                ..Default::default()
            })),
            ..Default::default()
        };
        let disabled = Disambiguator::new(&bib, &disabled_config, &locale);
        let disabled_flags = disabled.disambiguation_flags();
        // year_suffix=false → title_key must be None
        let disabled_cache = disabled.build_reference_cache(&refs, disabled_flags.year_suffix);
        assert!(
            disabled_cache
                .iter()
                .all(|reference| reference.data.title_key.is_none())
        );

        let enabled_config = Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                base: None,
                disambiguate: Some(Disambiguation {
                    names: false,
                    add_givenname: false,
                    givenname_rule: GivennameRule::default(),
                    year_suffix: true,
                }),
                ..Default::default()
            })),
            ..Default::default()
        };
        let enabled = Disambiguator::new(&bib, &enabled_config, &locale);
        let enabled_flags = enabled.disambiguation_flags();
        // year_suffix=true → title_key must be Some regardless of group_sort
        let enabled_cache = enabled.build_reference_cache(&refs, enabled_flags.year_suffix);
        assert!(
            enabled_cache
                .iter()
                .all(|reference| reference.data.title_key.is_some())
        );
    }

    #[test]
    fn test_reference_cache_key_uses_reference_id_or_index_fallback() {
        let with_id = make_ref("r1", "Smith", "John", 2020);
        let without_id = make_ref_without_id("missing-id", "Smith", "Jane", 2020);

        assert_eq!(
            Disambiguator::reference_cache_key(7, &with_id),
            ReferenceCacheKey::Id("r1".to_string())
        );
        assert_eq!(
            Disambiguator::reference_cache_key(7, &without_id),
            ReferenceCacheKey::Index(7)
        );

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), with_id);
        bib.insert("missing".to_string(), without_id);
        let refs: Vec<&Reference> = bib.values().collect();
        let cache = Disambiguator::new(&bib, &Config::default(), &Locale::en_us())
            .build_reference_cache(&refs, false);

        assert_eq!(cache[0].key, ReferenceCacheKey::Id("r1".to_string()));
        assert_eq!(cache[1].key, ReferenceCacheKey::Index(1));
    }

    #[test]
    fn test_anonymous_refs_do_not_receive_year_suffix() {
        // Anonymous entries (no author) sharing the same year must each be placed in
        // their own singleton group, even when an embedded reference id is empty or missing.
        use citum_schema::options::{Disambiguation, Processing, ProcessingCustom};

        let mut bib = Bibliography::new();
        bib.insert("a1".to_string(), make_ref("a1", "", "", 2020));
        bib.insert("a2".to_string(), make_ref("a2", "", "", 2020));
        bib.insert("a3".to_string(), make_ref("", "", "", 2020));
        bib.insert(
            "a4".to_string(),
            make_ref_without_id("missing-id", "", "", 2020),
        );
        let locale = Locale::en_us();
        let config = Config {
            processing: Some(Processing::Custom(ProcessingCustom {
                base: None,
                disambiguate: Some(Disambiguation {
                    names: true,
                    add_givenname: true,
                    givenname_rule: GivennameRule::default(),
                    year_suffix: true,
                }),
                ..Default::default()
            })),
            ..Default::default()
        };
        let disambiguator = Disambiguator::new(&bib, &config, &locale);
        let refs: Vec<&Reference> = bib.values().collect();
        let cache = disambiguator.build_reference_cache(&refs, false);
        let grouped = disambiguator.group_references(&cache);

        assert_eq!(grouped.len(), 4);
        assert!(!grouped.contains_key("anon:"));
        assert!(grouped.values().all(|group| group.len() == 1));
    }

    #[test]
    fn test_push_lowercased_matches_str_lowercase_for_non_ascii() {
        let mut key = String::new();
        let value = "ΟΣ";

        Disambiguator::push_lowercased(&mut key, value);

        assert_eq!(key, value.to_lowercase());
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
                base: None,
                disambiguate: Some(Disambiguation {
                    names: true,
                    add_givenname: false,
                    givenname_rule: GivennameRule::default(),
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

    /// Build a reference whose author is `Contributor::Multilingual` with distinct
    /// `original` but a shared `transliterations` entry keyed by `translit_tag`.
    fn make_multilingual_ref(
        id: &str,
        original_family: &str,
        translit_family: &str,
        translit_tag: &str,
        year: i32,
    ) -> Reference {
        use citum_schema::reference::contributor::MultilingualName;
        use std::collections::HashMap;

        let mut transliterations = HashMap::new();
        transliterations.insert(
            translit_tag.to_string(),
            StructuredName {
                family: MultilingualString::Simple(translit_family.to_string()),
                given: MultilingualString::Simple("A.".to_string()),
                ..Default::default()
            },
        );
        Reference::Monograph(Box::new(Monograph {
            id: Some(id.into()),
            r#type: MonographType::Book,
            title: Some(Title::Single(format!("Title {id}"))),
            author: Some(Contributor::Multilingual(MultilingualName {
                original: StructuredName {
                    family: MultilingualString::Simple(original_family.to_string()),
                    given: MultilingualString::Simple("A.".to_string()),
                    ..Default::default()
                },
                lang: Some("ja".into()),
                sort_as: None,
                transliterations,
                translations: HashMap::new(),
            })),
            issued: EdtfString(year.to_string()),
            ..Default::default()
        }))
    }

    /// DISAMBIGUATION.md §4: when display mode is `Transliterated`, two references
    /// whose transliterations collide must produce the same author key (→ one
    /// collision group). When mode is `Primary` (distinct originals), keys must differ.
    #[test]
    fn test_multilingual_key_generation_respects_display_mode() {
        use citum_schema::options::MultilingualConfig;
        use citum_schema::options::MultilingualMode;

        // Two distinct Japanese authors that share the same romanisation.
        // Original families differ ("田中" vs "谷中"), but transliteration is "Tanaka".
        let r1 = make_multilingual_ref("r1", "田中", "Tanaka", "ja-Latn", 2020);
        let r2 = make_multilingual_ref("r2", "谷中", "Tanaka", "ja-Latn", 2020);

        let mut bib = Bibliography::new();
        bib.insert("r1".to_string(), r1);
        bib.insert("r2".to_string(), r2);

        let locale = Locale::en_us();

        // --- case 1: Transliterated mode → same key (collision) ---
        let config_translit = Config {
            multilingual: Some(MultilingualConfig {
                name_mode: Some(MultilingualMode::Transliterated),
                preferred_transliteration: Some(vec!["ja-Latn".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let cache_translit = Disambiguator::new(&bib, &config_translit, &locale)
            .build_reference_cache(&[bib.get("r1").unwrap(), bib.get("r2").unwrap()], false);

        let ck_r1 = ReferenceCacheKey::Id("r1".to_string());
        let ck_r2 = ReferenceCacheKey::Id("r2".to_string());
        let ak_r1 = &cache_translit
            .iter()
            .find(|reference| reference.key == ck_r1)
            .expect("r1 cache entry")
            .data
            .author_key;
        let ak_r2 = &cache_translit
            .iter()
            .find(|reference| reference.key == ck_r2)
            .expect("r2 cache entry")
            .data
            .author_key;

        assert_eq!(
            ak_r1, ak_r2,
            "transliterated mode: colliding transliterations must produce the same author key"
        );
        assert_eq!(
            ak_r1, "tanaka",
            "key should be the lowercased transliteration"
        );

        // --- case 2: Primary mode → distinct keys (no collision) ---
        let config_primary = Config::default(); // multilingual: None → falls through to original

        let cache_primary = Disambiguator::new(&bib, &config_primary, &locale)
            .build_reference_cache(&[bib.get("r1").unwrap(), bib.get("r2").unwrap()], false);

        let ak_r1_primary = &cache_primary
            .iter()
            .find(|reference| reference.key == ck_r1)
            .expect("r1 cache entry")
            .data
            .author_key;
        let ak_r2_primary = &cache_primary
            .iter()
            .find(|reference| reference.key == ck_r2)
            .expect("r2 cache entry")
            .data
            .author_key;

        assert_ne!(
            ak_r1_primary, ak_r2_primary,
            "primary mode: distinct originals must produce different author keys"
        );
    }
}
