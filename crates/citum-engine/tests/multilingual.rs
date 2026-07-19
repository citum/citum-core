/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "test")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in test, benchmark, and example code."
)]

mod common;
use common::announce_behavior;

use citum_engine::Processor;
use citum_io::load_bibliography;
use citum_schema::Style;
use citum_schema::citation::{Citation, CitationItem};
use citum_schema::reference::InputReference;
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

/// Project root path resolver.
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Load a YAML style file.
fn load_style(path: &PathBuf) -> Style {
    let bytes = fs::read(path).expect("style file should be readable");
    serde_yaml::from_slice(&bytes).expect("style file should parse as YAML")
}

/// Create a single-item citation from an item ID.
fn single_item_citation(id: &str) -> Citation {
    Citation {
        items: vec![CitationItem {
            id: id.to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn test_cjk_name_rendering_asian_glyphs() {
    announce_behavior("CJK author names are preserved and rendered with native glyphs.");
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-cjk.json"))
            .expect("CJK fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("CSL-ASIAN-GLYPHS"))
        .expect("Asian glyphs citation should render");

    // Plain CJK name with no parallel variants: the §1.3 fallback renders the
    // original glyphs even under the APA romanized-translated preset.
    assert_eq!(citation, "(我妻, 1960)");
}

#[test]
fn test_cjk_et_al_rendering() {
    announce_behavior("CJK name lists are truncated with et al. for APA-style citations.");
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-cjk.json"))
            .expect("CJK fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("CJK-ET-AL-KANJI-AUTHORS"))
        .expect("et al. citation should render");

    // Three kanji-named authors under APA shorten {min: 3, use-first: 1}:
    // first author's family glyphs followed by the et-al term.
    assert_eq!(citation, "(山田 et al., 2020)");
}

#[test]
fn test_arabic_short_forms_with_diacritics() {
    announce_behavior("Arabic author names are rendered with diacritical marks intact.");
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-arabic.json"))
            .expect("Arabic fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("ARABIC-ASWANI-DIACRITICS"))
        .expect("Arabic citation with diacritics should render");

    // Romanized Arabic family name with macron and ʿayn must survive intact.
    assert_eq!(citation, "(al-Aswānī, 2015)");
}

#[test]
fn test_romanized_translated_preset_uses_parallel_metadata() {
    announce_behavior(
        "APA romanized-translated preset: names render romanized; titles render romanized [translated].",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-cjk.json"))
            .expect("CJK fixture should parse");

    let processor = Processor::new(style, bibliography);

    let citation = processor
        .process_citation(&single_item_citation("CJK-JAPANESE-BOOK"))
        .expect("Japanese book citation should render");
    let entry = processor
        .render_selected_bibliography_with_format_standalone::<citum_engine::render::plain::PlainText, _>(
            vec!["CJK-JAPANESE-BOOK".to_string()],
        );

    // §2.1 romanized-translated: the ja-Latn-hepburn name transliteration is
    // preferred over the original kana in the citation.
    assert_eq!(citation, "(Torusutoi, 1869)");
    // §2.1 romanized-translated: the title renders as the ja-Latn-hepburn
    // transliteration followed by the bracketed English translation.
    assert_eq!(
        entry,
        "Torusutoi, L. (1869). _Sensō to Heiwa [War and Peace]_. Iwanami Shoten."
    );
}

#[test]
fn test_bibliography_locales_switch_full_entry_layouts() {
    announce_behavior(
        "Bibliography entries switch to a locale-specific layout when the reference language matches.",
    );
    let root = project_root();
    let style =
        load_style(&root.join("styles/experimental/locale-specific-bibliography-layouts.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-cjk.json"))
            .expect("CJK fixture should parse");

    let processor = Processor::new(style, bibliography);

    let japanese_entry = processor
        .render_selected_bibliography_with_format_standalone::<citum_engine::render::plain::PlainText, _>(
            vec!["CJK-JAPANESE-LANGUAGE-TAGGED".to_string()],
        );
    let default_entry = processor
        .render_selected_bibliography_with_format_standalone::<citum_engine::render::plain::PlainText, _>(
            vec!["CSL-ET-AL-LATIN".to_string()],
        );

    assert!(
        japanese_entry.contains("Tokyo Academic Press, 2018. 日本語の書誌"),
        "Japanese entry should use localized publisher-year-title order: {japanese_entry}"
    );
    assert!(
        !japanese_entry.contains("日本語の書誌. Tokyo Academic Press"),
        "Japanese entry should not use default title-publisher order: {japanese_entry}"
    );
    assert!(
        default_entry.contains("Test Book. Test Publisher, 2020"),
        "Default entry should use title-publisher-year order: {default_entry}"
    );
}

#[test]
fn test_chinese_article_three_part_title() {
    announce_behavior(
        "Chicago: Chinese article renders as romanized + original script + [translation].",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/chicago-notes-18th-script.yaml"));
    let bibliography = load_bibliography(
        &root.join("tests/fixtures/multilingual/multilingual-east-asian-chicago.yaml"),
    )
    .expect("East Asian Chicago fixture should parse");

    let processor = Processor::new(style, bibliography);
    let entry = processor
        .process_citation(&single_item_citation("hua-linfu-article"))
        .expect("Chinese article citation should render");

    // Matches the Chicago 18th source example: native family-first ordering
    // with the original script appended after the romanized name.
    assert_eq!(
        entry,
        "Hua Linfu 华林甫, “Qingdai yilai Sanxia diqu shuihan zaihai de chubu yanjiu \
         清代以来三峡地区水旱灾害的初步研究 [A preliminary study of floods and droughts \
         in the Three Gorges region since the Qing dynasty]”, _Zhongguo shehui kexue \
         中国社会科学_ 1 (1999): 168–79."
    );
}

#[test]
fn test_korean_book_three_part_title() {
    announce_behavior(
        "Chicago: Korean book renders as romanized + original script + [translation].",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/chicago-notes-18th-script.yaml"));
    let bibliography = load_bibliography(
        &root.join("tests/fixtures/multilingual/multilingual-east-asian-chicago.yaml"),
    )
    .expect("East Asian Chicago fixture should parse");

    let processor = Processor::new(style, bibliography);
    let entry = processor
        .process_citation(&single_item_citation("kang-ubang-book"))
        .expect("Korean book citation should render");

    // Matches the Chicago 18th source example: native family-first ordering
    // with the original Hanja appended after the romanized name.
    assert_eq!(
        entry,
        "Kang U-bang 姜友邦, _Wŏnyung kwa chohwa: Han’guk kodae chogaksa ŭi wŏlli \
         圓融과調和: 韓國古代彫刻史의原理 [Synthesis and harmony: Principle of the \
         history of ancient Korean sculpture]_ (Yŏrhwadang, 1990)."
    );
}

#[test]
fn test_japanese_book_two_authors() {
    announce_behavior(
        "Chicago: Japanese book with two authors — both names and three-part title render.",
    );
    let root = project_root();
    let style = load_style(&root.join("styles/embedded/chicago-notes-18th-script.yaml"));
    let bibliography = load_bibliography(
        &root.join("tests/fixtures/multilingual/multilingual-east-asian-chicago.yaml"),
    )
    .expect("East Asian Chicago fixture should parse");

    let processor = Processor::new(style, bibliography);
    let entry = processor
        .process_citation(&single_item_citation("abe-yoshio-book"))
        .expect("Japanese book citation should render");

    // Matches the Chicago 18th source example: both names render family-first
    // with their original kanji appended after the romanized form.
    assert_eq!(
        entry,
        "Abe Yoshio 阿部善雄 and Kaneko Hideo 金子英生, _Saigo no “Nihonjin”: Asakawa Kan’ichi no shōgai \
         最後の「日本人」: 朝河貫一の生涯 [The last “Japanese”: Life of Kan’ichi Asakawa]_ \
         (Iwanami Shoten, 1983)."
    );
}

/// Build a minimal book reference for the GB/T script-aware punctuation tests.
fn punctuation_test_book(
    id: &str,
    title: &str,
    language: &str,
    publisher_place: &str,
    publisher: &str,
) -> InputReference {
    let fixture = serde_json::json!({
        "id": id,
        "type": "book",
        "title": title,
        "language": language,
        "publisher": publisher,
        "publisher-place": publisher_place,
        "issued": { "date-parts": [[1988]] },
    });
    let legacy: csl_legacy::csl_json::Reference =
        serde_json::from_value(fixture).expect("punctuation test fixture should parse");
    legacy.into()
}

#[test]
fn given_gb_t_numeric_style_when_rendering_latin_script_book_then_delimiters_are_latin_punctuation()
{
    announce_behavior(
        "GB/T 7714 numeric renders Latin-script references with Latin punctuation \
         (`: , ( )`), not the style's CJK-facing full-width delimiters — csl26-fn9x.",
    );
    let style = citum_schema::embedded::get_embedded_style("gb-t-7714-2025-numeric")
        .expect("gb-t-7714-2025-numeric should be embedded")
        .expect("gb-t-7714-2025-numeric should parse")
        .into_resolved();
    let bibliography = IndexMap::from([(
        "latin-book".to_string(),
        punctuation_test_book(
            "latin-book",
            "AI and the future of banking",
            "en",
            "New York",
            "Wiley",
        ),
    )]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(
        rendered,
        "[1]AI and the future of banking[M]. New York: Wiley, 1988"
    );
}

#[test]
fn given_gb_t_numeric_style_when_rendering_cjk_script_book_then_delimiters_stay_full_width() {
    announce_behavior(
        "GB/T 7714 numeric keeps full-width CJK delimiters for CJK-script references — \
         the Latin-script remap does not affect the style's native-script items.",
    );
    let style = citum_schema::embedded::get_embedded_style("gb-t-7714-2025-numeric")
        .expect("gb-t-7714-2025-numeric should be embedded")
        .expect("gb-t-7714-2025-numeric should parse")
        .into_resolved();
    let bibliography = IndexMap::from([(
        "cjk-book".to_string(),
        punctuation_test_book(
            "cjk-book",
            "银行业的未来与人工智能",
            "zh",
            "北京",
            "清华大学出版社",
        ),
    )]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(
        rendered,
        "[1]银行业的未来与人工智能[M]. 北京：清华大学出版社，1988"
    );
}

#[test]
fn given_style_without_latin_punctuation_option_when_rendering_latin_book_then_full_width_delimiters_are_unchanged()
 {
    announce_behavior(
        "options.multilingual.scripts.latin.punctuation is opt-in: without it, full-width \
         delimiters render unchanged even for a Latin-script item.",
    );
    let yaml = r#"
info:
  id: latin-punctuation-gate-test
  title: Latin Punctuation Gate Test
  default-locale: en-US
bibliography:
  template:
    - group:
        - variable: publisher-place
        - variable: publisher
      delimiter: ：
"#;
    let style: Style = serde_yaml::from_str(yaml).expect("minimal gate-test style should parse");
    let bibliography = IndexMap::from([(
        "latin-book".to_string(),
        punctuation_test_book(
            "latin-book",
            "AI and the future of banking",
            "en",
            "New York",
            "Wiley",
        ),
    )]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(rendered, "New York：Wiley");
}

/// A book fixture with an optional `language` field, for the
/// `realization-default` gate tests below — untagged items must omit the
/// field entirely (a JSON-null `language` is not the same as no evidence).
fn realization_test_book(
    id: &str,
    title: &str,
    language: Option<&str>,
    publisher_place: &str,
) -> InputReference {
    let mut fixture = serde_json::json!({
        "id": id,
        "type": "book",
        "title": title,
        "publisher-place": publisher_place,
        "issued": { "date-parts": [[1988]] },
    });
    if let Some(language) = language {
        fixture["language"] = serde_json::Value::String(language.to_string());
    }
    let legacy: csl_legacy::csl_json::Reference =
        serde_json::from_value(fixture).expect("realization test fixture should parse");
    legacy.into()
}

/// Style used by the `realization-default` gate tests: opts in to CJK
/// realization and wraps `publisher-place` with the semantic
/// `wrap: parentheses` token so its rendered width follows each item's
/// effective script (docs/specs/PUNCTUATION_REALIZATION.md, increment 1).
const REALIZATION_DEFAULT_CJK_GATE_STYLE_YAML: &str = r#"
info:
  id: realization-default-gate-test
  title: Realization Default Gate Test
  default-locale: en-US
options:
  multilingual:
    realization-default: cjk
bibliography:
  template:
    - variable: publisher-place
      wrap: parentheses
"#;

#[test]
fn realization_default_cjk_wraps_cjk_script_item_full_width() {
    announce_behavior(
        "options.multilingual.realization-default: cjk makes wrap: parentheses realize \
         full-width for a CJK-script item — csl26-k2kp increment 1.",
    );
    let style: Style = serde_yaml::from_str(REALIZATION_DEFAULT_CJK_GATE_STYLE_YAML)
        .expect("realization-default gate-test style should parse");
    let bibliography = IndexMap::from([(
        "cjk-book".to_string(),
        realization_test_book("cjk-book", "银行业的未来与人工智能", Some("zh"), "Beijing"),
    )]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(rendered, "（Beijing）");
}

#[test]
fn realization_default_cjk_still_wraps_latin_evidence_item_half_width() {
    announce_behavior(
        "options.multilingual.realization-default: cjk still realizes half-width for a \
         positively Latin-script item — evidence overrides the style's declared default.",
    );
    let style: Style = serde_yaml::from_str(REALIZATION_DEFAULT_CJK_GATE_STYLE_YAML)
        .expect("realization-default gate-test style should parse");
    let bibliography = IndexMap::from([(
        "latin-book".to_string(),
        realization_test_book(
            "latin-book",
            "AI and the future of banking",
            Some("en"),
            "New York",
        ),
    )]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(rendered, "(New York)");
}

#[test]
fn realization_default_cjk_wraps_untagged_item_full_width() {
    announce_behavior(
        "options.multilingual.realization-default: cjk applies to an untagged item with no \
         usable script evidence — the positive-evidence rule: absence of evidence never \
         moves an item away from the style's declared default.",
    );
    let style: Style = serde_yaml::from_str(REALIZATION_DEFAULT_CJK_GATE_STYLE_YAML)
        .expect("realization-default gate-test style should parse");
    let bibliography = IndexMap::from([(
        "untagged-book".to_string(),
        realization_test_book("untagged-book", "Untitled Work", None, "Unknown"),
    )]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(rendered, "（Unknown）");
}

fn render_realization_group(style_yaml: &str, language: &str) -> String {
    let style: Style =
        serde_yaml::from_str(style_yaml).expect("punctuation realization style should parse");
    let bibliography = IndexMap::from([(
        "book".to_string(),
        punctuation_test_book("book", "测试", language, "Left", "Right"),
    )]);

    Processor::new(style, bibliography).render_bibliography()
}

#[test]
fn semantic_comma_uses_script_default_but_literal_comma_name_is_untouched() {
    announce_behavior(
        "A { mark: comma } separator follows the effective item script while the scalar \
         text `comma` remains literal — csl26-w6wf.",
    );
    let semantic_style = r#"
info: { id: semantic-comma, title: Semantic Comma }
options:
  multilingual:
    realization-default: cjk
bibliography:
  template:
    - group:
        - variable: publisher-place
        - variable: publisher
      delimiter: { mark: comma }
"#;
    let literal_style = r#"
info: { id: literal-comma, title: Literal Comma }
options:
  multilingual:
    realization-default: cjk
bibliography:
  template:
    - group:
        - variable: publisher-place
        - variable: publisher
      delimiter: comma
"#;

    assert_eq!(
        render_realization_group(semantic_style, "zh"),
        "Left，Right"
    );
    assert_eq!(
        render_realization_group(semantic_style, "en"),
        "Left, Right"
    );
    assert_eq!(
        render_realization_group(literal_style, "zh"),
        "LeftcommaRight"
    );
}

#[test]
fn per_script_realization_override_wins_and_unset_marks_use_defaults() {
    announce_behavior(
        "Per-script realization overrides replace only named marks; other semantic marks \
         continue to use the engine defaults — csl26-w6wf.",
    );
    let style = r#"
info: { id: cjk-realization-override, title: CJK Realization Override }
options:
  multilingual:
    realization-default: cjk
    scripts:
      cjk:
        realization:
          comma: "、"
bibliography:
  template:
    - group:
        - variable: publisher-place
        - variable: publisher
      delimiter: { mark: comma }
      suffix: { mark: period }
"#;

    assert_eq!(render_realization_group(style, "zh"), "Left、Right。");
}

#[test]
fn literal_affixes_and_delimiters_are_not_realized_for_cjk_items() {
    announce_behavior(
        "Literal prefix, suffix, and delimiter strings retain their authored glyphs even \
         when the style opts into CJK realization — csl26-w6wf.",
    );
    let style = r#"
info: { id: literal-realization-boundary, title: Literal Realization Boundary }
options:
  multilingual:
    realization-default: cjk
bibliography:
  template:
    - group:
        - variable: publisher-place
        - variable: publisher
      delimiter: ", "
      prefix: "("
      suffix: ")"
"#;

    assert_eq!(render_realization_group(style, "zh"), "(Left, Right)");
}

fn render_contributor_delimiter(style_yaml: &str, language: &str) -> String {
    let style: Style =
        serde_yaml::from_str(style_yaml).expect("contributor realization style should parse");
    let fixture = serde_json::json!({
        "id": "book",
        "type": "book",
        "title": "测试",
        "language": language,
        "author": [
            { "family": "Alpha", "given": "A" },
            { "family": "Beta", "given": "B" }
        ],
        "editor": [
            { "family": "Alpha", "given": "A" },
            { "family": "Beta", "given": "B" }
        ]
    });
    let reference: csl_legacy::csl_json::Reference =
        serde_json::from_value(fixture).expect("contributor fixture should parse");
    let bibliography = IndexMap::from([("book".to_string(), reference.into())]);

    Processor::new(style, bibliography).render_bibliography()
}

#[test]
fn semantic_contributor_delimiter_realizes_by_script_while_scalar_stays_literal() {
    announce_behavior(
        "Contributor-list delimiters use the same explicit mark contract as template \
         delimiters, including literal scalar preservation — csl26-w6wf.",
    );
    let semantic_style = r#"
info: { id: semantic-contributor-comma, title: Semantic Contributor Comma }
options:
  multilingual:
    realization-default: cjk
  contributors:
    delimiter: { mark: comma }
bibliography:
  template:
    - contributor: editor
      form: family-only
"#;
    let literal_style = r#"
info: { id: literal-contributor-comma, title: Literal Contributor Comma }
options:
  multilingual:
    realization-default: cjk
  contributors:
    delimiter: comma
bibliography:
  template:
    - contributor: author
      form: family-only
"#;
    let component_style = r#"
info: { id: semantic-component-punctuation, title: Semantic Component Punctuation }
options:
  multilingual:
    realization-default: cjk
  contributors:
    delimiter: " | "
bibliography:
  template:
    - contributor: editor
      form: family-only
      delimiter: { mark: semicolon }
      label:
        term: editor
        form: long
        placement: suffix
        prefix: { mark: colon }
"#;

    assert_eq!(
        render_contributor_delimiter(semantic_style, "zh"),
        "Alpha，Beta"
    );
    assert_eq!(
        render_contributor_delimiter(semantic_style, "en"),
        "Alpha, Beta"
    );
    assert_eq!(
        render_contributor_delimiter(literal_style, "zh"),
        "AlphacommaBeta"
    );
    assert_eq!(
        render_contributor_delimiter(component_style, "zh"),
        "Alpha；Beta：editors"
    );
    assert_eq!(
        render_contributor_delimiter(component_style, "en"),
        "Alpha; Beta: editors"
    );
}
