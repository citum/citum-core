/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

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
    use crate::locale::Locale;
    use crate::options::{Config, MultilingualMode};
    use crate::reference::contributor::MultilingualName;
    use crate::reference::types::{Monograph, Title};

    #[test]
    fn test_multilingual_title_deserialization() {
        let yaml = r#"
original: "战争与和平"
lang: "zh"
sort-as: "Zhanzheng yu Heping"
transliterations:
  zh-Latn-pinyin: "Zhànzhēng yǔ Hépíng"
translations:
  en: "War and Peace"
"#;
        let title: Title = serde_yaml::from_str(yaml).unwrap();
        if let Title::Multilingual(m) = title {
            assert_eq!(m.original, "战争与和平");
            assert_eq!(m.lang, Some("zh".into()));
            assert_eq!(m.sort_as.as_deref(), Some("Zhanzheng yu Heping"));
            assert_eq!(m.translations.get("en").unwrap(), "War and Peace");
        } else {
            panic!("Expected Title::Multilingual");
        }
    }

    #[test]
    fn test_multilingual_contributor_deserialization() {
        let yaml = r#"
original:
  family: "Tolstoy"
  given: "Leo"
lang: "ru"
sort-as: "Tolstoy"
transliterations:
  Latn:
    family: "Tolstoy"
    given: "Leo"
"#;
        let name: MultilingualName = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(name.original.family.to_string(), "Tolstoy");
        assert_eq!(name.lang, Some("ru".into()));
        assert_eq!(name.sort_as.as_deref(), Some("Tolstoy"));
        assert!(name.transliterations.contains_key("Latn"));
    }

    #[test]
    fn test_multilingual_style_options() {
        let yaml = r#"
multilingual:
  title-mode: "transliterated"
  name-mode: "combined"
  preferred-script: "Latn"
  scripts:
    cjk:
      use-native-ordering: true
      delimiter: ""
    katakana:
      delimiter: "・"
      sort-separator: "、"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let mlt = config.multilingual.unwrap();
        assert_eq!(mlt.title_mode, Some(MultilingualMode::Transliterated));
        assert_eq!(mlt.name_mode, Some(MultilingualMode::Combined));
        assert!(mlt.scripts.get("cjk").unwrap().use_native_ordering);
        assert_eq!(
            mlt.scripts
                .get("katakana")
                .and_then(|script| script.sort_separator.as_deref()),
            Some("、")
        );
    }

    #[test]
    fn test_multiple_transliteration_methods() {
        let yaml = r#"
original: "東京"
lang: "ja"
transliterations:
  ja-Latn-hepburn: "Tōkyō"
  ja-Latn-kunrei: "Tôkyô"
translations:
  en: "Tokyo"
"#;
        let title: Title = serde_yaml::from_str(yaml).unwrap();
        if let Title::Multilingual(m) = title {
            assert_eq!(m.original, "東京");
            assert_eq!(m.transliterations.get("ja-Latn-hepburn").unwrap(), "Tōkyō");
            assert_eq!(m.transliterations.get("ja-Latn-kunrei").unwrap(), "Tôkyô");
        } else {
            panic!("Expected Title::Multilingual");
        }
    }

    #[test]
    fn test_title_locale_overrides_deserialization() {
        let yaml = r#"
titles:
  component:
    quote: true
    primary-delimiter: ". "
    subtitle-delimiter: "; "
    locale-overrides:
      de:
        emph: true
        primary-delimiter: ": "
      en-US:
        quote: false
        subtitle-delimiter: ". "
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let titles = config.titles.unwrap();
        let component = titles.component.unwrap();
        assert_eq!(component.primary_delimiter.as_deref(), Some(". "));
        assert_eq!(component.subtitle_delimiter.as_deref(), Some("; "));
        let overrides = component.locale_overrides.unwrap();
        assert_eq!(overrides.get("de").unwrap().emph, Some(true));
        assert_eq!(
            overrides.get("de").unwrap().primary_delimiter.as_deref(),
            Some(": ")
        );
        assert_eq!(overrides.get("en-US").unwrap().quote, Some(false));
        assert_eq!(
            overrides
                .get("en-US")
                .unwrap()
                .subtitle_delimiter
                .as_deref(),
            Some(". ")
        );
    }

    #[test]
    fn test_locale_title_delimiters_deserialization() {
        let yaml = r#"
locale: x-test
grammar-options:
  title-subtitle-delimiter: " — "
  subtitle-delimiter: " / "
"#;
        let locale = Locale::from_yaml_str(yaml).unwrap();
        assert_eq!(
            locale.grammar_options.title_subtitle_delimiter.as_str(),
            " — "
        );
        assert_eq!(locale.grammar_options.subtitle_delimiter.as_str(), " / ");
    }

    #[test]
    fn test_locale_punctuation_collision_defaults_deserialization() {
        let yaml = r#"
locale: x-test
grammar-options:
  strong-terminal-comma-policy: keep-terminal
  delimiter-suppressing-terminal-marks: "?!…"
"#;
        let locale = Locale::from_yaml_str(yaml).unwrap();
        assert_eq!(
            locale.grammar_options.strong_terminal_comma_policy,
            crate::options::StrongTerminalCommaPolicy::KeepTerminal
        );
        assert_eq!(
            locale.grammar_options.delimiter_suppressing_terminal_marks,
            "?!…"
        );
    }

    #[test]
    fn test_field_languages_deserialization() {
        let yaml = r#"
id: chapter-1
type: book
title: Haupttitel
issued: "2024"
language: de
field-languages:
  title: en
  parent-monograph.title: de
"#;
        let monograph: Monograph = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            monograph.field_languages.get("title").unwrap().as_ref(),
            "en"
        );
        assert_eq!(
            monograph
                .field_languages
                .get("parent-monograph.title")
                .unwrap()
                .as_ref(),
            "de"
        );
    }
}
