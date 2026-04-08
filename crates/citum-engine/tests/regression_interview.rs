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
    let style = load_style("styles/apa-7th.yaml");

    // Create the interview reference using native Citum structs
    let reference = InputReference::Monograph(Box::new(Monograph {
        id: Some("sr-interview".to_string()),
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

    println!("Rendered output:\n{}", result);

    // APA expected output for interview (simplified check for now)
    // Arendt, H. (1975). _Thinking in Public_ (E. Young-Bruehl, Interviewer). Schocken Books.

    assert!(result.contains("Arendt, H."), "Author output incorrect");
    assert!(
        result.contains("(1975)"),
        "Date output missing or incorrect"
    );
    assert!(
        result.contains("Thinking in Public"),
        "Title output missing or incorrect"
    );
    assert!(
        result.contains("Young-Bruehl, E."),
        "Interviewer output missing or incorrect"
    );
    assert!(
        result.contains("Interviewer"),
        "Interviewer role label missing"
    );
    assert!(
        result.contains("Schocken Books"),
        "Publisher output missing or incorrect"
    );
}
