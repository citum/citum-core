/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use super::Locale;

/// Convert a kebab-case key to a human-readable display string.
///
/// Splits on `-`, capitalizes the first character of the first word, and joins with spaces.
fn kebab_to_display(key: &str) -> String {
    let mut words = key.split('-');
    let mut result = String::new();
    if let Some(first) = words.next() {
        let mut chars = first.chars();
        if let Some(c) = chars.next() {
            result.extend(c.to_uppercase());
            result.push_str(chars.as_str());
        }
        for word in words {
            result.push(' ');
            result.push_str(word);
        }
    }
    result
}

impl Locale {
    /// Look up display text for a genre canonical key.
    ///
    /// Falls back to a readable form of the key if no translation found.
    pub fn lookup_genre(&self, key: &str) -> String {
        self.vocab
            .genre
            .get(key)
            .cloned()
            .unwrap_or_else(|| kebab_to_display(key))
    }

    /// Look up display text for a medium canonical key.
    ///
    /// Falls back to a readable form of the key if no translation found.
    pub fn lookup_medium(&self, key: &str) -> String {
        self.vocab
            .medium
            .get(key)
            .cloned()
            .unwrap_or_else(|| kebab_to_display(key))
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

    #[test]
    fn test_lookup_genre_known_key() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  genre:
    phd-thesis: "PhD thesis"
"#,
        )
        .unwrap();
        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
    }

    #[test]
    fn test_lookup_medium_known_key() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  medium:
    television: "Television"
"#,
        )
        .unwrap();
        assert_eq!(locale.lookup_medium("television"), "Television");
    }

    #[test]
    fn test_lookup_genre_fallback() {
        let locale = Locale::en_us();
        // Unknown key -> title-case first word + spaces
        assert_eq!(locale.lookup_genre("unknown-key"), "Unknown key");
    }

    #[test]
    fn test_en_us_locale_uses_embedded_vocab() {
        let locale = Locale::en_us();

        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
        assert_eq!(locale.lookup_medium("audio-cd"), "Audio CD");
    }

    #[test]
    fn test_from_yaml_str_inherits_embedded_vocab_defaults() {
        let locale = Locale::from_yaml_str("locale: en-US\n").unwrap();

        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
    }

    #[test]
    fn test_partial_genre_vocab_override_preserves_medium_defaults() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  genre:
    phd-thesis: "Doctoral dissertation"
"#,
        )
        .unwrap();

        assert_eq!(locale.lookup_genre("phd-thesis"), "Doctoral dissertation");
        assert_eq!(locale.lookup_medium("audio-cd"), "Audio CD");
    }

    #[test]
    fn test_partial_medium_vocab_override_preserves_genre_defaults() {
        let locale = Locale::from_yaml_str(
            r#"
locale: en-US
vocab:
  medium:
    television: "Broadcast television"
"#,
        )
        .unwrap();

        assert_eq!(locale.lookup_medium("television"), "Broadcast television");
        assert_eq!(locale.lookup_genre("phd-thesis"), "PhD thesis");
    }

    #[test]
    fn test_kebab_to_display_single_word() {
        assert_eq!(kebab_to_display("video"), "Video");
    }

    #[test]
    fn test_kebab_to_display_multiple_words() {
        assert_eq!(kebab_to_display("phd-thesis"), "Phd thesis");
        assert_eq!(kebab_to_display("audio-cd"), "Audio cd");
    }
}
