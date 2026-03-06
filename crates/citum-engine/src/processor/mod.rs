/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! The CSLN processor for rendering citations and bibliographies.
//!
//! ## Architecture
//!
//! The processor is intentionally "dumb" - it applies the style as written
//! without implicit logic. Style-specific behavior (e.g., suppress publisher
//! for journals) should be expressed in the style YAML via `overrides`, not
//! hardcoded here.
//!
//! ## CSL 1.0 Compatibility
//!
//! The processor implements the CSL 1.0 "variable-once" rule:
//! > "Substituted variables are suppressed in the rest of the output to
//! > prevent duplication."
//!
//! This is tracked via `rendered_vars` in `process_template()`.

/// Author/date disambiguation and year-suffix assignment.
pub mod disambiguation;
pub mod document;
pub mod labels;
/// Matching helpers for substitution and repeated-contributor detection.
pub mod matching;
/// Template rendering orchestration and per-component state handling.
pub mod rendering;
/// Citation and bibliography sorting helpers.
pub mod sorting;

#[cfg(test)]
mod tests;

use crate::error::ProcessorError;
use crate::reference::{Bibliography, Citation, CitationItem, Reference};
use crate::render::bibliography::render_entry_body_with_format;
use crate::render::{ProcEntry, ProcTemplate};
use crate::values::ProcHints;
use citum_schema::Style;
use citum_schema::citation::Position;
use citum_schema::locale::Locale;
use citum_schema::options::Config;
use citum_schema::template::{DelimiterPunctuation, WrapPunctuation};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// Get a canonical locator string for ibid comparison.
///
/// Accounts for both flat (`label`/`locator`) and compound (`locators`) forms.
/// Returns `None` when no locator is present.
fn effective_locator_string(item: &CitationItem) -> Option<String> {
    use citum_schema::citation::ResolvedLocator;
    match item.resolved_locator() {
        Some(ResolvedLocator::Flat { value, .. }) => Some(value),
        Some(ResolvedLocator::Compound(segments)) => {
            let parts: Vec<String> = segments
                .iter()
                .map(|s| format!("{:?}:{}", s.label, s.value.value_str()))
                .collect();
            Some(parts.join(","))
        }
        None => None,
    }
}

use self::disambiguation::Disambiguator;
use self::matching::Matcher;
use self::rendering::{CompoundRenderData, Renderer};
use self::sorting::Sorter;

/// The CSLN processor.
///
/// Takes a style, bibliography, and citations, and produces formatted output.
#[derive(Debug)]
pub struct Processor {
    /// The style definition.
    pub style: Style,
    /// The bibliography (references keyed by ID).
    pub bibliography: Bibliography,
    /// The locale for terms and formatting.
    pub locale: Locale,
    /// Default configuration.
    pub default_config: Config,
    /// Pre-calculated processing hints.
    pub hints: HashMap<String, ProcHints>,
    /// Citation numbers assigned to references (for numeric styles).
    pub citation_numbers: RefCell<HashMap<String, usize>>,
    /// IDs of items that were cited in a visible way.
    pub cited_ids: RefCell<HashSet<String>>,
    /// Compound sets keyed by set ID.
    pub compound_sets: IndexMap<String, Vec<String>>,
    /// Reverse lookup for set membership by reference ID.
    pub compound_set_by_ref: HashMap<String, String>,
    /// Position within a set (0-based) for each reference ID.
    pub compound_member_index: HashMap<String, usize>,
    /// Compound numeric groups: citation number → ordered ref IDs in the group.
    pub compound_groups: RefCell<IndexMap<usize, Vec<String>>>,
}

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
        }
    }
}
/// Processed output containing citations and bibliography.
#[derive(Debug, Default)]
pub struct ProcessedReferences {
    /// Rendered bibliography entries with metadata.
    pub bibliography: Vec<ProcEntry>,
    /// Rendered citations as formatted strings.
    ///
    /// None if no citations were processed; Some(vec) otherwise.
    pub citations: Option<Vec<String>>,
}

impl Processor {
    fn build_processor(
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
        };

        // Pre-calculate hints for disambiguation.
        processor.hints = processor.calculate_hints();
        processor
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
    fn is_note_style(&self) -> bool {
        self.get_config()
            .processing
            .as_ref()
            .is_some_and(|p| matches!(p, citum_schema::options::Processing::Note))
    }

    fn resolved_bibliography_sort(&self) -> Option<citum_schema::grouping::GroupSort> {
        if let Some(sort_spec) = self
            .style
            .bibliography
            .as_ref()
            .and_then(|b| b.sort.as_ref())
        {
            return Some(sort_spec.resolve());
        }

        self.get_config()
            .processing
            .as_ref()
            .and_then(|processing| processing.default_bibliography_sort())
            .map(|preset| preset.group_sort())
    }

    /// Detect and annotate citation positions.
    ///
    /// Analyzes citations in order and assigns positions based on whether an item
    /// has been cited before:
    /// - First: Item not cited before
    /// - Subsequent: Item cited before but not immediately preceding
    /// - Ibid: Same single item as immediately preceding citation, no locators
    /// - IbidWithLocator: Same single item as preceding, different locators
    ///
    /// Multi-item citations are never marked as Ibid (only First or Subsequent).
    /// Only sets position if currently None (respects explicit caller values).
    fn annotate_positions(&self, citations: &mut [Citation]) {
        let mut seen_items: HashMap<String, Option<String>> = HashMap::new(); // item_id -> last_locator
        let mut previous_items: Option<Vec<(String, Option<String>)>> = None;

        for citation in citations.iter_mut() {
            // Skip if position already explicitly set
            if citation.position.is_some() {
                // Update history even if position was explicit
                let current_items: Vec<(String, Option<String>)> = citation
                    .items
                    .iter()
                    .map(|item| {
                        let locator = effective_locator_string(item);
                        (item.id.clone(), locator)
                    })
                    .collect();
                previous_items = Some(current_items);
                for item in &citation.items {
                    seen_items.insert(item.id.clone(), effective_locator_string(item));
                }
                continue;
            }

            // Single-item citation: check for ibid cases
            if citation.items.len() == 1 {
                let current_id = &citation.items[0].id;
                let current_locator = effective_locator_string(&citation.items[0]);

                // Check if this is immediately after the previous citation with same item
                if let Some(ref prev_items) = previous_items
                    && prev_items.len() == 1
                    && prev_items[0].0 == *current_id
                {
                    // Same item as immediately preceding
                    let prev_locator = &prev_items[0].1;
                    if prev_locator.is_none() && current_locator.is_none() {
                        // No locators on either: plain ibid
                        citation.position = Some(Position::Ibid);
                    } else if *prev_locator != current_locator {
                        // Different locators: ibid with locator
                        citation.position = Some(Position::IbidWithLocator);
                    }
                    // else: same locator, treat as subsequent
                }

                // If not ibid, check if item was ever cited before
                if citation.position.is_none() {
                    if seen_items.contains_key(current_id) {
                        citation.position = Some(Position::Subsequent);
                    } else {
                        citation.position = Some(Position::First);
                    }
                }

                seen_items.insert(current_id.clone(), current_locator);
            } else {
                // Multi-item citation: never ibid, just First or Subsequent
                let all_seen = citation
                    .items
                    .iter()
                    .all(|item| seen_items.contains_key(&item.id));

                citation.position = if all_seen {
                    Some(Position::Subsequent)
                } else {
                    Some(Position::First)
                };

                for item in &citation.items {
                    seen_items.insert(item.id.clone(), effective_locator_string(item));
                }
            }

            // Update history for next iteration
            let current_items: Vec<(String, Option<String>)> = citation
                .items
                .iter()
                .map(|item| {
                    let locator = effective_locator_string(item);
                    (item.id.clone(), locator)
                })
                .collect();
            previous_items = Some(current_items);
        }
    }

    /// Normalize citation note context for note styles.
    ///
    /// Document/plugin layers should provide explicit `note_number` values.
    /// When missing, this method assigns sequential note numbers in citation order.
    pub fn normalize_note_context(&self, citations: &[Citation]) -> Vec<Citation> {
        if !self.is_note_style() {
            return citations.to_vec();
        }

        let mut next_note = 1_u32;
        citations
            .iter()
            .cloned()
            .map(|mut c| {
                if let Some(n) = c.note_number {
                    if n >= next_note {
                        next_note = n.saturating_add(1);
                    }
                } else {
                    c.note_number = Some(next_note);
                    next_note = next_note.saturating_add(1);
                }
                c
            })
            .collect()
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
    fn initialize_numeric_citation_numbers(&self) {
        let is_numeric = self
            .get_config()
            .processing
            .as_ref()
            .is_some_and(|p| matches!(p, citum_schema::options::Processing::Numeric));
        if !is_numeric {
            return;
        }

        let mut numbers = self.citation_numbers.borrow_mut();
        if !numbers.is_empty() {
            return;
        }

        let ordered_ids: Vec<String> = if let Some(sort_spec) = self.resolved_bibliography_sort() {
            let sorter = crate::grouping::GroupSorter::new(&self.locale);
            sorter
                .sort_references(self.bibliography.values().collect(), &sort_spec)
                .into_iter()
                .filter_map(|reference| reference.id())
                .collect()
        } else {
            self.bibliography.keys().cloned().collect()
        };

        let compound_config = self
            .get_config()
            .bibliography
            .as_ref()
            .and_then(|b| b.compound_numeric.as_ref())
            .cloned();

        if compound_config.is_some() {
            let mut set_first_seen: IndexMap<String, usize> = IndexMap::new();
            let mut current_number = 1usize;
            let mut compound_groups = self.compound_groups.borrow_mut();
            compound_groups.clear();

            for ref_id in &ordered_ids {
                if let Some(set_id) = self.compound_set_by_ref.get(ref_id) {
                    if let Some(&num) = set_first_seen.get(set_id) {
                        numbers.insert(ref_id.clone(), num);
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
                numbers.insert(ref_id.clone(), index + 1);
            }
        }
    }

    /// Create a new processor with default English locale (en-US).
    pub fn new(style: Style, bibliography: Bibliography) -> Self {
        Self::with_compound_sets(style, bibliography, IndexMap::new())
    }

    /// Create a new processor with explicit compound sets, returning an error for invalid sets.
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
    pub fn with_compound_sets(
        style: Style,
        bibliography: Bibliography,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Self {
        let validated_sets = crate::io::validate_compound_sets(Some(compound_sets), &bibliography)
            .ok()
            .flatten()
            .unwrap_or_default();
        Self::build_processor(style, bibliography, Locale::en_us(), validated_sets)
    }

    /// Create a new processor with a specified locale.
    ///
    /// The locale determines term translations and locale-specific formatting behavior.
    pub fn with_locale(style: Style, bibliography: Bibliography, locale: Locale) -> Self {
        Self::with_locale_and_compound_sets(style, bibliography, locale, IndexMap::new())
    }

    /// Create a new processor with explicit locale and compound sets, returning
    /// an error for invalid sets.
    pub fn try_with_locale_and_compound_sets(
        style: Style,
        bibliography: Bibliography,
        locale: Locale,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Result<Self, ProcessorError> {
        let validated_sets = crate::io::validate_compound_sets(Some(compound_sets), &bibliography)?
            .unwrap_or_default();
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
    pub fn with_locale_and_compound_sets(
        style: Style,
        bibliography: Bibliography,
        locale: Locale,
        compound_sets: IndexMap<String, Vec<String>>,
    ) -> Self {
        let validated_sets = crate::io::validate_compound_sets(Some(compound_sets), &bibliography)
            .ok()
            .flatten()
            .unwrap_or_default();
        Self::build_processor(style, bibliography, locale, validated_sets)
    }

    /// Create a new processor, loading the locale from disk.
    ///
    /// Loads the locale specified in the style's `default_locale` field from the given directory,
    /// falling back to en-US if not found or not specified.
    pub fn with_style_locale(
        style: Style,
        bibliography: Bibliography,
        locales_dir: &std::path::Path,
    ) -> Self {
        let locale = if let Some(ref locale_id) = style.info.default_locale {
            Locale::load(locale_id, locales_dir)
        } else {
            Locale::en_us()
        };
        Self::with_locale_and_compound_sets(style, bibliography, locale, IndexMap::new())
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
            .and_then(|c| c.options.as_ref())
        {
            Some(cite_opts) => std::borrow::Cow::Owned(Config::merged(base, cite_opts)),
            None => std::borrow::Cow::Borrowed(base),
        }
    }

    /// Return merged config for bibliography rendering.
    ///
    /// Combines global style options with bibliography-specific overrides.
    pub fn get_bibliography_config(&self) -> std::borrow::Cow<'_, Config> {
        let base = self.get_config();
        match self
            .style
            .bibliography
            .as_ref()
            .and_then(|b| b.options.as_ref())
        {
            Some(bib_opts) => std::borrow::Cow::Owned(Config::merged(base, bib_opts)),
            None => std::borrow::Cow::Borrowed(base),
        }
    }

    /// Process all bibliography references and render them.
    ///
    /// Returns sorted and formatted bibliography entries. For numeric styles,
    /// citations must have been processed first to assign citation numbers.
    pub fn process_references(&self) -> ProcessedReferences {
        self.initialize_numeric_citation_numbers();
        let sorted_refs = self.sort_references(self.bibliography.values().collect());
        let mut bibliography: Vec<ProcEntry> = Vec::new();
        let mut prev_reference: Option<&Reference> = None;

        let bib_config = self.get_config().bibliography.as_ref();
        let substitute = bib_config.and_then(|c| c.subsequent_author_substitute.as_ref());

        for (index, reference) in sorted_refs.iter().enumerate() {
            // For numeric styles, use the citation number assigned when first cited.
            // For other styles, use position in sorted bibliography.
            let ref_id = reference.id().unwrap_or_default();
            let entry_number = self
                .citation_numbers
                .borrow()
                .get(&ref_id)
                .copied()
                .unwrap_or(index + 1);
            if let Some(mut proc) = self.process_bibliography_entry(reference, entry_number) {
                // Apply subsequent author substitution if enabled
                if let Some(sub_string) = substitute
                    && let Some(prev) = prev_reference
                {
                    // Check if primary contributor matches
                    if self.contributors_match(prev, reference) {
                        let bib_config = self.get_bibliography_config();
                        let renderer = Renderer::new(
                            &self.style,
                            &self.bibliography,
                            &self.locale,
                            &bib_config,
                            &self.hints,
                            &self.citation_numbers,
                            CompoundRenderData {
                                set_by_ref: &self.compound_set_by_ref,
                                member_index: &self.compound_member_index,
                                sets: &self.compound_sets,
                            },
                        );
                        renderer.apply_author_substitution(&mut proc, sub_string);
                    }
                }

                bibliography.push(ProcEntry {
                    id: ref_id.clone(),
                    template: proc,
                    metadata: self.extract_metadata(reference),
                });
                prev_reference = Some(reference);
            }
        }

        ProcessedReferences {
            bibliography,
            citations: None,
        }
    }

    /// Extract basic metadata for interactivity.
    fn extract_metadata(&self, reference: &Reference) -> crate::render::format::ProcEntryMetadata {
        use crate::render::format::ProcEntryMetadata;
        use crate::values::RenderOptions;

        let options = RenderOptions {
            config: self.get_config(),
            locale: &self.locale,
            context: crate::values::RenderContext::Bibliography,
            mode: citum_schema::citation::CitationMode::NonIntegral,
            suppress_author: false,
            locator: None,
            locator_label: None,
        };

        ProcEntryMetadata {
            author: reference
                .author()
                .map(|a| crate::values::format_contributors_short(&a.to_names_vec(), &options)),
            year: reference.issued().map(|i| i.year().to_string()),
            title: reference.title().map(|t| t.to_string()),
        }
    }

    /// Render a single citation to plain text.
    ///
    /// This is the primary entry point for citation processing. It handles:
    /// 1. Looking up references in the bibliography.
    /// 2. Annotating positions (ibid, subsequent, etc.).
    /// 3. Resolving disambiguation (name expansion, year suffixes).
    /// 4. Applying the style's citation template.
    ///
    /// Returns the formatted citation string or an error if processing fails.
    pub fn process_citation(&self, citation: &Citation) -> Result<String, ProcessorError> {
        self.process_citation_with_format::<crate::render::plain::PlainText>(citation)
    }

    /// Process and render a bibliography entry.
    ///
    /// Returns a processed template with metadata if the entry matches the style.
    pub fn process_bibliography_entry(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate> {
        // Use bibliography-specific merged config
        let bib_config = self.get_bibliography_config();

        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            &bib_config,
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );
        renderer.process_bibliography_entry(reference, entry_number)
    }

    /// Sort references according to the style's bibliography sort specification.
    ///
    /// Uses style-specified sort keys (author, title, issued, etc.) and sort order.
    pub fn sort_references<'a>(&self, references: Vec<&'a Reference>) -> Vec<&'a Reference> {
        if let Some(sort_spec) = self.resolved_bibliography_sort() {
            let sorter = crate::grouping::GroupSorter::new(&self.locale);
            return sorter.sort_references(references, &sort_spec);
        }

        let sorter = Sorter::new(self.get_config(), &self.locale);
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
                .filter_map(|item| self.bibliography.get(&item.id).map(|r| (item, r)))
                .collect();

            let resolved_sort = sort_spec.resolve();
            let sorter = crate::grouping::GroupSorter::new(&self.locale);
            items_with_refs.sort_by(|a, b| {
                for sort_key in &resolved_sort.template {
                    let cmp = sorter.compare_by_key(a.1, b.1, sort_key);
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }
                std::cmp::Ordering::Equal
            });

            return items_with_refs.into_iter().map(|(item, _)| item).collect();
        }
        items
    }

    /// Calculate disambiguation hints needed for the style.
    ///
    /// Analyzes the bibliography to determine which items need disambiguation
    /// (year suffixes, etc.) and calculates hints for efficient rendering.
    pub fn calculate_hints(&self) -> HashMap<String, ProcHints> {
        let cite_config = self.get_citation_config();
        let config = cite_config.as_ref();

        let bib_sort_resolved = self.resolved_bibliography_sort();

        let disambiguator = if let Some(resolved) = &bib_sort_resolved {
            Disambiguator::with_group_sort(&self.bibliography, config, &self.locale, resolved)
        } else {
            Disambiguator::new(&self.bibliography, config, &self.locale)
        };

        disambiguator.calculate_hints()
    }

    /// Check whether primary contributors match between two references.
    ///
    /// Used for subsequent author substitution in bibliographies.
    pub fn contributors_match(&self, prev: &Reference, current: &Reference) -> bool {
        let matcher = Matcher::new(&self.style, &self.default_config);
        matcher.contributors_match(prev, current)
    }

    /// Replace the primary contributor in a bibliography entry with a substitution string.
    ///
    /// Used for subsequent author substitution (e.g., "———" for repeating authors).
    pub fn apply_author_substitution(&self, proc: &mut ProcTemplate, substitute: &str) {
        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            self.get_config(),
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );
        renderer.apply_author_substitution(proc, substitute);
    }

    /// Merge compound numeric groups in the bibliography.
    ///
    /// Entries sharing a compound group are collapsed: the first entry's
    /// rendered content is prefixed with "a)" and subsequent entries are
    /// appended with "b)", "c)", etc., joined by the configured sub-delimiter.
    fn merge_compound_entries<F>(&self, entries: Vec<ProcEntry>) -> Vec<ProcEntry>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let compound_groups = self.compound_groups.borrow();
        if compound_groups.is_empty() {
            return entries;
        }

        let compound_config = match self
            .get_config()
            .bibliography
            .as_ref()
            .and_then(|b| b.compound_numeric.as_ref())
        {
            Some(c) => c.clone(),
            None => return entries,
        };

        // Build lookup: ref_id -> group_number for all compound groups.
        let mut ref_to_group: HashMap<String, usize> = HashMap::new();
        for (&num, ids) in compound_groups.iter() {
            if ids.len() > 1 {
                for id in ids {
                    ref_to_group.insert(id.clone(), num);
                }
            }
        }

        if ref_to_group.is_empty() {
            return entries;
        }

        // Helper: is this component a citation-number label (e.g. `[1]`)?
        let is_label_component = |comp: &crate::render::component::ProcTemplateComponent| -> bool {
            matches!(
                &comp.template_component,
                citum_schema::template::TemplateComponent::Number(n)
                    if n.number
                        == citum_schema::template::NumberVariable::CitationNumber
            )
        };

        // First pass: render each entry's content WITHOUT the citation-number label.
        // This prevents the label from appearing inside each sub-entry when merged
        // (e.g. "a) [1] Zwart..." is wrong; content should be "a) Zwart...").
        // Use refs_to_string_with_format (not citation renderer) so bibliography
        // separators are applied correctly between author/title/year etc.
        let mut rendered_strings: HashMap<String, String> = HashMap::new();
        for entry in &entries {
            let content_components: Vec<_> = entry
                .template
                .iter()
                .filter(|c| !is_label_component(c))
                .cloned()
                .collect();
            let content_entry = ProcEntry {
                id: entry.id.clone(),
                template: content_components,
                metadata: entry.metadata.clone(),
            };
            let rendered = render_entry_body_with_format::<F>(&content_entry);
            rendered_strings.insert(entry.id.clone(), rendered.trim().to_string());
        }

        let entries_by_id: HashMap<String, ProcEntry> = entries
            .iter()
            .map(|entry| (entry.id.clone(), entry.clone()))
            .collect();

        let mut group_members_present: HashMap<usize, Vec<String>> = HashMap::new();
        for entry in &entries {
            if let Some(&group_num) = ref_to_group.get(&entry.id) {
                group_members_present
                    .entry(group_num)
                    .or_default()
                    .push(entry.id.clone());
            }
        }

        let first_present_by_group: HashMap<usize, String> = group_members_present
            .iter()
            .filter_map(|(&group_num, ids)| ids.first().cloned().map(|id| (group_num, id)))
            .collect();

        // Second pass: build merged output
        let mut result: Vec<ProcEntry> = Vec::new();

        for entry in entries {
            if let Some(&group_num) = ref_to_group.get(&entry.id) {
                let Some(present_ids) = group_members_present.get(&group_num) else {
                    result.push(entry);
                    continue;
                };

                if present_ids.len() == 1 {
                    result.push(entry);
                    continue;
                }

                if first_present_by_group.get(&group_num) == Some(&entry.id) {
                    // First in group — build merged entry
                    let group_ids = &compound_groups[&group_num];
                    let mut parts: Vec<String> = Vec::new();

                    for (i, id) in group_ids.iter().enumerate() {
                        if !entries_by_id.contains_key(id) {
                            continue;
                        }
                        let sub_label = match compound_config.sub_label {
                            citum_schema::options::bibliography::SubLabelStyle::Alphabetic => {
                                format!(
                                    "{}{}",
                                    crate::values::int_to_letter((i + 1) as u32)
                                        .unwrap_or_else(|| "a".to_string()),
                                    compound_config.sub_label_suffix
                                )
                            }
                            citum_schema::options::bibliography::SubLabelStyle::Numeric => {
                                format!("{}{}", i + 1, compound_config.sub_label_suffix)
                            }
                        };
                        if let Some(rendered) = rendered_strings.get(id) {
                            parts.push(format!("{} {}", sub_label, rendered));
                        }
                    }

                    let merged_content = parts.join(&compound_config.sub_delimiter);

                    // Keep citation-number label components intact so the bibliography
                    // renderer outputs the label (e.g. "[1] ") once, then appends the
                    // merged sub-entries as a single pre-formatted content component.
                    let mut merged_template: Vec<_> = entry
                        .template
                        .iter()
                        .filter(|c| is_label_component(c))
                        .cloned()
                        .collect();
                    merged_template.push(crate::render::component::ProcTemplateComponent {
                        template_component: citum_schema::template::TemplateComponent::default(),
                        value: merged_content,
                        pre_formatted: true,
                        config: entry.template.first().and_then(|c| c.config.clone()),
                        ..Default::default()
                    });

                    result.push(ProcEntry {
                        id: entry.id.clone(),
                        template: merged_template,
                        metadata: entry.metadata,
                    });
                }
                // else: skip non-first members of a group
            } else {
                // Not in any compound group — pass through
                result.push(entry);
            }
        }

        result
    }

    /// Render the bibliography to a string using a specific format.
    pub fn render_bibliography_with_format<F>(&self) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.render_selected_bibliography_with_format::<F, _>(
            self.bibliography.keys().cloned().collect::<Vec<_>>(),
        )
    }

    /// Render a selected bibliography subset to a string using a specific format.
    pub fn render_selected_bibliography_with_format<F, I>(&self, item_ids: I) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
        I: IntoIterator<Item = String>,
    {
        self.initialize_numeric_citation_numbers();
        let selected: HashSet<String> = item_ids.into_iter().collect();
        let sorted_refs = self.sort_references(self.bibliography.values().collect());
        let mut bibliography: Vec<ProcEntry> = Vec::new();
        let mut prev_reference: Option<&Reference> = None;

        let bib_config = self.get_config().bibliography.as_ref();
        let substitute = bib_config.and_then(|c| c.subsequent_author_substitute.as_ref());

        for (index, reference) in sorted_refs.iter().enumerate() {
            let ref_id = reference.id().unwrap_or_default();
            if !selected.contains(&ref_id) {
                continue;
            }
            let entry_number = self
                .citation_numbers
                .borrow()
                .get(&ref_id)
                .copied()
                .unwrap_or(index + 1);

            if let Some(mut proc) =
                self.process_bibliography_entry_with_format::<F>(reference, entry_number)
            {
                if let Some(sub_string) = substitute
                    && let Some(prev) = prev_reference
                    && self.contributors_match(prev, reference)
                {
                    let bib_config = self.get_bibliography_config();
                    let renderer = Renderer::new(
                        &self.style,
                        &self.bibliography,
                        &self.locale,
                        &bib_config,
                        &self.hints,
                        &self.citation_numbers,
                        CompoundRenderData {
                            set_by_ref: &self.compound_set_by_ref,
                            member_index: &self.compound_member_index,
                            sets: &self.compound_sets,
                        },
                    );
                    renderer.apply_author_substitution_with_format::<F>(&mut proc, sub_string);
                }

                bibliography.push(ProcEntry {
                    id: ref_id.clone(),
                    template: proc,
                    metadata: self.extract_metadata(reference),
                });
                prev_reference = Some(reference);
            }
        }

        let bibliography = self.merge_compound_entries::<F>(bibliography);
        crate::render::refs_to_string_with_format::<F>(bibliography, None, None)
    }

    /// Process a bibliography entry with specific format.
    pub fn process_bibliography_entry_with_format<F>(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        // Use bibliography-specific merged config
        let bib_config = self.get_bibliography_config();

        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            &bib_config,
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );
        renderer.process_bibliography_entry_with_format::<F>(reference, entry_number)
    }

    /// Render a citation to a string using a specific format.
    pub fn process_citation_with_format<F>(
        &self,
        citation: &Citation,
    ) -> Result<String, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.initialize_numeric_citation_numbers();
        // Track cited IDs
        for item in &citation.items {
            self.cited_ids.borrow_mut().insert(item.id.clone());
        }

        // Resolve the effective citation spec (position first, then mode)
        let default_spec = citum_schema::CitationSpec::default();
        let effective_spec = self.style.citation.as_ref().map_or_else(
            || std::borrow::Cow::Borrowed(&default_spec),
            |cs| {
                // Resolve position first (owned), then mode on the owned spec
                let position_resolved = cs.resolve_for_position(citation.position.as_ref());
                let spec_for_mode = position_resolved.into_owned();
                std::borrow::Cow::Owned(spec_for_mode.resolve_for_mode(&citation.mode).into_owned())
            },
        );

        // Sort items if sort spec is present
        let sorted_items = self.sort_citation_items(citation.items.clone(), &effective_spec);

        let intra_delimiter = effective_spec.delimiter.as_deref().unwrap_or(", ");
        let renderer_delimiter = if matches!(
            DelimiterPunctuation::from_csl_string(intra_delimiter),
            DelimiterPunctuation::None
        ) {
            ""
        } else {
            intra_delimiter
        };

        let inter_delimiter = effective_spec
            .multi_cite_delimiter
            .as_deref()
            .unwrap_or("; ");
        let renderer_inter_delimiter = if matches!(
            DelimiterPunctuation::from_csl_string(inter_delimiter),
            DelimiterPunctuation::None
        ) {
            ""
        } else {
            inter_delimiter
        };

        let cite_config = self.get_citation_config();
        let processing = cite_config.processing.clone().unwrap_or_default();
        let is_author_date = !matches!(
            processing,
            citum_schema::options::Processing::Numeric
                | citum_schema::options::Processing::Label(_)
        );
        let renderer = Renderer::new(
            &self.style,
            &self.bibliography,
            &self.locale,
            &cite_config,
            &self.hints,
            &self.citation_numbers,
            CompoundRenderData {
                set_by_ref: &self.compound_set_by_ref,
                member_index: &self.compound_member_index,
                sets: &self.compound_sets,
            },
        );

        // Process group components
        let rendered_groups = if is_author_date {
            renderer.render_grouped_citation_with_format::<F>(
                &sorted_items,
                &effective_spec,
                &citation.mode,
                renderer_delimiter,
                citation.suppress_author,
                citation.position.as_ref(),
            )?
        } else {
            renderer.render_ungrouped_citation_with_format::<F>(
                &sorted_items,
                &effective_spec,
                &citation.mode,
                renderer_delimiter,
                citation.suppress_author,
                citation.position.as_ref(),
            )?
        };

        let fmt = F::default();
        let content = fmt.join(rendered_groups, renderer_inter_delimiter);

        // Apply citation-level prefix/suffix from input
        let citation_prefix = citation.prefix.as_deref().unwrap_or("");
        let citation_suffix = citation.suffix.as_deref().unwrap_or("");

        // Ensure proper spacing for prefix/suffix
        let formatted_prefix =
            if !citation_prefix.is_empty() && !citation_prefix.ends_with(char::is_whitespace) {
                format!("{} ", citation_prefix)
            } else {
                citation_prefix.to_string()
            };

        let formatted_suffix =
            if !citation_suffix.is_empty() && !citation_suffix.starts_with(char::is_whitespace) {
                format!(" {}", citation_suffix)
            } else {
                citation_suffix.to_string()
            };

        let output = if !citation_prefix.is_empty() || !citation_suffix.is_empty() {
            fmt.affix(&formatted_prefix, content, &formatted_suffix)
        } else {
            content
        };

        // Get wrap/prefix/suffix from citation spec
        let wrap = effective_spec
            .wrap
            .as_ref()
            .unwrap_or(&WrapPunctuation::None);
        let spec_prefix = effective_spec.prefix.as_deref().unwrap_or("");
        let spec_suffix = effective_spec.suffix.as_deref().unwrap_or("");

        // For integral (narrative) citations, don't apply wrapping
        // (they're part of the narrative text, not parenthetical)
        let wrapped = if matches!(
            citation.mode,
            citum_schema::citation::CitationMode::Integral
        ) {
            // Integral mode: skip wrapping, apply only prefix/suffix
            if !spec_prefix.is_empty() || !spec_suffix.is_empty() {
                fmt.affix(spec_prefix, output, spec_suffix)
            } else {
                output
            }
        } else if *wrap != WrapPunctuation::None {
            // Non-integral mode: apply wrap
            fmt.wrap_punctuation(wrap, output)
        } else if !spec_prefix.is_empty() || !spec_suffix.is_empty() {
            fmt.affix(spec_prefix, output, spec_suffix)
        } else {
            output
        };

        Ok(fmt.finish(wrapped))
    }

    /// Render multiple citations in document order.
    ///
    /// For note-based styles, normalizes context and assigns citation positions.
    pub fn process_citations(&self, citations: &[Citation]) -> Result<Vec<String>, ProcessorError> {
        self.process_citations_with_format::<crate::render::plain::PlainText>(citations)
    }

    /// Render multiple citations with a custom output format.
    pub fn process_citations_with_format<F>(
        &self,
        citations: &[Citation],
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut normalized = self.normalize_note_context(citations);
        self.annotate_positions(&mut normalized);
        normalized
            .iter()
            .map(|c| self.process_citation_with_format::<F>(c))
            .collect()
    }

    /// Render the entire bibliography to a formatted string.
    pub fn render_bibliography(&self) -> String {
        self.render_bibliography_with_format::<crate::render::plain::PlainText>()
    }

    /// Render the bibliography with grouping for uncited (nocite) items.
    ///
    /// If `style.bibliography.groups` is defined, uses configurable grouping
    /// with per-group sorting. Otherwise, falls back to hardcoded cited/uncited
    /// grouping for backward compatibility.
    pub fn render_grouped_bibliography_with_format<F>(&self) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let processed = self.process_references();
        let merged_bibliography = self.merge_compound_entries::<F>(processed.bibliography);

        // Check if style defines custom groups
        if let Some(bib_spec) = &self.style.bibliography
            && let Some(groups) = &bib_spec.groups
        {
            return self.render_with_custom_groups::<F>(&merged_bibliography, groups);
        }

        // Fallback to hardcoded cited/uncited grouping
        self.render_with_legacy_grouping::<F>(&merged_bibliography)
    }

    /// Render bibliography for a specific group selector.
    pub(crate) fn render_bibliography_for_group<F>(
        &self,
        group: &citum_schema::BibliographyGroup,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        use crate::grouping::{GroupSorter, SelectorEvaluator};

        let processed = self.process_references();
        let merged_bibliography = self.merge_compound_entries::<F>(processed.bibliography);
        let bibliography = &merged_bibliography;

        let fmt = F::default();
        let cited_ids = self.cited_ids.borrow();
        let evaluator = SelectorEvaluator::new(&cited_ids);
        let sorter = GroupSorter::new(&self.locale);

        let matching_refs: Vec<&Reference> = bibliography
            .iter()
            .filter_map(|entry| {
                self.bibliography
                    .get(&entry.id)
                    .filter(|reference| evaluator.matches(reference, &group.selector))
            })
            .collect();

        if matching_refs.is_empty() {
            return fmt.finish(String::new());
        }

        let sorted_refs = if let Some(sort_spec) = &group.sort {
            sorter.sort_references(matching_refs, &sort_spec.resolve())
        } else {
            matching_refs
        };

        let entries: Vec<ProcEntry> = sorted_refs
            .into_iter()
            .filter_map(|r| {
                let id = r.id()?;
                bibliography.iter().find(|e| e.id == id).cloned()
            })
            .collect();

        fmt.finish(crate::render::refs_to_string_with_format::<F>(
            entries, None, None,
        ))
    }

    fn resolve_group_heading(&self, heading: &citum_schema::GroupHeading) -> Option<String> {
        match heading {
            citum_schema::GroupHeading::Literal { literal } => Some(literal.clone()),
            citum_schema::GroupHeading::Term { term, form } => self
                .locale
                .general_term(term, form.unwrap_or(citum_schema::locale::TermForm::Long))
                .map(ToOwned::to_owned),
            citum_schema::GroupHeading::Localized { localized } => {
                self.resolve_localized_heading(localized)
            }
        }
    }

    fn resolve_localized_heading(&self, localized: &HashMap<String, String>) -> Option<String> {
        fn language_tag(locale: &str) -> &str {
            locale.split('-').next().unwrap_or(locale)
        }

        let mut candidates: Vec<String> = Vec::new();
        let mut push_candidate = |locale: &str| {
            let candidate = locale.to_string();
            if !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        };

        push_candidate(&self.locale.locale);
        push_candidate(language_tag(&self.locale.locale));

        if let Some(default_locale) = self.style.info.default_locale.as_deref() {
            push_candidate(default_locale);
            push_candidate(language_tag(default_locale));
        }

        push_candidate("en-US");
        push_candidate("en");

        for locale in candidates {
            if let Some(value) = localized.get(&locale) {
                return Some(value.clone());
            }
        }

        localized
            .iter()
            .min_by(|a, b| a.0.cmp(b.0))
            .map(|(_, value)| value.clone())
    }

    /// Render bibliography with configurable groups.
    fn render_with_custom_groups<F>(
        &self,
        bibliography: &[ProcEntry],
        groups: &[citum_schema::BibliographyGroup],
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        use crate::grouping::{GroupSorter, SelectorEvaluator};
        use citum_schema::grouping::DisambiguationScope;
        use std::collections::HashSet;

        let fmt = F::default();
        let cited_ids = self.cited_ids.borrow();

        let evaluator = SelectorEvaluator::new(&cited_ids);
        let sorter = GroupSorter::new(&self.locale);

        let mut assigned: HashSet<String> = HashSet::new();
        let mut result = String::new();

        for group in groups {
            // Find items matching this group's selector
            let matching_refs: Vec<&Reference> = bibliography
                .iter()
                .filter(|entry| !assigned.contains(&entry.id))
                .filter_map(|entry| {
                    self.bibliography
                        .get(&entry.id)
                        .filter(|reference| evaluator.matches(reference, &group.selector))
                })
                .collect();

            if matching_refs.is_empty() {
                continue;
            }

            // Mark as assigned (first-match semantics)
            for r in &matching_refs {
                if let Some(id) = r.id() {
                    assigned.insert(id);
                }
            }

            // Sort using per-group or global sort
            let sorted_refs = if let Some(sort_spec) = &group.sort {
                sorter.sort_references(matching_refs, &sort_spec.resolve())
            } else {
                // references in `matching_refs` are in original global-sort order
                matching_refs
            };

            // Handle local disambiguation if requested
            let local_hints = if matches!(group.disambiguate, Some(DisambiguationScope::Locally)) {
                let mut group_bib = Bibliography::new();
                for r in &sorted_refs {
                    group_bib.insert(r.id().unwrap_or_default(), (*r).clone());
                }
                let group_sort_resolved = group.sort.as_ref().map(|s| s.resolve());
                let disambiguator = if let Some(resolved) = &group_sort_resolved {
                    Disambiguator::with_group_sort(
                        &group_bib,
                        self.get_config(),
                        &self.locale,
                        resolved,
                    )
                } else {
                    Disambiguator::new(&group_bib, self.get_config(), &self.locale)
                };
                Some(disambiguator.calculate_hints())
            } else {
                None
            };

            // Re-render entries if local hints or local template is present
            let entries_vec: Vec<ProcEntry> = if local_hints.is_some() || group.template.is_some() {
                let hints = local_hints.as_ref().unwrap_or(&self.hints);
                let bib_config = self.get_bibliography_config();

                // Create a local style if we have a group-specific template
                let effective_style = if let Some(group_template) = &group.template {
                    let mut local_style = self.style.clone();
                    if let Some(bib_spec) = local_style.bibliography.as_mut() {
                        bib_spec.template = Some(group_template.clone());
                    }
                    std::borrow::Cow::Owned(local_style)
                } else {
                    std::borrow::Cow::Borrowed(&self.style)
                };

                let renderer = Renderer::new(
                    &effective_style,
                    &self.bibliography,
                    &self.locale,
                    &bib_config,
                    hints,
                    &self.citation_numbers,
                    CompoundRenderData {
                        set_by_ref: &self.compound_set_by_ref,
                        member_index: &self.compound_member_index,
                        sets: &self.compound_sets,
                    },
                );

                sorted_refs
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| ProcEntry {
                        id: r.id().unwrap_or_default(),
                        template: renderer
                            .process_bibliography_entry(r, i + 1)
                            .unwrap_or_default(),
                        metadata: self.extract_metadata(r),
                    })
                    .collect()
            } else {
                // Use pre-rendered entries in sorted order
                sorted_refs
                    .into_iter()
                    .filter_map(|r| {
                        let id = r.id()?;
                        bibliography.iter().find(|e| e.id == id).cloned()
                    })
                    .collect()
            };

            // Add group heading
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            if let Some(heading) = &group.heading
                && let Some(resolved_heading) = self.resolve_group_heading(heading)
            {
                result.push_str(&format!("# {}\n\n", resolved_heading));
            }

            // Render entries
            result.push_str(&crate::render::refs_to_string_with_format::<F>(
                entries_vec,
                None,
                None,
            ));
        }

        // Fallback for ungrouped items
        let unassigned: Vec<ProcEntry> = bibliography
            .iter()
            .filter(|e| !assigned.contains(&e.id))
            .cloned()
            .collect();

        if !unassigned.is_empty() {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&crate::render::refs_to_string_with_format::<F>(
                unassigned, None, None,
            ));
        }

        fmt.finish(result)
    }

    /// Legacy hardcoded cited/uncited grouping.
    fn render_with_legacy_grouping<F>(&self, bibliography: &[ProcEntry]) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let cited_ids = self.cited_ids.borrow();

        // Items cited visibly
        let cited_entries: Vec<ProcEntry> = bibliography
            .iter()
            .filter(|e| cited_ids.contains(&e.id))
            .cloned()
            .collect();

        let mut result = String::new();

        if !cited_entries.is_empty() {
            result.push_str(&crate::render::refs_to_string_with_format::<F>(
                cited_entries,
                None,
                None,
            ));
        }

        fmt.finish(result)
    }
}
