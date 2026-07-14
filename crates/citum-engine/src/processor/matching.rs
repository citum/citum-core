/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Matching logic for determining if references share primary contributors.
//!
//! This module implements contributor matching according to substitution rules,
//! allowing comparison of references (particularly for "ibid" tracking) based on author,
//! editor, translator, or title fallback logic.

use crate::reference::Reference;
use citum_schema::Style;
use citum_schema::locale::Locale;
use citum_schema::options::{Config, Substitute};
use std::borrow::Cow;

/// Matcher for determining if references share the same primary contributors.
///
/// Uses the style's substitution configuration to determine which contributor
/// (author, editor, translator) should be used for comparison.
pub struct Matcher<'a> {
    /// The active citation style.
    style: &'a Style,
    /// The effective style configuration used for name resolution and fallbacks.
    config: &'a Config,
    /// The locale used to resolve multilingual and merged-list names.
    locale: &'a Locale,
}

impl<'a> Matcher<'a> {
    /// Build a matcher from the active style, effective configuration, and locale.
    #[must_use]
    pub fn new(style: &'a Style, config: &'a Config, locale: &'a Locale) -> Self {
        Self {
            style,
            config,
            locale,
        }
    }

    /// Check if primary contributors (authors/editors) match between two references.
    ///
    /// Delegates to the shared effective-primary resolver
    /// ([`crate::values::contributor::substitute::effective_primary_names`]) so
    /// matching honors type overrides and merged-role candidates identically to
    /// rendering, sorting, and disambiguation. Two references match only when
    /// both resolve non-empty, equal name lists; a title-substitute result
    /// (empty names) never matches.
    #[must_use]
    pub fn contributors_match(&self, prev: &Reference, current: &Reference) -> bool {
        let substitute = self.get_substitute_config();
        let prev_names = crate::values::contributor::substitute::effective_primary_names(
            prev,
            substitute.as_ref(),
            self.config,
            self.locale,
        );
        let curr_names = crate::values::contributor::substitute::effective_primary_names(
            current,
            substitute.as_ref(),
            self.config,
            self.locale,
        );
        !prev_names.is_empty() && !curr_names.is_empty() && prev_names == curr_names
    }

    /// Gets the substitute configuration from the style or falls back to defaults.
    ///
    /// Resolves the substitute template from the style's options if available,
    /// otherwise falls back to the default configuration's substitute settings.
    fn get_substitute_config(&self) -> Cow<'_, Substitute> {
        if let Some(config) = self
            .style
            .options
            .as_ref()
            .and_then(|o| o.substitute.as_ref())
        {
            return config.resolve_ref();
        }
        self.config.substitute.as_ref().map_or_else(
            || Cow::Owned(Substitute::default()),
            citum_schema::options::SubstituteConfig::resolve_ref,
        )
    }
}
