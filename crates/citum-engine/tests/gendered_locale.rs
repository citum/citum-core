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
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in test, benchmark, and example code."
)]

//! Snapshot coverage for gendered locale term resolution.
//!
//! These tests exercise `MaybeGendered<T>` dispatch on role labels and locator
//! terms across French (fr-FR) and Arabic (ar-AR) locales, and confirm that
//! plain-string forms in existing locales are unaffected.

use citum_schema::citation::LocatorType;
use citum_schema::locale::{GrammaticalGender, Locale, TermForm};
use citum_schema::template::ContributorRole;

fn load_locale(tag: &str) -> Locale {
    let bytes = citum_schema::embedded::get_locale_bytes(tag)
        .unwrap_or_else(|| panic!("locale bytes not found: {tag}"));
    Locale::from_yaml_str(std::str::from_utf8(bytes).expect("invalid UTF-8"))
        .unwrap_or_else(|e| panic!("failed to parse {tag} locale: {e}"))
}

/// French editor role label dispatches masculine and feminine forms correctly.
#[test]
fn french_gendered_editor_role_label() {
    let locale = load_locale("fr-FR");

    assert_eq!(
        locale.resolved_role_term(
            &ContributorRole::Editor,
            false,
            &TermForm::Long,
            Some(GrammaticalGender::Feminine)
        ),
        Some("éditrice".to_string()),
        "fr-FR singular feminine editor"
    );

    assert_eq!(
        locale.resolved_role_term(
            &ContributorRole::Editor,
            false,
            &TermForm::Long,
            Some(GrammaticalGender::Masculine)
        ),
        Some("éditeur".to_string()),
        "fr-FR singular masculine editor"
    );

    assert_eq!(
        locale.resolved_role_term(
            &ContributorRole::Editor,
            true,
            &TermForm::Long,
            Some(GrammaticalGender::Feminine)
        ),
        Some("éditrices".to_string()),
        "fr-FR plural feminine editor"
    );

    assert_eq!(
        locale.resolved_role_term(
            &ContributorRole::Editor,
            true,
            &TermForm::Long,
            Some(GrammaticalGender::Masculine)
        ),
        Some("éditeurs".to_string()),
        "fr-FR plural masculine editor"
    );
}

/// Arabic page locator term is present with feminine lexical gender, and
/// the MF2-driven editor role dispatches masculine/feminine/neutral forms.
#[test]
fn arabic_gendered_page_locator_and_role() {
    let locale = load_locale("ar-AR");

    // Page locator text
    assert_eq!(
        locale.resolved_locator_term(&LocatorType::Page, false, &TermForm::Long, None),
        Some("صفحة".to_string()),
        "ar-AR page singular"
    );
    assert_eq!(
        locale.resolved_locator_term(&LocatorType::Page, true, &TermForm::Long, None),
        Some("صفحات".to_string()),
        "ar-AR page plural"
    );

    // Lexical gender field on the locator term
    assert_eq!(
        locale.locators[&LocatorType::Page].gender,
        Some(GrammaticalGender::Feminine),
        "ar-AR page has feminine lexical gender"
    );

    // MF2-driven gendered editor role
    assert_eq!(
        locale.resolved_role_term(
            &ContributorRole::Editor,
            false,
            &TermForm::Long,
            Some(GrammaticalGender::Masculine)
        ),
        Some("مُحَرِّر".to_string()),
        "ar-AR masculine singular editor"
    );
    assert_eq!(
        locale.resolved_role_term(
            &ContributorRole::Editor,
            false,
            &TermForm::Long,
            Some(GrammaticalGender::Feminine)
        ),
        Some("مُحَرِّرَة".to_string()),
        "ar-AR feminine singular editor"
    );
    assert_eq!(
        locale.resolved_role_term_neutral(&ContributorRole::Editor, false, &TermForm::Long),
        Some("تحقيق".to_string()),
        "ar-AR neutral editor falls back to common/verbal noun"
    );
}

/// Existing plain-string locale fixtures resolve unchanged after adding gendered variants.
#[test]
fn plain_locale_fixtures_unchanged() {
    // fr-FR: no gender requested → common fallback from the new gendered editor
    let fr = load_locale("fr-FR");
    assert_eq!(
        fr.resolved_role_term(&ContributorRole::Editor, false, &TermForm::Long, None),
        Some("éditeur".to_string()),
        "fr-FR editor without gender request returns common fallback"
    );

    // en-US: page locator term is unchanged plain string
    let en = load_locale("en-US");
    assert_eq!(
        en.resolved_locator_term(&LocatorType::Page, false, &TermForm::Long, None),
        Some("page".to_string()),
        "en-US page locator unchanged"
    );
    assert_eq!(
        en.resolved_locator_term(&LocatorType::Page, true, &TermForm::Long, None),
        Some("pages".to_string()),
        "en-US pages (plural) unchanged"
    );
}
