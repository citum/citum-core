//! Rendering logic for numeric variables (volume, issue, pages, citation numbers, etc.).
//!
//! This module handles number component rendering with support for page range formatting,
//! edition labels, and numeric citation identifiers.

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::locale::{GrammaticalGender, TermForm};
use citum_schema::template::{NumberVariable, TemplateNumber};

/// Resolve the raw value string for a number variable from a reference.
fn resolve_number_value(
    number: &NumberVariable,
    reference: &Reference,
    hints: &ProcHints,
    options: &RenderOptions<'_>,
    show_with_locator: bool,
) -> Option<String> {
    match number {
        NumberVariable::Volume => reference.volume().map(|v| v.to_string()),
        NumberVariable::Issue => reference.issue().map(|v| v.to_string()),
        NumberVariable::Pages => {
            let suppress = !show_with_locator
                && options.context == crate::values::RenderContext::Citation
                && options.locator_raw.is_some()
                && matches!(
                    options.config.processing,
                    Some(citum_schema::options::Processing::Note)
                );
            if suppress {
                None
            } else {
                reference.pages().map(|p| {
                    format_page_range(&p.to_string(), options.config.page_range_format.as_ref())
                })
            }
        }
        NumberVariable::ChapterNumber => match reference {
            Reference::Statute(r) => r.chapter_number.clone(),
            _ => reference.numbering_value(&citum_schema::reference::NumberingType::Chapter),
        },
        NumberVariable::Edition => reference.edition(),
        NumberVariable::CollectionNumber => reference.collection_number(),
        NumberVariable::Number => reference.number(),
        NumberVariable::Custom(kind) => reference.numbering_value(
            &citum_schema::reference::NumberingType::Custom(kind.clone()),
        ),
        NumberVariable::DocketNumber => match reference {
            Reference::Brief(r) => r.docket_number.clone(),
            _ => None,
        },
        NumberVariable::PatentNumber => match reference {
            Reference::Patent(r) => Some(r.patent_number.clone()),
            _ => None,
        },
        NumberVariable::StandardNumber => match reference {
            Reference::Standard(r) => Some(r.standard_number.clone()),
            _ => None,
        },
        NumberVariable::ReportNumber => reference.report_number(),
        NumberVariable::PartNumber
        | NumberVariable::SupplementNumber
        | NumberVariable::PrintingNumber => None,
        NumberVariable::CitationNumber => hints.citation_number.map(|n| {
            if options.context == crate::values::RenderContext::Citation
                && let Some(sub_label) = &hints.citation_sub_label
            {
                return format!("{n}{sub_label}");
            }
            n.to_string()
        }),
        NumberVariable::CitationLabel => {
            let Some(citum_schema::options::Processing::Label(config)) =
                options.config.processing.as_ref()
            else {
                return None;
            };
            let params = config.effective_params();
            let base = crate::processor::labels::generate_base_label(reference, &params);
            if base.is_empty() {
                return None;
            }
            let suffix = if hints.disamb_condition && hints.group_index > 0 {
                crate::values::int_to_letter(hints.group_index as u32).unwrap_or_default()
            } else {
                String::new()
            };
            Some(format!("{base}{suffix}"))
        }
        _ => None,
    }
}

/// Resolve a label prefix for a number variable if `label_form` is configured.
fn resolve_number_label<F: crate::render::format::OutputFormat<Output = String>>(
    number: &NumberVariable,
    label_form: &citum_schema::template::LabelForm,
    value: &str,
    requested_gender: Option<GrammaticalGender>,
    effective_rendering: &citum_schema::template::Rendering,
    options: &RenderOptions<'_>,
    fmt: &F,
) -> Option<String> {
    if let Some(locator_type) = number_var_to_locator_type(number) {
        // Check pluralization
        let plural = check_plural(value, &locator_type);

        let term_form = match label_form {
            citum_schema::template::LabelForm::Long => TermForm::Long,
            citum_schema::template::LabelForm::Short => TermForm::Short,
            citum_schema::template::LabelForm::Symbol => TermForm::Symbol,
        };

        options
            .locale
            .resolved_locator_term(&locator_type, plural, term_form, requested_gender)
            .map(|t| {
                let term_str = if crate::values::should_strip_periods(effective_rendering, options)
                {
                    crate::values::strip_trailing_periods(&t)
                } else {
                    t
                };
                fmt.text(&format!("{term_str} "))
            })
    } else {
        None
    }
}

impl ComponentValues for TemplateNumber {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        let fmt = F::default();

        let value = resolve_number_value(
            &self.number,
            reference,
            hints,
            options,
            self.show_with_locator.unwrap_or(false),
        );

        value.filter(|s| !s.is_empty()).map(|value| {
            // Resolve effective rendering options
            let effective_rendering = &self.rendering;

            // Handle label if label_form is specified
            let prefix = if let Some(label_form) = &self.label_form {
                resolve_number_label(
                    &self.number,
                    label_form,
                    &value,
                    self.gender,
                    effective_rendering,
                    options,
                    &fmt,
                )
            } else {
                None
            };

            ProcValues {
                value,
                prefix,
                suffix: None,
                url: crate::values::resolve_effective_url(
                    self.links.as_ref(),
                    options.config.links.as_ref(),
                    reference,
                    citum_schema::options::LinkAnchor::Component,
                ),
                substituted_key: None,
                pre_formatted: false,
            }
        })
    }
}

/// Maps a number variable to its corresponding locator type.
///
/// Determines which `LocatorType` corresponds to a given numeric variable,
/// allowing proper label selection when rendering page, volume, or issue information.
/// Returns `None` for variables with no locator equivalent (e.g. edition, version).
#[must_use]
pub fn number_var_to_locator_type(
    var: &NumberVariable,
) -> Option<citum_schema::citation::LocatorType> {
    use citum_schema::citation::LocatorType;
    match var {
        NumberVariable::Volume => Some(LocatorType::Volume),
        NumberVariable::Pages => Some(LocatorType::Page),
        NumberVariable::ChapterNumber => Some(LocatorType::Chapter),
        NumberVariable::NumberOfPages => Some(LocatorType::Page),
        NumberVariable::NumberOfVolumes => Some(LocatorType::Volume),
        NumberVariable::Number
        | NumberVariable::DocketNumber
        | NumberVariable::PatentNumber
        | NumberVariable::StandardNumber
        | NumberVariable::ReportNumber
        | NumberVariable::PrintingNumber => Some(LocatorType::Number),
        NumberVariable::PartNumber => Some(LocatorType::Part),
        NumberVariable::SupplementNumber => Some(LocatorType::Supplement),
        NumberVariable::Issue => Some(LocatorType::Issue),
        NumberVariable::Custom(kind) => Some(LocatorType::Custom(kind.clone())),
        _ => None,
    }
}

/// Heuristically detect whether a locator string should use plural labeling.
///
/// Returns `true` if the value contains range or list separators — hyphens (`-`),
/// en-dashes (`–`), commas (`,`), or ampersands (`&`) — indicating multiple items
/// such as `"1-10"`, `"1, 3"`, or `"1 & 3"`.
#[must_use]
pub fn check_plural(value: &str, _locator_type: &citum_schema::citation::LocatorType) -> bool {
    // Simple heuristic: if contains ranges or separators, it's plural.
    // "1-10", "1, 3", "1 & 3"
    value.contains('–') || value.contains('-') || value.contains(',') || value.contains('&')
}

/// Format a page range according to the specified format.
///
/// Formats: expanded (default), minimal, minimal-two, chicago, chicago-16
#[must_use]
pub fn format_page_range(
    pages: &str,
    format: Option<&citum_schema::options::PageRangeFormat>,
) -> String {
    use citum_schema::options::PageRangeFormat;

    // First, replace hyphen with en-dash
    let pages = pages.replace('-', "–");

    // If no range or no format specified, return as-is
    let Some(format) = format else {
        return pages; // Default: just convert to en-dash
    };

    // Check if this is a range (contains en-dash)
    let parts: Vec<&str> = pages.split('–').collect();
    if parts.len() != 2 {
        return pages; // Not a simple range
    }

    let start = parts[0].trim();
    let end = parts[1].trim();

    // Parse as numbers
    let start_num: Option<u32> = start.parse().ok();
    let end_num: Option<u32> = end.parse().ok();

    match (start_num, end_num) {
        (Some(s), Some(e)) if e > s => {
            let formatted_end = match format {
                PageRangeFormat::Expanded => end.to_string(),
                PageRangeFormat::Minimal => format_minimal(start, end, 1),
                PageRangeFormat::MinimalTwo => format_minimal(start, end, 2),
                PageRangeFormat::Chicago | PageRangeFormat::Chicago16 => format_chicago(s, e),
                _ => end.to_string(), // Future variants: default to expanded
            };
            format!("{start}–{formatted_end}")
        }
        _ => pages, // Can't parse or invalid range
    }
}

/// Minimal format: keep only differing digits, with minimum `min_digits`
#[must_use]
pub fn format_minimal(start: &str, end: &str, min_digits: usize) -> String {
    let start_chars: Vec<char> = start.chars().collect();
    let end_chars: Vec<char> = end.chars().collect();

    if start_chars.len() != end_chars.len() {
        return end.to_string();
    }

    // Find first differing position
    let mut first_diff = 0;
    for (i, (s, e)) in start_chars.iter().zip(end_chars.iter()).enumerate() {
        if s != e {
            first_diff = i;
            break;
        }
    }

    // Keep at least min_digits from the end
    let keep_from = first_diff.min(end_chars.len().saturating_sub(min_digits));
    end_chars[keep_from..].iter().collect()
}

/// Chicago Manual of Style page range format
#[must_use]
pub fn format_chicago(start: u32, end: u32) -> String {
    // Chicago rules (simplified from CMOS 17th):
    // - Under 100: use all digits (3–10, 71–72, 96–117)
    // - 100+, same hundreds: use changed part only for 2+ digits (107–8, 321–28, 1536–38)
    // - Different hundreds: use all digits (107–108, 321–328 if change of hundreds)

    if start < 100 || end < 100 {
        return end.to_string();
    }

    let start_str = start.to_string();
    let end_str = end.to_string();

    if start_str.len() != end_str.len() {
        return end_str;
    }

    // Check if same hundreds
    let start_prefix = start / 100;
    let end_prefix = end / 100;

    if start_prefix != end_prefix {
        return end_str; // Different hundreds, use full number
    }

    // Same hundreds: use minimal-two style
    format_minimal(&start_str, &end_str, 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::options::PageRangeFormat;

    #[test]
    fn test_format_chicago() {
        for (start, end, expected) in [
            (3, 10, "10"),
            (71, 72, "72"),
            (96, 117, "117"),
            (107, 108, "08"),
            (321, 328, "28"),
            (1536, 1538, "38"),
            (107, 208, "208"),
            (321, 428, "428"),
        ] {
            assert_eq!(format_chicago(start, end), expected);
        }
    }

    #[test]
    fn test_format_minimal() {
        for (start, end, min_digits, expected) in [
            ("100", "105", 1, "5"),
            ("100", "105", 2, "05"),
            ("1536", "1538", 1, "8"),
            ("1536", "1538", 2, "38"),
            ("1536", "1538", 4, "1538"),
            ("12", "15", 1, "5"),
            ("12", "15", 2, "15"),
            ("10", "150", 1, "150"),
        ] {
            assert_eq!(format_minimal(start, end, min_digits), expected);
        }
    }

    #[test]
    fn test_format_page_range() {
        for (input, format, expected) in [
            ("10-15", None, "10–15"),
            ("10–15", None, "10–15"),
            ("321-328", None, "321–328"),
            ("10-15", Some(PageRangeFormat::Expanded), "10–15"),
            ("42-45", Some(PageRangeFormat::Expanded), "42–45"),
            ("107-108", Some(PageRangeFormat::Chicago), "107–08"),
            ("71-72", Some(PageRangeFormat::Chicago), "71–72"),
            ("321-328", Some(PageRangeFormat::Chicago), "321–28"),
            ("321-428", Some(PageRangeFormat::Chicago), "321–428"),
            ("1536-1538", Some(PageRangeFormat::Chicago), "1536–38"),
            ("100-105", Some(PageRangeFormat::Minimal), "100–5"),
            ("321-328", Some(PageRangeFormat::Minimal), "321–8"),
            ("42-45", Some(PageRangeFormat::Minimal), "42–5"),
            ("12-17", Some(PageRangeFormat::Minimal), "12–7"),
            ("100-105", Some(PageRangeFormat::MinimalTwo), "100–05"),
            ("42-45", Some(PageRangeFormat::MinimalTwo), "42–45"),
            ("10", Some(PageRangeFormat::Chicago), "10"),
            ("10-5", Some(PageRangeFormat::Chicago), "10–5"),
            ("X-Y", Some(PageRangeFormat::Chicago), "X–Y"),
            ("10-15-20", Some(PageRangeFormat::Chicago), "10–15–20"),
        ] {
            assert_eq!(format_page_range(input, format.as_ref()), expected);
        }
    }

    #[test]
    fn test_check_plural() {
        for (value, expected) in [
            ("1-10", true),
            ("1–10", true),
            ("1, 3", true),
            ("1 & 3", true),
            ("1", false),
            ("IV", false),
        ] {
            assert_eq!(
                check_plural(value, &citum_schema::citation::LocatorType::Page),
                expected
            );
        }
    }
}
