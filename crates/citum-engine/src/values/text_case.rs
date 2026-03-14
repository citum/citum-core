/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Title text-case transforms.
//!
//! Implements structured-title-aware casing for bibliography output.
//! All transforms operate on Djot-markup-bearing strings and respect
//! `.nocase` span protection via the rich-text renderer.

use citum_schema::options::titles::TextCase;

/// Apply a text-case transform to a single plain-text segment.
///
/// This function handles the core casing logic for a single string.
/// For structured titles with subtitles, use [`apply_to_structured_parts`].
///
/// `.nocase`-protected spans are handled at the Djot rendering layer,
/// not here — this function operates on already-resolved text segments.
pub fn apply_text_case(text: &str, case: TextCase) -> String {
    match case {
        TextCase::AsIs => text.to_string(),
        TextCase::Lowercase => text.to_lowercase(),
        TextCase::Uppercase => text.to_uppercase(),
        TextCase::CapitalizeFirst => capitalize_first_word(text),
        TextCase::Sentence | TextCase::SentenceApa | TextCase::SentenceNlm => {
            to_sentence_case(text)
        }
        TextCase::Title => to_title_case(text),
    }
}

/// Apply text-case to a structured title (main + subtitles).
///
/// The key difference between sentence-case variants:
/// - `SentenceApa`: capitalize first word of main title AND each subtitle
/// - `SentenceNlm`: capitalize first word of main title only
/// - Other variants: applied uniformly to each part
pub fn apply_to_structured_parts(
    main: &str,
    subtitles: &[&str],
    case: TextCase,
) -> (String, Vec<String>) {
    match case {
        TextCase::SentenceApa => {
            let main_cased = to_sentence_case(main);
            let subs_cased = subtitles.iter().map(|s| to_sentence_case(s)).collect();
            (main_cased, subs_cased)
        }
        TextCase::SentenceNlm => {
            let main_cased = to_sentence_case(main);
            // NLM: subtitles keep only explicit/protected capitals (lowercase the rest)
            let subs_cased = subtitles.iter().map(|s| s.to_lowercase()).collect();
            (main_cased, subs_cased)
        }
        _ => {
            let main_cased = apply_text_case(main, case);
            let subs_cased = subtitles.iter().map(|s| apply_text_case(s, case)).collect();
            (main_cased, subs_cased)
        }
    }
}

/// Returns true if the given language tag indicates English.
pub fn is_english_language(lang: Option<&str>) -> bool {
    match lang {
        Some(tag) => {
            let primary = tag.split('-').next().unwrap_or(tag);
            primary.eq_ignore_ascii_case("en")
        }
        // Default: assume English for backward compatibility
        None => true,
    }
}

/// Resolve the effective text-case, applying language fallback.
///
/// For non-English languages without defined transforms, returns `AsIs`.
pub fn resolve_text_case(case: TextCase, language: Option<&str>) -> TextCase {
    if !is_english_language(language) {
        // Non-English: only explicit as-is, lowercase, uppercase pass through.
        // All English-specific transforms fall back to as-is.
        match case {
            TextCase::AsIs | TextCase::Lowercase | TextCase::Uppercase => case,
            _ => TextCase::AsIs,
        }
    } else {
        case
    }
}

/// Convert text to sentence case: lowercase everything, then capitalize the first word.
fn to_sentence_case(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let lowered = text.to_lowercase();
    capitalize_first_word(&lowered)
}

/// Capitalize the first alphabetic character of the string,
/// preserving leading whitespace and punctuation.
pub(crate) fn capitalize_first_word(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut found_first = false;
    for ch in text.chars() {
        if !found_first && ch.is_alphabetic() {
            for upper in ch.to_uppercase() {
                result.push(upper);
            }
            found_first = true;
        } else {
            result.push(ch);
        }
    }
    result
}

// English title-case stop words (articles, short conjunctions, short prepositions).
const TITLE_CASE_STOP_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "but", "by", "for", "in", "nor", "of", "on", "or", "so", "the",
    "to", "up", "yet", "v", "vs",
];

/// Convert text to English headline-style title case.
///
/// Capitalizes the first and last word unconditionally.
/// Interior stop words (articles, short prepositions, conjunctions) stay lowercase.
fn to_title_case(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return text.to_string();
    }

    let last_idx = words.len() - 1;
    let mut parts: Vec<String> = Vec::with_capacity(words.len());

    for (i, word) in words.iter().enumerate() {
        if i == 0 || i == last_idx {
            parts.push(capitalize_first_word(&word.to_lowercase()));
        } else {
            let lower = word.to_lowercase();
            if TITLE_CASE_STOP_WORDS.contains(&lower.as_str()) {
                parts.push(lower);
            } else {
                parts.push(capitalize_first_word(&lower));
            }
        }
    }

    // Rebuild with original whitespace structure
    let mut result = String::with_capacity(text.len());
    let mut word_iter = parts.iter();
    let mut in_word = false;
    let mut current_word = word_iter.next();

    for ch in text.chars() {
        if ch.is_whitespace() {
            if in_word {
                in_word = false;
                current_word = word_iter.next();
            }
            result.push(ch);
        } else if !in_word && let Some(word) = current_word {
            result.push_str(word);
            in_word = true;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- capitalize_first_word ---

    #[test]
    fn test_capitalize_first_word_basic() {
        assert_eq!(capitalize_first_word("hello world"), "Hello world");
    }

    #[test]
    fn test_capitalize_first_word_leading_space() {
        assert_eq!(capitalize_first_word("  hello"), "  Hello");
    }

    #[test]
    fn test_capitalize_first_word_empty() {
        assert_eq!(capitalize_first_word(""), "");
    }

    #[test]
    fn test_capitalize_first_word_already_upper() {
        assert_eq!(capitalize_first_word("Hello"), "Hello");
    }

    // --- to_sentence_case ---

    #[test]
    fn test_sentence_case_basic() {
        assert_eq!(
            to_sentence_case("The Quick Brown Fox"),
            "The quick brown fox"
        );
    }

    #[test]
    fn test_sentence_case_all_caps() {
        assert_eq!(to_sentence_case("DNA REPLICATION"), "Dna replication");
    }

    #[test]
    fn test_sentence_case_empty() {
        assert_eq!(to_sentence_case(""), "");
    }

    // --- to_title_case ---

    #[test]
    fn test_title_case_basic() {
        assert_eq!(to_title_case("the quick brown fox"), "The Quick Brown Fox");
    }

    #[test]
    fn test_title_case_stop_words() {
        assert_eq!(
            to_title_case("a tale of two cities"),
            "A Tale of Two Cities"
        );
    }

    #[test]
    fn test_title_case_last_word_capitalized() {
        assert_eq!(
            to_title_case("the world we live in"),
            "The World We Live In"
        );
    }

    // --- apply_to_structured_parts ---

    #[test]
    fn test_sentence_apa_structured() {
        let (main, subs) = apply_to_structured_parts(
            "Understanding Citation Systems",
            &["History and Practice", "A Comparative View"],
            TextCase::SentenceApa,
        );
        assert_eq!(main, "Understanding citation systems");
        assert_eq!(subs, vec!["History and practice", "A comparative view"]);
    }

    #[test]
    fn test_sentence_nlm_structured() {
        let (main, subs) = apply_to_structured_parts(
            "Understanding Citation Systems",
            &["History and Practice"],
            TextCase::SentenceNlm,
        );
        assert_eq!(main, "Understanding citation systems");
        // NLM: subtitles lowercased (no first-word capitalization)
        assert_eq!(subs, vec!["history and practice"]);
    }

    #[test]
    fn test_title_case_structured() {
        let (main, subs) =
            apply_to_structured_parts("the dna of empire", &["a new perspective"], TextCase::Title);
        assert_eq!(main, "The Dna of Empire");
        assert_eq!(subs, vec!["A New Perspective"]);
    }

    // --- resolve_text_case ---

    #[test]
    fn test_english_language_detection() {
        assert!(is_english_language(Some("en")));
        assert!(is_english_language(Some("en-US")));
        assert!(is_english_language(Some("en-GB")));
        assert!(is_english_language(None));
        assert!(!is_english_language(Some("de")));
        assert!(!is_english_language(Some("fr-FR")));
    }

    #[test]
    fn test_resolve_non_english_falls_back() {
        assert_eq!(
            resolve_text_case(TextCase::SentenceApa, Some("de")),
            TextCase::AsIs
        );
        assert_eq!(
            resolve_text_case(TextCase::Title, Some("fr")),
            TextCase::AsIs
        );
        // Explicit lowercase/uppercase pass through for any language
        assert_eq!(
            resolve_text_case(TextCase::Lowercase, Some("de")),
            TextCase::Lowercase
        );
    }

    #[test]
    fn test_resolve_english_passes_through() {
        assert_eq!(
            resolve_text_case(TextCase::SentenceApa, Some("en")),
            TextCase::SentenceApa
        );
        assert_eq!(
            resolve_text_case(TextCase::Title, Some("en-US")),
            TextCase::Title
        );
    }
}
