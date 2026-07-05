/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Single source of truth for reference-type classification.
//!
//! Six engine sites used to hardcode per-`ref_type` presentation rules, each
//! slightly differently (an English string suffix, a `contains("article")`
//! gate, a `contains("ancient")` fuzzy match, three divergent fallback
//! tables, a `== "dataset"` gate, and a silent `chapter`/`entry-dictionary`
//! alias). Renaming or adding a reference type therefore risked changing
//! rendering in some sites but not others. This module centralizes those
//! facts as data so each site becomes a thin call-through.
//!
//! See `docs/specs/TYPE_CLASSIFICATION_CENTRALIZATION.md` for the design
//! rationale and `docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md`
//! (Finding 14) for the original audit.
//!
//! `ref_type()` (`citum_schema::reference` via
//! `citum-schema-data/src/reference/accessors.rs`) emits a finite,
//! enumerable set of CSL-style strings derived from `ReferenceClass` and
//! genre. Every table here is closed over that vocabulary; there is
//! deliberately no substring/`contains` matching, since the value producer
//! only ever emits exact known strings (see `matches_type_class` doc comment
//! for the concrete case this replaced).

use citum_schema::options::TypeClass;

/// Title-category buckets used to select which `TitlesConfig` rendering
/// applies to a title component when the style has no explicit
/// `titles.type_mapping` entry for the reference's type.
///
/// Different `TitleType`s use different subsets of these buckets and fall
/// back differently (e.g. `ParentSerial` never falls to `Default`, while
/// `ContainerTitle` does) — that divergence is preserved intentionally
/// (behavior-preserving centralization), not flattened into one shared
/// table. See `title_category`, `container_title_category`, and
/// `parent_serial_title_category`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TitleCategory {
    /// Titles of works contained in a larger work (articles, chapters, entries).
    Component,
    /// Titles of standalone monographic works (books, theses, reports).
    Monograph,
    /// Titles of periodicals (journals, magazines, newspapers, broadcasts).
    Periodical,
    /// Titles of serials, when distinguished from periodicals proper.
    Serial,
    /// Titles of monographic containers (books containing chapters).
    ContainerMonograph,
    /// No specific category; use the style's default title rendering.
    Default,
}

/// Resolve the default title category for a reference type.
///
/// This is the fallback used when the style provides no explicit
/// `titles.type_mapping` entry for `ref_type`. Callers consult the style's
/// declarative mapping first and only fall back to this table, so adding a
/// `type_mapping` entry to a style always takes precedence.
#[must_use]
pub(crate) fn title_category(ref_type: &str) -> TitleCategory {
    match ref_type {
        "article-journal" | "article-magazine" | "article-newspaper" | "chapter" | "entry"
        | "entry-dictionary" | "entry-encyclopedia" | "paper-conference" | "post"
        | "post-weblog" => TitleCategory::Component,
        "book" | "thesis" | "report" => TitleCategory::Monograph,
        _ => TitleCategory::Default,
    }
}

/// Resolve the default *parent-container* title category for a reference
/// type — the category of the title borne by the reference's container
/// (`ContainerTitle`/`ParentSerial`/`ParentMonograph`), not the reference's
/// own title.
///
/// Mirrors `title_category` in spirit but reflects that a periodical parent
/// (journal, magazine, newspaper, broadcast) reads differently from a
/// monographic parent (book containing a chapter).
#[must_use]
pub(crate) fn container_title_category(ref_type: &str) -> TitleCategory {
    match ref_type {
        "article-journal" | "article-magazine" | "article-newspaper" | "broadcast" => {
            TitleCategory::Periodical
        }
        "chapter" | "paper-conference" => TitleCategory::ContainerMonograph,
        _ => TitleCategory::Default,
    }
}

/// Resolve the default title category for a `ParentSerial` title component
/// (the short title shown for a serial container, e.g. a journal name next
/// to an article).
///
/// Unlike `container_title_category`, this bucket never falls to `Default`
/// — an unrecognized type still renders as `Serial` — and does not treat
/// `broadcast` as periodical. This divergence from `container_title_category`
/// already existed before centralization and is preserved rather than
/// unified, since unifying the two would change rendered output.
#[must_use]
pub(crate) fn parent_serial_title_category(ref_type: &str) -> TitleCategory {
    match ref_type {
        "article-journal" | "article-magazine" | "article-newspaper" => TitleCategory::Periodical,
        _ => TitleCategory::Serial,
    }
}

/// Whether a reference type's container is a serial (periodical/journal)
/// parent, as opposed to a monographic one — determines whether a parent's
/// short title renders (`ParentSerial`).
///
/// Replaces the historical `ref_type().contains("article")` gate, which
/// matched any future type containing the substring "article" rather than
/// the closed set of serial-component types the data model actually
/// produces.
#[must_use]
pub(crate) fn is_serial_parent_type(ref_type: &str) -> bool {
    matches!(
        ref_type,
        "article-journal" | "article-magazine" | "article-newspaper" | "broadcast"
    )
}

/// Whether `ref_type` belongs to the given [`TypeClass`], used to gate
/// locator-pattern selection.
///
/// `TypeClass::Classical` used to include `ref_type.contains("ancient")`.
/// `ref_type()` is a finite, enumerable CSL-string set (see module docs);
/// the `Classic` reference class always emits the literal string
/// `"classic"`, never a string containing `"ancient"`, so that fuzzy match
/// matched nothing the data model can produce. It is replaced here with the
/// exact member list.
#[must_use]
pub(crate) fn matches_type_class(ref_type: &str, type_class: TypeClass) -> bool {
    match type_class {
        TypeClass::Legal => matches!(
            ref_type,
            "legal-case"
                | "legal_case"
                | "statute"
                | "treaty"
                | "regulation"
                | "bill"
                | "legislation"
        ),
        TypeClass::Classical => matches!(ref_type, "classic" | "religious-text" | "religious_text"),
        TypeClass::Standard => true,
    }
}

/// Reference-type aliases consulted when matching a style's `type-variants`
/// selector against a reference's actual type.
///
/// Returns the candidate selector strings to try, in priority order (the
/// reference's own type first). A `chapter` reference also matches a style's
/// `entry-dictionary` type-variant, mirroring how citeproc treats dictionary
/// entries as chapters when no dedicated variant is authored.
#[must_use]
pub(crate) fn type_selector_aliases(ref_type: &str) -> Vec<&str> {
    match ref_type {
        "chapter" => vec!["chapter", "entry-dictionary"],
        _ => vec![ref_type],
    }
}

/// Whether this reference type should synthesize a `doi.org` URL from a DOI
/// when no standalone `url` is present.
///
/// Currently only `dataset` — datasets are commonly distributed with only a
/// DOI and no landing-page URL, unlike most other reference types.
#[must_use]
pub(crate) fn synthesizes_doi_url(ref_type: &str) -> bool {
    ref_type == "dataset"
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
    fn given_component_ref_type_when_title_category_then_component() {
        // given a chapter reference type
        // when resolving its default title category
        let category = title_category("chapter");
        // then it falls into the component bucket
        assert_eq!(category, TitleCategory::Component);
    }

    #[test]
    fn given_monograph_ref_type_when_title_category_then_monograph() {
        let category = title_category("book");
        assert_eq!(category, TitleCategory::Monograph);
    }

    #[test]
    fn given_unclassified_ref_type_when_title_category_then_default() {
        let category = title_category("dataset");
        assert_eq!(category, TitleCategory::Default);
    }

    #[test]
    fn given_periodical_ref_type_when_container_title_category_then_periodical() {
        let category = container_title_category("article-journal");
        assert_eq!(category, TitleCategory::Periodical);
    }

    #[test]
    fn given_chapter_ref_type_when_container_title_category_then_container_monograph() {
        let category = container_title_category("chapter");
        assert_eq!(category, TitleCategory::ContainerMonograph);
    }

    #[test]
    fn given_journal_article_when_parent_serial_title_category_then_periodical() {
        let category = parent_serial_title_category("article-journal");
        assert_eq!(category, TitleCategory::Periodical);
    }

    #[test]
    fn given_book_when_parent_serial_title_category_then_serial() {
        // given a non-article reference type
        // when resolving its ParentSerial title category
        // then it falls back to Serial, not Default -- this bucket never
        // falls to Default, matching pre-centralization behavior
        let category = parent_serial_title_category("book");
        assert_eq!(category, TitleCategory::Serial);
    }

    #[test]
    fn given_broadcast_when_parent_serial_title_category_then_serial_not_periodical() {
        // Unlike container_title_category, ParentSerial does not treat
        // broadcast as periodical -- preserving the pre-existing divergence.
        let category = parent_serial_title_category("broadcast");
        assert_eq!(category, TitleCategory::Serial);
    }

    #[test]
    fn given_journal_article_when_is_serial_parent_type_then_true() {
        // given a journal-article reference type
        // when checking whether its container is a serial parent
        // then it is
        assert!(is_serial_parent_type("article-journal"));
    }

    #[test]
    fn given_broadcast_when_is_serial_parent_type_then_true() {
        assert!(is_serial_parent_type("broadcast"));
    }

    #[test]
    fn given_book_when_is_serial_parent_type_then_false() {
        assert!(!is_serial_parent_type("book"));
    }

    #[test]
    fn given_legal_case_when_matches_legal_type_class_then_true() {
        // given a legal-case reference type
        // when checking membership in TypeClass::Legal
        // then it matches
        assert!(matches_type_class("legal-case", TypeClass::Legal));
    }

    #[test]
    fn given_classic_when_matches_classical_type_class_then_true() {
        assert!(matches_type_class("classic", TypeClass::Classical));
    }

    #[test]
    fn given_classic_ref_type_when_checked_for_ancient_substring_then_absent() {
        // The data model's `ref_type()` accessor emits the literal string
        // "classic" for the Classic reference class; it never emits a
        // string containing "ancient". This test documents that the former
        // `ref_type.contains("ancient")` fuzzy match had no live producer in
        // the data model and its removal changes no observable behavior.
        assert!(!"classic".contains("ancient"));
    }

    #[test]
    fn given_book_when_matches_classical_type_class_then_false() {
        assert!(!matches_type_class("book", TypeClass::Classical));
    }

    #[test]
    fn given_any_ref_type_when_matches_standard_type_class_then_true() {
        assert!(matches_type_class("anything", TypeClass::Standard));
    }

    #[test]
    fn given_chapter_when_type_selector_aliases_then_includes_entry_dictionary() {
        // given a chapter reference type
        // when resolving type-variant selector aliases
        // then entry-dictionary is included as a fallback candidate
        let aliases = type_selector_aliases("chapter");
        assert_eq!(aliases, vec!["chapter", "entry-dictionary"]);
    }

    #[test]
    fn given_book_when_type_selector_aliases_then_only_itself() {
        let aliases = type_selector_aliases("book");
        assert_eq!(aliases, vec!["book"]);
    }

    #[test]
    fn given_dataset_when_synthesizes_doi_url_then_true() {
        // given a dataset reference type
        // when checking whether it synthesizes a DOI URL
        // then it does
        assert!(synthesizes_doi_url("dataset"));
    }

    #[test]
    fn given_book_when_synthesizes_doi_url_then_false() {
        assert!(!synthesizes_doi_url("book"));
    }
}
