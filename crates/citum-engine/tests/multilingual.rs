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
        .render_selected_bibliography_with_format::<citum_engine::render::plain::PlainText, _>(
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
        .render_selected_bibliography_with_format::<citum_engine::render::plain::PlainText, _>(
            vec!["CJK-JAPANESE-LANGUAGE-TAGGED".to_string()],
        );
    let default_entry = processor
        .render_selected_bibliography_with_format::<citum_engine::render::plain::PlainText, _>(
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
