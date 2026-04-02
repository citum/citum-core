use super::super::{
    GroupRenderParams, Renderer, TemplateComponentTracker, TemplateRenderParams,
    TemplateRenderRequest, find_grouping_component, get_variable_key, has_contributor_component,
    leading_group_affix, strip_author_component, strip_leading_group_affixes,
};
use super::group_citation_items_by_author;
use crate::error::ProcessorError;
use crate::reference::Reference;
use crate::render::{ProcTemplate, ProcTemplateComponent};
use crate::values::{ComponentValues, ProcHints, RenderContext, RenderOptions};
use citum_schema::{
    options::ArticleJournalNoPageFallback,
    reference::NumOrStr,
    template::{DateVariable, NumberVariable, SimpleVariable, TemplateComponent},
};
use std::borrow::Cow;

#[derive(Clone, Copy)]
enum ArticleJournalBibliographyMode {
    StandardDetail,
    DoiFallback,
}

#[derive(Clone, Copy)]
enum AnonymousEntryBibliographyMode {
    ContainerLed,
    SuppressPrintLike,
}

struct GroupRenderState<'a> {
    first_item: &'a crate::reference::CitationItem,
    first_ref: &'a Reference,
    template: Cow<'a, [TemplateComponent]>,
}

struct ItemRenderState<'a> {
    item: &'a crate::reference::CitationItem,
    reference: &'a Reference,
    template: Cow<'a, [TemplateComponent]>,
}

struct GroupItemRenderRequest<'a> {
    item: &'a crate::reference::CitationItem,
    template: &'a [TemplateComponent],
    mode: &'a citum_schema::citation::CitationMode,
    suppress_author: bool,
    position: Option<&'a citum_schema::citation::Position>,
    delimiter: &'a str,
}

/// Returns the first type-variant template whose selector matches `ref_type`,
/// or `None` if there are no variants or none match.
fn resolve_type_variant<'a>(
    type_variants: Option<
        &'a indexmap::IndexMap<citum_schema::template::TypeSelector, citum_schema::Template>,
    >,
    ref_type: &str,
) -> Option<&'a [TemplateComponent]> {
    let selector_candidates = aliased_type_selector_candidates(ref_type);
    type_variants?.iter().find_map(|(selector, template)| {
        selector_candidates
            .iter()
            .any(|candidate| selector.matches(candidate))
            .then_some(template.as_slice())
    })
}

impl Renderer<'_> {
    fn strip_redundant_leading_group_punctuation<'a>(
        &self,
        value: &'a str,
        delimiter: &str,
    ) -> &'a str {
        let Some(delimiter_char) = delimiter.chars().find(|ch| !ch.is_whitespace()) else {
            return value;
        };

        let trimmed = value.trim_start();
        if !trimmed.starts_with(delimiter_char) {
            return value;
        }

        trimmed[delimiter_char.len_utf8()..].trim_start()
    }

    fn join_integral_group_item_parts(&self, item_parts: &[String], delimiter: &str) -> String {
        let repeated_item_delimiter = if delimiter.trim().is_empty() {
            ", "
        } else {
            delimiter
        };

        let mut joined = String::new();
        for (index, part) in item_parts.iter().enumerate() {
            if index > 0 {
                joined.push_str(repeated_item_delimiter);
            }

            let normalized = if index == 0 {
                part.as_str()
            } else {
                self.strip_redundant_leading_group_punctuation(part, repeated_item_delimiter)
            };
            joined.push_str(normalized);
        }

        joined
    }

    /// Render citation items with author grouping, using plain text format.
    ///
    /// # Errors
    ///
    /// Returns an error when a referenced item is missing or grouped rendering fails.
    pub fn render_grouped_citation(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Vec<String>, ProcessorError> {
        self.render_grouped_citation_with_format::<crate::render::plain::PlainText>(
            items,
            spec,
            mode,
            intra_delimiter,
            suppress_author,
            position,
        )
    }

    /// Render a group of items that must not be author-collapsed (legal cases,
    /// personal communications). Returns the rendered citation strings.
    fn render_special_type_items<F>(
        &self,
        group: &[&crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
        intra_delimiter: &str,
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let mut rendered_items = Vec::new();
        for item in group {
            let state = self.resolve_item_render_state(item, spec)?;
            if let Some(item_str) = self.render_group_item_from_template_with_format::<F>(
                state.reference,
                GroupItemRenderRequest {
                    item: state.item,
                    template: &state.template,
                    mode,
                    suppress_author,
                    position,
                    delimiter: intra_delimiter,
                },
            ) && let Some((ids, content)) = self.build_citation_chunk(
                &fmt,
                vec![item.id.clone()],
                item_str,
                item.prefix.as_deref(),
                item.suffix.as_deref(),
            ) {
                rendered_items.push(fmt.citation(ids, content));
            }
        }
        Ok(rendered_items)
    }

    /// Render one citation group using the explicit integral template.
    ///
    /// Returns `Ok(Some(citation))` if the group rendered (caller should push and `continue`),
    /// or `Ok(None)` if no items produced output (caller should fall through to other branches).
    fn render_integral_explicit_group<F>(
        &self,
        group: &[&crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Option<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let component_delimiter = spec.delimiter.as_deref().unwrap_or(" ");
        let item_join_delim = spec.multi_cite_delimiter.as_deref().unwrap_or(", ");
        let mut group_items_str = Vec::new();
        let mut all_ids = Vec::new();

        for item in group {
            let state = self.resolve_item_render_state(item, spec)?;
            if let Some(item_str) = self.render_group_item_from_template_with_format::<F>(
                state.reference,
                GroupItemRenderRequest {
                    item: state.item,
                    template: &state.template,
                    mode,
                    suppress_author,
                    position,
                    delimiter: component_delimiter,
                },
            ) && !item_str.is_empty()
            {
                group_items_str.push(self.affix_content(
                    &fmt,
                    item_str,
                    item.prefix.as_deref(),
                    item.suffix.as_deref(),
                ));
                all_ids.push(item.id.clone());
            }
        }

        if group_items_str.is_empty() {
            return Ok(None);
        }

        let combined_str = group_items_str.join(item_join_delim);
        Ok(Some(fmt.citation(all_ids, combined_str)))
    }

    /// This preserves per-item output when grouping rules require items to stay
    /// separate, and otherwise applies the requested renderer format to the
    /// grouped citation output.
    ///
    /// # Errors
    ///
    /// Returns an error when a referenced item is missing or grouped rendering
    /// fails.
    pub fn render_grouped_citation_with_format<F>(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let groups = group_citation_items_by_author(self, items);
        let mut rendered_groups = Vec::new();
        for (_author_key, group) in groups {
            rendered_groups.extend(self.render_grouped_citation_group_with_format::<F>(
                &group,
                spec,
                mode,
                intra_delimiter,
                suppress_author,
                position,
            )?);
        }

        Ok(rendered_groups)
    }

    fn render_grouped_citation_group_with_format<F>(
        &self,
        group: &[&crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Vec<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let state = self.resolve_group_render_state(group, spec)?;

        if let Some(citation) = self.try_render_integral_group_with_format::<F>(
            group,
            spec,
            mode,
            suppress_author,
            position,
        )? {
            return Ok(vec![citation]);
        }

        if self.requires_full_group_item_rendering(mode, state.first_ref) {
            return self.render_special_type_items::<F>(
                group,
                spec,
                mode,
                suppress_author,
                position,
                intra_delimiter,
            );
        }

        Ok(self
            .render_fallback_grouped_citation_with_format::<F>(
                group,
                state.first_ref,
                state.first_item,
                &state.template,
                &GroupRenderParams {
                    spec,
                    mode,
                    intra_delimiter,
                    suppress_author,
                    position,
                },
            )?
            .into_iter()
            .collect())
    }

    fn render_fallback_grouped_citation_with_format<F>(
        &self,
        group: &[&crate::reference::CitationItem],
        first_ref: &Reference,
        first_item: &crate::reference::CitationItem,
        template: &[TemplateComponent],
        params: &GroupRenderParams<'_>,
    ) -> Result<Option<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let author_part = self.render_author_for_grouping_with_format::<F>(
            first_ref,
            first_item,
            template,
            params.mode,
            params.suppress_author,
            params.position,
        );
        let (item_parts, group_delimiter) =
            self.render_group_item_parts_with_format::<F>(&fmt, group, params)?;
        let Some(content) = self.build_grouped_citation_content(
            &author_part,
            &item_parts,
            params,
            group_delimiter.as_deref(),
        ) else {
            return Ok(None);
        };
        let group_ids = group.iter().map(|item| item.id.clone()).collect();
        let prefix = first_item.prefix.as_deref().unwrap_or("");
        // Suffix is embedded in item_parts by render_group_item_parts_with_format when
        // item_parts is non-empty. Apply it here only when item_parts was empty (author-only output).
        let suffix = if item_parts.is_empty() {
            first_item.suffix.as_deref()
        } else {
            None
        };

        Ok(Some(fmt.citation(
            group_ids,
            self.affix_content(&fmt, content, Some(prefix), suffix),
        )))
    }

    fn build_grouped_citation_content(
        &self,
        author_part: &str,
        item_parts: &[String],
        params: &GroupRenderParams<'_>,
        group_delimiter: Option<&str>,
    ) -> Option<String> {
        if !author_part.is_empty() && !item_parts.is_empty() {
            let author_item_delimiter = group_delimiter.unwrap_or(params.intra_delimiter);
            let joined_items = match params.mode {
                citum_schema::citation::CitationMode::Integral => {
                    self.join_integral_group_item_parts(item_parts, author_item_delimiter)
                }
                citum_schema::citation::CitationMode::NonIntegral => {
                    let repeated_item_delimiter = if author_item_delimiter.trim().is_empty() {
                        ", "
                    } else {
                        author_item_delimiter
                    };
                    item_parts.join(repeated_item_delimiter)
                }
            };
            return Some(match params.mode {
                citum_schema::citation::CitationMode::Integral => self
                    .format_integral_grouped_items(
                        author_part,
                        &joined_items,
                        params.suppress_author,
                    ),
                citum_schema::citation::CitationMode::NonIntegral => self
                    .format_non_integral_grouped_items(
                        author_part,
                        author_item_delimiter,
                        &joined_items,
                        params.suppress_author,
                    ),
            });
        }

        if !author_part.is_empty() {
            return Some(author_part.to_string());
        }

        if !item_parts.is_empty() {
            return Some(item_parts.join(params.intra_delimiter));
        }

        None
    }

    fn format_integral_grouped_items(
        &self,
        author_part: &str,
        joined_items: &str,
        suppress_author: bool,
    ) -> String {
        if suppress_author {
            format!("({joined_items})")
        } else {
            format!("{author_part} ({joined_items})")
        }
    }

    fn format_non_integral_grouped_items(
        &self,
        author_part: &str,
        author_item_delimiter: &str,
        joined_items: &str,
        suppress_author: bool,
    ) -> String {
        if suppress_author {
            return joined_items.to_string();
        }

        if let Some(adjusted) =
            self.adjust_grouped_author_quote_punctuation(author_part, author_item_delimiter)
        {
            return format!("{adjusted}{joined_items}");
        }

        format!("{author_part}{author_item_delimiter}{joined_items}")
    }

    fn adjust_grouped_author_quote_punctuation(
        &self,
        author_part: &str,
        author_item_delimiter: &str,
    ) -> Option<String> {
        if !self.config.punctuation_in_quote
            || !author_item_delimiter.starts_with(',')
            || !(author_part.ends_with('"') || author_part.ends_with('\u{201D}'))
        {
            return None;
        }

        let is_curly = author_part.ends_with('\u{201D}');
        let quote_char = if is_curly { '\u{201D}' } else { '"' };
        let trimmed = &author_part[..author_part.len() - quote_char.len_utf8()];
        Some(format!(
            "{trimmed},{quote_char}{}",
            &author_item_delimiter[1..]
        ))
    }

    fn render_group_item_parts_with_format<F>(
        &self,
        fmt: &F,
        group: &[&crate::reference::CitationItem],
        params: &GroupRenderParams<'_>,
    ) -> Result<(Vec<String>, Option<String>), ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut item_parts = Vec::new();
        let mut group_delimiter: Option<String> = None;
        for (index, item) in group.iter().enumerate() {
            let state = self.resolve_item_render_state(item, params.spec)?;
            let (filtered_template, leading_affix) = filter_author_from_template(&state.template);
            if group_delimiter.is_none() {
                group_delimiter = leading_affix
                    .as_ref()
                    .filter(|value| !value.is_empty())
                    .cloned();
            }
            let item_delimiter = if leading_affix.is_some() {
                ""
            } else {
                params.intra_delimiter
            };
            if let Some(item_str) = self.render_group_item_from_template_with_format::<F>(
                state.reference,
                GroupItemRenderRequest {
                    item: state.item,
                    template: &filtered_template,
                    mode: params.mode,
                    suppress_author: params.suppress_author,
                    position: params.position,
                    delimiter: item_delimiter,
                },
            ) && !item_str.is_empty()
            {
                let prefix = (index > 0).then_some(item.prefix.as_deref()).flatten();
                item_parts.push(self.affix_content(fmt, item_str, prefix, item.suffix.as_deref()));
            }
        }
        Ok((item_parts, group_delimiter))
    }

    fn resolve_group_render_state<'b>(
        &'b self,
        group: &'b [&'b crate::reference::CitationItem],
        spec: &'b citum_schema::CitationSpec,
    ) -> Result<GroupRenderState<'b>, ProcessorError> {
        let first_item = group[0];
        let first_ref = self
            .bibliography
            .get(&first_item.id)
            .ok_or_else(|| ProcessorError::ReferenceNotFound(first_item.id.clone()))?;
        let first_language = crate::values::effective_item_language(first_ref);
        let default_template = spec
            .resolve_template_for_language(first_language.as_deref())
            .map(Cow::Owned);

        let ref_type = first_ref.ref_type();
        let first_template = resolve_type_variant(spec.type_variants.as_ref(), &ref_type)
            .map(Cow::Borrowed)
            .or(default_template);

        Ok(GroupRenderState {
            first_item,
            first_ref,
            template: first_template.unwrap_or(Cow::Borrowed(&[])),
        })
    }

    fn resolve_item_render_state<'b>(
        &'b self,
        item: &'b crate::reference::CitationItem,
        spec: &'b citum_schema::CitationSpec,
    ) -> Result<ItemRenderState<'b>, ProcessorError> {
        let reference = self
            .bibliography
            .get(&item.id)
            .ok_or_else(|| ProcessorError::ReferenceNotFound(item.id.clone()))?;
        let item_language = crate::values::effective_item_language(reference);
        let default_template = spec
            .resolve_template_for_language(item_language.as_deref())
            .map(Cow::Owned);

        let ref_type = reference.ref_type();
        let item_template = resolve_type_variant(spec.type_variants.as_ref(), &ref_type)
            .map(Cow::Borrowed)
            .or(default_template);

        Ok(ItemRenderState {
            item,
            reference,
            template: item_template.unwrap_or(Cow::Borrowed(&[])),
        })
    }

    fn try_render_integral_group_with_format<F>(
        &self,
        group: &[&crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Option<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        if !matches!(mode, citum_schema::citation::CitationMode::Integral)
            || !self.has_explicit_integral_template()
        {
            return Ok(None);
        }

        self.render_integral_explicit_group::<F>(group, spec, mode, suppress_author, position)
    }

    fn requires_full_group_item_rendering(
        &self,
        mode: &citum_schema::citation::CitationMode,
        reference: &Reference,
    ) -> bool {
        matches!(mode, citum_schema::citation::CitationMode::NonIntegral)
            && matches!(
                reference.ref_type().as_str(),
                "legal-case" | "personal-communication"
            )
    }

    /// Render just the author part for citation grouping.
    pub(crate) fn render_author_for_grouping_with_format<F>(
        &self,
        reference: &Reference,
        item: &crate::reference::CitationItem,
        template: &[TemplateComponent],
        mode: &citum_schema::citation::CitationMode,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> String
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let is_note_processing = self.config.processing.as_ref().is_some_and(|processing| {
            matches!(processing, citum_schema::options::Processing::Note)
        });
        if is_note_processing
            && matches!(
                position,
                Some(
                    citum_schema::citation::Position::Ibid
                        | citum_schema::citation::Position::IbidWithLocator
                )
            )
            && !template.iter().any(has_contributor_component)
        {
            return String::new();
        }

        let options = self.citation_render_options(mode.clone(), suppress_author, None, None);

        // Try to use the first semantically relevant component (including nested lists)
        // so disambiguation hints and component-specific formatting are preserved.
        // This ensures substitution, shortening, and mode-dependent conjunctions are respected.
        if let Some(comp) = template.first().and_then(find_grouping_component) {
            let base_hints = self
                .hints
                .get(&reference.id().unwrap_or_default())
                .cloned()
                .unwrap_or_default();
            // Inject citation position so subsequent et-al thresholds are applied.
            let hints = ProcHints {
                position: position.cloned(),
                integral_name_state: item.integral_name_state,
                ..base_hints
            };
            if let Some(vals) = comp.values::<F>(reference, &hints, &options)
                && !vals.value.is_empty()
            {
                return vals.value;
            }
        }

        // Fallback for cases where first component isn't suitable or returned empty
        if let Some(authors) = reference.author() {
            let names_vec = self.resolve_contributor_names(&authors);
            F::default().text(&crate::values::format_contributors_short(
                &names_vec, &options,
            ))
        } else {
            String::new()
        }
    }

    /// Render the prose anchor for an integral citation without any trailing note text.
    pub(crate) fn render_integral_anchor_with_format<F>(
        &self,
        items: &[crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        inter_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<String, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let groups = group_citation_items_by_author(self, items);

        let mut rendered_groups = Vec::new();
        let fmt = F::default();
        for (_author_key, group) in groups {
            let first_item = group[0];
            let reference = self
                .bibliography
                .get(&first_item.id)
                .ok_or_else(|| ProcessorError::ReferenceNotFound(first_item.id.clone()))?;
            let item_language = crate::values::effective_item_language(reference);
            let template = spec.resolve_template_for_language(item_language.as_deref());
            let effective_template = template.as_deref().unwrap_or(&[]);
            let author_part = self.render_author_for_grouping_with_format::<F>(
                reference,
                first_item,
                effective_template,
                &citum_schema::citation::CitationMode::Integral,
                suppress_author,
                position,
            );
            if !author_part.is_empty() {
                rendered_groups.push(author_part);
            }
        }

        Ok(fmt.join(rendered_groups, inter_delimiter))
    }

    /// Get the citation number for a reference, assigning one if not yet cited.
    #[must_use]
    pub fn get_or_assign_citation_number(&self, ref_id: &str) -> usize {
        let mut numbers = self.citation_numbers.borrow_mut();
        let next_num = numbers.len() + 1;
        *numbers.entry(ref_id.to_string()).or_insert(next_num)
    }

    /// Process a bibliography entry.
    #[must_use]
    pub fn process_bibliography_entry(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate> {
        self.process_bibliography_entry_with_format::<crate::render::plain::PlainText>(
            reference,
            entry_number,
        )
    }

    /// Process a bibliography entry with specific format.
    #[must_use]
    pub fn process_bibliography_entry_with_format<F>(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let bib_spec = self.style.bibliography.as_ref()?;

        let item_language = crate::values::effective_item_language(reference);
        let default_template = bib_spec
            .resolve_template_for_language(item_language.as_deref())
            .map(Cow::Owned);

        let ref_type = reference.ref_type();
        let template = resolve_type_variant(bib_spec.type_variants.as_ref(), &ref_type)
            .map(Cow::Borrowed)
            .or(default_template)?;

        let template = self.apply_anonymous_entry_bibliography_policy(reference, template)?;
        let template = self.apply_article_journal_bibliography_policy(reference, template);

        self.process_template_request_with_format::<F>(
            reference,
            TemplateRenderRequest {
                template: template.as_ref(),
                context: RenderContext::Bibliography,
                mode: citum_schema::citation::CitationMode::NonIntegral,
                suppress_author: false,
                locator_raw: None,
                citation_number: entry_number,
                position: None,
                integral_name_state: None,
            },
        )
    }

    /// Process a template for a reference using plain text format.
    ///
    /// Accepts a [`TemplateRenderParams`] bundle rather than individual arguments
    /// to keep the call site readable and avoid argument-count lint issues.
    #[must_use]
    pub fn process_template_with_number(
        &self,
        reference: &Reference,
        params: TemplateRenderParams<'_>,
    ) -> Option<ProcTemplate> {
        self.process_template_with_number_with_format::<crate::render::plain::PlainText>(
            reference, params,
        )
    }

    /// Process a template for a reference with a specific output format.
    ///
    /// Accepts a [`TemplateRenderParams`] bundle rather than individual arguments
    /// to keep the call site readable and avoid argument-count lint issues.
    pub fn process_template_with_number_with_format<F>(
        &self,
        reference: &Reference,
        params: TemplateRenderParams<'_>,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.process_template_request_with_format::<F>(
            reference,
            TemplateRenderRequest {
                template: params.template,
                context: params.context,
                mode: params.mode,
                suppress_author: params.suppress_author,
                locator_raw: params.locator_raw,
                citation_number: params.citation_number,
                position: params.position.cloned(),
                integral_name_state: params.integral_name_state,
            },
        )
    }

    /// Process a template request with a specific output format.
    #[must_use]
    pub fn process_template_request_with_format<F>(
        &self,
        reference: &Reference,
        request: TemplateRenderRequest<'_>,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let TemplateRenderRequest {
            template,
            context,
            mode,
            suppress_author,
            locator_raw,
            citation_number,
            position,
            integral_name_state,
        } = request;
        let ref_type = reference.ref_type();
        let options = RenderOptions {
            config: self.config,
            bibliography_config: self.bibliography_config.clone(),
            locale: self.locale,
            context,
            mode,
            suppress_author,
            locator_raw,
            ref_type: Some(ref_type.clone()),
            show_semantics: self.show_semantics,
            current_template_index: None,
        };
        let hint = self.build_template_render_hint(
            reference,
            options.context,
            citation_number,
            position,
            integral_name_state,
        );
        let mut tracker = TemplateComponentTracker::default();
        let components: Vec<ProcTemplateComponent> = template
            .iter()
            .enumerate()
            .filter_map(|(template_index, component)| {
                let mut component_options = options.clone();
                component_options.current_template_index =
                    self.inject_ast_indices.then_some(template_index);
                self.render_template_component_with_format::<F>(
                    reference,
                    &ref_type,
                    &component_options,
                    &hint,
                    template_index,
                    component,
                    &mut tracker,
                )
            })
            .collect();

        if components.is_empty() {
            None
        } else {
            Some(components)
        }
    }

    fn apply_article_journal_bibliography_policy<'a>(
        &self,
        reference: &Reference,
        template: Cow<'a, [TemplateComponent]>,
    ) -> Cow<'a, [TemplateComponent]> {
        let Some(mode) = self.article_journal_bibliography_mode(reference) else {
            return template;
        };

        if !article_journal_template_needs_filter(template.as_ref(), mode) {
            return template;
        }

        Cow::Owned(filter_article_journal_template_components(
            template.as_ref(),
            mode,
        ))
    }

    fn apply_anonymous_entry_bibliography_policy<'a>(
        &self,
        reference: &Reference,
        template: Cow<'a, [TemplateComponent]>,
    ) -> Option<Cow<'a, [TemplateComponent]>> {
        let Some(mode) = self.anonymous_entry_bibliography_mode(reference, template.as_ref())
        else {
            return Some(template);
        };

        match mode {
            AnonymousEntryBibliographyMode::ContainerLed => {
                Some(Cow::Owned(rewrite_anonymous_entry_template(
                    template.as_ref(),
                    mode,
                    reference.ref_type().as_str(),
                    reference_has_doi(reference),
                )))
            }
            AnonymousEntryBibliographyMode::SuppressPrintLike => None,
        }
    }

    fn article_journal_bibliography_mode(
        &self,
        reference: &Reference,
    ) -> Option<ArticleJournalBibliographyMode> {
        if reference.ref_type() != "article-journal" {
            return None;
        }

        let fallback = self
            .bibliography_config
            .as_ref()?
            .article_journal
            .as_ref()?
            .no_page_fallback?;

        match fallback {
            ArticleJournalNoPageFallback::Doi => {
                if reference_has_pages(reference) {
                    Some(ArticleJournalBibliographyMode::StandardDetail)
                } else if reference_has_doi(reference) {
                    Some(ArticleJournalBibliographyMode::DoiFallback)
                } else {
                    None
                }
            }
        }
    }

    fn anonymous_entry_bibliography_mode(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
    ) -> Option<AnonymousEntryBibliographyMode> {
        if !matches!(
            reference.ref_type().as_str(),
            "entry-dictionary" | "entry-encyclopedia" | "chapter"
        ) {
            return None;
        }

        if reference.ref_type() == "chapter" && !template_has_dictionary_entry_shape(template) {
            return None;
        }

        if self.reference_has_visible_author(reference) {
            return None;
        }

        if !template_has_primary_title(template) || !template_has_parent_container_title(template) {
            return None;
        }

        if reference_has_online_access(reference) {
            Some(AnonymousEntryBibliographyMode::ContainerLed)
        } else {
            Some(AnonymousEntryBibliographyMode::SuppressPrintLike)
        }
    }

    fn reference_has_visible_author(&self, reference: &Reference) -> bool {
        reference
            .author()
            .is_some_and(|author| !self.resolve_contributor_names(&author).is_empty())
    }

    fn build_template_render_hint(
        &self,
        reference: &Reference,
        context: RenderContext,
        citation_number: usize,
        position: Option<citum_schema::citation::Position>,
        integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    ) -> ProcHints {
        let default_hint = ProcHints::default();
        let base_hint = self
            .hints
            .get(&reference.id().unwrap_or_default())
            .unwrap_or(&default_hint);
        ProcHints {
            citation_number: (citation_number > 0).then_some(citation_number),
            citation_sub_label: if context == RenderContext::Citation {
                reference
                    .id()
                    .as_deref()
                    .and_then(|id| self.citation_sub_label_for_ref(id))
            } else {
                None
            },
            position,
            integral_name_state,
            ..base_hint.clone()
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Template rendering needs the resolved context plus the source template index."
    )]
    fn render_template_component_with_format<F>(
        &self,
        reference: &Reference,
        ref_type: &str,
        options: &RenderOptions<'_>,
        hint: &ProcHints,
        template_index: usize,
        component: &TemplateComponent,
        tracker: &mut TemplateComponentTracker,
    ) -> Option<ProcTemplateComponent>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        if let TemplateComponent::Group(group) = component {
            return self.render_group_component_with_format::<F>(
                reference,
                ref_type,
                options,
                hint,
                template_index,
                group,
                tracker,
            );
        }

        let resolved_component = component;
        let var_key = get_variable_key(resolved_component);
        if tracker.should_skip(var_key.as_deref()) {
            return None;
        }

        let mut values = resolved_component.values::<F>(reference, hint, options)?;
        if values.value.is_empty() {
            return None;
        }
        self.apply_issued_no_date_fallback(reference, options, resolved_component, &mut values);
        self.apply_entry_link_fallback(reference, options, &mut values);

        let item_language =
            crate::values::effective_component_language(reference, resolved_component);
        tracker.mark_rendered(var_key, values.substituted_key.as_deref());

        Some(ProcTemplateComponent {
            template_component: resolved_component.clone(),
            template_index: self.inject_ast_indices.then_some(template_index),
            value: values.value,
            prefix: values.prefix,
            suffix: values.suffix,
            url: values.url,
            ref_type: Some(ref_type.to_string()),
            config: Some(options.config.clone()),
            bibliography_config: options.bibliography_config.clone(),
            item_language,
            pre_formatted: values.pre_formatted,
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Nested group rendering reuses the same tracker and template metadata."
    )]
    fn render_group_component_with_format<F>(
        &self,
        reference: &Reference,
        ref_type: &str,
        options: &RenderOptions<'_>,
        hint: &ProcHints,
        template_index: usize,
        group: &citum_schema::template::TemplateGroup,
        tracker: &mut TemplateComponentTracker,
    ) -> Option<ProcTemplateComponent>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let delimiter = group
            .delimiter
            .as_ref()
            .unwrap_or(&citum_schema::template::DelimiterPunctuation::Comma)
            .to_string_with_space();
        let fmt = F::default();
        let mut has_meaningful_content = false;
        let mut values = Vec::new();

        for item in &group.group {
            let Some(rendered) = self.render_template_component_with_format::<F>(
                reference,
                ref_type,
                options,
                hint,
                template_index,
                item,
                tracker,
            ) else {
                continue;
            };
            let rendered_str = crate::render::render_component_with_format_and_renderer::<F>(
                &rendered,
                &fmt,
                options.show_semantics,
            );
            if rendered_str.is_empty() {
                continue;
            }
            if !is_term_only_component(item) {
                has_meaningful_content = true;
            }
            values.push(rendered_str);
        }

        if values.is_empty() || !has_meaningful_content {
            return None;
        }

        let group_component = TemplateComponent::Group(group.clone());
        Some(ProcTemplateComponent {
            template_component: group_component.clone(),
            template_index: self.inject_ast_indices.then_some(template_index),
            value: fmt.join(values, &delimiter),
            prefix: None,
            suffix: None,
            url: None,
            ref_type: Some(ref_type.to_string()),
            config: Some(options.config.clone()),
            bibliography_config: options.bibliography_config.clone(),
            item_language: crate::values::effective_component_language(reference, &group_component),
            pre_formatted: true,
        })
    }

    fn apply_issued_no_date_fallback(
        &self,
        reference: &Reference,
        options: &RenderOptions<'_>,
        component: &TemplateComponent,
        values: &mut crate::values::ProcValues<String>,
    ) {
        if !matches!(
            component,
            TemplateComponent::Date(citum_schema::template::TemplateDate {
                date: citum_schema::template::DateVariable::Issued,
                ..
            })
        ) || !reference.issued().is_none_or(|issued| issued.0.is_empty())
            || self.preferred_no_date_term_form() != citum_schema::locale::TermForm::Long
        {
            return;
        }

        if let Some(long) = options.locale.resolved_general_term(
            &citum_schema::locale::GeneralTerm::NoDate,
            citum_schema::locale::TermForm::Long,
        ) {
            values.value = long;
        }
    }

    fn apply_entry_link_fallback(
        &self,
        reference: &Reference,
        options: &RenderOptions<'_>,
        values: &mut crate::values::ProcValues<String>,
    ) {
        if values.url.is_some() {
            return;
        }

        let Some(links) = &options.config.links else {
            return;
        };
        use citum_schema::options::LinkAnchor;
        if matches!(links.anchor, Some(LinkAnchor::Entry)) {
            values.url = crate::values::resolve_url(links, reference);
        }
    }

    /// Apply the substitution string to the primary contributor component.
    pub fn apply_author_substitution(&self, proc: &mut ProcTemplate, substitute: &str) {
        self.apply_author_substitution_with_format::<crate::render::plain::PlainText>(
            proc, substitute,
        );
    }

    /// Apply the substitution string to the primary contributor component with specific format.
    pub fn apply_author_substitution_with_format<F>(
        &self,
        proc: &mut ProcTemplate,
        substitute: &str,
    ) where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        if let Some(component) = proc
            .iter_mut()
            .find(|c| matches!(c.template_component, TemplateComponent::Contributor(_)))
        {
            let fmt = F::default();
            component.value = fmt.text(substitute);
        }
    }

    fn preferred_no_date_term_form(&self) -> citum_schema::locale::TermForm {
        match self
            .style
            .info
            .source
            .as_ref()
            .map(|source| source.csl_id.as_str())
        {
            Some("http://www.zotero.org/styles/harvard-cite-them-right") => {
                citum_schema::locale::TermForm::Long
            }
            _ => citum_schema::locale::TermForm::Short,
        }
    }

    fn render_group_item_from_template_with_format<F>(
        &self,
        reference: &Reference,
        item_request: GroupItemRenderRequest<'_>,
    ) -> Option<String>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let request = self.citation_render_request(
            item_request.item,
            item_request.template,
            item_request.mode,
            item_request.suppress_author,
            item_request.position,
        );
        self.render_item_from_template_with_format::<F>(reference, request, item_request.delimiter)
    }
}

fn filter_article_journal_template_components(
    components: &[TemplateComponent],
    mode: ArticleJournalBibliographyMode,
) -> Vec<TemplateComponent> {
    components
        .iter()
        .filter_map(|component| filter_article_journal_template_component(component, mode))
        .collect()
}

fn filter_article_journal_template_component(
    component: &TemplateComponent,
    mode: ArticleJournalBibliographyMode,
) -> Option<TemplateComponent> {
    if should_suppress_article_journal_component(component, mode) {
        return None;
    }

    match component {
        TemplateComponent::Group(list) => {
            let mut filtered = list.clone();
            filtered.group = filter_article_journal_template_components(&list.group, mode);
            (!filtered.group.is_empty()).then_some(TemplateComponent::Group(filtered))
        }
        _ => Some(component.clone()),
    }
}

fn article_journal_template_needs_filter(
    components: &[TemplateComponent],
    mode: ArticleJournalBibliographyMode,
) -> bool {
    components
        .iter()
        .any(|component| article_journal_component_needs_filter(component, mode))
}

fn rewrite_anonymous_entry_template(
    template: &[TemplateComponent],
    mode: AnonymousEntryBibliographyMode,
    ref_type: &str,
    prefer_doi: bool,
) -> Vec<TemplateComponent> {
    match mode {
        AnonymousEntryBibliographyMode::ContainerLed => {
            let mut rewritten = Vec::new();

            if let Some(container_title) =
                find_preferred_parent_container_component(template, ref_type)
            {
                rewritten.push(container_title.clone());
            }
            if let Some(issued) = find_first_component(template, is_issued_date_component) {
                rewritten.push(issued.clone());
            }
            if let Some(primary_title) = find_first_component(template, is_primary_title_component)
            {
                rewritten.push(primary_title.clone());
            }
            if let Some(volume) = find_first_component(template, is_volume_component) {
                rewritten.push(volume.clone());
            }

            if prefer_doi {
                if let Some(doi) = find_first_component(template, is_doi_component) {
                    rewritten.push(doi.clone());
                }
            } else if let Some(url) = find_first_component(template, is_url_component) {
                rewritten.push(url.clone());
            }

            if rewritten.is_empty() {
                template.to_vec()
            } else {
                rewritten
            }
        }
        AnonymousEntryBibliographyMode::SuppressPrintLike => template.to_vec(),
    }
}

fn find_first_component(
    template: &[TemplateComponent],
    predicate: impl Fn(&TemplateComponent) -> bool,
) -> Option<&TemplateComponent> {
    template.iter().find(|component| predicate(component))
}

fn find_preferred_parent_container_component<'a>(
    template: &'a [TemplateComponent],
    ref_type: &str,
) -> Option<&'a TemplateComponent> {
    if ref_type == "chapter"
        && let Some(parent_monograph) =
            find_first_component(template, is_parent_monograph_title_component)
    {
        return Some(parent_monograph);
    }

    find_first_component(template, is_parent_container_title_component)
}

fn article_journal_component_needs_filter(
    component: &TemplateComponent,
    mode: ArticleJournalBibliographyMode,
) -> bool {
    if should_suppress_article_journal_component(component, mode) {
        return true;
    }

    match component {
        TemplateComponent::Group(group) => {
            article_journal_template_needs_filter(&group.group, mode)
        }
        _ => false,
    }
}

fn is_term_only_component(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Term(_) => true,
        TemplateComponent::Group(group) => group.group.iter().all(is_term_only_component),
        _ => false,
    }
}

fn should_suppress_article_journal_component(
    component: &TemplateComponent,
    mode: ArticleJournalBibliographyMode,
) -> bool {
    match mode {
        ArticleJournalBibliographyMode::StandardDetail => is_doi_component(component),
        ArticleJournalBibliographyMode::DoiFallback => is_article_detail_component(component),
    }
}

fn template_has_primary_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(is_primary_title_component)
}

fn template_has_parent_container_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(is_parent_container_title_component)
}

fn template_has_dictionary_entry_shape(template: &[TemplateComponent]) -> bool {
    template.iter().any(|component| {
        matches!(
            component,
            TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Version
        )
    })
}

fn is_primary_title_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if title.title == citum_schema::template::TitleType::Primary
    )
}

fn is_parent_container_title_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if matches!(
                title.title,
                citum_schema::template::TitleType::ParentSerial
                    | citum_schema::template::TitleType::ParentMonograph
            )
    )
}

fn is_parent_monograph_title_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Title(title)
            if title.title == citum_schema::template::TitleType::ParentMonograph
    )
}

fn is_issued_date_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(date) if date.date == DateVariable::Issued
    )
}

fn is_volume_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Number(number) if number.number == NumberVariable::Volume
    )
}

fn is_url_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Url
    )
}

fn is_doi_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Doi
    )
}

fn is_article_detail_component(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(date) if date.date == DateVariable::Issued
    ) || matches!(
        component,
        TemplateComponent::Number(number)
            if matches!(
                number.number,
                NumberVariable::Volume | NumberVariable::Issue | NumberVariable::Pages
            )
    )
}

fn reference_has_pages(reference: &Reference) -> bool {
    match reference.pages() {
        Some(NumOrStr::Str(pages)) => !pages.trim().is_empty(),
        Some(NumOrStr::Number(_)) => true,
        None => false,
    }
}

fn reference_has_doi(reference: &Reference) -> bool {
    reference.doi().is_some_and(|doi| !doi.trim().is_empty())
}

fn reference_has_url(reference: &Reference) -> bool {
    reference.url().is_some()
}

fn reference_has_online_access(reference: &Reference) -> bool {
    reference_has_doi(reference) || reference_has_url(reference)
}

fn aliased_type_selector_candidates(ref_type: &str) -> Vec<&str> {
    match ref_type {
        "chapter" => vec!["chapter", "entry-dictionary"],
        _ => vec![ref_type],
    }
}

pub(super) fn filter_author_from_template(
    template: &[TemplateComponent],
) -> (Vec<TemplateComponent>, Option<String>) {
    let mut filtered: Vec<TemplateComponent> =
        template.iter().filter_map(strip_author_component).collect();
    let leading_affix = filtered.first().and_then(leading_group_affix);
    if let Some(first) = filtered.first_mut() {
        strip_leading_group_affixes(first);
    }
    (filtered, leading_affix)
}
