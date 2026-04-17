/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Title text-case transforms.
//!
//! Implements structured-title-aware casing for bibliography output.
//! All transforms operate on Djot-markup-bearing strings and respect
//! `.nocase` span protection via the rich-text renderer.

use citum_schema::NoteStartTextCase;
use citum_schema::options::titles::TextCase;

/// Apply a text-case transform to a single plain-text segment.
///
/// This function handles the core casing logic for a single string.
/// For structured titles with subtitles, use [`apply_to_structured_parts`].
///
/// `.nocase`-protected spans are handled at the Djot rendering layer,
/// not here — this function operates on already-resolved text segments.
#[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
pub fn resolve_text_case(case: TextCase, language: Option<&str>) -> TextCase {
    if is_english_language(language) {
        case
    } else {
        // Non-English: only explicit as-is, lowercase, uppercase pass through.
        // All English-specific transforms fall back to as-is.
        match case {
            TextCase::AsIs | TextCase::Lowercase | TextCase::Uppercase => case,
            _ => TextCase::AsIs,
        }
    }
}

/// Apply a note-start text-case transform using the same language fallback rules
/// as other locale-backed casing behavior.
#[must_use]
pub(crate) fn apply_note_start_text_case(
    value: &str,
    text_case: NoteStartTextCase,
    language: Option<&str>,
) -> String {
    let case = match text_case {
        NoteStartTextCase::CapitalizeFirst => TextCase::CapitalizeFirst,
        NoteStartTextCase::Lowercase => TextCase::Lowercase,
    };
    apply_text_case(value, resolve_text_case(case, language))
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
    "a", "an", "and", "as", "at", "but", "by", "for", "from", "in", "nor", "of", "on", "or", "so",
    "the", "to", "up", "yet", "v", "vs",
];

/// Capitalize each component of a hyphenated compound word for title case.
///
/// When `force_all` is true (first/last word, post-punctuation), every component
/// is capitalized. Otherwise interior stop-word components stay lowercase.
fn capitalize_hyphenated(word: &str, force_all: bool) -> String {
    word.split('-')
        .map(|part| {
            if force_all {
                capitalize_first_word(part)
            } else {
                let alpha_core = part.trim_matches(|c: char| !c.is_alphanumeric());
                if TITLE_CASE_STOP_WORDS.contains(&alpha_core) {
                    part.to_string()
                } else {
                    capitalize_first_word(part)
                }
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn trim_trailing_closing_punctuation(word: &str) -> &str {
    word.trim_end_matches(['"', '\'', ')', ']', '}', '»', '”', '’'])
}

/// Convert text to English headline-style title case.
///
/// Capitalizes the first and last word unconditionally.
/// Interior stop words (articles, short prepositions, conjunctions) stay lowercase.
/// The first word after `:`, `?`, or `!` is always capitalized.
/// Hyphenated compounds capitalize each non-stop-word component.
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
    let mut capitalize_next = false;

    for (i, word) in words.iter().enumerate() {
        let lower = word.to_lowercase();
        if i == 0 || i == last_idx || capitalize_next {
            if lower.contains('-') {
                parts.push(capitalize_hyphenated(&lower, true));
            } else {
                parts.push(capitalize_first_word(&lower));
            }
        } else {
            // Strip leading/trailing punctuation when checking stop words so that
            // words like "(and" or "and)" are still treated as the stop word "and".
            let alpha_core = lower.trim_matches(|c: char| !c.is_alphanumeric());
            if TITLE_CASE_STOP_WORDS.contains(&alpha_core) {
                parts.push(lower);
            } else if lower.contains('-') {
                parts.push(capitalize_hyphenated(&lower, false));
            } else {
                parts.push(capitalize_first_word(&lower));
            }
        }
        // Capitalize the next word after sentence-ending punctuation or a colon,
        // even when that punctuation is followed by a closing quote or bracket.
        let punctuation_core = trim_trailing_closing_punctuation(word);
        capitalize_next = punctuation_core.ends_with(':')
            || punctuation_core.ends_with('?')
            || punctuation_core.ends_with('!');
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

    #[test]
    fn test_title_case_after_colon() {
        assert_eq!(
            to_title_case("the title: a subtitle"),
            "The Title: A Subtitle"
        );
    }

    #[test]
    fn test_title_case_after_colon_stop_word() {
        // First word after colon is a stop word but must still be capitalized
        assert_eq!(
            to_title_case("history of the world: a new perspective"),
            "History of the World: A New Perspective"
        );
    }

    #[test]
    fn test_title_case_after_question_mark() {
        assert_eq!(
            to_title_case("who's black and why? a hidden chapter"),
            "Who's Black and Why? A Hidden Chapter"
        );
    }

    #[test]
    fn test_title_case_after_question_mark_with_closing_quote() {
        assert_eq!(
            to_title_case("who's black and why?\" a hidden chapter"),
            "Who's Black and Why?\" A Hidden Chapter"
        );
    }

    #[test]
    fn test_title_case_from_is_stop_word() {
        assert_eq!(
            to_title_case("a hidden chapter from the eighteenth-century invention of race"),
            "A Hidden Chapter from the Eighteenth-Century Invention of Race"
        );
    }

    #[test]
    fn test_title_case_hyphenated_compound() {
        assert_eq!(
            to_title_case("eighteenth-century studies"),
            "Eighteenth-Century Studies"
        );
    }

    #[test]
    fn test_title_case_hyphenated_stop_word_part() {
        // "well-to-do": "to" is a stop word → stays lowercase in interior position
        assert_eq!(to_title_case("a well-to-do family"), "A Well-to-Do Family");
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

    #[test]
    fn test_note_start_capitalize_first_uses_english_language_rules() {
        assert_eq!(
            apply_note_start_text_case(
                "edited by",
                NoteStartTextCase::CapitalizeFirst,
                Some("en-US"),
            ),
            "Edited by"
        );
    }

    #[test]
    fn test_note_start_capitalize_first_falls_back_to_as_is_for_non_english() {
        assert_eq!(
            apply_note_start_text_case(
                "hg. von",
                NoteStartTextCase::CapitalizeFirst,
                Some("de-DE"),
            ),
            "hg. von"
        );
    }

    #[test]
    fn test_note_start_capitalize_first_is_no_op_for_uncased_scripts() {
        assert_eq!(
            apply_note_start_text_case("ابن سينا", NoteStartTextCase::CapitalizeFirst, Some("ar"),),
            "ابن سينا"
        );
    }
}
