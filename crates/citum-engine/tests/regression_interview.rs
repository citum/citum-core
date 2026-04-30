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
#![allow(missing_docs, reason = "test")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod common;
use common::*;

use citum_engine::Processor;
use citum_schema::reference::{
    Contributor, ContributorEntry, ContributorRole, EdtfString, InputReference, Monograph,
    MonographType, Publisher, StructuredName, Title,
};
use indexmap::IndexMap;

#[test]
fn test_apa_interview_fidelity_regression() {
    let style = load_style("styles/embedded/apa-7th.yaml");

    // Create the interview reference using native Citum structs
    let reference = InputReference::Monograph(Box::new(Monograph {
        id: Some("sr-interview".into()),
        r#type: MonographType::Interview,
        title: Some(Title::Single("Thinking in Public".to_string())),
        author: Some(Contributor::StructuredName(StructuredName {
            family: "Arendt".into(),
            given: "Hannah".into(),
            ..Default::default()
        })),
        contributors: vec![ContributorEntry {
            role: ContributorRole::Interviewer,
            contributor: Contributor::StructuredName(StructuredName {
                family: "Young-Bruehl".into(),
                given: "Elisabeth".into(),
                ..Default::default()
            }),
            gender: None,
        }],
        issued: EdtfString("1975".to_string()),
        publisher: Some(Publisher {
            name: "Schocken Books".into(),
            place: None,
        }),
        ..Default::default()
    }));

    let mut bib = IndexMap::new();
    bib.insert("sr-interview".to_string(), reference);

    let processor = Processor::new(style, bib);
    let result = processor.render_bibliography();

    // APA expected output for interview
    assert_eq!(
        result,
        "Arendt, H. (1975). Thinking in Public (E. Young-Bruehl, Interviewer) Schocken Books."
    );
}
