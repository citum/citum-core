/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use super::Locale;

impl Locale {
    /// Strip leading articles from a string for sorting.
    ///
    /// Uses locale-specific articles (e.g., "the", "a", "an" for English;
    /// "der", "die", "das" for German). Falls back to English articles
    /// if no locale-specific articles are defined.
    pub fn strip_sort_articles<'a>(&self, s: &'a str) -> &'a str {
        let s = s.trim();

        // Default English articles
        const DEFAULT_ARTICLES: &[&str] = &["the", "a", "an"];

        if self.sort_articles.is_empty() {
            // Use default English articles
            for article in DEFAULT_ARTICLES {
                let prefix = format!("{} ", article);
                if s.to_lowercase().starts_with(&prefix) {
                    #[allow(
                        clippy::string_slice,
                        reason = "prefix is derived from ASCII article"
                    )]
                    return &s[prefix.len()..];
                }
            }
        } else {
            // Use locale-specific articles
            for article in &self.sort_articles {
                let prefix = format!("{} ", article);
                if s.to_lowercase().starts_with(&prefix) {
                    #[allow(
                        clippy::string_slice,
                        reason = "prefix is derived from a defined article"
                    )]
                    return &s[prefix.len()..];
                }
            }
        }
        s
    }
}
