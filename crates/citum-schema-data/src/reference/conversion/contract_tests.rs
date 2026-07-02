/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Contract test: every CSL 1.0.2 item type in
//! [`csl_legacy::csl_json::CSL_TYPES`] converts to an `InputReference`
//! whose `ref_type()` is a faithful round trip, not a silent collapse into
//! the generic document/monograph fallback. See
//! `docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md` for the canonicalization
//! rules and the rationale behind each intentional divergence documented
//! inline in [`EXPECTATIONS`] below.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "Panicking is acceptable and often desired in tests."
)]

use super::*;
use csl_legacy::csl_json::CSL_TYPES;

/// Build the minimal legacy reference the contract test converts for a
/// given CSL type: an id, the type under test, a title, and an issued
/// year. This is deliberately the *smallest* shape a real CSL-JSON export
/// would carry. A type that needs more than this to avoid the generic
/// document/monograph fallback is either a genuine routing gap (fix the
/// converter, not this helper) or a documented, shape-dependent
/// divergence (see the comments on [`EXPECTATIONS`] and the spec).
fn minimal_reference(ref_type: &str) -> csl_legacy::csl_json::Reference {
    csl_legacy::csl_json::Reference {
        id: format!("contract-{ref_type}"),
        ref_type: ref_type.to_string(),
        title: Some("Contract Test Title".to_string()),
        issued: Some(csl_legacy::csl_json::DateVariable {
            date_parts: Some(vec![vec![2024]]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Expected `ref_type()` output for every CSL 1.0.2 type, given the
/// minimal shape [`minimal_reference`] builds. Most entries are the
/// identity mapping; the ones that are not are intentional and documented
/// inline (also recorded in
/// `docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md`'s canonicalization table).
const EXPECTATIONS: &[(&str, &str)] = &[
    // Bare `article` carries no container-title, so the converter treats
    // it as a standalone preprint rather than a truncated journal article
    // — this mirrors real-world CSL-JSON exports where a container-less
    // `article` is an arXiv/SSRN-style preprint. See `from_preprint_ref`.
    ("article", "preprint"),
    ("article-journal", "article-journal"),
    ("article-magazine", "article-magazine"),
    ("article-newspaper", "article-newspaper"),
    // A minimal `bill` (no `authority`, `chapter-number`, or
    // `container-title`/`volume`/`page` combination) carries none of the
    // shape signals `from_bill_ref` uses to distinguish a hearing,
    // bill-proceeding, or bill-record from a generic government
    // document. See `reference/tests.rs` for the shapes that *do*
    // round-trip distinctly (`test_parse_csl_bill_*`).
    ("bill", "document"),
    ("book", "book"),
    ("broadcast", "broadcast"),
    ("chapter", "chapter"),
    ("classic", "classic"),
    ("collection", "collection"),
    ("dataset", "dataset"),
    ("document", "document"),
    ("entry", "entry"),
    ("entry-dictionary", "entry-dictionary"),
    ("entry-encyclopedia", "entry-encyclopedia"),
    ("event", "event"),
    ("figure", "figure"),
    ("graphic", "graphic"),
    ("hearing", "hearing"),
    ("interview", "interview"),
    // `legal_case` (the CSL 1.0.2 spelling) canonicalizes to the
    // hyphenated `legal-case` on output, matching this codebase's
    // convention of canonicalizing underscore CSL spellings to hyphens
    // (see also `motion_picture`, `musical_score`,
    // `personal_communication`).
    ("legal_case", "legal-case"),
    // `legislation` is the CSL 1.0.2 closed-vocabulary type; it routes to
    // the same converter as the `statute` extension spelling and shares
    // its canonical output.
    ("legislation", "statute"),
    ("manuscript", "manuscript"),
    ("map", "map"),
    ("motion_picture", "motion-picture"),
    ("musical_score", "musical-score"),
    ("pamphlet", "pamphlet"),
    ("paper-conference", "paper-conference"),
    ("patent", "patent"),
    ("performance", "performance"),
    ("periodical", "periodical"),
    ("personal_communication", "personal-communication"),
    ("post", "post"),
    ("post-weblog", "post-weblog"),
    ("regulation", "regulation"),
    ("report", "report"),
    ("review", "review"),
    ("review-book", "review-book"),
    ("software", "software"),
    ("song", "song"),
    ("speech", "speech"),
    ("standard", "standard"),
    ("thesis", "thesis"),
    ("treaty", "treaty"),
    ("webpage", "webpage"),
];

#[test]
fn every_csl_1_0_2_type_has_an_expectation_table_entry() {
    for csl_type in CSL_TYPES {
        assert!(
            EXPECTATIONS.iter().any(|(input, _)| input == csl_type),
            "CSL_TYPES entry `{csl_type}` has no entry in the contract test's \
             EXPECTATIONS table; every CSL 1.0.2 type must be covered"
        );
    }
    assert_eq!(
        EXPECTATIONS.len(),
        CSL_TYPES.len(),
        "EXPECTATIONS table size has drifted from CSL_TYPES; add or remove \
         an entry so the two stay in lockstep"
    );
}

#[test]
fn every_csl_1_0_2_type_round_trips_through_ref_type() {
    // Collect every mismatch instead of stopping at the first one: a
    // routing regression usually breaks more than one type at a time, and
    // seeing the whole set in one run is what turns this into a fast
    // diagnostic instead of a bisection exercise (see the epic's problem
    // statement in bean csl26-cvfy).
    let failures: Vec<String> = EXPECTATIONS
        .iter()
        .filter_map(|(csl_type, expected)| {
            let legacy = minimal_reference(csl_type);
            let actual = InputReference::from(legacy).ref_type();
            (&actual != expected)
                .then(|| format!("{csl_type}: expected `{expected}`, got `{actual}`"))
        })
        .collect();

    assert!(
        failures.is_empty(),
        "CSL type round-trip mismatches (see docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md):\n{}",
        failures.join("\n")
    );
}

/// Regression test for the `TLIB-SEL-MAP-1` fixture case
/// (`tests/fixtures/references-expanded.json`): a `map` reference whose
/// user-supplied genre uses a capitalized export label (`"Map"`, as
/// Zotero emits) must still round-trip to the canonical lowercase `map`,
/// not fall through the genre back-map into `document`. The back-map has
/// matched genre case-insensitively since before the routing closure;
/// this pins that behavior for all genre-discriminated document types.
#[test]
fn map_with_capitalized_export_genre_still_round_trips_as_map() {
    let mut legacy = minimal_reference("map");
    legacy.genre = Some("Map".to_string());

    let converted = InputReference::from(legacy);

    assert_eq!(converted.ref_type(), "map");
}

/// Regression test for the `chi-manuscript` fixture case (bean
/// `csl26-shco`): a reference with top-level `"type": "manuscript"` and a
/// note-field override `"type: collection"` must round-trip as
/// `collection`, not silently collapse into the generic `document`
/// fallback. CSL 1.0.2's `collection` is an *archival* collection
/// (author, archive, archive-place), so it converts to the archival
/// monograph/document shape — which carries those fields — with a
/// genre-discriminated round trip, not to the editorial
/// `ClassExtension::Collection` (anthology/proceedings), which has no
/// author or archive fields and would drop them. See
/// `docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md`.
#[test]
fn manuscript_with_recognized_collection_note_override_converts_to_collection() {
    let mut legacy = minimal_reference("manuscript");
    legacy.note = Some("type: collection".to_string());
    legacy.archive = Some("University of Georgia Library".to_string());
    legacy.parse_note_field_hacks();

    assert_eq!(legacy.ref_type, "collection");

    let converted = InputReference::from(legacy);

    assert_eq!(converted.ref_type(), "collection");
    let crate::reference::ClassExtension::Monograph(monograph) = converted.extension() else {
        panic!(
            "expected the archival Monograph shape for a `collection` conversion, got `{}`",
            converted.ref_type()
        );
    };
    assert_eq!(
        monograph.archive.as_deref(),
        Some("University of Georgia Library"),
        "archival fields must survive the collection conversion"
    );
}
