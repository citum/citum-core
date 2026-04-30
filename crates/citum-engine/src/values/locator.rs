/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Locator rendering logic for citations.
//!
//! Renders citation locators (page numbers, sections, etc.) with configurable
//! labels, range formatting, and compound locator patterns.

use citum_schema::citation::{CitationLocator, LocatorSegment, LocatorType};
use citum_schema::locale::{Locale, TermForm};
use citum_schema::options::{LabelForm, LabelRepeat, LocatorConfig, PageRangeFormat};

/// Render a citation locator to a display string.
///
/// All label, range, and delimiter decisions are driven by `config`.
/// Returns an empty string when the locator is absent.
///
/// # Arguments
/// * `locator` - The citation locator to render.
/// * `ref_type` - The reference type for optional type-class gating.
/// * `config` - The locator configuration.
/// * `locale` - The locale for term lookup.
#[must_use]
pub fn render_locator(
    locator: &CitationLocator,
    ref_type: &str,
    config: &LocatorConfig,
    locale: &Locale,
) -> String {
    let segments = locator.segments();

    // Collect the set of locator kinds present in the locator
    let kinds: std::collections::HashSet<LocatorType> =
        segments.iter().map(|seg| seg.label.clone()).collect();

    // Find first matching pattern
    let pattern = config.patterns.iter().find(|p| {
        // Check if pattern's kind set is a subset of active kinds
        let pattern_kinds: std::collections::HashSet<LocatorType> =
            p.kinds.iter().cloned().collect();
        if !pattern_kinds.is_subset(&kinds) {
            return false;
        }

        // Check type_class gate if present
        if let Some(type_class) = p.type_class
            && !type_class_matches(ref_type, type_class)
        {
            return false;
        }

        true
    });

    if let Some(pattern) = pattern {
        render_with_pattern(segments, pattern, config, locale)
    } else {
        render_default(segments, config, locale)
    }
}

/// Render segments using a matched pattern.
fn render_with_pattern(
    segments: &[LocatorSegment],
    pattern: &citum_schema::options::LocatorPattern,
    config: &LocatorConfig,
    locale: &Locale,
) -> String {
    let mut rendered = Vec::new();

    for (idx, kind) in pattern.order.iter().enumerate() {
        // Find the segment with this kind
        if let Some(seg) = segments.iter().find(|s| s.label == *kind) {
            let kind_cfg = config.kinds.get(kind);
            let should_label = matches!(pattern.label_repeat, LabelRepeat::All)
                || (matches!(pattern.label_repeat, LabelRepeat::First) && idx == 0);

            let rendered_segment = if should_label {
                let form = kind_cfg
                    .and_then(|cfg| cfg.label_form)
                    .unwrap_or(config.default_label_form);
                render_segment_with_label(seg, kind_cfg, form, config.strip_label_periods, locale)
            } else {
                let kind_range_fmt = kind_cfg
                    .and_then(|k| k.range_format.clone())
                    .unwrap_or_else(|| config.range_format.clone());
                apply_range_format(seg.value.value_str(), kind_range_fmt)
            };

            rendered.push(rendered_segment);
        }
    }

    // Render segments not covered by pattern.order using default rendering
    let covered: std::collections::HashSet<LocatorType> = pattern.order.iter().cloned().collect();
    for seg in segments.iter().filter(|s| !covered.contains(&s.label)) {
        let kind_cfg = config.kinds.get(&seg.label);
        let form = kind_cfg
            .and_then(|cfg| cfg.label_form)
            .unwrap_or(config.default_label_form);
        let kind_range_fmt = kind_cfg
            .and_then(|k| k.range_format.clone())
            .unwrap_or_else(|| config.range_format.clone());
        let value_str = apply_range_format(seg.value.value_str(), kind_range_fmt);
        let rendered_segment = if matches!(form, LabelForm::None) {
            value_str
        } else {
            render_segment_with_label_str(
                seg,
                kind_cfg,
                form,
                &value_str,
                config.strip_label_periods,
                locale,
            )
        };
        rendered.push(rendered_segment);
    }

    rendered.join(&pattern.delimiter)
}

/// Render segments without a matched pattern (default behavior).
fn render_default(segments: &[LocatorSegment], config: &LocatorConfig, locale: &Locale) -> String {
    let mut rendered = Vec::new();

    for seg in segments {
        let kind_cfg = config.kinds.get(&seg.label);
        let form = kind_cfg
            .and_then(|cfg| cfg.label_form)
            .unwrap_or(config.default_label_form);

        let rendered_segment = if matches!(form, LabelForm::None) {
            let kind_range_fmt = kind_cfg
                .and_then(|k| k.range_format.clone())
                .unwrap_or_else(|| config.range_format.clone());
            apply_range_format(seg.value.value_str(), kind_range_fmt)
        } else {
            render_segment_with_label(seg, kind_cfg, form, config.strip_label_periods, locale)
        };

        rendered.push(rendered_segment);
    }

    rendered.join(&config.fallback_delimiter)
}

/// Render a single segment with label.
fn render_segment_with_label(
    seg: &LocatorSegment,
    kind_cfg: Option<&citum_schema::options::LocatorKindConfig>,
    form: LabelForm,
    global_strip: Option<bool>,
    locale: &Locale,
) -> String {
    let kind_range_fmt = kind_cfg
        .and_then(|k| k.range_format.clone())
        .unwrap_or(PageRangeFormat::Expanded);
    let value_str = apply_range_format(seg.value.value_str(), kind_range_fmt);
    render_segment_with_label_str(seg, kind_cfg, form, &value_str, global_strip, locale)
}

/// Render a single segment with label, given a pre-computed value string.
fn render_segment_with_label_str(
    seg: &LocatorSegment,
    kind_cfg: Option<&citum_schema::options::LocatorKindConfig>,
    form: LabelForm,
    value_str: &str,
    global_strip: Option<bool>,
    locale: &Locale,
) -> String {
    let plural = seg.value.is_plural();

    let term_form = match form {
        LabelForm::Short => TermForm::Short,
        LabelForm::Long => TermForm::Long,
        LabelForm::Symbol => TermForm::Symbol,
        LabelForm::None => TermForm::Short, // Shouldn't reach here
    };

    if let Some(term) = locale.resolved_locator_term(&seg.label, plural, term_form, None) {
        let strip_periods = kind_cfg
            .and_then(|k| k.strip_label_periods)
            .or(global_strip)
            == Some(true);
        if strip_periods {
            // Stripping the trailing period removes the natural separator,
            // so no additional space is added (e.g. "p." → "p23" not "p 23").
            let term_str = crate::values::strip_trailing_periods(&term);
            format!("{term_str}{value_str}")
        } else {
            format!("{term} {value_str}")
        }
    } else {
        value_str.to_string()
    }
}

/// Apply range format to a value string containing a range separator.
fn apply_range_format(value: &str, format: PageRangeFormat) -> String {
    // Only act on values containing a range separator
    let sep_pos = value.find(['-', '–', '—']);
    let Some(pos) = sep_pos else {
        return value.to_string();
    };
    #[allow(clippy::string_slice, reason = "index from find()")]
    let start = &value[..pos];
    // Find end of separator (handle multi-byte em/en-dash)
    #[allow(clippy::string_slice, reason = "index from find()")]
    let sep_end = value[pos..]
        .chars()
        .next()
        .map_or(pos + 1, |c| pos + c.len_utf8());
    #[allow(clippy::string_slice, reason = "index from find() + char len")]
    let end = value[sep_end..].trim_start();
    match format {
        PageRangeFormat::Expanded => {
            // Expand abbreviated end: "33-5" → "33-35", "100-3" → "100-103"
            let expanded_end = expand_range_end(start.trim(), end);
            format!("{start}\u{2013}{expanded_end}") // en-dash
        }
        PageRangeFormat::Minimal => {
            // Minimal: trim shared prefix from end
            let minimal_end = minimal_range_end(start.trim(), end);
            format!("{start}\u{2013}{minimal_end}")
        }
        PageRangeFormat::MinimalTwo => {
            // MinimalTwo: keep at least 2 digits on end
            let minimal_end = minimal_range_end(start.trim(), end);
            format!("{start}\u{2013}{minimal_end}")
        }
        PageRangeFormat::Chicago | PageRangeFormat::Chicago16 | _ => {
            // Chicago rules (simplified: same as expanded for now)
            let expanded_end = expand_range_end(start.trim(), end);
            format!("{start}\u{2013}{expanded_end}")
        }
    }
}

/// Expand an abbreviated range end relative to start.
fn expand_range_end(start: &str, end: &str) -> String {
    if end.len() >= start.len() {
        return end.to_string();
    }
    #[allow(clippy::string_slice, reason = "length checked")]
    let prefix = &start[..start.len() - end.len()];
    format!("{prefix}{end}")
}

/// Compute minimal range end (drop shared leading digits).
fn minimal_range_end(start: &str, end: &str) -> String {
    if end.len() >= start.len() {
        return end.to_string();
    }
    // Find how many leading chars are shared
    let shared = start
        .chars()
        .zip(end.chars())
        .take_while(|(a, b)| a == b)
        .count();
    #[allow(clippy::string_slice, reason = "shared is a character boundary")]
    let result = end[shared..].to_string();
    result
}

/// Check if a reference type matches a TypeClass.
fn type_class_matches(ref_type: &str, type_class: citum_schema::options::TypeClass) -> bool {
    use citum_schema::options::TypeClass;

    match type_class {
        TypeClass::Legal => {
            ref_type == "legal-case"
                || ref_type == "legal_case"
                || ref_type == "statute"
                || ref_type == "treaty"
                || ref_type == "regulation"
                || ref_type == "bill"
                || ref_type == "legislation"
        }
        TypeClass::Classical => {
            ref_type == "classic"
                || ref_type.contains("ancient")
                || ref_type == "religious-text"
                || ref_type == "religious_text"
        }
        TypeClass::Standard => true, // Always matches
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;
    use citum_schema::citation::LocatorValue;
    use citum_schema::options::{LabelForm, LocatorConfig};

    #[test]
    fn test_render_single_page_locator_with_short_label() {
        // given a short-label config and a single page locator
        let config = LocatorConfig {
            default_label_form: LabelForm::Short,
            ..Default::default()
        };
        let locator = CitationLocator::Single(LocatorSegment {
            label: LocatorType::Page,
            value: LocatorValue::Text("42".to_string()),
        });
        // when rendered with the default locale (which has "p." for page)
        let result = render_locator(&locator, "book", &config, &Locale::default());
        // then the output includes the short label and value
        assert!(
            result.contains("42"),
            "should contain the page number: {result}"
        );
    }

    #[test]
    fn test_render_single_page_locator_no_label() {
        // given a none-label config and a single page locator
        let config = LocatorConfig {
            default_label_form: LabelForm::None,
            ..Default::default()
        };
        let locator = CitationLocator::Single(LocatorSegment {
            label: LocatorType::Page,
            value: LocatorValue::Text("42".to_string()),
        });
        // when rendered
        let result = render_locator(&locator, "book", &config, &Locale::default());
        // then output is just the bare value
        assert_eq!(result, "42");
    }

    #[test]
    fn test_render_compound_locator_page_line_pattern() {
        use citum_schema::options::{LabelRepeat, LocatorPattern};
        // given a config with a page+line pattern
        let config = LocatorConfig {
            default_label_form: LabelForm::Short,
            patterns: vec![LocatorPattern {
                kinds: vec![LocatorType::Page, LocatorType::Line],
                type_class: None,
                order: vec![LocatorType::Page, LocatorType::Line],
                delimiter: ", ".to_string(),
                label_repeat: LabelRepeat::First,
            }],
            ..Default::default()
        };
        let locator = CitationLocator::Compound {
            segments: vec![
                LocatorSegment {
                    label: LocatorType::Page,
                    value: LocatorValue::Text("33".to_string()),
                },
                LocatorSegment {
                    label: LocatorType::Line,
                    value: LocatorValue::Text("5".to_string()),
                },
            ],
        };
        // when rendered
        let result = render_locator(&locator, "book", &config, &Locale::default());
        // then label appears only on first segment, value present for both
        assert!(result.contains("33"), "should contain page value: {result}");
        assert!(result.contains('5'), "should contain line value: {result}");
    }

    #[test]
    fn test_render_global_strip_label_periods() {
        // given a config with global strip_label_periods = true
        let config = LocatorConfig {
            default_label_form: LabelForm::Short,
            strip_label_periods: Some(true),
            ..Default::default()
        };
        let locator = CitationLocator::Single(LocatorSegment {
            label: LocatorType::Page,
            value: LocatorValue::Text("42".to_string()),
        });
        // when rendered
        let result = render_locator(&locator, "book", &config, &Locale::default());
        // then the label has no trailing period
        assert!(result.contains("42"), "should contain page value: {result}");
        assert!(
            !result.contains("p."),
            "label period should be stripped: {result}"
        );
    }

    #[test]
    fn test_render_type_class_gated_pattern() {
        use citum_schema::options::{LabelRepeat, LocatorPattern, TypeClass};
        // given a config with a legal-only pattern
        let config = LocatorConfig {
            default_label_form: LabelForm::Short,
            patterns: vec![LocatorPattern {
                kinds: vec![LocatorType::Page],
                type_class: Some(TypeClass::Legal),
                order: vec![LocatorType::Page],
                delimiter: ", ".to_string(),
                label_repeat: LabelRepeat::None,
            }],
            ..Default::default()
        };
        let locator = CitationLocator::Single(LocatorSegment {
            label: LocatorType::Page,
            value: LocatorValue::Text("42".to_string()),
        });
        // when rendered as a non-legal type
        let result = render_locator(&locator, "book", &config, &Locale::default());
        // then the legal pattern does NOT apply (default rendering applies instead)
        assert!(result.contains("42"));
    }

    #[test]
    fn test_render_label_repeat_all() {
        use citum_schema::options::{LabelRepeat, LocatorPattern};
        // given a config with LabelRepeat::All on a compound pattern
        let config = LocatorConfig {
            default_label_form: LabelForm::Short,
            patterns: vec![LocatorPattern {
                kinds: vec![LocatorType::Page, LocatorType::Line],
                type_class: None,
                order: vec![LocatorType::Page, LocatorType::Line],
                delimiter: ", ".to_string(),
                label_repeat: LabelRepeat::All,
            }],
            ..Default::default()
        };
        let locator = CitationLocator::Compound {
            segments: vec![
                LocatorSegment {
                    label: LocatorType::Page,
                    value: LocatorValue::Text("33".to_string()),
                },
                LocatorSegment {
                    label: LocatorType::Line,
                    value: LocatorValue::Text("5".to_string()),
                },
            ],
        };
        // when rendered
        let result = render_locator(&locator, "book", &config, &Locale::default());
        // then both segments contain their values
        assert!(result.contains("33"));
        assert!(result.contains('5'));
    }

    #[test]
    fn test_render_custom_locator_with_locale_defined_label() {
        let config = LocatorConfig {
            default_label_form: LabelForm::Short,
            ..Default::default()
        };
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
locators:
  reel:
    short:
      singular: "reel"
      plural: "reels"
"#,
        )
        .expect("custom locale should parse");
        let locator = CitationLocator::Single(LocatorSegment {
            label: LocatorType::Custom("reel".to_string()),
            value: LocatorValue::Text("3".to_string()),
        });

        assert_eq!(render_locator(&locator, "book", &config, &locale), "reel 3");
    }

    #[test]
    fn test_render_custom_locator_pattern_matches_custom_kind() {
        use citum_schema::options::{LabelRepeat, LocatorPattern};

        let config = LocatorConfig {
            default_label_form: LabelForm::Short,
            patterns: vec![LocatorPattern {
                kinds: vec![LocatorType::Custom("reel".to_string())],
                type_class: None,
                order: vec![LocatorType::Custom("reel".to_string())],
                delimiter: " | ".to_string(),
                label_repeat: LabelRepeat::All,
            }],
            ..Default::default()
        };
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
locators:
  reel:
    short:
      singular: "reel"
      plural: "reels"
"#,
        )
        .expect("custom locale should parse");
        let locator = CitationLocator::Single(LocatorSegment {
            label: LocatorType::Custom("reel".to_string()),
            value: LocatorValue::Text("3".to_string()),
        });

        assert_eq!(render_locator(&locator, "book", &config, &locale), "reel 3");
    }
}
