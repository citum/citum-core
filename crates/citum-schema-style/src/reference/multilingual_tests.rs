/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(test)]
mod tests {
    use crate::options::{Config, MultilingualMode};
    use crate::reference::contributor::MultilingualName;
    use crate::reference::types::{Monograph, Title};

    #[test]
    fn test_multilingual_title_deserialization() {
        let yaml = r#"
original: "战争与和平"
lang: "zh"
transliterations:
  zh-Latn-pinyin: "Zhànzhēng yǔ Hépíng"
translations:
  en: "War and Peace"
"#;
        let title: Title = serde_yaml::from_str(yaml).unwrap();
        if let Title::Multilingual(m) = title {
            assert_eq!(m.original, "战争与和平");
            assert_eq!(m.lang, Some("zh".to_string()));
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
transliterations:
  Latn:
    family: "Tolstoy"
    given: "Leo"
"#;
        let name: MultilingualName = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(name.original.family.to_string(), "Tolstoy");
        assert_eq!(name.lang, Some("ru".to_string()));
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
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let mlt = config.multilingual.unwrap();
        assert_eq!(mlt.title_mode, Some(MultilingualMode::Transliterated));
        assert_eq!(mlt.name_mode, Some(MultilingualMode::Combined));
        assert!(mlt.scripts.get("cjk").unwrap().use_native_ordering);
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
    locale-overrides:
      de:
        emph: true
      en-US:
        quote: false
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let titles = config.titles.unwrap();
        let component = titles.component.unwrap();
        let overrides = component.locale_overrides.unwrap();
        assert_eq!(overrides.get("de").unwrap().emph, Some(true));
        assert_eq!(overrides.get("en-US").unwrap().quote, Some(false));
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
        assert_eq!(monograph.field_languages.get("title").unwrap(), "en");
        assert_eq!(
            monograph
                .field_languages
                .get("parent-monograph.title")
                .unwrap(),
            "de"
        );
    }
}
