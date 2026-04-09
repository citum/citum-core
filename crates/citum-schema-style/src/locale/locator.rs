/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use crate::citation::LocatorType;
use crate::locale::Locale;
use std::collections::HashMap;

/// Normalize a textual locator string into the canonical locator model using locale-aware aliases.
pub fn normalize_locator_text(
    locator: &str,
    locale: &Locale,
) -> Option<citum_schema_data::citation::CitationLocator> {
    citum_schema_data::citation::normalize_locator_text(locator, &locator_aliases(locale))
}

/// Generate a list of locator aliases for a given locale, ordered by length descending.
pub fn locator_aliases(locale: &Locale) -> Vec<(String, LocatorType)> {
    let mut aliases = HashMap::<String, LocatorType>::new();

    for (alias, label) in english_locator_aliases() {
        aliases.insert((*alias).to_string(), label.clone());
    }

    for (label, term) in &locale.locators {
        for form in [&term.long, &term.short, &term.symbol]
            .into_iter()
            .flatten()
        {
            aliases
                .entry(form.singular.to_lowercase())
                .or_insert_with(|| label.clone());
            aliases
                .entry(form.plural.to_lowercase())
                .or_insert_with(|| label.clone());
        }
    }

    let mut aliases: Vec<(String, LocatorType)> = aliases.into_iter().collect();
    aliases.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    aliases
}

/// Returns a static list of common English locator aliases.
pub fn english_locator_aliases() -> &'static [(&'static str, LocatorType)] {
    &[
        ("algorithm", LocatorType::Algorithm),
        ("alg.", LocatorType::Algorithm),
        ("book", LocatorType::Book),
        ("bk.", LocatorType::Book),
        ("chapter", LocatorType::Chapter),
        ("chap.", LocatorType::Chapter),
        ("ch.", LocatorType::Chapter),
        ("chapters", LocatorType::Chapter),
        ("chs.", LocatorType::Chapter),
        ("clause", LocatorType::Clause),
        ("cl.", LocatorType::Clause),
        ("column", LocatorType::Column),
        ("col.", LocatorType::Column),
        ("columns", LocatorType::Column),
        ("cols.", LocatorType::Column),
        ("corollary", LocatorType::Corollary),
        ("cor.", LocatorType::Corollary),
        ("definition", LocatorType::Definition),
        ("def.", LocatorType::Definition),
        ("division", LocatorType::Division),
        ("div.", LocatorType::Division),
        ("figure", LocatorType::Figure),
        ("fig.", LocatorType::Figure),
        ("figures", LocatorType::Figure),
        ("figs.", LocatorType::Figure),
        ("folio", LocatorType::Folio),
        ("fol.", LocatorType::Folio),
        ("line", LocatorType::Line),
        ("l.", LocatorType::Line),
        ("lines", LocatorType::Line),
        ("ll.", LocatorType::Line),
        ("lemma", LocatorType::Lemma),
        ("lem.", LocatorType::Lemma),
        ("note", LocatorType::Note),
        ("n.", LocatorType::Note),
        ("notes", LocatorType::Note),
        ("nn.", LocatorType::Note),
        ("number", LocatorType::Number),
        ("no.", LocatorType::Number),
        ("numbers", LocatorType::Number),
        ("nos.", LocatorType::Number),
        ("opus", LocatorType::Opus),
        ("op.", LocatorType::Opus),
        ("page", LocatorType::Page),
        ("p.", LocatorType::Page),
        ("pages", LocatorType::Page),
        ("pp.", LocatorType::Page),
        ("paragraph", LocatorType::Paragraph),
        ("para.", LocatorType::Paragraph),
        ("paragraphs", LocatorType::Paragraph),
        ("paras.", LocatorType::Paragraph),
        ("part", LocatorType::Part),
        ("pt.", LocatorType::Part),
        ("parts", LocatorType::Part),
        ("pts.", LocatorType::Part),
        ("problem", LocatorType::Problem),
        ("prob.", LocatorType::Problem),
        ("proposition", LocatorType::Proposition),
        ("prop.", LocatorType::Proposition),
        ("recital", LocatorType::Recital),
        ("rec.", LocatorType::Recital),
        ("schedule", LocatorType::Schedule),
        ("sched.", LocatorType::Schedule),
        ("section", LocatorType::Section),
        ("sec.", LocatorType::Section),
        ("sections", LocatorType::Section),
        ("secs.", LocatorType::Section),
        ("§", LocatorType::Section),
        ("§§", LocatorType::Section),
        ("surah", LocatorType::Surah),
        ("theorem", LocatorType::Theorem),
        ("thm.", LocatorType::Theorem),
        ("sub verbo", LocatorType::SubVerbo),
        ("s.v.", LocatorType::SubVerbo),
        ("supplement", LocatorType::Supplement),
        ("suppl.", LocatorType::Supplement),
        ("verse", LocatorType::Verse),
        ("v.", LocatorType::Verse),
        ("verses", LocatorType::Verse),
        ("vv.", LocatorType::Verse),
        ("volume", LocatorType::Volume),
        ("vol.", LocatorType::Volume),
        ("volumes", LocatorType::Volume),
        ("vols.", LocatorType::Volume),
        ("issue", LocatorType::Issue),
        ("issues", LocatorType::Issue),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_locale_locator_terms_become_parsing_aliases() {
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

        let locator =
            normalize_locator_text("reel 3", &locale).expect("custom locator should parse");
        let segment = &locator.segments()[0];
        assert_eq!(segment.label, LocatorType::Custom("reel".to_string()));
        assert_eq!(segment.value.value_str(), "3");
    }
}
