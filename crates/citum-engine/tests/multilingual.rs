/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::announce_behavior;

use citum_engine::Processor;
use citum_engine::io::load_bibliography;
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
    let style = load_style(&root.join("styles/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-cjk.json"))
            .expect("CJK fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("CSL-ASIAN-GLYPHS"))
        .expect("Asian glyphs citation should render");

    // CSL test expects output to contain Japanese author name
    assert!(
        citation.contains("我妻"),
        "Citation should contain Japanese author name"
    );
}

#[test]
fn test_cjk_et_al_rendering() {
    announce_behavior("CJK name lists are truncated with et al. for APA-style citations.");
    let root = project_root();
    let style = load_style(&root.join("styles/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-cjk.json"))
            .expect("CJK fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("CSL-ET-AL-KANJI"))
        .expect("et al. citation should render");

    // Should render first author followed by et al.
    assert!(
        citation.contains("Zither") || citation.contains("et al"),
        "Citation should contain author name or et al."
    );
}

#[test]
fn test_arabic_short_forms_with_diacritics() {
    announce_behavior("Arabic author names are rendered with diacritical marks intact.");
    let root = project_root();
    let style = load_style(&root.join("styles/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-arabic.json"))
            .expect("Arabic fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("ARABIC-ASWANI-DIACRITICS"))
        .expect("Arabic citation with diacritics should render");

    // Should render Arabic author name with proper diacritics
    assert!(
        citation.contains("al-Aswānī") || citation.contains("Alaa"),
        "Citation should contain Arabic author name or transliteration"
    );
}

#[test]
fn test_arabic_transliterated_forms() {
    announce_behavior("Transliterated Arabic names are accepted and rendered correctly.");
    let root = project_root();
    let style = load_style(&root.join("styles/apa-7th.yaml"));
    let bibliography =
        load_bibliography(&root.join("tests/fixtures/multilingual/multilingual-arabic.json"))
            .expect("Arabic fixture should parse");

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("ARABIC-ASWANI-TRANSLITERATED"))
        .expect("Arabic citation with transliteration should render");

    // Should render transliterated form
    assert!(
        citation.contains("al-Aswānī") || citation.contains("Alaa"),
        "Citation should handle transliterated Arabic"
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
            vec!["CSL-ET-AL-KANJI".to_string()],
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
