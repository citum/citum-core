/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Final standalone style assembly for `citum-migrate`.

use crate::{
    bib_postprocess::{
        is_inferred_bib_source, merge_inferred_type_templates, postprocess_bibliography_templates,
    },
    citation_validate,
    template_diff::{TypeTemplateMap, TypeVariantMap, build_type_variants},
};
use citum_migrate::{
    analysis,
    compilation::XmlCompilationOutput as XmlFallback,
    fixups::{
        ensure_numeric_locator_citation_component, ensure_personal_communication_omitted,
        gate_web_only_url_accessed, move_group_wrap_to_citation_items,
        normalize_author_date_locator_citation_component,
        normalize_wrapped_numeric_locator_citation_component,
    },
    passes::suppression::{normalize_visible_suppress, strip_inert_suppressed_placeholders},
    template_resolver,
};
use citum_schema::{
    BibliographySpec, CitationCollapse, CitationSpec, Style, StyleInfo,
    options::{ContributorConfig, DisplayAsSort},
    template::{
        NameOrder, Rendering, TemplateComponent, TemplateContributor, TemplateTitle, TitleType,
        TypeSelector, WrapPunctuation,
    },
};
use std::path::Path;

/// Borrowed pipeline state needed to assemble a standalone style variant.
pub(crate) struct StandaloneAssembly<'a> {
    /// Parsed legacy CSL style.
    pub(crate) legacy_style: &'a csl_legacy::model::Style,
    /// Resolved hand-authored, inferred, or XML-fallback template sources.
    pub(crate) resolved: &'a template_resolver::ResolvedTemplates,
    /// XML-compiled fallback candidate used when resolved templates are absent.
    pub(crate) xml_fallback: &'a Option<XmlFallback>,
    /// Extracted global migration options.
    pub(crate) options: &'a citum_schema::options::Config,
    /// Extracted bibliography-specific options.
    pub(crate) bibliography_options: &'a Option<citum_schema::BibliographyOptions>,
    /// Citation contributor overrides extracted from legacy CSL.
    pub(crate) citation_contributor_overrides: &'a Option<citum_schema::options::ContributorConfig>,
    /// Bibliography contributor overrides extracted from legacy CSL.
    pub(crate) bibliography_contributor_overrides:
        &'a Option<citum_schema::options::ContributorConfig>,
}

/// Template-source suppression switches used for measured candidate selection.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TemplateSourceSelection {
    /// Force bibliography assembly through XML fallback.
    pub(crate) suppress_inferred_bibliography: bool,
    /// Force citation assembly through XML fallback.
    pub(crate) suppress_inferred_citation: bool,
}

/// All compiled template and option data needed to build the final Style.
struct CompiledOutput {
    options: citum_schema::options::Config,
    citation_contributor_overrides: Option<citum_schema::options::ContributorConfig>,
    bibliography_options: Option<citum_schema::BibliographyOptions>,
    bibliography_contributor_overrides: Option<citum_schema::options::ContributorConfig>,
    new_cit: Vec<TemplateComponent>,
    new_bib: Vec<TemplateComponent>,
    type_templates: Option<TypeTemplateMap>,
    citation_wrap: Option<WrapPunctuation>,
    citation_prefix: Option<String>,
    citation_suffix: Option<String>,
    citation_delimiter: Option<String>,
    citation_subsequent_override: Option<Vec<TemplateComponent>>,
    citation_ibid_override: Option<Vec<TemplateComponent>>,
}

impl StandaloneAssembly<'_> {
    /// Assemble a standalone style using the requested source suppression.
    pub(crate) fn assemble_with_selection(&self, selection: TemplateSourceSelection) -> Style {
        let masked;
        let resolved =
            if selection.suppress_inferred_bibliography || selection.suppress_inferred_citation {
                masked = template_resolver::ResolvedTemplates {
                    bibliography: if selection.suppress_inferred_bibliography {
                        None
                    } else {
                        self.resolved.bibliography.clone()
                    },
                    citation: if selection.suppress_inferred_citation {
                        None
                    } else {
                        self.resolved.citation.clone()
                    },
                };
                &masked
            } else {
                self.resolved
            };

        let mut bibliography_options = self.bibliography_options.clone();
        let (new_bib, mut type_templates, inferred_bib_source) =
            select_and_process_bibliography_template(
                resolved,
                self.xml_fallback,
                self.legacy_style,
            );

        let (mut new_cit, citation_subsequent_override, citation_ibid_override) =
            select_citation_template(
                resolved,
                self.xml_fallback,
                inferred_bib_source,
                self.legacy_style,
                &mut type_templates,
            );

        override_bibliography_options_if_inferred(
            resolved,
            self.legacy_style,
            &mut bibliography_options,
        );

        let (citation_wrap, citation_prefix, citation_suffix, citation_delimiter) =
            resolve_citation_metadata(resolved, self.legacy_style, self.options, &mut new_cit);

        build_final_style(
            self.legacy_style,
            CompiledOutput {
                options: self.options.clone(),
                citation_contributor_overrides: self.citation_contributor_overrides.clone(),
                bibliography_options,
                bibliography_contributor_overrides: self.bibliography_contributor_overrides.clone(),
                new_cit,
                new_bib,
                type_templates,
                citation_wrap,
                citation_prefix,
                citation_suffix,
                citation_delimiter,
                citation_subsequent_override,
                citation_ibid_override,
            },
        )
    }
}

/// Replace the current citation template when a measured candidate renders
/// closer to citeproc-js.
///
/// The inferrer's confidence score is self-referential and can rate templates
/// highly that render badly; this settles the choice empirically. Any scoring
/// failure keeps the inferred status quo.
pub(crate) fn apply_measured_citation_selection(
    current: Style,
    assembly: &StandaloneAssembly<'_>,
    source_selection: TemplateSourceSelection,
    style_name: &str,
    style_xml: &str,
    workspace_root: &Path,
) -> (Style, bool) {
    let Some(out) = assembly.xml_fallback.as_ref() else {
        return (current, false);
    };
    if out.citation.is_empty() {
        return (current, false);
    }

    let alternative = assembly.assemble_with_selection(TemplateSourceSelection {
        suppress_inferred_citation: true,
        ..source_selection
    });
    match citum_migrate::synthesis::synthesize_citation(
        &current,
        &alternative,
        style_name,
        style_xml,
        workspace_root,
    ) {
        Ok(selection) => apply_citation_selection_result(current, style_name, selection),
        Err(err) => measured_selection_unavailable(current, style_name, "citation", err),
    }
}

/// Replace the current bibliography template when a measured candidate renders
/// closer to citeproc-js.
pub(crate) fn apply_measured_bibliography_selection(
    current: Style,
    assembly: &StandaloneAssembly<'_>,
    source_selection: TemplateSourceSelection,
    style_name: &str,
    style_xml: &str,
    workspace_root: &Path,
) -> (Style, bool) {
    let Some(out) = assembly.xml_fallback.as_ref() else {
        return (current, false);
    };
    if out.bibliography.is_empty() {
        return (current, false);
    }

    let bibliography_source = assembly.assemble_with_selection(TemplateSourceSelection {
        suppress_inferred_bibliography: true,
        ..source_selection
    });
    let alternative = style_with_bibliography_from(current.clone(), bibliography_source);
    match citum_migrate::synthesis::synthesize_bibliography(
        &current,
        &alternative,
        style_name,
        style_xml,
        workspace_root,
    ) {
        Ok(selection) => apply_bibliography_selection_result(current, style_name, selection),
        Err(err) => measured_selection_unavailable(current, style_name, "bibliography", err),
    }
}

/// Resolve migrated bibliography sort only when it differs from the processing default.
pub(crate) fn resolve_migrated_bibliography_sort(
    processing: Option<&citum_schema::options::Processing>,
    legacy_sort: Option<&csl_legacy::model::Sort>,
) -> Option<citum_schema::grouping::GroupSortEntry> {
    let extracted_entry = legacy_sort.and_then(
        citum_migrate::options_extractor::bibliography::extract_group_sort_from_bibliography,
    )?;
    let extracted = extracted_entry.resolve();

    if bibliography_sort_matches_processing_default(processing, &extracted) {
        None
    } else {
        Some(extracted_entry)
    }
}

/// Assembles the final Citum Style from compiled output and legacy metadata.
fn build_final_style(legacy_style: &csl_legacy::model::Style, mut c: CompiledOutput) -> Style {
    let citation_scope_options =
        c.citation_contributor_overrides
            .map(|contributors| citum_schema::CitationOptions {
                contributors: Some(contributors),
                ..Default::default()
            });
    let mut bibliography_scope_options = c.bibliography_options.take().unwrap_or_default();
    if let Some(contributors) = c.bibliography_contributor_overrides.take() {
        bibliography_scope_options.contributors = Some(contributors);
    }
    let effective_citation_options = citation_scope_options.as_ref().map_or_else(
        || c.options.clone(),
        |options| options.merged_with(&c.options),
    );
    normalize_templates_against_options(
        &mut c.new_cit,
        TitleDefaultContext::Default,
        &effective_citation_options,
    );
    if let Some(template) = c.citation_subsequent_override.as_mut() {
        normalize_templates_against_options(
            template,
            TitleDefaultContext::Default,
            &effective_citation_options,
        );
    }
    if let Some(template) = c.citation_ibid_override.as_mut() {
        normalize_templates_against_options(
            template,
            TitleDefaultContext::Default,
            &effective_citation_options,
        );
    }

    let effective_bibliography_options = bibliography_scope_options.merged_with(&c.options);
    let bibliography_scope_options = (bibliography_scope_options
        != citum_schema::BibliographyOptions::default())
    .then_some(bibliography_scope_options);
    let bibliography_sort = resolve_migrated_bibliography_sort(
        c.options.processing.as_ref(),
        legacy_style
            .bibliography
            .as_ref()
            .and_then(|bib| bib.sort.as_ref()),
    );
    let (new_bib, type_variants) = finalize_bibliography_variants(
        c.new_bib,
        c.type_templates,
        &effective_bibliography_options,
    );

    // [PRUNING] Prune redundant citation modes (e.g. ibid/subsequent if they match base).
    let subsequent = c
        .citation_subsequent_override
        .filter(|t| t != &c.new_cit)
        .map(|t| {
            Box::new(CitationSpec {
                template: Some(t),
                ..Default::default()
            })
        });

    let ibid = c
        .citation_ibid_override
        .filter(|t| t != &c.new_cit)
        .map(|t| {
            Box::new(CitationSpec {
                template: Some(t),
                ..Default::default()
            })
        });

    Style {
        info: StyleInfo {
            title: Some(legacy_style.info.title.clone()),
            id: Some(legacy_style.info.id.clone()),
            default_locale: legacy_style.default_locale.clone(),
            ..Default::default()
        },
        templates: None,
        options: Some(c.options),
        citation: Some(CitationSpec {
            options: citation_scope_options,
            template_ref: None,
            template: Some(c.new_cit),
            collapse: extract_citation_collapse(&legacy_style.citation),
            wrap: c.citation_wrap.map(Into::into),
            prefix: c.citation_prefix,
            suffix: c.citation_suffix,
            delimiter: c.citation_delimiter,
            multi_cite_delimiter: legacy_style.citation.layout.delimiter.clone(),
            subsequent,
            ibid,
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            options: bibliography_scope_options,
            template_ref: None,
            template: Some(new_bib),
            type_variants,
            sort: bibliography_sort,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn select_and_process_bibliography_template(
    resolved: &template_resolver::ResolvedTemplates,
    xml_fallback: &Option<XmlFallback>,
    legacy_style: &csl_legacy::model::Style,
) -> (Vec<TemplateComponent>, Option<TypeTemplateMap>, bool) {
    let (mut new_bib, mut type_templates, inferred_bib_source) =
        if let Some(ref resolved_bib) = resolved.bibliography {
            let inferred_bib = is_inferred_bib_source(&resolved_bib.source);
            let merged_type_templates = if inferred_bib {
                xml_fallback
                    .as_ref()
                    .and_then(|out| merge_inferred_type_templates(out, &resolved_bib.template))
            } else {
                None
            };
            (
                resolved_bib.template.clone(),
                merged_type_templates,
                inferred_bib,
            )
        } else {
            #[allow(clippy::expect_used, reason = "fatal bootstrap error")]
            let out = xml_fallback
                .as_ref()
                .expect("XML fallback must exist when bibliography is unresolved");
            (out.bibliography.clone(), out.type_templates.clone(), false)
        };

    // Phase 1: semantic fixups operate only on full concrete templates.
    postprocess_bibliography_templates(&mut new_bib, &mut type_templates, legacy_style);

    (new_bib, type_templates, inferred_bib_source)
}

fn select_citation_template(
    resolved: &template_resolver::ResolvedTemplates,
    xml_fallback: &Option<XmlFallback>,
    inferred_bib_source: bool,
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<TypeTemplateMap>,
) -> (
    Vec<TemplateComponent>,
    Option<Vec<TemplateComponent>>,
    Option<Vec<TemplateComponent>>,
) {
    let mut citation_subsequent_override: Option<Vec<TemplateComponent>> = None;
    let mut citation_ibid_override: Option<Vec<TemplateComponent>> = None;
    let new_cit = if let Some(ref resolved_cit) = resolved.citation {
        // Inferred templates only capture the first-position form. For note
        // styles, the subsequent/ibid short forms exist solely in the XML
        // pipeline's position extraction — attach them so the inferred first
        // form keeps its repeat behavior. Hand-authored templates are left
        // alone: their repeat shape is a deliberate authoring decision.
        if legacy_style.class == "note"
            && citation_validate::is_inferred_source(&resolved_cit.source)
            && let Some(out) = xml_fallback.as_ref()
        {
            citation_subsequent_override = out.citation_overrides.subsequent.clone();
            citation_ibid_override = out.citation_overrides.ibid.clone();
        }
        resolved_cit.template.clone()
    } else {
        #[allow(clippy::expect_used, reason = "fatal bootstrap error")]
        let out = xml_fallback
            .as_ref()
            .expect("XML fallback must exist when citation is unresolved");
        citation_subsequent_override = out.citation_overrides.subsequent.clone();
        citation_ibid_override = out.citation_overrides.ibid.clone();
        out.citation.clone()
    };

    if inferred_bib_source {
        ensure_personal_communication_omitted(legacy_style, &new_cit, type_templates);
    }

    (
        new_cit,
        citation_subsequent_override,
        citation_ibid_override,
    )
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
fn override_bibliography_options_if_inferred(
    resolved: &template_resolver::ResolvedTemplates,
    legacy_style: &csl_legacy::model::Style,
    options: &mut Option<citum_schema::BibliographyOptions>,
) {
    if let Some(ref resolved_bib) = resolved.bibliography {
        let allow_bib_punctuation_override =
            !(legacy_style.class == "note" && is_inferred_bib_source(&resolved_bib.source));

        if allow_bib_punctuation_override {
            if let Some(ref delim) = resolved_bib.delimiter {
                tracing::debug!("  Overriding bibliography separator: {delim:?}");
                let bib_cfg = options.get_or_insert_with(Default::default);
                bib_cfg.separator = Some(delim.clone());
            }

            if let Some(ref suffix) = resolved_bib.entry_suffix {
                tracing::debug!("  Overriding bibliography entry suffix: {suffix:?}");
                let bib_cfg = options.get_or_insert_with(Default::default);
                bib_cfg.entry_suffix = Some(suffix.clone());
            }
        } else {
            tracing::debug!(
                "  Skipping inferred bibliography separator/entry-suffix override for note style."
            );
        }
    }
}

fn resolve_citation_metadata(
    resolved: &template_resolver::ResolvedTemplates,
    legacy_style: &csl_legacy::model::Style,
    options: &citum_schema::options::Config,
    new_cit: &mut Vec<TemplateComponent>,
) -> (
    Option<citum_schema::template::WrapPunctuation>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let (mut citation_wrap, mut citation_prefix, mut citation_suffix) =
        analysis::citation::infer_citation_wrapping(&legacy_style.citation.layout);
    let mut citation_delimiter = analysis::citation::extract_citation_delimiter(
        &legacy_style.citation.layout,
        &legacy_style.macros,
    );

    if let Some(ref resolved_cit) = resolved.citation {
        if let Some(ref wrap) = resolved_cit.wrap {
            citation_wrap = Some(wrap.clone());
            citation_prefix = None;
            citation_suffix = None;
        }
        if let Some(ref delim) = resolved_cit.delimiter {
            citation_delimiter = Some(delim.clone());
        }
    }

    if matches!(
        options.processing,
        Some(citum_schema::options::Processing::Numeric)
    ) {
        ensure_numeric_locator_citation_component(&legacy_style.citation.layout, new_cit);
        normalize_wrapped_numeric_locator_citation_component(
            &legacy_style.citation.layout,
            new_cit,
            &mut citation_delimiter,
        );
        move_group_wrap_to_citation_items(
            &legacy_style.citation.layout,
            new_cit,
            &mut citation_wrap,
        );
    } else if legacy_style.class == "in-text" {
        normalize_author_date_locator_citation_component(
            &legacy_style.citation.layout,
            &legacy_style.macros,
            new_cit,
        );
    }

    (
        citation_wrap,
        citation_prefix,
        citation_suffix,
        citation_delimiter,
    )
}

fn finalize_bibliography_variants(
    mut new_bib: Vec<TemplateComponent>,
    mut type_templates: Option<TypeTemplateMap>,
    effective_options: &citum_schema::options::Config,
) -> (Vec<TemplateComponent>, Option<TypeVariantMap>) {
    // Phase 1: final semantic fixups still see full templates. This render-time
    // citeproc-js gate is applied here after measured selection and before any
    // diff encoding, so the cleaned base and type templates stay coherent.
    {
        let mut empty = TypeTemplateMap::new();
        let templates = type_templates.as_mut().unwrap_or(&mut empty);
        gate_web_only_url_accessed(&mut new_bib, templates);
    }
    strip_inert_suppressed_placeholders(&mut new_bib);
    if let Some(type_templates) = type_templates.as_mut() {
        for template in type_templates.values_mut() {
            strip_inert_suppressed_placeholders(template);
        }
    }

    // Phase 2: compression is a serialization pass over finalized Full templates.
    if let Some(type_templates) = type_templates.as_mut() {
        type_templates.retain(|_, template| template != &new_bib);
    }
    normalize_templates_against_options(
        &mut new_bib,
        TitleDefaultContext::Default,
        effective_options,
    );
    if let Some(type_templates) = type_templates.as_mut() {
        for (selector, template) in type_templates {
            let context = selector_title_context(selector);
            normalize_templates_against_options(template, context, effective_options);
        }
    }
    let mut type_variants = type_templates
        .take()
        .map(|type_templates| build_type_variants(&new_bib, type_templates));

    // Normalize suppress:Some(false) → None on all serialized full template
    // lists. This clears the occurrence-compiler's "visible by default" marker,
    // which is semantically identical to None but serializes as noise. Must run
    // after build_type_variants so diff `modify` operations (which legitimately
    // reference suppress:false to un-suppress a parent-suppressed component)
    // are already encoded and are not affected.
    normalize_visible_suppress(&mut new_bib);
    if let Some(variants) = type_variants.as_mut() {
        for variant in variants.values_mut() {
            if let citum_schema::TemplateVariant::Full(components) = variant {
                normalize_visible_suppress(components);
            }
        }
    }

    (new_bib, type_variants)
}

#[derive(Clone, Copy)]
enum TitleDefaultContext<'a> {
    Default,
    RefType(&'a str),
    Unknown,
}

fn selector_title_context(selector: &TypeSelector) -> TitleDefaultContext<'_> {
    match selector {
        TypeSelector::Single(ref_type) => TitleDefaultContext::RefType(ref_type),
        TypeSelector::Multiple(_) => TitleDefaultContext::Unknown,
    }
}

fn normalize_templates_against_options(
    template: &mut [TemplateComponent],
    title_context: TitleDefaultContext<'_>,
    options: &citum_schema::options::Config,
) {
    for component in template {
        match component {
            TemplateComponent::Contributor(contributor) => {
                if let Some(defaults) = options.contributors.as_ref() {
                    compact_inherited_contributor(contributor, defaults);
                }
            }
            TemplateComponent::Title(title) => {
                compact_inherited_title(title, title_context, options);
            }
            TemplateComponent::Group(group) => {
                normalize_templates_against_options(&mut group.group, title_context, options);
            }
            _ => {}
        }
    }
}

fn compact_inherited_title(
    title: &mut TemplateTitle,
    title_context: TitleDefaultContext<'_>,
    options: &citum_schema::options::Config,
) {
    let Some(defaults) = effective_title_rendering(&title.title, title_context, options) else {
        return;
    };

    clear_rendering_field(&mut title.rendering.text_case, defaults.text_case);
    clear_rendering_field(&mut title.rendering.emph, defaults.emph);
    clear_rendering_field(&mut title.rendering.quote, defaults.quote);
    clear_rendering_field(&mut title.rendering.strong, defaults.strong);
    clear_rendering_field(&mut title.rendering.small_caps, defaults.small_caps);
    clear_rendering_field(&mut title.rendering.prefix, defaults.prefix);
    clear_rendering_field(&mut title.rendering.suffix, defaults.suffix);
}

fn clear_rendering_field<T: PartialEq>(field: &mut Option<T>, inherited: Option<T>) {
    if field.as_ref() == inherited.as_ref() {
        *field = None;
    }
}

fn effective_title_rendering(
    title_type: &TitleType,
    title_context: TitleDefaultContext<'_>,
    options: &citum_schema::options::Config,
) -> Option<Rendering> {
    let ref_type = match title_context {
        TitleDefaultContext::Default => None,
        TitleDefaultContext::RefType(ref_type) => Some(ref_type),
        TitleDefaultContext::Unknown => return None,
    };
    let titles = options.titles.as_ref()?;
    let mapped_category = ref_type.and_then(|rt| titles.type_mapping.get(rt));

    let rendering = match title_type {
        TitleType::ContainerTitle => {
            if let Some(category) = mapped_category {
                match category.as_str() {
                    "periodical" => titles.periodical.as_ref(),
                    "serial" => titles.serial.as_ref(),
                    "monograph" | "collection" => titles
                        .container_monograph
                        .as_ref()
                        .or(titles.monograph.as_ref()),
                    _ => titles.default.as_ref(),
                }
            } else if let Some(ref_type) = ref_type {
                if matches!(
                    ref_type,
                    "article-journal" | "article-magazine" | "article-newspaper" | "broadcast"
                ) {
                    titles.periodical.as_ref()
                } else if matches!(ref_type, "chapter" | "paper-conference") {
                    titles
                        .container_monograph
                        .as_ref()
                        .or(titles.monograph.as_ref())
                } else {
                    titles.default.as_ref()
                }
            } else {
                titles.default.as_ref()
            }
        }
        TitleType::ParentSerial => {
            if let Some(category) = mapped_category {
                match category.as_str() {
                    "periodical" => titles.periodical.as_ref(),
                    "serial" => titles.serial.as_ref(),
                    _ => titles.periodical.as_ref(),
                }
            } else if let Some(ref_type) = ref_type {
                if matches!(
                    ref_type,
                    "article-journal" | "article-magazine" | "article-newspaper"
                ) {
                    titles.periodical.as_ref()
                } else {
                    titles.serial.as_ref()
                }
            } else {
                titles.periodical.as_ref()
            }
        }
        TitleType::ParentMonograph => titles
            .container_monograph
            .as_ref()
            .or(titles.monograph.as_ref()),
        TitleType::CollectionTitle => titles
            .container_monograph
            .as_ref()
            .or(titles.monograph.as_ref())
            .or(titles.default.as_ref()),
        TitleType::Primary => {
            if let Some(category) = mapped_category {
                match category.as_str() {
                    "component" => titles.component.as_ref(),
                    "monograph" => titles.monograph.as_ref(),
                    _ => titles.default.as_ref(),
                }
            } else if let Some(ref_type) = ref_type {
                if matches!(
                    ref_type,
                    "article-journal"
                        | "article-magazine"
                        | "article-newspaper"
                        | "chapter"
                        | "entry"
                        | "entry-dictionary"
                        | "entry-encyclopedia"
                        | "paper-conference"
                        | "post"
                        | "post-weblog"
                ) {
                    titles.component.as_ref()
                } else if matches!(ref_type, "book" | "thesis" | "report") {
                    titles.monograph.as_ref()
                } else {
                    titles.default.as_ref()
                }
            } else {
                titles.default.as_ref()
            }
        }
        _ => None,
    };

    Some(rendering.or(titles.default.as_ref())?.to_rendering())
}

fn compact_inherited_contributor(
    contributor: &mut TemplateContributor,
    defaults: &ContributorConfig,
) {
    if contributor.and.as_ref() == defaults.and.as_ref() {
        contributor.and = None;
    }
    if contributor.shorten.as_ref() == defaults.shorten.as_ref() {
        contributor.shorten = None;
    }
    if contributor.sort_separator.as_ref() == defaults.sort_separator.as_ref() {
        contributor.sort_separator = None;
    }
    if contributor.name_form.as_ref() == defaults.name_form.as_ref() {
        contributor.name_form = None;
    }
    if contributor
        .name_order
        .as_ref()
        .is_some_and(|name_order| inherited_name_order_matches(contributor, defaults, name_order))
    {
        contributor.name_order = None;
    }
}

fn inherited_name_order_matches(
    contributor: &TemplateContributor,
    defaults: &ContributorConfig,
    name_order: &NameOrder,
) -> bool {
    if defaults
        .effective_role_name_order(&contributor.contributor)
        .is_some_and(|default| default == name_order)
    {
        return true;
    }

    matches!(
        (defaults.display_as_sort, name_order),
        (
            Some(DisplayAsSort::All | DisplayAsSort::First),
            NameOrder::FamilyFirst
        ) | (Some(DisplayAsSort::First), NameOrder::FamilyFirstOnly)
    )
}

fn apply_citation_selection_result(
    current: Style,
    style_name: &str,
    selection: citum_migrate::measured_citation::MeasuredCitationSelection,
) -> (Style, bool) {
    if selection.use_xml {
        tracing::debug!(
            "Measured citation selection for {style_name}: XML candidate wins ({} vs {} passes over {} items); replacing inferred citation template.",
            selection.xml_passes,
            selection.inferred_passes,
            selection.items
        );
        return (selection.selected_style, true);
    }
    if selection.selected_candidate != "inferred" {
        tracing::debug!(
            "Measured citation selection for {style_name}: {} candidate wins ({} vs {} current passes over {} items; {} synthesis rounds: {:?}).",
            selection.selected_candidate,
            selection.selected_passes,
            selection.inferred_passes,
            selection.items,
            selection.synthesis_rounds,
            selection.accepted_mutations
        );
        return (selection.selected_style, false);
    }
    tracing::debug!(
        "Measured citation selection for {style_name}: keeping inferred citation template ({} vs {} passes over {} items).",
        selection.inferred_passes,
        selection.xml_passes,
        selection.items
    );
    (current, false)
}

fn apply_bibliography_selection_result(
    current: Style,
    style_name: &str,
    selection: citum_migrate::measured_citation::MeasuredBibliographySelection,
) -> (Style, bool) {
    if selection.use_xml {
        tracing::debug!(
            "Measured bibliography selection for {style_name}: XML candidate wins ({} vs {} current passes over {} items); replacing current bibliography template.",
            selection.xml_passes,
            selection.inferred_passes,
            selection.items
        );
        return (selection.selected_style, true);
    }
    if selection.selected_candidate != "inferred" {
        tracing::debug!(
            "Measured bibliography selection for {style_name}: {} candidate wins (family={}, section={}, types={:?}; {} vs {} current passes over {} items; {} synthesis rounds: {:?}).",
            selection.selected_candidate,
            selection.selected_family.as_deref().unwrap_or("unknown"),
            selection.selected_section.as_deref().unwrap_or("unknown"),
            selection.selected_affected_types,
            selection.selected_passes,
            selection.inferred_passes,
            selection.items,
            selection.synthesis_rounds,
            selection.accepted_mutations
        );
        return (selection.selected_style, false);
    }
    tracing::debug!(
        "Measured bibliography selection for {style_name}: keeping inferred bibliography template ({} vs {} passes over {} items).",
        selection.inferred_passes,
        selection.xml_passes,
        selection.items
    );
    (current, false)
}

fn style_with_bibliography_from(mut current: Style, bibliography_source: Style) -> Style {
    current.bibliography = bibliography_source.bibliography;
    current
}

fn measured_selection_unavailable(
    current: Style,
    style_name: &str,
    section: &str,
    err: String,
) -> (Style, bool) {
    tracing::debug!("Measured {section} selection unavailable for {style_name}: {err}");
    (current, false)
}

fn extract_citation_collapse(citation: &csl_legacy::model::Citation) -> Option<CitationCollapse> {
    match citation.collapse.as_deref() {
        Some("citation-number") => Some(CitationCollapse::CitationNumber),
        _ => None,
    }
}

fn bibliography_sort_matches_processing_default(
    processing: Option<&citum_schema::options::Processing>,
    sort: &citum_schema::grouping::GroupSort,
) -> bool {
    processing
        .and_then(citum_schema::options::Processing::default_bibliography_sort)
        .is_some_and(|preset| preset.group_sort() == *sort)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;
    use citum_schema::{
        BibliographySpec, CitationSpec,
        options::{
            AndOptions, DateConfig, DisplayAsSort, MonthFormat, NameForm, ShortenListOptions,
            TextCase, TitleRendering, TitlesConfig,
        },
        template::{
            ContributorForm, ContributorRole, DateForm, DateVariable, NameOrder, Rendering,
            SimpleVariable, TemplateContributor, TemplateDate, TemplateGroup, TemplateTitle,
            TemplateVariable, TitleType,
        },
    };
    use csl_legacy::model::{
        Citation, Info, Layout, Sort as LegacySort, SortKey as LegacySortKey, Style as LegacyStyle,
    };

    #[test]
    fn bibliography_candidate_preserves_current_citation_section() {
        let current = Style {
            citation: Some(CitationSpec {
                delimiter: Some(String::new()),
                ..CitationSpec::default()
            }),
            bibliography: Some(BibliographySpec {
                template: Some(vec![TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::Url,
                    ..TemplateVariable::default()
                })]),
                ..BibliographySpec::default()
            }),
            ..Style::default()
        };
        let bibliography_source = Style {
            citation: Some(CitationSpec {
                delimiter: Some(", ".to_string()),
                ..CitationSpec::default()
            }),
            bibliography: Some(BibliographySpec {
                template: Some(vec![TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::Doi,
                    ..TemplateVariable::default()
                })]),
                ..BibliographySpec::default()
            }),
            ..Style::default()
        };

        let alternative =
            style_with_bibliography_from(current.clone(), bibliography_source.clone());

        assert_eq!(
            serde_json::to_value(&alternative.citation).expect("citation should serialize"),
            serde_json::to_value(&current.citation).expect("citation should serialize")
        );
        assert_eq!(
            serde_json::to_value(&alternative.bibliography).expect("bibliography should serialize"),
            serde_json::to_value(&bibliography_source.bibliography)
                .expect("bibliography should serialize")
        );
    }

    #[test]
    fn suppresses_author_date_default_bibliography_sort() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::AuthorDate),
            Some(&legacy_sort(&["author", "issued", "title"])),
        );

        assert_eq!(sort, None);
    }

    #[test]
    fn suppresses_note_default_bibliography_sort() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::Note),
            Some(&legacy_sort(&["author", "title", "issued"])),
        );

        assert_eq!(sort, None);
    }

    #[test]
    fn preserves_numeric_bibliography_sort() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::Numeric),
            Some(&legacy_sort(&["author", "issued", "title"])),
        );

        assert_eq!(
            sort,
            Some(citum_schema::grouping::GroupSortEntry::Preset(
                citum_schema::presets::SortPreset::AuthorDateTitle
            ))
        );
    }

    #[test]
    fn preserves_note_family_exceptions() {
        let sort = resolve_migrated_bibliography_sort(
            Some(&citum_schema::options::Processing::Note),
            Some(&legacy_sort(&["author", "issued", "title"])),
        );

        assert!(sort.is_some());
    }

    #[test]
    fn maps_legacy_citation_number_collapse() {
        let mut style = minimal_legacy_style();
        style.citation.collapse = Some("citation-number".to_string());

        let migrated = build_final_style(
            &style,
            CompiledOutput {
                options: citum_schema::options::Config::default(),
                bibliography_options: None,
                citation_contributor_overrides: None,
                bibliography_contributor_overrides: None,
                new_cit: vec![],
                new_bib: vec![],
                type_templates: None,
                citation_wrap: None,
                citation_prefix: None,
                citation_suffix: None,
                citation_delimiter: None,
                citation_subsequent_override: None,
                citation_ibid_override: None,
            },
        );

        assert_eq!(
            migrated
                .citation
                .as_ref()
                .and_then(|citation| citation.collapse.clone()),
            Some(CitationCollapse::CitationNumber)
        );
    }

    #[test]
    fn inherited_contributor_defaults_are_compacted() {
        let defaults = ContributorConfig {
            display_as_sort: Some(DisplayAsSort::First),
            and: Some(AndOptions::Text),
            shorten: Some(ShortenListOptions {
                min: 3,
                use_first: 1,
                ..ShortenListOptions::default()
            }),
            sort_separator: Some(", ".to_string()),
            name_form: Some(NameForm::Initials),
            ..ContributorConfig::default()
        };
        let mut template = vec![
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                form: ContributorForm::Long,
                name_order: Some(NameOrder::FamilyFirstOnly),
                and: Some(AndOptions::Text),
                shorten: defaults.shorten.clone(),
                sort_separator: Some(", ".to_string()),
                name_form: Some(NameForm::Initials),
                ..TemplateContributor::default()
            }),
            TemplateComponent::Group(TemplateGroup {
                group: vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Editor,
                    form: ContributorForm::Long,
                    name_order: Some(NameOrder::FamilyFirst),
                    and: Some(AndOptions::Text),
                    shorten: defaults.shorten.clone(),
                    sort_separator: Some(", ".to_string()),
                    name_form: Some(NameForm::Initials),
                    ..TemplateContributor::default()
                })],
                ..TemplateGroup::default()
            }),
        ];

        normalize_templates_against_options(
            &mut template,
            TitleDefaultContext::Default,
            &citum_schema::options::Config {
                contributors: Some(defaults),
                ..citum_schema::options::Config::default()
            },
        );

        let TemplateComponent::Contributor(author) =
            template.first().expect("first component should exist")
        else {
            panic!("first component should remain a contributor");
        };
        assert_eq!(author.form, ContributorForm::Long);
        assert_eq!(author.name_order, None);
        assert_eq!(author.and, None);
        assert_eq!(author.shorten, None);
        assert_eq!(author.sort_separator, None);
        assert_eq!(author.name_form, None);

        let TemplateComponent::Group(group) =
            template.get(1).expect("second component should exist")
        else {
            panic!("second component should remain a group");
        };
        let TemplateComponent::Contributor(editor) =
            group.group.first().expect("group child should exist")
        else {
            panic!("group child should remain a contributor");
        };
        assert_eq!(editor.name_order, None);
        assert_eq!(editor.and, None);
        assert_eq!(editor.shorten, None);
        assert_eq!(editor.sort_separator, None);
        assert_eq!(editor.name_form, None);
    }

    #[test]
    fn differing_contributor_fields_are_preserved() {
        let defaults = ContributorConfig {
            display_as_sort: Some(DisplayAsSort::First),
            and: Some(AndOptions::Text),
            sort_separator: Some(", ".to_string()),
            name_form: Some(NameForm::Initials),
            ..ContributorConfig::default()
        };
        let mut template = vec![TemplateComponent::Contributor(TemplateContributor {
            contributor: ContributorRole::Author,
            form: ContributorForm::Long,
            name_order: Some(NameOrder::GivenFirst),
            and: Some(AndOptions::Symbol),
            sort_separator: Some(" ".to_string()),
            name_form: Some(NameForm::Full),
            ..TemplateContributor::default()
        })];

        normalize_templates_against_options(
            &mut template,
            TitleDefaultContext::Default,
            &citum_schema::options::Config {
                contributors: Some(defaults),
                ..citum_schema::options::Config::default()
            },
        );

        let TemplateComponent::Contributor(author) =
            template.first().expect("component should exist")
        else {
            panic!("component should remain a contributor");
        };
        assert_eq!(author.name_order, Some(NameOrder::GivenFirst));
        assert_eq!(author.and, Some(AndOptions::Symbol));
        assert_eq!(author.sort_separator, Some(" ".to_string()));
        assert_eq!(author.name_form, Some(NameForm::Full));
    }

    #[test]
    fn citation_contributor_defaults_are_compacted() {
        let style = minimal_legacy_style();
        let migrated = build_final_style(
            &style,
            CompiledOutput {
                options: citum_schema::options::Config {
                    contributors: Some(ContributorConfig {
                        display_as_sort: Some(DisplayAsSort::First),
                        and: Some(AndOptions::Symbol),
                        ..ContributorConfig::default()
                    }),
                    ..citum_schema::options::Config::default()
                },
                citation_contributor_overrides: Some(ContributorConfig {
                    and: Some(AndOptions::Text),
                    ..ContributorConfig::default()
                }),
                bibliography_options: None,
                bibliography_contributor_overrides: None,
                new_cit: vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    name_order: Some(NameOrder::FamilyFirstOnly),
                    and: Some(AndOptions::Text),
                    ..TemplateContributor::default()
                })],
                new_bib: vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    name_order: Some(NameOrder::FamilyFirstOnly),
                    and: Some(AndOptions::Symbol),
                    ..TemplateContributor::default()
                })],
                type_templates: None,
                citation_wrap: None,
                citation_prefix: None,
                citation_suffix: None,
                citation_delimiter: None,
                citation_subsequent_override: None,
                citation_ibid_override: None,
            },
        );

        let citation_template = migrated
            .citation
            .as_ref()
            .and_then(|citation| citation.template.as_ref())
            .expect("citation template should be present");
        let TemplateComponent::Contributor(citation_author) = citation_template
            .first()
            .expect("citation component should exist")
        else {
            panic!("citation component should remain a contributor");
        };
        assert_eq!(citation_author.name_order, None);
        assert_eq!(citation_author.and, None);

        let bibliography_template = migrated
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.template.as_ref())
            .expect("bibliography template should be present");
        let TemplateComponent::Contributor(bibliography_author) = bibliography_template
            .first()
            .expect("bibliography component should exist")
        else {
            panic!("bibliography component should remain a contributor");
        };
        assert_eq!(bibliography_author.name_order, None);
        assert_eq!(bibliography_author.and, None);
    }

    #[test]
    fn citation_contributor_overrides_are_preserved_when_they_differ() {
        let style = minimal_legacy_style();
        let migrated = build_final_style(
            &style,
            CompiledOutput {
                options: citum_schema::options::Config {
                    contributors: Some(ContributorConfig {
                        display_as_sort: Some(DisplayAsSort::First),
                        and: Some(AndOptions::Text),
                        ..ContributorConfig::default()
                    }),
                    ..citum_schema::options::Config::default()
                },
                bibliography_options: None,
                citation_contributor_overrides: Some(ContributorConfig {
                    and: Some(AndOptions::Symbol),
                    ..ContributorConfig::default()
                }),
                bibliography_contributor_overrides: None,
                new_cit: vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    name_order: Some(NameOrder::FamilyFirstOnly),
                    and: Some(AndOptions::Text),
                    ..TemplateContributor::default()
                })],
                new_bib: vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: ContributorForm::Long,
                    name_order: Some(NameOrder::FamilyFirstOnly),
                    and: Some(AndOptions::Text),
                    ..TemplateContributor::default()
                })],
                type_templates: None,
                citation_wrap: None,
                citation_prefix: None,
                citation_suffix: None,
                citation_delimiter: None,
                citation_subsequent_override: None,
                citation_ibid_override: None,
            },
        );

        let citation_template = migrated
            .citation
            .as_ref()
            .and_then(|citation| citation.template.as_ref())
            .expect("citation template should be present");
        let TemplateComponent::Contributor(citation_author) = citation_template
            .first()
            .expect("citation component should exist")
        else {
            panic!("citation component should remain a contributor");
        };
        assert_eq!(citation_author.name_order, None);
        assert_eq!(citation_author.and, Some(AndOptions::Text));

        let bibliography_template = migrated
            .bibliography
            .as_ref()
            .and_then(|bibliography| bibliography.template.as_ref())
            .expect("bibliography template should be present");
        let TemplateComponent::Contributor(bibliography_author) = bibliography_template
            .first()
            .expect("bibliography component should exist")
        else {
            panic!("bibliography component should remain a contributor");
        };
        assert_eq!(bibliography_author.name_order, None);
        assert_eq!(bibliography_author.and, None);
    }

    #[test]
    fn inherited_title_rendering_defaults_are_compacted() {
        let mut template = vec![TemplateComponent::Title(TemplateTitle {
            title: TitleType::Primary,
            rendering: Rendering {
                text_case: Some(TextCase::SentenceApa),
                emph: Some(true),
                quote: Some(true),
                ..Rendering::default()
            },
            ..TemplateTitle::default()
        })];

        normalize_templates_against_options(
            &mut template,
            TitleDefaultContext::Default,
            &citum_schema::options::Config {
                titles: Some(TitlesConfig {
                    default: Some(TitleRendering {
                        text_case: Some(TextCase::SentenceApa),
                        emph: Some(true),
                        quote: Some(true),
                        ..TitleRendering::default()
                    }),
                    ..TitlesConfig::default()
                }),
                ..citum_schema::options::Config::default()
            },
        );

        let TemplateComponent::Title(title) = template.first().expect("component should exist")
        else {
            panic!("component should remain a title");
        };
        assert_eq!(title.rendering.text_case, None);
        assert_eq!(title.rendering.emph, None);
        assert_eq!(title.rendering.quote, None);
    }

    #[test]
    fn category_specific_title_defaults_are_compacted() {
        let mut template = vec![TemplateComponent::Title(TemplateTitle {
            title: TitleType::Primary,
            rendering: Rendering {
                quote: Some(true),
                ..Rendering::default()
            },
            ..TemplateTitle::default()
        })];

        normalize_templates_against_options(
            &mut template,
            TitleDefaultContext::RefType("article-journal"),
            &citum_schema::options::Config {
                titles: Some(TitlesConfig {
                    component: Some(TitleRendering {
                        quote: Some(true),
                        ..TitleRendering::default()
                    }),
                    monograph: Some(TitleRendering {
                        emph: Some(true),
                        ..TitleRendering::default()
                    }),
                    ..TitlesConfig::default()
                }),
                ..citum_schema::options::Config::default()
            },
        );

        let TemplateComponent::Title(title) = template.first().expect("component should exist")
        else {
            panic!("component should remain a title");
        };
        assert_eq!(title.rendering.quote, None);
    }

    #[test]
    fn differing_title_rendering_is_preserved() {
        let mut template = vec![TemplateComponent::Title(TemplateTitle {
            title: TitleType::Primary,
            rendering: Rendering {
                text_case: Some(TextCase::Title),
                emph: Some(false),
                quote: Some(true),
                ..Rendering::default()
            },
            ..TemplateTitle::default()
        })];

        normalize_templates_against_options(
            &mut template,
            TitleDefaultContext::Default,
            &citum_schema::options::Config {
                titles: Some(TitlesConfig {
                    default: Some(TitleRendering {
                        text_case: Some(TextCase::SentenceApa),
                        emph: Some(true),
                        quote: Some(true),
                        ..TitleRendering::default()
                    }),
                    ..TitlesConfig::default()
                }),
                ..citum_schema::options::Config::default()
            },
        );

        let TemplateComponent::Title(title) = template.first().expect("component should exist")
        else {
            panic!("component should remain a title");
        };
        assert_eq!(title.rendering.text_case, Some(TextCase::Title));
        assert_eq!(title.rendering.emph, Some(false));
        assert_eq!(title.rendering.quote, None);
    }

    #[test]
    fn date_forms_are_not_compacted() {
        let mut template = vec![TemplateComponent::Date(TemplateDate {
            date: DateVariable::Issued,
            form: DateForm::YearMonth,
            ..TemplateDate::default()
        })];

        normalize_templates_against_options(
            &mut template,
            TitleDefaultContext::Default,
            &citum_schema::options::Config {
                dates: Some(DateConfig {
                    month: MonthFormat::Short,
                    ..DateConfig::default()
                }),
                ..citum_schema::options::Config::default()
            },
        );

        let TemplateComponent::Date(date) = template.first().expect("component should exist")
        else {
            panic!("component should remain a date");
        };
        assert_eq!(date.form, DateForm::YearMonth);
    }

    fn legacy_sort(keys: &[&str]) -> LegacySort {
        LegacySort {
            keys: keys
                .iter()
                .map(|key| LegacySortKey {
                    variable: Some((*key).to_string()),
                    macro_name: None,
                    sort: None,
                })
                .collect(),
        }
    }

    fn minimal_legacy_style() -> LegacyStyle {
        LegacyStyle {
            version: "1.0".to_string(),
            xmlns: "http://purl.org/net/xbiblio/csl".to_string(),
            class: "in-text".to_string(),
            default_locale: None,
            initialize_with: None,
            initialize_with_hyphen: None,
            names_delimiter: None,
            name_as_sort_order: None,
            sort_separator: None,
            delimiter_precedes_last: None,
            delimiter_precedes_et_al: None,
            demote_non_dropping_particle: None,
            and: None,
            page_range_format: None,
            info: Info::default(),
            locale: vec![],
            macros: vec![],
            citation: Citation {
                layout: Layout {
                    prefix: None,
                    suffix: None,
                    delimiter: None,
                    children: vec![],
                },
                sort: None,
                collapse: None,
                et_al_min: None,
                et_al_use_first: None,
                disambiguate_add_year_suffix: None,
                disambiguate_add_names: None,
                disambiguate_add_givenname: None,
                disambiguate_givenname_rule: None,
            },
            bibliography: None,
        }
    }
}
