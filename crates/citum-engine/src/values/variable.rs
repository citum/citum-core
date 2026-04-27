//! Rendering logic for simple variables (DOI, URL, ISBN, etc.).
//!
//! This module handles template variable rendering, including proper localization
//! of locator labels and special handling for reference-type-specific variables.

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::locale::ArchiveHierarchyField;
use citum_schema::options::titles::TextCase;
use citum_schema::reference::RichText;
use citum_schema::template::{SimpleVariable, TemplateVariable};

/// Extracts the short title from a parent reference if available.
///
/// Returns the `short_title` from the embedded parent of collection or serial
/// components, or None if the parent is an ID reference or the component
/// type doesn't support short titles.
fn container_title_short(reference: &Reference) -> Option<String> {
    reference.container_title().and_then(|t| match t {
        citum_schema::reference::types::Title::Shorthand(short, _) => Some(short),
        citum_schema::reference::types::Title::Single(s) => Some(s),
        _ => None,
    })
}

fn resolve_archive_name(reference: &Reference, options: &RenderOptions<'_>) -> Option<String> {
    let archive_name = reference.archive_name()?;
    let multilingual = options.config.multilingual.as_ref();

    Some(crate::values::resolve_multilingual_string(
        &archive_name,
        multilingual.and_then(|ml| ml.name_mode.as_ref()),
        multilingual.and_then(|ml| ml.preferred_transliteration.as_deref()),
        multilingual.and_then(|ml| ml.preferred_script.as_ref()),
        options.locale.locale.as_str(),
    ))
}

fn assemble_archive_hierarchy(
    reference: &Reference,
    options: &RenderOptions<'_>,
) -> Option<String> {
    let locale = options.locale;
    let mut parts: Vec<String> = Vec::new();

    // collection (with optional collection_id in parens)
    if let Some(collection) = reference.archive_collection() {
        let label = locale
            .resolved_archive_term(ArchiveHierarchyField::Collection)
            .map(|l| format!("{l} "))
            .unwrap_or_default();
        if let Some(cid) = reference.archive_collection_id() {
            parts.push(format!("{label}{collection} ({cid})"));
        } else {
            parts.push(format!("{label}{collection}"));
        }
    }

    // series
    if let Some(series) = reference.archive_series() {
        let label = locale
            .resolved_archive_term(ArchiveHierarchyField::Series)
            .map(|l| format!("{l} "))
            .unwrap_or_default();
        parts.push(format!("{label}{series}"));
    }

    // box
    if let Some(b) = reference.archive_box() {
        let label = locale
            .resolved_archive_term(ArchiveHierarchyField::Box)
            .map(|l| format!("{l} "))
            .unwrap_or_default();
        parts.push(format!("{label}{b}"));
    }

    // folder
    if let Some(folder) = reference.archive_folder() {
        let label = locale
            .resolved_archive_term(ArchiveHierarchyField::Folder)
            .map(|l| format!("{l} "))
            .unwrap_or_default();
        parts.push(format!("{label}{folder}"));
    }

    // item
    if let Some(item) = reference.archive_item() {
        let label = locale
            .resolved_archive_term(ArchiveHierarchyField::Item)
            .map(|l| format!("{l} "))
            .unwrap_or_default();
        parts.push(format!("{label}{item}"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn make_rich_text_case_transform(case: TextCase) -> impl FnMut(&str) -> String {
    let mut seen_alpha = false;
    move |text: &str| match case {
        TextCase::Sentence | TextCase::SentenceApa | TextCase::SentenceNlm => {
            let lowered = text.to_lowercase();
            if seen_alpha {
                lowered
            } else {
                let result = crate::values::text_case::capitalize_first_word(&lowered);
                if result.chars().any(char::is_alphabetic) {
                    seen_alpha = true;
                }
                result
            }
        }
        _ => crate::values::text_case::apply_text_case(text, case),
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
        SimpleVariable::Url => reference.url().map(|u| u.to_string()).or_else(|| {
            (reference.ref_type() == "dataset")
                .then(|| reference.doi().map(|doi| format!("https://doi.org/{doi}")))
                .flatten()
        }),
        SimpleVariable::Isbn => reference.isbn(),
        SimpleVariable::Issn => reference.issn(),
        SimpleVariable::Publisher => reference.publisher_str(),
        SimpleVariable::PublisherPlace => reference.publisher_place(),
        SimpleVariable::OriginalPublisher => reference.original_publisher_str(),
        SimpleVariable::OriginalPublisherPlace => reference.original_publisher_place(),
        SimpleVariable::Genre => reference.genre().map(|k| options.locale.lookup_genre(&k)),
        SimpleVariable::Medium => reference.medium().map(|k| options.locale.lookup_medium(&k)),
        SimpleVariable::Status => reference.status(),
        SimpleVariable::Abstract | SimpleVariable::Note => None,
        SimpleVariable::Archive => reference.archive(),
        SimpleVariable::ArchiveLocation => reference
            .archive_location()
            .or_else(|| assemble_archive_hierarchy(reference, options)),
        SimpleVariable::ArchiveName => resolve_archive_name(reference, options),
        SimpleVariable::ArchivePlace => reference.archive_place(),
        SimpleVariable::ArchiveCollection => reference.archive_collection(),
        SimpleVariable::ArchiveCollectionId => reference.archive_collection_id(),
        SimpleVariable::ArchiveSeries => reference.archive_series(),
        SimpleVariable::ArchiveBox => reference.archive_box(),
        SimpleVariable::ArchiveFolder => reference.archive_folder(),
        SimpleVariable::ArchiveItem => reference.archive_item(),
        SimpleVariable::ArchiveUrl => reference.archive_url().map(|url| url.to_string()),
        SimpleVariable::EprintId => reference.eprint_id(),
        SimpleVariable::EprintServer => reference.eprint_server(),
        SimpleVariable::EprintClass => reference.eprint_class(),
        SimpleVariable::Authority => reference.authority(),
        SimpleVariable::Code => reference.code(),
        SimpleVariable::Reporter => reference.reporter(),
        SimpleVariable::Page => reference.pages().map(|v| v.to_string()),
        SimpleVariable::Section => reference.section(),
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
        SimpleVariable::ReportNumber => reference.report_number(),
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
        // Rich-text variables carry format metadata — handle before the plain-string path.
        let rich_text: Option<RichText> = match self.variable {
            SimpleVariable::Note => reference.note(),
            SimpleVariable::Abstract => reference.abstract_text(),
            _ => None,
        };

        if let Some(rt) = rich_text {
            if rt.is_empty() {
                return None;
            }
            let fmt = F::default();
            let (value, pre_formatted) = match (rt, self.rendering.text_case) {
                (RichText::Plain(s), Some(tc)) => {
                    (crate::values::text_case::apply_text_case(&s, tc), false)
                }
                (RichText::Plain(s), None) => (s, false),
                (RichText::Djot { djot }, Some(tc)) => (
                    crate::render::rich_text::render_djot_inline_with_transform(
                        &djot,
                        &fmt,
                        make_rich_text_case_transform(tc),
                    )
                    .0,
                    true,
                ),
                (RichText::Djot { djot }, None) => {
                    (crate::render::render_djot_inline(&djot, &fmt), true)
                }
            };
            return Some(ProcValues {
                value,
                prefix: None,
                suffix: None,
                url: None,
                substituted_key: None,
                pre_formatted,
            });
        }

        // Plain-string path for all other variables.
        let value = resolve_variable_value(&self.variable, reference, options);

        value.filter(|s: &String| !s.is_empty()).map(|value| {
            let value = if let Some(tc) = self.rendering.text_case {
                crate::values::text_case::apply_text_case(&value, tc)
            } else {
                value
            };
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
