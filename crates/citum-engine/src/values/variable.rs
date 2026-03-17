//! Rendering logic for simple variables (DOI, URL, ISBN, etc.).
//!
//! This module handles template variable rendering, including proper localization
//! of locator labels and special handling for reference-type-specific variables.

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::reference::Parent;
use citum_schema::template::{SimpleVariable, TemplateVariable};

/// Extracts the short title from a parent reference if available.
///
/// Returns the `short_title` from the embedded parent of collection or serial
/// components, or None if the parent is an ID reference or the component
/// type doesn't support short titles.
fn container_title_short(reference: &Reference) -> Option<String> {
    match reference {
        Reference::CollectionComponent(component) => match &component.parent {
            Parent::Embedded(parent) => parent.short_title.clone(),
            Parent::Id(_) => None,
        },
        Reference::SerialComponent(component) => match &component.parent {
            Parent::Embedded(parent) => parent.short_title.clone(),
            Parent::Id(_) => None,
        },
        _ => None,
    }
}

/// Resolve the raw value string for a simple variable from a reference.
fn resolve_variable_value(
    variable: &SimpleVariable,
    reference: &Reference,
    options: &RenderOptions<'_>,
) -> Option<String> {
    match variable {
        SimpleVariable::Doi => reference.doi(),
        SimpleVariable::Url => reference.url().map(|u| u.to_string()),
        SimpleVariable::Isbn => reference.isbn(),
        SimpleVariable::Issn => reference.issn(),
        SimpleVariable::Publisher => reference.publisher_str(),
        SimpleVariable::PublisherPlace => reference.publisher_place(),
        SimpleVariable::Genre => reference.genre(),
        SimpleVariable::Medium => reference.medium(),
        SimpleVariable::Abstract => reference.abstract_text(),
        SimpleVariable::Note => reference.note(),
        SimpleVariable::Archive => reference.archive(),
        SimpleVariable::ArchiveLocation => reference.archive_location(),
        SimpleVariable::Authority => reference.authority(),
        SimpleVariable::Reporter => reference.reporter(),
        SimpleVariable::Page => reference.pages().map(|v| v.to_string()),
        SimpleVariable::Volume => reference.volume().map(|v| v.to_string()),
        SimpleVariable::Number => reference.number(),
        SimpleVariable::DocketNumber => match reference {
            Reference::Brief(r) => r.docket_number.clone(),
            _ => None,
        },
        SimpleVariable::PatentNumber => match reference {
            Reference::Patent(r) => Some(r.patent_number.clone()),
            _ => None,
        },
        SimpleVariable::StandardNumber => match reference {
            Reference::Standard(r) => Some(r.standard_number.clone()),
            _ => None,
        },
        SimpleVariable::AdsBibcode => reference.ads_bibcode(),
        SimpleVariable::ReportNumber => match reference {
            Reference::Monograph(r) => r.report_number.clone(),
            _ => None,
        },
        SimpleVariable::Version => reference.version(),
        SimpleVariable::ContainerTitleShort => container_title_short(reference),
        SimpleVariable::Locator => options.locator_raw.map(|loc| {
            // When no explicit locators config is set, derive a default from the
            // processing mode so note styles automatically suppress page labels.
            let derived;
            let cfg = if let Some(c) = options.config.locators.as_ref() {
                c
            } else {
                derived = if matches!(
                    options.config.processing,
                    Some(citum_schema::options::Processing::Note)
                ) {
                    citum_schema::options::LocatorPreset::Note.config()
                } else {
                    citum_schema::options::LocatorConfig::default()
                };
                &derived
            };
            let ref_type = options.ref_type.as_deref().unwrap_or("");
            crate::values::locator::render_locator(loc, ref_type, cfg, options.locale)
        }),
        _ => None,
    }
}

impl ComponentValues for TemplateVariable {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        _hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        let value = resolve_variable_value(&self.variable, reference, options);

        value.filter(|s: &String| !s.is_empty()).map(|value| {
            use citum_schema::options::{LinkAnchor, LinkTarget};
            let component_anchor = match self.variable {
                SimpleVariable::Url => LinkAnchor::Url,
                SimpleVariable::Doi => LinkAnchor::Doi,
                _ => LinkAnchor::Component,
            };

            let mut url = crate::values::resolve_effective_url(
                self.links.as_ref(),
                options.config.links.as_ref(),
                reference,
                component_anchor,
            );

            // Fallback for simple legacy config
            if url.is_none()
                && let Some(links) = &self.links
            {
                if self.variable == SimpleVariable::Url
                    && (links.url == Some(true)
                        || matches!(links.target, Some(LinkTarget::Url | LinkTarget::UrlOrDoi)))
                {
                    url = reference.url().map(|u| u.to_string());
                } else if self.variable == SimpleVariable::Doi
                    && (links.doi == Some(true)
                        || matches!(links.target, Some(LinkTarget::Doi | LinkTarget::UrlOrDoi)))
                {
                    url = reference.doi().map(|d| format!("https://doi.org/{d}"));
                }
            }

            ProcValues {
                value,
                prefix: None,
                suffix: None,
                url,
                substituted_key: None,
                pre_formatted: false,
            }
        })
    }
}
