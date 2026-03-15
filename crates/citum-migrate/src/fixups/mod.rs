#![allow(missing_docs)]

use citum_schema::template::{TemplateComponent, TypeSelector, WrapPunctuation};
use csl_legacy::model::Layout;

mod locator;
mod media;
mod template;

/// Returns whether a type selector matches any candidate type name.
pub fn selector_matches_any(selector: &TypeSelector, candidates: &[&str]) -> bool {
    media::selector_matches_any(selector, candidates)
}

/// Normalizes legal-case type templates after migration from legacy CSL.
pub fn normalize_legal_case_type_template(
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
) {
    media::normalize_legal_case_type_template(legacy_style, type_templates);
}

/// Ensures inferred media-oriented type templates exist when legacy signals require them.
pub fn ensure_inferred_media_type_templates(
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
    bibliography_template: &[TemplateComponent],
) {
    media::ensure_inferred_media_type_templates(
        legacy_style,
        type_templates,
        bibliography_template,
    );
}

/// Ensures personal communication types are omitted when legacy behavior suppresses them.
pub fn ensure_personal_communication_omitted(
    legacy_style: &csl_legacy::model::Style,
    citation_template: &[TemplateComponent],
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
) {
    media::ensure_personal_communication_omitted(legacy_style, citation_template, type_templates);
}

/// Ensures inferred patent templates exist when legacy output expects them.
pub fn ensure_inferred_patent_type_template(
    legacy_style: &csl_legacy::model::Style,
    type_templates: &mut Option<std::collections::HashMap<TypeSelector, Vec<TemplateComponent>>>,
    bibliography_template: &[TemplateComponent],
) {
    media::ensure_inferred_patent_type_template(
        legacy_style,
        type_templates,
        bibliography_template,
    );
}

/// Adds a numeric locator component when the legacy citation layout uses one.
pub fn ensure_numeric_locator_citation_component(
    layout: &Layout,
    template: &mut [TemplateComponent],
) {
    locator::ensure_numeric_locator_citation_component(layout, template);
}

/// Normalizes wrapped numeric locator formatting extracted from legacy layout groups.
pub fn normalize_wrapped_numeric_locator_citation_component(
    layout: &Layout,
    template: &mut [TemplateComponent],
    citation_delimiter: &mut Option<String>,
) {
    locator::normalize_wrapped_numeric_locator_citation_component(
        layout,
        template,
        citation_delimiter,
    );
}

/// Normalizes author-date locator formatting based on legacy citation layout hints.
pub fn normalize_author_date_locator_citation_component(
    layout: &Layout,
    macros: &[csl_legacy::model::Macro],
    template: &mut Vec<TemplateComponent>,
) {
    locator::normalize_author_date_locator_citation_component(layout, macros, template);
}

/// Moves citation-number wrapping from a legacy layout group to migrated citation items.
pub fn move_group_wrap_to_citation_items(
    layout: &Layout,
    template: &mut [TemplateComponent],
    citation_wrap: &mut Option<WrapPunctuation>,
) {
    locator::move_group_wrap_to_citation_items(layout, template, citation_wrap);
}

/// Returns whether a citation template contains a citation-number component.
pub fn citation_template_has_citation_number(template: &[TemplateComponent]) -> bool {
    locator::citation_template_has_citation_number(template)
}

/// Returns whether a note citation template is too small to preserve note semantics.
pub fn note_citation_template_is_underfit(template: &[TemplateComponent]) -> bool {
    template::note_citation_template_is_underfit(template)
}

/// Returns whether a citation template contains only author and year structure.
pub fn citation_template_is_author_year_only(template: &[TemplateComponent]) -> bool {
    template::citation_template_is_author_year_only(template)
}

/// Converts long contributor forms to short forms throughout a migrated template.
pub fn normalize_contributor_form_to_short(template: &mut [TemplateComponent]) -> bool {
    template::normalize_contributor_form_to_short(template)
}

/// Normalizes inferred contributor rendering for author-date citation templates.
pub fn normalize_author_date_inferred_contributors(
    template: &mut [TemplateComponent],
    drop_component_shorten: bool,
) -> bool {
    template::normalize_author_date_inferred_contributors(template, drop_component_shorten)
}

/// Returns whether a migrated type template should be merged into inferred output.
pub fn should_merge_inferred_type_template(
    type_name: &str,
    inferred_template: &[TemplateComponent],
    candidate_template: &[TemplateComponent],
) -> bool {
    template::should_merge_inferred_type_template(type_name, inferred_template, candidate_template)
}

/// Scrubs literal artifacts introduced by inferred template fragments.
pub fn scrub_inferred_literal_artifacts(component: &mut TemplateComponent) {
    template::scrub_inferred_literal_artifacts(component);
}
