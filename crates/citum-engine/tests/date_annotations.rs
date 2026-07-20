/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Integration coverage for the opaque date `note` (calendar-date
//! annotation) feature — `docs/specs/CALENDAR_DATE_ANNOTATIONS.md`. Exercises
//! the embedded GB/T 7714—2025 author-date style end to end: bibliography
//! renders the wrapped note, citations do not.

#![allow(missing_docs, reason = "test")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Panicking is acceptable and often desired in test code."
)]

mod common;
use common::announce_behavior;

use citum_engine::Processor;
use citum_schema::citation::{Citation, CitationItem};
use citum_schema::reference::{
    Contributor, ContributorEntry, ContributorRole, DateValue, InputReference, Monograph,
    MonographType, StructuredName, Title,
};
use indexmap::IndexMap;

/// Build a minimal book reference with an annotated `issued` date.
fn annotated_book(id: &str, title: &str, year: &str, note: &str) -> InputReference {
    InputReference::Monograph(Box::new(Monograph {
        id: Some(id.into()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title.to_string())),
        issued: DateValue {
            value: year.to_string(),
            note: Some(note.to_string()),
        },
        ..Default::default()
    }))
}

/// Same as `annotated_book`, with a shared author for disambiguation tests.
fn annotated_book_by(
    id: &str,
    title: &str,
    author_family: &str,
    year: &str,
    note: &str,
) -> InputReference {
    InputReference::Monograph(Box::new(Monograph {
        id: Some(id.into()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title.to_string())),
        contributors: vec![ContributorEntry {
            roles: ContributorRole::Author.into(),
            contributor: Contributor::StructuredName(StructuredName {
                family: author_family.into(),
                ..Default::default()
            }),
            gender: None,
        }],
        issued: DateValue {
            value: year.to_string(),
            note: Some(note.to_string()),
        },
        ..Default::default()
    }))
}

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
fn given_gb_t_author_date_style_when_rendering_bibliography_then_calendar_note_is_wrapped() {
    announce_behavior(
        "GB/T 7714 author-date wraps an issued date's opaque note (e.g. a Minguo \
         calendar annotation) in the bibliography, per the style's `note-wrap: \
         parentheses` bibliography-scoped `dates` option — csl26-epu6.",
    );
    let style = citum_schema::embedded::get_embedded_style("gb-t-7714-2025-author-date")
        .expect("gb-t-7714-2025-author-date should be embedded")
        .expect("gb-t-7714-2025-author-date should parse")
        .into_resolved();
    let bibliography = IndexMap::from([(
        "minguo-1947".to_string(),
        annotated_book("minguo-1947", "戰後臺灣史", "1947", "民国三十六年"),
    )]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(rendered, "[1]戰後臺灣史. [M]. 1947（民国三十六年）");
}

#[test]
fn given_gb_t_author_date_style_when_rendering_citation_then_calendar_note_is_absent() {
    announce_behavior(
        "GB/T 7714 author-date citations carry no `note-wrap` option (it is set only \
         under bibliography.options.dates), so an annotated date's note never appears \
         in citation output — bibliography-only scoping, csl26-epu6.",
    );
    let style = citum_schema::embedded::get_embedded_style("gb-t-7714-2025-author-date")
        .expect("gb-t-7714-2025-author-date should be embedded")
        .expect("gb-t-7714-2025-author-date should parse")
        .into_resolved();
    let bibliography = IndexMap::from([(
        "minguo-1947".to_string(),
        annotated_book("minguo-1947", "戰後臺灣史", "1947", "民国三十六年"),
    )]);

    let processor = Processor::new(style, bibliography);
    let citation = processor
        .process_citation(&single_item_citation("minguo-1947"))
        .expect("annotated-date citation should render");

    assert_eq!(citation, "（“戰後臺灣史”，1947）");
}

#[test]
fn given_two_refs_same_author_year_different_note_when_disambiguating_then_still_collide() {
    announce_behavior(
        "A date's opaque note never enters author-date collision grouping — two \
         references by the same author with the same issued year but different \
         calendar notes still collide and receive year-suffix disambiguators, exactly \
         as if neither carried a note. `year()`/`effective_issued_date` read only \
         `value`; `note` is invisible to `build_group_key` — csl26-epu6.",
    );
    let style = citum_schema::embedded::get_embedded_style("gb-t-7714-2025-author-date")
        .expect("gb-t-7714-2025-author-date should be embedded")
        .expect("gb-t-7714-2025-author-date should parse")
        .into_resolved();
    let bibliography = IndexMap::from([
        (
            "kang-a".to_string(),
            annotated_book_by("kang-a", "First Work", "Kang", "1947", "民国三十六年"),
        ),
        (
            "kang-b".to_string(),
            annotated_book_by("kang-b", "Second Work", "Kang", "1947", "不同的注释"),
        ),
    ]);

    let processor = Processor::new(style, bibliography);
    let rendered = processor.render_bibliography();

    assert_eq!(
        rendered,
        "[1]Kang. First Work[M]. 1947a（民国三十六年）\n\n\
         [2]Kang. Second Work[M]. 1947b（不同的注释）"
    );
}
