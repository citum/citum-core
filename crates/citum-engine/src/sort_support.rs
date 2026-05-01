/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Shared bibliography sort-key normalization and collation helpers.

use std::cmp::Ordering;

use crate::reference::Reference;
use citum_schema::grouping::NameSortOrder;
use citum_schema::locale::Locale;
use icu_collator::options::{AlternateHandling, CaseLevel, CollatorOptions, Strength};
use icu_collator::{CollatorBorrowed, CollatorPreferences};
use icu_locale::Locale as IcuLocale;

/// Locale-aware comparator used by bibliography sorting paths.
pub(crate) struct TextCollator {
    collator: CollatorBorrowed<'static>,
}

impl TextCollator {
    /// Create a collator for the active Citum locale.
    ///
    /// Configures the collator with:
    /// - Secondary strength (base letters + accents, no case distinction)
    /// - Case level off (case-insensitive via collator, not preprocessing)
    /// - Alternate handling shifted (punctuation/spaces ignorable at primary/secondary)
    #[must_use]
    pub(crate) fn new(locale: &Locale) -> Self {
        let mut options = CollatorOptions::default();
        options.strength = Some(Strength::Secondary);
        options.case_level = Some(CaseLevel::Off);
        options.alternate_handling = Some(AlternateHandling::Shifted);
        // Note: Numeric ordering and script reordering are not explicitly
        // configurable at the ICU4X collator API level; they follow CLDR
        // defaults for the resolved locale.
        #[allow(clippy::expect_used, reason = "ICU bootstrap failure is fatal")]
        let collator = CollatorBorrowed::try_new(collator_preferences(locale), options)
            .expect("ICU4X compiled collation data should be available");
        Self { collator }
    }

    /// Compare two already-normalized sort keys.
    #[must_use]
    pub(crate) fn compare(&self, left: &str, right: &str) -> Ordering {
        self.collator.compare(left, right)
    }
}

/// Build the normalized author sort key using existing fallback rules.
#[must_use]
pub(crate) fn author_sort_key_opt(
    reference: &Reference,
    name_order: NameSortOrder,
    locale: &Locale,
    fallback_to_title: bool,
) -> Option<String> {
    reference
        .author()
        .and_then(|c| c.to_names_vec().first().cloned())
        .map(|name| match name_order {
            NameSortOrder::FamilyGiven | NameSortOrder::GivenFamily => {
                normalize_sort_text(name.family_or_literal())
            }
        })
        .filter(|key| !key.is_empty())
        .or_else(|| {
            reference
                .editor()
                .and_then(|c| c.to_names_vec().first().cloned())
                .map(|name| normalize_sort_text(name.family_or_literal()))
                .filter(|key| !key.is_empty())
        })
        .or_else(|| fallback_to_title.then(|| title_sort_key(reference, locale)))
        .filter(|key| !key.is_empty())
}

/// Build the normalized title sort key with locale article stripping.
#[must_use]
pub(crate) fn title_sort_key(reference: &Reference, locale: &Locale) -> String {
    let title = reference.title().map(|t| t.to_string()).unwrap_or_default();
    normalize_sort_text(locale.strip_sort_articles(&title))
}

/// Normalize plain text for bibliography sorting.
///
/// Returns the text unchanged; collator handles case-insensitive comparison
/// internally via CaseLevel::Off configuration, preserving original text.
#[must_use]
pub(crate) fn normalize_sort_text(text: &str) -> String {
    text.to_string()
}

fn collator_preferences(locale: &Locale) -> CollatorPreferences {
    parse_icu_locale(&locale.locale)
        .unwrap_or_else(default_icu_locale)
        .into()
}

fn parse_icu_locale(locale_id: &str) -> Option<IcuLocale> {
    let mut candidate = locale_id.trim();
    while !candidate.is_empty() {
        if let Ok(locale) = candidate.parse::<IcuLocale>() {
            return Some(locale);
        }
        match candidate.rsplit_once('-') {
            Some((prefix, _)) => candidate = prefix,
            None => break,
        }
    }
    None
}

fn default_icu_locale() -> IcuLocale {
    #[allow(clippy::expect_used, reason = "ICU bootstrap failure is fatal")]
    "en-US"
        .parse::<IcuLocale>()
        .expect("en-US should always be a valid ICU locale")
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

    #[test]
    fn test_parse_icu_locale_trims_unparseable_override_suffix() {
        let parsed = parse_icu_locale("de-DE-foo_bar")
            .expect("fallback parsing should produce a base locale");
        assert_eq!(parsed.to_string(), "de-DE");
    }

    #[test]
    fn test_text_collator_sorts_accented_names_near_ascii_peers() {
        let collator = TextCollator::new(&Locale::en_us());
        assert_eq!(collator.compare("celik", "çelik"), Ordering::Less);
        assert_eq!(collator.compare("ó tuathail", "zukin"), Ordering::Less);
    }

    #[test]
    fn test_normalize_sort_text_preserves_locale_sensitive_case_points() {
        assert_eq!(normalize_sort_text("İnce"), "İnce");
    }

    #[test]
    fn test_text_collator_is_case_insensitive() {
        let collator = TextCollator::new(&Locale::en_us());
        // "smith" and "Smith" should compare equal at primary/secondary levels
        assert_eq!(collator.compare("smith", "Smith"), Ordering::Equal);
        assert_eq!(collator.compare("Jones", "jones"), Ordering::Equal);
    }

    #[test]
    fn test_text_collator_nfc_nfd_equivalence() {
        let collator = TextCollator::new(&Locale::en_us());
        // é as single codepoint (NFC) vs e + combining acute (NFD) should compare equal
        let nfc = "café"; // é as U+00E9
        let nfd = "cafe\u{0301}"; // e + U+0301 combining acute
        assert_eq!(collator.compare(nfc, nfd), Ordering::Equal);
    }

    #[test]
    fn test_text_collator_hangul_latin_consistent_order() {
        let collator = TextCollator::new(&Locale::en_us());
        // Under en-US tailored collator, these should have a consistent order.
        // Hangul block (U+AC00 onwards) sorts after Latin-1 Supplement.
        let latin = "Smith";
        let hangul = "김"; // Hangul syllable "Kim"
        // Just verify consistent comparison (both directions give opposite results)
        let fwd = collator.compare(latin, hangul);
        let rev = collator.compare(hangul, latin);
        assert_ne!(fwd, rev); // One is Less, the other is Greater
        assert_eq!(fwd.reverse(), rev); // Reverse of Less is Greater
    }

    #[test]
    fn test_text_collator_arabic_latin_consistent_order() {
        let collator = TextCollator::new(&Locale::en_us());
        // Under en-US tailored collator, Arabic script sorts consistently.
        let latin = "Smith";
        let arabic = "محمد"; // Arabic "Muhammad"
        let fwd = collator.compare(latin, arabic);
        let rev = collator.compare(arabic, latin);
        assert_ne!(fwd, rev);
        assert_eq!(fwd.reverse(), rev);
    }

    #[test]
    fn test_text_collator_punctuation_ignorable() {
        let collator = TextCollator::new(&Locale::en_us());
        // With AlternateHandling::Shifted, punctuation and spaces should be ignorable
        // at primary/secondary levels, so names with and without apostrophes/hyphens compare equal.
        assert_eq!(collator.compare("O'Brien", "Obrien"), Ordering::Equal);
        assert_eq!(collator.compare("al-Rashid", "alRashid"), Ordering::Equal);
    }
}
