#![allow(missing_docs, reason = "test/bench/bin crate")]

use citum_schema_style::citation::{CitationItem, IntegralNameState};
use citum_schema_style::options::{IntegralNameContexts, IntegralNameRule, IntegralNameScope};
use citum_schema_style::reference::{InputReference, Monograph};
use citum_schema_style::{InputBibliography, Style};

#[test]
fn test_monograph_doi_alias() {
    let json = r#"{
        "type": "book",
        "title": "Test Book",
        "issued": "2023",
        "DOI": "10.1001/test"
    }"#;
    let monograph: Monograph = serde_json::from_str(json).unwrap();
    assert_eq!(monograph.doi, Some("10.1001/test".to_string()));
}

#[test]
fn test_input_reference_doi_alias() {
    let json = r#"{
        "class": "monograph",
        "type": "book",
        "title": "Test Book",
        "issued": "2023",
        "DOI": "10.1001/test"
    }"#;
    let reference: InputReference = serde_json::from_str(json).unwrap();
    if let InputReference::Monograph(m) = reference {
        assert_eq!(m.doi, Some("10.1001/test".to_string()));
    } else {
        panic!("Expected Monograph");
    }
}

#[test]
fn test_input_reference_url_alias() {
    let json = r#"{
        "class": "monograph",
        "type": "book",
        "title": "Test Book",
        "issued": "2023",
        "URL": "https://example.com"
    }"#;
    let reference: InputReference = serde_json::from_str(json).unwrap();
    if let InputReference::Monograph(m) = reference {
        assert_eq!(m.url.unwrap().to_string(), "https://example.com/");
    } else {
        panic!("Expected Monograph");
    }
}

#[test]
fn test_input_bibliography_sets_round_trip() {
    let json = r#"{
        "references": [
            {
                "class": "monograph",
                "id": "ref-a",
                "type": "book",
                "title": "Book A",
                "issued": "2020"
            },
            {
                "class": "monograph",
                "id": "ref-b",
                "type": "book",
                "title": "Book B",
                "issued": "2021"
            }
        ],
        "sets": {
            "compound-1": ["ref-a", "ref-b"]
        }
    }"#;

    let bibliography: InputBibliography =
        serde_json::from_str(json).expect("input bibliography should parse");
    let sets = bibliography.sets.as_ref().expect("sets should exist");
    assert_eq!(
        sets.get("compound-1"),
        Some(&vec!["ref-a".to_string(), "ref-b".to_string()])
    );

    let serialized = serde_json::to_string(&bibliography).expect("serialization should work");
    let reparsed: InputBibliography =
        serde_json::from_str(&serialized).expect("round-trip parse should work");
    assert_eq!(reparsed.sets, bibliography.sets);
}

#[test]
fn test_style_integral_names_round_trip() {
    let json = r#"{
        "version": "0.8.0",
        "info": { "title": "Test Style" },
        "options": {
            "integral-names": {
                "rule": "full-then-short",
                "scope": "chapter",
                "contexts": "body-and-notes",
                "subsequent-form": "short"
            }
        }
    }"#;

    let style: Style = serde_json::from_str(json).expect("style should parse");
    let config = style
        .options
        .as_ref()
        .and_then(|options| options.integral_names.as_ref())
        .expect("integral-names should exist");
    assert_eq!(config.rule, Some(IntegralNameRule::FullThenShort));
    assert_eq!(config.scope, Some(IntegralNameScope::Chapter));
    assert_eq!(config.contexts, Some(IntegralNameContexts::BodyAndNotes));

    let serialized = serde_json::to_string(&style).expect("style should serialize");
    let reparsed: Style = serde_json::from_str(&serialized).expect("style should round-trip");
    assert!(
        reparsed
            .options
            .and_then(|options| options.integral_names)
            .is_some()
    );
}

#[test]
fn test_citation_item_integral_name_state_round_trip() {
    let json = r#"{
        "id": "item1",
        "integral-name-state": "subsequent"
    }"#;

    let item: CitationItem = serde_json::from_str(json).expect("citation item should parse");
    assert_eq!(
        item.integral_name_state,
        Some(IntegralNameState::Subsequent)
    );

    let serialized = serde_json::to_string(&item).expect("citation item should serialize");
    let reparsed: CitationItem =
        serde_json::from_str(&serialized).expect("citation item should round-trip");
    assert_eq!(
        reparsed.integral_name_state,
        Some(IntegralNameState::Subsequent)
    );
}
