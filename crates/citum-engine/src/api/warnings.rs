/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Input-compatibility warning scanners.
//!
//! Each function inspects already-loaded inputs (style, bibliography) for
//! constructs the engine tolerated but cannot act on — unknown reference
//! classes, fields captured by the forward-compat `unknown_fields`
//! catch-all, and unknown enum variants — and reports them as structured
//! [`Warning`]s. Adapters (CLI, WASM, FFI) present these; they must never
//! re-derive their own checks.

use crate::processor::Processor;
use crate::reference::Bibliography;
use citum_schema::locale::{GeneralTerm, TermForm};
use citum_schema::reference::{
    ClassExtension, CollectionType, ContributorRole as ReferenceRole, MonographComponentType,
    MonographType, ReferenceClass, SerialComponentType,
};
use citum_schema::template::ContributorRole as TemplateRole;

use super::{Warning, WarningLevel};

/// Scan the bibliography for unknown reference classes and return compatibility warnings.
pub fn unknown_reference_class_warnings(bibliography: &Bibliography) -> Vec<Warning> {
    bibliography
        .iter()
        .filter_map(|(ref_id, reference)| {
            let ReferenceClass::Unknown(class) = reference.class() else {
                return None;
            };
            Some(Warning {
                level: WarningLevel::Warning,
                code: "unknown_reference_class".to_string(),
                citation_id: None,
                ref_id: Some(ref_id.clone()),
                message: format!(
                    "Reference '{ref_id}' uses unknown class '{class}'; rendering will use only fields this engine understands."
                ),
            })
        })
        .collect()
}

/// Scan the bibliography for fields captured by the forward-compat
/// `unknown_fields` catch-all and return per-reference warnings.
///
/// Unknown-class references are skipped here; they are already reported by
/// [`unknown_reference_class_warnings`].
pub fn unknown_reference_field_warnings(bibliography: &Bibliography) -> Vec<Warning> {
    bibliography
        .iter()
        .filter_map(|(ref_id, reference)| {
            let unknown = reference.unknown_fields()?;
            if unknown.is_empty() {
                return None;
            }
            let keys: Vec<&str> = unknown.keys().map(String::as_str).collect();
            Some(Warning {
                level: WarningLevel::Warning,
                code: "unknown_reference_field".to_string(),
                citation_id: None,
                ref_id: Some(ref_id.clone()),
                message: format!(
                    "Reference '{ref_id}' has unknown field(s): {}; these fields are ignored during rendering.",
                    keys.join(", ")
                ),
            })
        })
        .collect()
}

/// Scan the style and bibliography for unknown enum variants and term keys.
///
/// Returns a list of structured compatibility warnings for encounter of
/// unknown variants that were captured via the tolerant-enum mechanism.
pub fn unknown_enum_warnings(processor: &Processor) -> Vec<Warning> {
    let mut warnings = Vec::new();

    // 1. Scan bibliography
    for (ref_id, reference) in &processor.bibliography {
        match reference.extension() {
            ClassExtension::Monograph(r) => {
                if let MonographType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown monograph type '{s}'; rendering will use default monograph formatting."),
                    });
                }
            }
            ClassExtension::Collection(r) => {
                if let CollectionType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown collection type '{s}'; rendering will use default collection formatting."),
                    });
                }
            }
            ClassExtension::CollectionComponent(r) => {
                if let MonographComponentType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown monograph component type '{s}'; rendering will use default chapter formatting."),
                    });
                }
            }
            ClassExtension::SerialComponent(r) => {
                if let SerialComponentType::Unknown(s) = &r.r#type {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: Some(ref_id.clone()),
                        message: format!("Reference '{ref_id}' uses unknown serial component type '{s}'; rendering will use default article formatting."),
                    });
                }
            }
            _ => {}
        }

        for contributor in reference.all_contributor_entries() {
            if let ReferenceRole::Unknown(s) = &contributor.role {
                warnings.push(Warning {
                    level: WarningLevel::Warning,
                    code: "unknown_enum_variant".to_string(),
                    citation_id: None,
                    ref_id: Some(ref_id.clone()),
                    message: format!("Reference '{ref_id}' uses unknown contributor role '{s}'; this role may be ignored during rendering."),
                });
            }
        }
    }

    // 2. Scan Style
    if let Some(templates) = &processor.style.templates {
        for (name, template) in templates {
            scan_template_for_unknowns(template, &format!("template '{name}'"), &mut warnings);
        }
    }
    if let Some(citation) = &processor.style.citation {
        scan_citation_spec_for_unknowns(citation, "citation layout", &mut warnings);
    }
    if let Some(bib) = &processor.style.bibliography {
        if let Some(template) = &bib.template {
            scan_template_for_unknowns(template, "bibliography layout", &mut warnings);
        }
        if let Some(type_variants) = &bib.type_variants {
            for variant in type_variants.values() {
                if let Some(template) = variant.as_template() {
                    scan_template_for_unknowns(template, "bibliography layout", &mut warnings);
                }
            }
        }
        if let Some(locales) = &bib.locales {
            for locale_spec in locales {
                scan_template_for_unknowns(
                    &locale_spec.template,
                    "bibliography layout",
                    &mut warnings,
                );
            }
        }
    }

    warnings
}

/// Recursively scan a [`citum_schema::CitationSpec`] and its mode/position
/// sub-specs (`integral`, `non-integral`, `subsequent`, `ibid`), plus its
/// `type-variants` and per-locale templates, for unknown enum variants.
///
/// The top-level `unknown_enum_warnings` scan previously inspected only the
/// main citation template, missing unknown terms/roles/date-forms nested in
/// sub-specs, type-variants, and localized templates.
fn scan_citation_spec_for_unknowns(
    spec: &citum_schema::CitationSpec,
    location: &str,
    warnings: &mut Vec<Warning>,
) {
    if let Some(template) = &spec.template {
        scan_template_for_unknowns(template, location, warnings);
    }
    if let Some(type_variants) = &spec.type_variants {
        for variant in type_variants.values() {
            if let Some(template) = variant.as_template() {
                scan_template_for_unknowns(template, location, warnings);
            }
        }
    }
    if let Some(locales) = &spec.locales {
        for locale_spec in locales {
            scan_template_for_unknowns(&locale_spec.template, location, warnings);
        }
    }

    if let Some(child) = &spec.integral {
        scan_citation_spec_for_unknowns(child, &format!("{location} (integral)"), warnings);
    }
    if let Some(child) = &spec.non_integral {
        scan_citation_spec_for_unknowns(child, &format!("{location} (non-integral)"), warnings);
    }
    if let Some(child) = &spec.subsequent {
        scan_citation_spec_for_unknowns(child, &format!("{location} (subsequent)"), warnings);
    }
    if let Some(child) = &spec.ibid {
        scan_citation_spec_for_unknowns(child, &format!("{location} (ibid)"), warnings);
    }
}

fn scan_template_for_unknowns(
    components: &[citum_schema::template::TemplateComponent],
    location: &str,
    warnings: &mut Vec<Warning>,
) {
    use citum_schema::template::TemplateComponent;
    for component in components {
        match component {
            TemplateComponent::Term(t) => {
                if let GeneralTerm::Unknown(s) = &t.term {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown locale term key '{s}'; this term may render as empty."),
                    });
                }
                if let Some(TermForm::Unknown(s)) = &t.form {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown term form '{s}'; falling back to long form."),
                    });
                }
            }
            TemplateComponent::Contributor(c) => {
                if let TemplateRole::Unknown(s) = &c.contributor {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown contributor role '{s}'; this role may be ignored."),
                    });
                }
            }
            TemplateComponent::Date(d) => {
                if let citum_schema::template::DateForm::Unknown(s) = &d.form {
                    warnings.push(Warning {
                        level: WarningLevel::Warning,
                        code: "unknown_enum_variant".to_string(),
                        citation_id: None,
                        ref_id: None,
                        message: format!("Style {location} uses unknown date form '{s}'; falling back to year only."),
                    });
                }
            }
            TemplateComponent::Group(g) => {
                scan_template_for_unknowns(&g.group, location, warnings);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn unknown_enum_warnings_reports_unknown_term_in_integral_sub_spec() {
        let yaml = "info:\n  title: Test\ncitation:\n  integral:\n    template:\n      - term: not-a-real-term\n";
        let style = citum_schema::Style::from_yaml_str(yaml).unwrap();
        let processor = Processor::new(style, Bibliography::new());

        let warnings = unknown_enum_warnings(&processor);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("not-a-real-term") && w.message.contains("(integral)")),
            "expected a warning for the unknown term in citation.integral.template, got: {warnings:?}"
        );
    }

    #[test]
    fn unknown_enum_warnings_reports_unknown_term_in_type_variants() {
        let yaml = "info:\n  title: Test\ncitation:\n  type-variants:\n    book:\n      - term: not-a-real-term-2\n";
        let style = citum_schema::Style::from_yaml_str(yaml).unwrap();
        let processor = Processor::new(style, Bibliography::new());

        let warnings = unknown_enum_warnings(&processor);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("not-a-real-term-2")),
            "expected a warning for the unknown term in citation.type-variants, got: {warnings:?}"
        );
    }
}
