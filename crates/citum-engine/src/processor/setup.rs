/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Processor construction and configuration helpers.
//!
//! This module owns setup-time concerns for [`Processor`]: constructor paths,
//! locale/config resolution, compound-set validation, and numeric citation
//! number initialization. It intentionally does not contain citation or
//! bibliography rendering logic.

use super::Processor;
use super::disambiguation::Disambiguator;
use super::sorting::Sorter;
use crate::error::ProcessorError;
use crate::reference::{Bibliography, CitationItem, Reference};
use crate::values::ProcHints;
use citum_schema::Style;
use citum_schema::locale::Locale;
use citum_schema::options::{Config, bibliography::BibliographyConfig};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

impl Default for Processor {
    fn default() -> Self {
        let compound_sets = IndexMap::new();
        let (compound_set_by_ref, compound_member_index) =
            Self::build_compound_set_indexes(&compound_sets);
        Self {
            style: Style::default(),
            bibliography: Bibliography::default(),
            locale: Locale::en_us(),
            default_config: Config::default(),
            hints: HashMap::new(),
            citation_numbers: RefCell::new(HashMap::new()),
            cited_ids: RefCell::new(HashSet::new()),
            compound_sets,
            compound_set_by_ref,
            compound_member_index,
            compound_groups: RefCell::new(IndexMap::new()),
            show_semantics: true,
            inject_ast_indices: false,
        }
    }
}

impl Processor {
    fn build_processor(
        style: Style,
        bibliography: Bibliography,
        locale: Locale,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Self {
        let style = style.into_resolved();
        Self::build_processor_pre_resolved(style, bibliography, locale, compound_sets)
    }

    /// Build a processor from an already-resolved style, skipping preset resolution.
    ///
    /// Use this when the style was cloned from a processor that has already
    /// called `into_resolved()`, to avoid a second resolution pass that would
    /// re-apply preset defaults and overwrite null-cleared fields.
    pub(super) fn build_processor_pre_resolved(
        style: Style,
        bibliography: Bibliography,
        locale: Locale,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Self {
        let (compound_set_by_ref, compound_member_index) =
            Self::build_compound_set_indexes(&compound_sets);
        let mut processor = Processor {
            style,
            bibliography,
            locale,
            default_config: Config::default(),
            hints: HashMap::new(),
            citation_numbers: RefCell::new(HashMap::new()),
            cited_ids: RefCell::new(HashSet::new()),
            compound_sets,
            compound_set_by_ref,
            compound_member_index,
            compound_groups: RefCell::new(IndexMap::new()),
            show_semantics: true,
            inject_ast_indices: false,
        };

        // Pre-calculate hints for disambiguation.
        processor.hints = processor.calculate_hints();
        processor
    }

    fn try_validate_compound_sets(
        bibliography: &Bibliography,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Result<IndexMap<String, Vec<String>>, ProcessorError> {
        crate::io::validate_compound_sets(Some(compound_sets), bibliography)
            .map(Option::unwrap_or_default)
    }

    fn validate_compound_sets_or_default(
        bibliography: &Bibliography,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> IndexMap<String, Vec<String>> {
        Self::try_validate_compound_sets(bibliography, compound_sets).unwrap_or_default()
    }

    fn build_compound_set_indexes(
        sets: &IndexMap<String, Vec<String>>,
    ) -> (HashMap<String, String>, HashMap<String, usize>) {
        let mut by_ref = HashMap::new();
        let mut member_index = HashMap::new();
        for (set_id, members) in sets {
            for (idx, member) in members.iter().enumerate() {
                by_ref.insert(member.clone(), set_id.clone());
                member_index.insert(member.clone(), idx);
            }
        }
        (by_ref, member_index)
    }

    /// Check whether the style uses note-based citations (footnotes/endnotes).
    pub(crate) fn is_note_style(&self) -> bool {
        self.get_config()
            .processing
            .as_ref()
            .is_some_and(|processing| matches!(processing, citum_schema::options::Processing::Note))
    }

    /// Check whether the style uses numeric citation rendering.
    fn is_numeric_style(&self) -> bool {
        self.get_config()
            .processing
            .as_ref()
            .is_some_and(|processing| {
                matches!(processing, citum_schema::options::Processing::Numeric)
            })
    }

    fn is_numeric_bibliography_style(&self) -> bool {
        self.get_bibliography_config()
            .processing
            .as_ref()
            .is_some_and(|processing| {
                matches!(processing, citum_schema::options::Processing::Numeric)
            })
    }

    fn resolved_bibliography_sort(&self) -> Option<citum_schema::grouping::GroupSort> {
        if let Some(sort_spec) = self
            .style
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.sort.as_ref())
        {
            return Some(sort_spec.resolve());
        }

        self.get_bibliography_config()
            .processing
            .as_ref()
            .and_then(citum_schema::options::Processing::default_bibliography_sort)
            .map(|preset| preset.group_sort())
    }

    /// Initialize numeric citation numbers from bibliography insertion order.
    ///
    /// citeproc-js registers all bibliography items before citation rendering in
    /// the oracle workflow, so numeric labels are stable by reference registry
    /// order rather than first-citation order.
    ///
    /// When the style declares an explicit bibliography sort, or the
    /// processing family provides a bibliography default, citation numbers
    /// must follow that resolved bibliography order.
    pub(crate) fn initialize_numeric_citation_numbers(&self) {
        if !self.is_numeric_style() {
            return;
        }

        self.initialize_numeric_numbers(self.sort_citation_number_order());
    }

    /// Initialize numeric bibliography numbers from resolved bibliography order.
    pub(crate) fn initialize_numeric_bibliography_numbers(&self) {
        if !self.is_numeric_bibliography_style() {
            return;
        }

        self.initialize_numeric_numbers(self.sort_bibliography_number_order());
    }

    fn initialize_numeric_numbers(&self, ordered_ids: Vec<String>) {
        if !self.citation_numbers.borrow().is_empty() {
            return;
        }

        self.initialize_numeric_citation_numbers_from_ordered_ids(ordered_ids);
    }

    fn sort_citation_number_order(&self) -> Vec<String> {
        if let Some(sort_spec) = self
            .style
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.sort.as_ref())
        {
            let sorter = crate::grouping::GroupSorter::new(&self.locale);
            return sorter
                .sort_references(self.bibliography.values().collect(), &sort_spec.resolve())
                .into_iter()
                .filter_map(citum_schema::reference::InputReference::id)
                .map(String::from)
                .collect();
        }

        let bibliography_config = self.get_bibliography_config();
        Sorter::new(&bibliography_config, &self.locale)
            .sort_references(self.bibliography.values().collect())
            .into_iter()
            .filter_map(citum_schema::reference::InputReference::id)
            .map(String::from)
            .collect()
    }

    fn sort_bibliography_number_order(&self) -> Vec<String> {
        self.sort_references(self.bibliography.values().collect())
            .into_iter()
            .filter_map(citum_schema::reference::InputReference::id)
            .map(String::from)
            .collect()
    }

    /// Assign numeric citation numbers from a pre-resolved reference order.
    fn initialize_numeric_citation_numbers_from_ordered_ids(&self, ordered_ids: Vec<String>) {
        let mut numbers = self.citation_numbers.borrow_mut();
        if !numbers.is_empty() {
            return;
        }

        let compound_config = self.get_bibliography_options().compound_numeric.clone();

        if compound_config.is_some() {
            let mut set_first_seen: IndexMap<String, usize> = IndexMap::new();
            let mut current_number = 1usize;
            let mut compound_groups = self.compound_groups.borrow_mut();
            compound_groups.clear();

            for ref_id in &ordered_ids {
                if let Some(set_id) = self.compound_set_by_ref.get(ref_id) {
                    if let Some(&number) = set_first_seen.get(set_id) {
                        numbers.insert(ref_id.clone(), number);
                    } else {
                        set_first_seen.insert(set_id.clone(), current_number);
                        if let Some(members) = self.compound_sets.get(set_id) {
                            let present_members: Vec<String> = members
                                .iter()
                                .filter(|id| self.bibliography.contains_key(*id))
                                .cloned()
                                .collect();
                            for member in &present_members {
                                numbers.insert(member.clone(), current_number);
                            }
                            if present_members.len() > 1 {
                                compound_groups.insert(current_number, present_members);
                            }
                        } else {
                            numbers.insert(ref_id.clone(), current_number);
                        }
                        current_number += 1;
                    }
                } else if !numbers.contains_key(ref_id) {
                    numbers.insert(ref_id.clone(), current_number);
                    current_number += 1;
                }
            }
        } else {
            for (index, ref_id) in ordered_ids.into_iter().enumerate() {
                numbers.insert(ref_id, index + 1);
            }
        }
    }

    /// Create a new processor with default English locale (en-US).
    #[must_use]
    pub fn new(style: Style, bibliography: Bibliography) -> Self {
        Self::with_compound_sets(style, bibliography, IndexMap::new())
    }

    /// Create a new processor with explicit compound sets, returning an error for invalid sets.
    ///
    /// # Errors
    ///
    /// Returns an error when any compound set references unknown bibliography
    /// entries or reuses the same member more than once.
    pub fn try_with_compound_sets(
        style: Style,
        bibliography: Bibliography,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Result<Self, ProcessorError> {
        Self::try_with_locale_and_compound_sets(style, bibliography, Locale::en_us(), compound_sets)
    }

    /// Create a new processor with explicit compound sets.
    ///
    /// If `compound_sets` is invalid, this constructor ignores the supplied sets
    /// and falls back to a processor without compound sets.
    #[must_use]
    pub fn with_compound_sets(
        style: Style,
        bibliography: Bibliography,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Self {
        let validated_sets = Self::validate_compound_sets_or_default(&bibliography, compound_sets);
        Self::build_processor(style, bibliography, Locale::en_us(), validated_sets)
    }

    /// Create a new processor with a specified locale.
    ///
    /// The locale determines term translations and locale-specific formatting behavior.
    #[must_use]
    pub fn with_locale(style: Style, bibliography: Bibliography, locale: Locale) -> Self {
        Self::with_locale_and_compound_sets(style, bibliography, locale, IndexMap::new())
    }

    /// Create a new processor with explicit locale and compound sets, returning
    /// an error for invalid sets.
    ///
    /// # Errors
    ///
    /// Returns an error when any compound set references unknown bibliography
    /// entries or reuses the same member more than once.
    pub fn try_with_locale_and_compound_sets(
        style: Style,
        bibliography: Bibliography,
        locale: Locale,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Result<Self, ProcessorError> {
        let validated_sets = Self::try_validate_compound_sets(&bibliography, compound_sets)?;
        Ok(Self::build_processor(
            style,
            bibliography,
            locale,
            validated_sets,
        ))
    }

    /// Create a new processor with a specified locale and explicit compound sets.
    ///
    /// The locale determines term translations and locale-specific formatting behavior.
    ///
    /// If `compound_sets` is invalid, this constructor ignores the supplied sets
    /// and falls back to a processor without compound sets.
    #[must_use]
    pub fn with_locale_and_compound_sets(
        style: Style,
        bibliography: Bibliography,
        locale: Locale,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Self {
        let validated_sets = Self::validate_compound_sets_or_default(&bibliography, compound_sets);
        Self::build_processor(style, bibliography, locale, validated_sets)
    }

    /// Create a new processor, loading the locale from disk.
    ///
    /// Loads the locale specified in the style's `default_locale` field from the given directory,
    /// falling back to en-US if not found or not specified.
    #[must_use]
    pub fn with_style_locale(
        style: Style,
        bibliography: Bibliography,
        locales_dir: &std::path::Path,
    ) -> Self {
        let style = style.into_resolved();
        let locale = if let Some(ref locale_id) = style.info.default_locale {
            Locale::load(locale_id, locales_dir)
        } else {
            Locale::en_us()
        };
        Self::with_locale_and_compound_sets(style, bibliography, locale, IndexMap::new())
    }

    /// Return a copy of the processor that injects source template indices into semantic HTML.
    #[must_use]
    pub fn with_inject_ast_indices(mut self, inject_ast_indices: bool) -> Self {
        self.inject_ast_indices = inject_ast_indices;
        self
    }

    /// Enable or disable source template index injection for semantic HTML output.
    pub fn set_inject_ast_indices(&mut self, inject_ast_indices: bool) {
        self.inject_ast_indices = inject_ast_indices;
    }

    /// Return the global style configuration.
    pub fn get_config(&self) -> &Config {
        self.style.options.as_ref().unwrap_or(&self.default_config)
    }

    /// Return merged config for citation rendering.
    ///
    /// Combines global style options with citation-specific overrides.
    pub fn get_citation_config(&self) -> std::borrow::Cow<'_, Config> {
        let base = self.get_config();
        match self
            .style
            .citation
            .as_ref()
            .and_then(|citation| citation.options.as_ref())
        {
            Some(citation_options) => std::borrow::Cow::Owned(citation_options.merged_with(base)),
            None => std::borrow::Cow::Borrowed(base),
        }
    }

    /// Return merged shared config for bibliography rendering.
    ///
    /// Combines global shared style options with bibliography-local shared overrides.
    pub fn get_bibliography_config(&self) -> std::borrow::Cow<'_, Config> {
        let base = self.get_config();
        match self
            .style
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.options.as_ref())
        {
            Some(bibliography_options) => {
                std::borrow::Cow::Owned(bibliography_options.merged_with(base))
            }
            None => std::borrow::Cow::Borrowed(base),
        }
    }

    /// Return effective bibliography-only configuration.
    pub fn get_bibliography_options(&self) -> std::borrow::Cow<'_, BibliographyConfig> {
        match self
            .style
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.options.as_ref())
        {
            Some(bibliography_options) => {
                std::borrow::Cow::Owned(bibliography_options.to_bibliography_config())
            }
            None => std::borrow::Cow::Owned(BibliographyConfig::default()),
        }
    }

    /// Sort references according to the style's bibliography sort specification.
    ///
    /// Uses style-specified sort keys (author, title, issued, etc.) and sort order.
    pub fn sort_references<'a>(&self, references: Vec<&'a Reference>) -> Vec<&'a Reference> {
        if let Some(sort_spec) = self.resolved_bibliography_sort() {
            let sorter = crate::grouping::GroupSorter::new(&self.locale);
            return sorter.sort_references(references, &sort_spec);
        }

        let bibliography_config = self.get_bibliography_config();
        let sorter = Sorter::new(&bibliography_config, &self.locale);
        sorter.sort_references(references)
    }

    /// Sort citation items according to the style's citation sort specification.
    pub fn sort_citation_items(
        &self,
        items: Vec<CitationItem>,
        spec: &citum_schema::CitationSpec,
    ) -> Vec<CitationItem> {
        if let Some(sort_spec) = &spec.sort {
            let mut items_with_refs: Vec<(CitationItem, &Reference)> = items
                .into_iter()
                .filter_map(|item| {
                    self.bibliography
                        .get(&item.id)
                        .map(|reference| (item, reference))
                })
                .collect();

            let resolved_sort = sort_spec.resolve();
            let sorter = crate::grouping::GroupSorter::new(&self.locale);
            items_with_refs.sort_by(|left, right| {
                for sort_key in &resolved_sort.template {
                    let cmp = sorter.compare_by_key(left.1, right.1, sort_key);
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }
                std::cmp::Ordering::Equal
            });

            return items_with_refs
                .into_iter()
                .map(|(item, _reference)| item)
                .collect();
        }

        items
    }

    /// Calculate disambiguation hints needed for the style.
    ///
    /// Analyzes the bibliography to determine which items need disambiguation
    /// (year suffixes, etc.) and calculates hints for efficient rendering.
    pub fn calculate_hints(&self) -> HashMap<String, ProcHints> {
        let citation_config = self.get_citation_config();
        let config = citation_config.as_ref();
        let bibliography_sort = self.resolved_bibliography_sort();

        let disambiguator = if let Some(resolved_sort) = &bibliography_sort {
            Disambiguator::with_group_sort(&self.bibliography, config, &self.locale, resolved_sort)
        } else {
            Disambiguator::new(&self.bibliography, config, &self.locale)
        };

        disambiguator.calculate_hints()
    }
}
