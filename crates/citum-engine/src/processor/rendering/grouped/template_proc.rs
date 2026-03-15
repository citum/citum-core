//! Template processing and bibliography entry rendering.

use super::super::Renderer;
use crate::reference::Reference;
use crate::render::{ProcTemplate, ProcTemplateComponent};
use crate::values::{ComponentValues, ProcHints, RenderContext, RenderOptions};
use citum_schema::citation::LocatorType;
use citum_schema::template::TemplateComponent;
use std::collections::HashSet;

/// Internal request for template processing.
pub(crate) struct TemplateRenderRequest<'a> {
    pub template: &'a [TemplateComponent],
    pub context: RenderContext,
    pub mode: citum_schema::citation::CitationMode,
    pub suppress_author: bool,
    pub locator: Option<String>,
    pub locator_label: Option<LocatorType>,
    pub citation_number: usize,
    pub position: Option<citum_schema::citation::Position>,
    pub integral_name_state: Option<citum_schema::citation::IntegralNameState>,
}

/// Tracks which template variables have been rendered to avoid duplication.
#[derive(Default)]
pub(crate) struct TemplateComponentTracker {
    rendered_vars: HashSet<String>,
    substituted_bases: HashSet<String>,
}

impl TemplateComponentTracker {
    pub(crate) fn should_skip(&self, var_key: Option<&str>) -> bool {
        let Some(var_key) = var_key else {
            return false;
        };
        let base = key_base(var_key);
        self.rendered_vars.contains(var_key) || self.substituted_bases.contains(&base)
    }

    pub(crate) fn mark_rendered(&mut self, var_key: Option<String>, substituted_key: Option<&str>) {
        if let Some(var_key) = var_key {
            self.rendered_vars.insert(var_key);
        }
        if let Some(substituted_key) = substituted_key {
            self.rendered_vars.insert(substituted_key.to_string());
            self.substituted_bases.insert(key_base(substituted_key));
        }
    }
}

impl<'a> Renderer<'a> {
    /// Process a bibliography entry using plain text format.
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

    /// Process a bibliography entry with a specific output format.
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

    /// Process a template for a reference using plain text format.
    pub fn process_template_with_number(
        &self,
        reference: &Reference,
        params: super::super::TemplateRenderParams<'_>,
    ) -> Option<ProcTemplate> {
        self.process_template_with_number_with_format::<crate::render::plain::PlainText>(
            reference, params,
        )
    }

    /// Process a template for a reference with a specific output format.
    pub fn process_template_with_number_with_format<F>(
        &self,
        reference: &Reference,
        params: super::super::TemplateRenderParams<'_>,
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
                locator: params.locator.map(str::to_string),
                locator_label: params.locator_label,
                citation_number: params.citation_number,
                position: params.position.cloned(),
                integral_name_state: params.integral_name_state,
            },
        )
    }

    /// Process a template request with a specific output format.
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

    /// Build render hints for a template from citation metadata.
    pub fn build_template_render_hint(
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

    /// Render a template component with a specific output format.
    pub fn render_template_component_with_format<F>(
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

    /// Apply no-date fallback if issued date is empty.
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

    /// Apply entry link fallback if no component URL is set.
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

    /// Get the preferred no-date term form for this style.
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
}

/// Get a unique key for a template component's variable.
pub(crate) fn get_variable_key(component: &TemplateComponent) -> Option<String> {
    use citum_schema::template::Rendering;

    fn context_suffix(rendering: &Rendering) -> String {
        match (&rendering.prefix, &rendering.suffix) {
            (Some(p), Some(s)) => format!(":{}_{}", p, s),
            (Some(p), None) => format!(":{}", p),
            (None, Some(s)) => format!(":{}", s),
            (None, None) => String::new(),
        }
    }

    fn make_key(kind: &str, value: impl std::fmt::Debug, ctx: String) -> Option<String> {
        Some(format!("{}:{:?}{}", kind, value, ctx))
    }

    match component {
        TemplateComponent::Contributor(c) => {
            make_key("contributor", &c.contributor, context_suffix(&c.rendering))
        }
        TemplateComponent::Date(d) => make_key("date", &d.date, context_suffix(&d.rendering)),
        TemplateComponent::Variable(v) => {
            make_key("variable", &v.variable, context_suffix(&v.rendering))
        }
        TemplateComponent::Title(t) => make_key("title", &t.title, context_suffix(&t.rendering)),
        TemplateComponent::Number(n) => make_key("number", &n.number, context_suffix(&n.rendering)),
        TemplateComponent::List(_) => None,
        _ => None,
    }
}

/// Resolves a template component by applying type-specific overrides.
fn resolve_component_for_ref_type(
    component: &TemplateComponent,
    ref_type: &str,
) -> TemplateComponent {
    use citum_schema::template::ComponentOverride;

    let Some(overrides) = component.overrides() else {
        return component.clone();
    };

    let mut specific: Option<TemplateComponent> = None;
    let mut default_fallback: Option<TemplateComponent> = None;
    let mut type_matched = false;

    for (selector, ov) in overrides {
        if selector.matches(ref_type) {
            type_matched = true;
            if let ComponentOverride::Component(c) = ov {
                specific = Some((**c).clone());
            }
        } else if selector.matches("default")
            && let ComponentOverride::Component(c) = ov
        {
            default_fallback = Some((**c).clone());
        }
    }

    if type_matched {
        specific.unwrap_or_else(|| component.clone())
    } else {
        default_fallback.unwrap_or_else(|| component.clone())
    }
}

/// Get the base key for a template variable (without context suffixes).
fn key_base(key: &str) -> String {
    let mut parts = key.splitn(3, ':');
    match (parts.next(), parts.next()) {
        (Some(kind), Some(var)) => format!("{kind}:{var}"),
        _ => key.to_string(),
    }
}
