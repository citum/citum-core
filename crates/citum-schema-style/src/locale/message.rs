/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Message evaluation for parameterized locale strings.
//!
//! This module provides the `MessageEvaluator` trait for evaluating
//! ICU MessageFormat messages at runtime. The trait acts as a seam
//! for future ICU4X migration, allowing different evaluator implementations
//! to be swapped without changing call sites.

/// Arguments passed to message evaluation.
///
/// Contains optional named variables that may be referenced in a message.
/// The engine populates these based on rendering context.
#[derive(Debug, Clone, Default)]
pub struct MessageArgs<'a> {
    /// Numeric count for plural dispatch.
    pub count: Option<u64>,
    /// Named string variable (e.g., a name list or URL).
    pub value: Option<&'a str>,
    /// Gender for select dispatch (MF2 `select` with gender keys).
    pub gender: Option<&'a str>,
    /// Pre-formatted name list string.
    pub names: Option<&'a str>,
    /// Start of a range (e.g., page range start).
    pub start: Option<&'a str>,
    /// End of a range (e.g., page range end).
    pub end: Option<&'a str>,
    /// URL string.
    pub url: Option<&'a str>,
    /// Pre-formatted date string.
    pub date: Option<&'a str>,
    /// Main contributor list for "et al." patterns.
    pub main_list: Option<&'a str>,
}

/// Evaluates a parameterized message string with runtime arguments.
///
/// # ICU4X swap path
///
/// This trait is the seam for future ICU4X migration. When
/// `icu_message_format` (ICU4X) reaches stable, replace `Mf2MessageEvaluator`
/// with an ICU4X-backed struct implementing this same trait.
/// See bean `csl26-qrpo` and <https://github.com/unicode-org/icu4x/issues/3028>.
///
/// # Implementation notes
///
/// Implementations are expected to be cheap to clone or wrap in `Arc<T>`
/// for concurrent rendering contexts.
pub trait MessageEvaluator: Send + Sync {
    /// Evaluate a message with the provided arguments.
    ///
    /// Returns `Some(result)` on successful evaluation, or `None` if:
    /// - The message body is unparseable as MF2
    /// - A required variable is missing from `args`
    /// - An unrecoverable error occurs
    ///
    /// The caller (engine) provides fallback behavior (e.g., returning a
    /// legacy term or a bare message ID) on `None`.
    fn evaluate(&self, message: &str, args: &MessageArgs<'_>) -> Option<String>;
}

/// MF2 message evaluator for ICU MessageFormat 2 syntax.
///
/// Evaluates MF2 messages with `.match` statements and variable substitution
/// without external dependencies.
#[derive(Debug, Clone)]
pub struct Mf2MessageEvaluator;

impl MessageEvaluator for Mf2MessageEvaluator {
    fn evaluate(&self, message: &str, args: &MessageArgs<'_>) -> Option<String> {
        let trimmed = message.trim();

        if trimmed.starts_with(".match") {
            evaluate_mf2_matcher(trimmed, args)
        } else {
            substitute_mf2_vars(trimmed, args)
        }
    }
}

/// Substitute `{$var}` references in a simple MF2 pattern.
///
/// Variable names resolve directly from `MessageArgs` fields.
/// Returns `None` if a referenced variable is missing.
fn substitute_mf2_vars(pattern: &str, args: &MessageArgs<'_>) -> Option<String> {
    if !pattern.contains('{') {
        return Some(pattern.to_string());
    }

    let mut result = String::new();
    let mut cursor = 0usize;

    while let Some(offset) = pattern[cursor..].find('{') {
        let open = cursor + offset;
        result.push_str(&pattern[cursor..open]);

        let close = find_matching_brace(pattern, open)?;
        let inner = pattern.get(open + 1..close)?.trim();

        if !inner.starts_with('$') {
            return None;
        }

        let var_name = &inner[1..];
        let var_value = resolve_var(var_name, args)?;
        result.push_str(var_value);
        cursor = close + 1;
    }

    result.push_str(&pattern[cursor..]);
    Some(result)
}

/// Resolve a variable name to its value in `MessageArgs`.
fn resolve_var<'a>(var_name: &str, args: &'a MessageArgs<'a>) -> Option<&'a str> {
    match var_name {
        "value" => args.value,
        "gender" => args.gender,
        "names" => args.names,
        "start" => args.start,
        "end" => args.end,
        "url" => args.url,
        "date" => args.date,
        "main_list" => args.main_list,
        _ => None,
    }
}

/// Evaluate a `.match` statement with selector and variants.
fn evaluate_mf2_matcher(message: &str, args: &MessageArgs<'_>) -> Option<String> {
    let trimmed = message.trim();
    if !trimmed.starts_with(".match") {
        return None;
    }

    let after_match = trimmed[".match".len()..].trim_start();
    let open_brace = after_match.find('{')?;
    let close_brace = find_matching_brace(after_match, open_brace)?;
    let selector_text = after_match.get(open_brace + 1..close_brace)?.trim();

    let variants_start = after_match.get(close_brace + 1..)?.trim_start();

    let (var_name, function) = parse_mf2_selector(selector_text)?;
    let match_key = determine_match_key(var_name, function, args)?;
    let matched_pattern = find_mf2_variant(variants_start, &match_key)?;

    substitute_mf2_vars(&matched_pattern, args)
}

/// Parse an MF2 selector like `$count :plural` or `$gender :select`.
///
/// Returns `(variable_name, optional_function)`.
fn parse_mf2_selector(selector: &str) -> Option<(&str, Option<&str>)> {
    let parts: Vec<&str> = selector.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let var_name = parts[0].strip_prefix('$')?;

    let function = parts
        .get(1)
        .and_then(|func_part| func_part.strip_prefix(':'));

    Some((var_name, function))
}

/// Determine the match key based on variable value and function.
fn determine_match_key(
    var_name: &str,
    function: Option<&str>,
    args: &MessageArgs<'_>,
) -> Option<String> {
    match function {
        Some("plural") => {
            // :plural dispatch is only valid for $count
            if var_name != "count" {
                return None;
            }
            let count = args.count?;
            if count == 1 {
                Some("one".to_string())
            } else {
                Some("*".to_string())
            }
        }
        Some("select") | None => {
            let value = match var_name {
                "count" => args.count.map(|c| c.to_string()),
                "value" => args.value.map(|s| s.to_string()),
                "gender" => args.gender.map(|s| s.to_string()),
                "names" => args.names.map(|s| s.to_string()),
                "start" => args.start.map(|s| s.to_string()),
                "end" => args.end.map(|s| s.to_string()),
                "url" => args.url.map(|s| s.to_string()),
                "date" => args.date.map(|s| s.to_string()),
                "main_list" => args.main_list.map(|s| s.to_string()),
                _ => None,
            }?;
            Some(value)
        }
        _ => None,
    }
}

/// Find the matched variant in MF2 when-blocks and return its pattern.
///
/// Scans for `when <key> { pattern }` lines. Returns the first exact match,
/// or falls back to `when * { pattern }`.
fn find_mf2_variant(variants_text: &str, match_key: &str) -> Option<String> {
    let mut wildcard_pattern: Option<String> = None;
    let mut rest = variants_text;

    loop {
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            break;
        }

        if !trimmed.starts_with("when") {
            break;
        }

        let after_when = trimmed["when".len()..].trim_start();
        let brace_pos = after_when.find('{')?;
        let key_str = after_when[..brace_pos].trim();

        let open_brace_index = rest.len() - after_when.len() + brace_pos;
        let close_brace_index = find_matching_brace(rest, open_brace_index)?;
        let pattern = rest
            .get(open_brace_index + 1..close_brace_index)?
            .to_string();

        if key_str == match_key {
            return Some(pattern);
        } else if key_str == "*" && wildcard_pattern.is_none() {
            wildcard_pattern = Some(pattern);
        }

        rest = rest.get(close_brace_index + 1..)?;
    }

    wildcard_pattern
}

/// Find the matching closing brace for an opening brace at a given index.
fn find_matching_brace(input: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;

    for (index, ch) in input
        .char_indices()
        .skip_while(|(index, _)| *index < open_index)
    {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_message() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs::default();
        let result = evaluator.evaluate("and", &args);
        assert_eq!(result, Some("and".to_string()));
    }

    #[test]
    fn test_simple_variable() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs {
            value: Some("Smith"),
            ..Default::default()
        };
        let result = evaluator.evaluate("retrieved from {$url}", &args);
        // Should return None because url is not set
        assert_eq!(result, None);

        let args = MessageArgs {
            url: Some("https://example.com"),
            ..Default::default()
        };
        let result = evaluator.evaluate("retrieved from {$url}", &args);
        assert_eq!(
            result,
            Some("retrieved from https://example.com".to_string())
        );
    }

    #[test]
    fn test_plural_one() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs {
            count: Some(1),
            ..Default::default()
        };
        let message = ".match {$count :plural}\nwhen one {p.}\nwhen * {pp.}";
        assert_eq!(evaluator.evaluate(message, &args), Some("p.".to_string()));
    }

    #[test]
    fn test_plural_other() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs {
            count: Some(5),
            ..Default::default()
        };
        let message = ".match {$count :plural}\nwhen one {p.}\nwhen * {pp.}";
        assert_eq!(evaluator.evaluate(message, &args), Some("pp.".to_string()));
    }

    #[test]
    fn test_select() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs {
            gender: Some("masc"),
            ..Default::default()
        };
        let message = ".match {$gender :select}\nwhen masc {él}\nwhen fem {ella}\nwhen * {elle}";
        assert_eq!(evaluator.evaluate(message, &args), Some("él".to_string()));
    }

    #[test]
    fn test_select_fallback_wildcard() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs {
            gender: Some("neuter"),
            ..Default::default()
        };
        let message = ".match {$gender :select}\nwhen masc {él}\nwhen fem {ella}\nwhen * {elle}";
        assert_eq!(evaluator.evaluate(message, &args), Some("elle".to_string()));
    }

    #[test]
    fn test_mixed_text_and_variable() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs {
            url: Some("https://example.com"),
            ..Default::default()
        };
        let result = evaluator.evaluate("retrieved from {$url}", &args);
        assert_eq!(
            result,
            Some("retrieved from https://example.com".to_string())
        );
    }

    #[test]
    fn test_missing_variable_plural() {
        let evaluator = Mf2MessageEvaluator;
        let args = MessageArgs::default();
        let message = ".match {$count :plural}\nwhen one {p.}\nwhen * {pp.}";
        let result = evaluator.evaluate(message, &args);
        // Should return None when count is missing
        assert_eq!(result, None);
    }
}
