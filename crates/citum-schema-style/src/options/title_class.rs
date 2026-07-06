/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Single source of truth for ref-type → title-category classification.
//!
//! Both `citum-engine` (title rendering defaults) and `citum-migrate` (SQI
//! pruning, which must be the exact inverse of engine defaulting) need this
//! table. It lives here, alongside `TitlesConfig`, so both crates consume
//! the same classification via the `citum-schema` facade rather than
//! maintaining separate copies that can silently drift apart.
//!
//! See `docs/specs/TYPE_CLASSIFICATION_CENTRALIZATION.md` for the original
//! design rationale (written when this lived in `citum-engine`) and
//! `docs/architecture/audits/2026-07-06_CITUM_MIGRATE_REVIEW.md` (Finding F1)
//! for why it moved here.

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
pub enum TitleCategory {
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
pub fn title_category(ref_type: &str) -> TitleCategory {
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
pub fn container_title_category(ref_type: &str) -> TitleCategory {
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
pub fn parent_serial_title_category(ref_type: &str) -> TitleCategory {
    match ref_type {
        "article-journal" | "article-magazine" | "article-newspaper" => TitleCategory::Periodical,
        _ => TitleCategory::Serial,
    }
}

/// Every reference type explicitly classified by at least one of
/// `title_category`, `container_title_category`, or
/// `parent_serial_title_category` (i.e. every type that does not simply take
/// each function's catch-all default).
///
/// Used to exhaustively exercise cross-crate agreement between engine
/// defaulting and SQI pruning inversion — see
/// `citum-migrate`'s `sqi_refinement` tests.
#[must_use]
pub fn classified_ref_types() -> &'static [&'static str] {
    &[
        "article-journal",
        "article-magazine",
        "article-newspaper",
        "chapter",
        "entry",
        "entry-dictionary",
        "entry-encyclopedia",
        "paper-conference",
        "post",
        "post-weblog",
        "book",
        "thesis",
        "report",
        "broadcast",
    ]
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
    fn given_classified_ref_types_when_categorized_then_none_panic() {
        // Sanity check: every enumerated type resolves through all three
        // functions without needing to touch new match arms.
        for ref_type in classified_ref_types() {
            let _ = title_category(ref_type);
            let _ = container_title_category(ref_type);
            let _ = parent_serial_title_category(ref_type);
        }
    }
}
