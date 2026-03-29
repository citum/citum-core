//! Rendering logic for date fields with locale-aware formatting.
//!
//! This module handles date component rendering with support for different date forms,
//! time formatting, and locale-specific date presentation.

use crate::reference::{EdtfString, Reference};
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_edtf::{Day, Edtf, MonthOrSeason, Timezone, UnspecifiedYear, Year};
use citum_schema::locale::{GeneralTerm, TermForm};
use citum_schema::options::dates::TimeFormat;
use citum_schema::reference::RefDate;
use citum_schema::template::{DateForm, DateVariable as TemplateDateVar, TemplateDate};

fn month_to_string(month: u32, months: &[String]) -> String {
    if month > 0 {
        let index = month - 1;
        if index < months.len() as u32 {
            months[index as usize].clone()
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

fn extract_month(date: &EdtfString, months: &[String]) -> String {
    let parsed_date = date.parse();
    let month: Option<u32> = match parsed_date {
        RefDate::Edtf(edtf) => edtf.month(),
        RefDate::Literal(_) => None,
    };
    match month {
        Some(month) => month_to_string(month, months),
        None => String::new(),
    }
}

fn format_display_year(year: &Year, before_era: Option<&str>) -> String {
    if year.unspecified != UnspecifiedYear::None {
        return year.to_string();
    }

    if year.value <= 0 {
        let historical_year = 1 - year.value;
        if let Some(term) = before_era.filter(|term| !term.is_empty()) {
            format!("{historical_year} {term}")
        } else {
            historical_year.to_string()
        }
    } else {
        year.value.to_string()
    }
}

fn extract_display_year(date: &EdtfString, before_era: Option<&str>) -> String {
    match date.parse() {
        RefDate::Edtf(edtf) => match edtf {
            Edtf::Date(date) => format_display_year(&date.year, before_era),
            Edtf::Interval(interval) => format_display_year(&interval.start.year, before_era),
            Edtf::IntervalFrom(date) | Edtf::IntervalTo(date) => {
                format_display_year(&date.year, before_era)
            }
        },
        RefDate::Literal(_) => String::new(),
    }
}

fn extract_range_end(
    date: &EdtfString,
    months: &[String],
    before_era: Option<&str>,
) -> Option<String> {
    match date.parse() {
        RefDate::Edtf(edtf) => match edtf {
            Edtf::Interval(interval) => {
                let end = &interval.end;
                let year = format_display_year(&end.year, before_era);
                let month = match end.month_or_season {
                    Some(MonthOrSeason::Month(m)) => Some(m),
                    _ => None,
                };
                let day = match end.day {
                    Some(Day::Day(d)) => Some(d),
                    _ => None,
                };

                match (month, day) {
                    (Some(m), Some(d)) if m > 0 && d > 0 => {
                        let month_str = month_to_string(m, months);
                        Some(format!("{} {}, {}", month_str, d, year))
                    }
                    (Some(m), _) if m > 0 => {
                        let month_str = month_to_string(m, months);
                        Some(format!("{} {}", month_str, year))
                    }
                    _ => Some(year),
                }
            }
            Edtf::IntervalFrom(_date) => None, // Open-ended
            Edtf::IntervalTo(date) => {
                let year = format_display_year(&date.year, before_era);
                Some(year)
            }
            _ => None,
        },
        RefDate::Literal(_) => None,
    }
}

/// Formats a time with the specified format, optionally including seconds and timezone.
///
/// Converts 24-hour time to 12-hour format if specified, and appends localized
/// AM/PM or timezone indicators as configured.
fn format_time(
    time: citum_edtf::Time,
    format: &TimeFormat,
    show_seconds: bool,
    show_timezone: bool,
    am_term: Option<&str>,
    pm_term: Option<&str>,
    utc_term: Option<&str>,
) -> String {
    let (display_hour, period) = match format {
        TimeFormat::Hour12 => {
            let (h, p) = if time.hour == 0 {
                (12u32, am_term.unwrap_or("AM"))
            } else if time.hour < 12 {
                (time.hour, am_term.unwrap_or("AM"))
            } else if time.hour == 12 {
                (12u32, pm_term.unwrap_or("PM"))
            } else {
                (time.hour - 12, pm_term.unwrap_or("PM"))
            };
            (h, Some(p))
        }
        TimeFormat::Hour24 => (time.hour, None),
    };

    let time_str = if show_seconds {
        format!("{:02}:{:02}:{:02}", display_hour, time.minute, time.second)
    } else {
        format!("{:02}:{:02}", display_hour, time.minute)
    };

    let with_period = match period {
        Some(p) => format!("{time_str} {p}"),
        None => time_str,
    };

    if show_timezone {
        let tz_str = match time.timezone {
            Some(Timezone::Utc) => utc_term.unwrap_or("UTC").to_string(),
            Some(Timezone::Offset(mins)) => {
                let sign = if mins >= 0 { '+' } else { '-' };
                let abs = mins.unsigned_abs();
                format!("{}{:02}:{:02}", sign, abs / 60, abs % 60)
            }
            None => String::new(),
        };
        if tz_str.is_empty() {
            with_period
        } else {
            format!("{with_period} {tz_str}")
        }
    } else {
        with_period
    }
}

/// Format the start portion of a date range according to the given form.
fn format_range_start(
    date: &EdtfString,
    form: &DateForm,
    locale: &citum_schema::locale::Locale,
) -> String {
    let before_era = locale.dates.before_era.as_deref();
    match form {
        DateForm::Year => extract_display_year(date, before_era),
        DateForm::YearMonth => {
            let month = extract_month(date, &locale.dates.months.long);
            let year = extract_display_year(date, before_era);
            if month.is_empty() {
                year
            } else {
                format!("{month} {year}")
            }
        }
        DateForm::MonthDay => {
            let month = extract_month(date, &locale.dates.months.long);
            let day = date.day();
            match day {
                Some(d) => format!("{month} {d}"),
                None => month,
            }
        }
        DateForm::Full => {
            let year = extract_display_year(date, before_era);
            let month = extract_month(date, &locale.dates.months.long);
            let day = date.day();
            match (month.is_empty(), day) {
                (true, _) => year,
                (false, None) => format!("{month} {year}"),
                (false, Some(d)) => format!("{month} {d}, {year}"),
            }
        }
        DateForm::YearMonthDay => {
            let year = extract_display_year(date, before_era);
            let month = extract_month(date, &locale.dates.months.long);
            let day = date.day();
            match (month.is_empty(), day) {
                (true, _) => year,
                (false, None) => format!("{year}, {month}"),
                (false, Some(d)) => format!("{year}, {month} {d}"),
            }
        }
        DateForm::DayMonthAbbrYear => {
            let year = extract_display_year(date, before_era);
            let month = extract_month(date, &locale.dates.months.short);
            let day = date.day();
            match (month.is_empty(), day) {
                (true, _) => year,
                (false, None) => format!("{month} {year}"),
                (false, Some(d)) => format!("{d} {month} {year}"),
            }
        }
    }
}

/// Format a date range with start date and delimiter.
fn format_date_range(
    start: String,
    date: &EdtfString,
    locale: &citum_schema::locale::Locale,
    date_config: Option<&citum_schema::options::dates::DateConfig>,
) -> Option<String> {
    if date.is_open_range() {
        // Open-ended range (e.g., "1990/..")
        if let Some(end_marker) = date_config
            .and_then(|c| c.open_range_marker.as_deref())
            .or(locale.dates.open_ended_term.as_deref())
        {
            // U+2013 en-dash is the Unicode standard range delimiter (not language-specific)
            let delimiter = date_config.map_or("–", |c| c.range_delimiter.as_str());
            Some(format!("{start}{delimiter}{end_marker}"))
        } else {
            // No open-ended term available - return start date only
            Some(start)
        }
    } else if let Some(end) = extract_range_end(
        date,
        &locale.dates.months.long,
        locale.dates.before_era.as_deref(),
    ) {
        // Closed range with end date
        // U+2013 en-dash is the Unicode standard range delimiter (not language-specific)
        let delimiter = date_config.map_or("–", |c| c.range_delimiter.as_str());
        Some(format!("{start}{delimiter}{end}"))
    } else {
        Some(start)
    }
}

/// Apply uncertainty and approximation markers to formatted date.
fn apply_date_markers(
    value: String,
    date: &EdtfString,
    date_config: Option<&citum_schema::options::dates::DateConfig>,
) -> String {
    let mut result = value;
    if date.is_approximate()
        && let Some(marker) = date_config.and_then(|c| c.approximation_marker.as_ref())
    {
        result = format!("{marker}{result}");
    }
    if date.is_uncertain()
        && let Some(marker) = date_config.and_then(|c| c.uncertainty_marker.as_ref())
    {
        result = format!("{result}{marker}");
    }
    result
}

/// Compute the disambiguation suffix for year-based citations.
fn compute_disamb_suffix<F: crate::render::format::OutputFormat<Output = String>>(
    formatted: &Option<String>,
    hints: &ProcHints,
    options: &RenderOptions<'_>,
    fmt: &F,
) -> Option<String> {
    if hints.disamb_condition && formatted.as_ref().is_some_and(|s| s.len() == 4) {
        // Check if year suffix is enabled. Fall back to AuthorDate default
        // (year_suffix: true) when processing is not explicitly set, matching
        // the behavior in disambiguation.rs which uses unwrap_or_default().
        let use_suffix = options
            .config
            .processing
            .as_ref()
            .unwrap_or(&citum_schema::options::Processing::AuthorDate)
            .config()
            .disambiguate
            .as_ref()
            .is_some_and(|d| d.year_suffix);

        if use_suffix {
            int_to_letter(hints.group_index as u32).map(|s| fmt.text(&s))
        } else {
            None
        }
    } else {
        None
    }
}

/// Format a single date (non-range) according to the given form.
fn format_single_date(
    date: &EdtfString,
    form: &DateForm,
    locale: &citum_schema::locale::Locale,
    date_config: Option<&citum_schema::options::dates::DateConfig>,
) -> Option<String> {
    let before_era = locale.dates.before_era.as_deref();
    match form {
        DateForm::Year => {
            let year = extract_display_year(date, before_era);
            if year.is_empty() { None } else { Some(year) }
        }
        DateForm::YearMonth => {
            let year = extract_display_year(date, before_era);
            if year.is_empty() {
                return None;
            }
            let month = extract_month(date, &locale.dates.months.long);
            if month.is_empty() {
                Some(year)
            } else {
                Some(format!("{month} {year}"))
            }
        }
        DateForm::MonthDay => {
            let month = extract_month(date, &locale.dates.months.long);
            if month.is_empty() {
                return None;
            }
            let day = date.day();
            match day {
                Some(d) => Some(format!("{month} {d}")),
                None => Some(month),
            }
        }
        DateForm::Full => {
            let year = extract_display_year(date, before_era);
            if year.is_empty() {
                return None;
            }
            let month = extract_month(date, &locale.dates.months.long);
            let day = date.day();
            let base = match (month.is_empty(), day) {
                (true, _) => year,
                (false, None) => format!("{month} {year}"),
                (false, Some(d)) => format!("{month} {d}, {year}"),
            };
            // Append time component if configured and present
            if let (Some(time_fmt), Some(time)) = (
                date_config.and_then(|c| c.time_format.as_ref()),
                date.time(),
            ) {
                let show_secs = date_config.is_some_and(|c| c.show_seconds);
                let show_tz = date_config.is_some_and(|c| c.show_timezone);
                let time_str = format_time(
                    time,
                    time_fmt,
                    show_secs,
                    show_tz,
                    locale.dates.am.as_deref(),
                    locale.dates.pm.as_deref(),
                    locale.dates.timezone_utc.as_deref(),
                );
                Some(format!("{base}, {time_str}"))
            } else {
                Some(base)
            }
        }
        DateForm::YearMonthDay => {
            let year = extract_display_year(date, before_era);
            if year.is_empty() {
                return None;
            }
            let month = extract_month(date, &locale.dates.months.long);
            let day = date.day();
            match (month.is_empty(), day) {
                (true, _) => Some(year),
                (false, None) => Some(format!("{year}, {month}")),
                (false, Some(d)) => Some(format!("{year}, {month} {d}")),
            }
        }
        DateForm::DayMonthAbbrYear => {
            let year = extract_display_year(date, before_era);
            if year.is_empty() {
                return None;
            }
            let month = extract_month(date, &locale.dates.months.short);
            let day = date.day();
            match (month.is_empty(), day) {
                (true, _) => Some(year),
                (false, None) => Some(format!("{month} {year}")),
                (false, Some(d)) => Some(format!("{d} {month} {year}")),
            }
        }
    }
}

impl ComponentValues for TemplateDate {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        let fmt = F::default();
        let date_opt: Option<EdtfString> = match self.date {
            TemplateDateVar::Issued => reference.issued(),
            TemplateDateVar::Accessed => reference.accessed(),
            _ => None,
        };

        if date_opt.is_none() || date_opt.as_ref().unwrap().0.is_empty() {
            // Handle fallback if date is missing
            if let Some(fallbacks) = &self.fallback {
                for component in fallbacks {
                    if let Some(values) = component.values::<F>(reference, hints, options) {
                        return Some(values);
                    }
                }
            }
            // For issued dates, substitute the locale's "no-date" term (e.g. "n.d.")
            if matches!(self.date, TemplateDateVar::Issued)
                && let Some(nd) = options
                    .locale
                    .resolved_general_term(&GeneralTerm::NoDate, TermForm::Short)
            {
                return Some(ProcValues {
                    value: nd,
                    prefix: None,
                    suffix: None,
                    url: None,
                    substituted_key: None,
                    pre_formatted: false,
                });
            }
            return None;
        }

        let date = date_opt.unwrap();
        let locale = options.locale;
        let date_config = options.config.dates.as_ref();
        let effective_form = if options.context == crate::values::RenderContext::Citation
            && reference.ref_type() == "personal-communication"
            && matches!(self.date, TemplateDateVar::Issued)
        {
            DateForm::Full
        } else {
            self.form.clone()
        };

        let formatted = if date.is_range() {
            // Handle date ranges
            let start = format_range_start(&date, &effective_form, locale);
            format_date_range(start, &date, locale, date_config)
        } else {
            // Single date (not a range)
            format_single_date(&date, &effective_form, locale, date_config)
        };

        // Apply uncertainty and approximation markers
        let formatted = formatted.map(|value| apply_date_markers(value, &date, date_config));

        // Handle disambiguation suffix (a, b, c...)
        let suffix = compute_disamb_suffix(&formatted, hints, options, &fmt);

        formatted.map(|value| ProcValues {
            value,
            prefix: None,
            suffix,
            url: crate::values::resolve_effective_url(
                self.links.as_ref(),
                options.config.links.as_ref(),
                reference,
                citum_schema::options::LinkAnchor::Component,
            ),
            substituted_key: None,
            pre_formatted: false,
        })
    }
}

/// Convert a 1-based index into an alphabetic suffix (`1 -> "a"`, `27 -> "aa"`).
#[must_use]
pub fn int_to_letter(n: u32) -> Option<String> {
    if n == 0 {
        return None;
    }

    let mut result = String::new();
    let mut num = n - 1;

    loop {
        result.push((b'a' + (num % 26) as u8) as char);
        if num < 26 {
            break;
        }
        num = num / 26 - 1;
    }

    Some(result.chars().rev().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_to_letter() {
        // Test basic single-letter conversions (1-26)
        assert_eq!(int_to_letter(1), Some("a".to_string()));
        assert_eq!(int_to_letter(2), Some("b".to_string()));
        assert_eq!(int_to_letter(26), Some("z".to_string()));

        // Test double-letter conversions (27+)
        assert_eq!(int_to_letter(27), Some("aa".to_string()));
        assert_eq!(int_to_letter(52), Some("az".to_string()));
        assert_eq!(int_to_letter(53), Some("ba".to_string()));

        // Test zero returns None
        assert_eq!(int_to_letter(0), None);
    }
}

#[cfg(test)]
mod time_tests {
    use super::*;
    use citum_edtf::{Time, Timezone};

    #[test]
    fn test_format_time_12h_utc() {
        let time = Time {
            hour: 23,
            minute: 20,
            second: 30,
            timezone: Some(Timezone::Utc),
        };
        let result = format_time(
            time,
            &TimeFormat::Hour12,
            false,
            true,
            Some("AM"),
            Some("PM"),
            Some("UTC"),
        );
        assert_eq!(result, "11:20 PM UTC");
    }

    #[test]
    fn test_format_time_24h_utc() {
        let time = Time {
            hour: 23,
            minute: 20,
            second: 30,
            timezone: Some(Timezone::Utc),
        };
        let result = format_time(
            time,
            &TimeFormat::Hour24,
            false,
            true,
            None,
            None,
            Some("UTC"),
        );
        assert_eq!(result, "23:20 UTC");
    }

    #[test]
    fn test_format_time_with_offset() {
        let time = Time {
            hour: 10,
            minute: 10,
            second: 10,
            timezone: Some(Timezone::Offset(330)),
        };
        let result = format_time(
            time,
            &TimeFormat::Hour24,
            false,
            true,
            None,
            None,
            Some("UTC"),
        );
        assert_eq!(result, "10:10 +05:30");
    }

    #[test]
    fn test_format_time_no_timezone() {
        let time = Time {
            hour: 14,
            minute: 30,
            second: 0,
            timezone: None,
        };
        let result = format_time(time, &TimeFormat::Hour24, false, false, None, None, None);
        assert_eq!(result, "14:30");
    }
}
