use super::*;

struct GroupRenderState<'a> {
    first_item: &'a crate::reference::CitationItem,
    first_ref: &'a Reference,
    template: Vec<TemplateComponent>,
}

struct GroupItemRenderRequest<'a> {
    item: &'a crate::reference::CitationItem,
    template: &'a [TemplateComponent],
    mode: &'a citum_schema::citation::CitationMode,
    suppress_author: bool,
    position: Option<&'a citum_schema::citation::Position>,
    delimiter: &'a str,
}

pub(super) fn group_citation_items_by_author<'a>(
    renderer: &Renderer<'_>,
    items: &'a [crate::reference::CitationItem],
) -> Vec<(String, Vec<&'a crate::reference::CitationItem>)> {
    let preserve_individual_citations = items.iter().any(|item| {
        renderer
            .hints
            .get(&item.id)
            .is_some_and(|hints| hints.min_names_to_show.is_some() || hints.expand_given_names)
    });

    let mut groups: Vec<(String, Vec<&'a crate::reference::CitationItem>)> = Vec::new();

    for item in items {
        let reference = renderer.bibliography.get(&item.id);
        let author_key = if preserve_individual_citations {
            item.id.clone()
        } else {
            reference.map(author_grouping_key).unwrap_or_default()
        };

        match groups.last_mut() {
            Some(group) if !author_key.is_empty() && group.0 == author_key => {
                group.1.push(item);
            }
            _ => groups.push((author_key, vec![item])),
        }
    }

    groups
}

impl<'a> Renderer<'a> {
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
                state.first_ref,
                GroupItemRenderRequest {
                    item,
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
        let first_item = group[0];
        let component_delimiter = spec.delimiter.as_deref().unwrap_or(" ");
        let item_join_delim = spec.multi_cite_delimiter.as_deref().unwrap_or("; ");
        let mut group_items_str = Vec::new();
        let mut all_ids = Vec::new();

        for item in group {
            let state = self.resolve_item_render_state(item, spec)?;
            if let Some(item_str) = self.render_group_item_from_template_with_format::<F>(
                state.first_ref,
                GroupItemRenderRequest {
                    item,
                    template: &state.template,
                    mode,
                    suppress_author,
                    position,
                    delimiter: component_delimiter,
                },
            ) {
                group_items_str.push(item_str);
                all_ids.push(item.id.clone());
            }
        }

        if group_items_str.is_empty() {
            return Ok(None);
        }

        let combined_str = group_items_str.join(item_join_delim);
        Ok(self
            .build_citation_chunk(
                &fmt,
                all_ids,
                combined_str,
                first_item.prefix.as_deref(),
                first_item.suffix.as_deref(),
            )
            .map(|(ids, content)| fmt.citation(ids, content)))
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
                spec,
                mode,
                intra_delimiter,
                suppress_author,
                position,
            )?
            .into_iter()
            .collect())
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
        let first_template = spec.resolve_template_for_language(first_language.as_deref());

        Ok(GroupRenderState {
            first_item,
            first_ref,
            template: first_template.unwrap_or_default(),
        })
    }

    fn resolve_item_render_state<'b>(
        &'b self,
        item: &'b crate::reference::CitationItem,
        spec: &'b citum_schema::CitationSpec,
    ) -> Result<GroupRenderState<'b>, ProcessorError> {
        let first_ref = self
            .bibliography
            .get(&item.id)
            .ok_or_else(|| ProcessorError::ReferenceNotFound(item.id.clone()))?;
        let item_language = crate::values::effective_item_language(first_ref);
        let item_template = spec.resolve_template_for_language(item_language.as_deref());

        Ok(GroupRenderState {
            first_item: item,
            first_ref,
            template: item_template.unwrap_or_default(),
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
    pub(super) fn get_or_assign_citation_number(&self, ref_id: &str) -> usize {
        let mut numbers = self.citation_numbers.borrow_mut();
        let next_num = numbers.len() + 1;
        *numbers.entry(ref_id.to_string()).or_insert(next_num)
    }

    /// Process a bibliography entry.
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
    pub fn process_bibliography_entry_with_format<F>(
        &self,
        reference: &Reference,
        entry_number: usize,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let bib_spec = self.style.bibliography.as_ref()?;

        // Resolve default template (handles preset vs explicit)
        let item_language = crate::values::effective_item_language(reference);
        let default_template = bib_spec.resolve_template_for_language(item_language.as_deref())?;

        // Determine effective template (override or default)
        let ref_type = reference.ref_type();
        let template = if let Some(type_templates) = &bib_spec.type_templates {
            let mut matched_template = None;
            for (selector, t) in type_templates {
                if selector.matches(&ref_type) {
                    matched_template = Some(t.clone());
                    break;
                }
            }
            matched_template.unwrap_or(default_template)
        } else {
            default_template
        };

        let template_ref = &template;

        self.process_template_request_with_format::<F>(
            reference,
            TemplateRenderRequest {
                template: template_ref,
                context: RenderContext::Bibliography,
                mode: citum_schema::citation::CitationMode::NonIntegral,
                suppress_author: false,
                locator: None,
                locator_label: None,
                citation_number: entry_number,
                position: None,
                integral_name_state: None,
            },
        )
    }

    /// Process a template for a reference with citation number.
    #[allow(clippy::too_many_arguments)]
    pub fn process_template_with_number(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
        context: RenderContext,
        mode: citum_schema::citation::CitationMode,
        suppress_author: bool,
        citation_number: usize,
        locator: Option<&str>,
        locator_label: Option<citum_schema::citation::LocatorType>,
        position: Option<&citum_schema::citation::Position>,
        integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    ) -> Option<ProcTemplate> {
        self.process_template_with_number_with_format::<crate::render::plain::PlainText>(
            reference,
            template,
            context,
            mode,
            suppress_author,
            citation_number,
            locator,
            locator_label,
            position,
            integral_name_state,
        )
    }

    /// Process a template for a reference with citation number and specific format.
    #[allow(clippy::too_many_arguments)]
    pub fn process_template_with_number_with_format<F>(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
        context: RenderContext,
        mode: citum_schema::citation::CitationMode,
        suppress_author: bool,
        citation_number: usize,
        locator: Option<&str>,
        locator_label: Option<citum_schema::citation::LocatorType>,
        position: Option<&citum_schema::citation::Position>,
        integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    ) -> Option<ProcTemplate>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        self.process_template_request_with_format::<F>(
            reference,
            TemplateRenderRequest {
                template,
                context,
                mode,
                suppress_author,
                locator: locator.map(str::to_string),
                locator_label,
                citation_number,
                position: position.cloned(),
                integral_name_state,
            },
        )
    }

    pub(super) fn process_template_request_with_format<F>(
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
            locator,
            locator_label,
            citation_number,
            position,
            integral_name_state,
        } = request;
        let options = RenderOptions {
            config: self.config,
            locale: self.locale,
            context,
            mode,
            suppress_author,
            locator: locator.as_deref(),
            locator_label,
        };
        let hint = self.build_template_render_hint(
            reference,
            options.context,
            citation_number,
            position,
            integral_name_state,
        );
        let ref_type = reference.ref_type().to_string();
        let mut tracker = TemplateComponentTracker::default();
        let components: Vec<ProcTemplateComponent> = template
            .iter()
            .filter_map(|component| {
                self.render_template_component_with_format::<F>(
                    reference,
                    &ref_type,
                    &options,
                    &hint,
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

    #[allow(clippy::too_many_arguments)]
    fn render_fallback_grouped_citation_with_format<F>(
        &self,
        group: &[&crate::reference::CitationItem],
        first_ref: &Reference,
        first_item: &crate::reference::CitationItem,
        template: &[TemplateComponent],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
    ) -> Result<Option<String>, ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let fmt = F::default();
        let author_part = self.render_author_for_grouping_with_format::<F>(
            first_ref,
            first_item,
            template,
            mode,
            suppress_author,
            position,
        );
        let (item_parts, group_delimiter) = self.render_group_item_parts_with_format::<F>(
            &fmt,
            group,
            spec,
            mode,
            suppress_author,
            position,
            intra_delimiter,
        )?;
        let Some(content) = self.build_grouped_citation_content(
            &author_part,
            &item_parts,
            mode,
            intra_delimiter,
            group_delimiter.as_deref(),
            suppress_author,
        ) else {
            return Ok(None);
        };
        let group_ids = group.iter().map(|item| item.id.clone()).collect();
        let prefix = first_item.prefix.as_deref().unwrap_or("");

        Ok(Some(fmt.citation(
            group_ids,
            self.affix_content(&fmt, content, Some(prefix), None),
        )))
    }

    fn build_grouped_citation_content(
        &self,
        author_part: &str,
        item_parts: &[String],
        mode: &citum_schema::citation::CitationMode,
        intra_delimiter: &str,
        group_delimiter: Option<&str>,
        suppress_author: bool,
    ) -> Option<String> {
        if !author_part.is_empty() && !item_parts.is_empty() {
            let author_item_delimiter = group_delimiter.unwrap_or(intra_delimiter);
            let repeated_item_delimiter = if author_item_delimiter.trim().is_empty() {
                ", "
            } else {
                author_item_delimiter
            };
            let joined_items = item_parts.join(repeated_item_delimiter);
            return Some(match mode {
                citum_schema::citation::CitationMode::Integral => {
                    self.format_integral_grouped_items(author_part, &joined_items, suppress_author)
                }
                citum_schema::citation::CitationMode::NonIntegral => self
                    .format_non_integral_grouped_items(
                        author_part,
                        author_item_delimiter,
                        &joined_items,
                        suppress_author,
                    ),
            });
        }

        if !author_part.is_empty() {
            return Some(author_part.to_string());
        }

        if !item_parts.is_empty() {
            return Some(item_parts.join(intra_delimiter));
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

    fn render_template_component_with_format<F>(
        &self,
        reference: &Reference,
        ref_type: &str,
        options: &RenderOptions<'_>,
        hint: &ProcHints,
        component: &TemplateComponent,
        tracker: &mut TemplateComponentTracker,
    ) -> Option<ProcTemplateComponent>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let resolved_component = resolve_component_for_ref_type(component, ref_type);
        let var_key = get_variable_key(&resolved_component);
        if tracker.should_skip(var_key.as_deref()) {
            return None;
        }

        let mut values = resolved_component.values::<F>(reference, hint, options)?;
        if values.value.is_empty() {
            return None;
        }
        self.apply_issued_no_date_fallback(reference, options, &resolved_component, &mut values);
        self.apply_entry_link_fallback(reference, options, &mut values);

        let item_language =
            crate::values::effective_component_language(reference, &resolved_component);
        tracker.mark_rendered(var_key, values.substituted_key.as_deref());

        Some(ProcTemplateComponent {
            template_component: resolved_component,
            value: values.value,
            prefix: values.prefix,
            suffix: values.suffix,
            url: values.url,
            ref_type: Some(ref_type.to_string()),
            config: Some(options.config.clone()),
            item_language,
            pre_formatted: values.pre_formatted,
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

        if let Some(long) = options.locale.general_term(
            &citum_schema::locale::GeneralTerm::NoDate,
            citum_schema::locale::TermForm::Long,
        ) {
            values.value = long.to_string();
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

    #[allow(clippy::too_many_arguments)]
    fn render_group_item_parts_with_format<F>(
        &self,
        fmt: &F,
        group: &[&crate::reference::CitationItem],
        spec: &citum_schema::CitationSpec,
        mode: &citum_schema::citation::CitationMode,
        suppress_author: bool,
        position: Option<&citum_schema::citation::Position>,
        intra_delimiter: &str,
    ) -> Result<(Vec<String>, Option<String>), ProcessorError>
    where
        F: crate::render::format::OutputFormat<Output = String>,
    {
        let mut item_parts = Vec::new();
        let mut group_delimiter: Option<String> = None;
        for item in group {
            let state = self.resolve_item_render_state(item, spec)?;
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
                intra_delimiter
            };
            if let Some(item_str) = self.render_group_item_from_template_with_format::<F>(
                state.first_ref,
                GroupItemRenderRequest {
                    item,
                    template: &filtered_template,
                    mode,
                    suppress_author,
                    position,
                    delimiter: item_delimiter,
                },
            ) && !item_str.is_empty()
            {
                let suffix = item.suffix.as_deref().unwrap_or("");
                if !suffix.is_empty() {
                    let spaced_suffix = Self::ensure_suffix_spacing(suffix);
                    item_parts.push(fmt.affix("", item_str, &spaced_suffix));
                } else {
                    item_parts.push(item_str);
                }
            }
        }
        Ok((item_parts, group_delimiter))
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

fn author_grouping_key(reference: &Reference) -> String {
    reference
        .author()
        .map_or_else(
            || {
                reference.editor().map_or_else(
                    || {
                        reference
                            .title()
                            .map_or_else(String::new, |title| title.to_string())
                    },
                    |editor| editor.to_string(),
                )
            },
            |author| author.to_string(),
        )
        .to_lowercase()
}

fn filter_author_from_template(
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
