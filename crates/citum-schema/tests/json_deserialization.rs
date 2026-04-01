#![allow(missing_docs, reason = "test/bench")]

use citum_schema::citation::{CitationItem, IntegralNameState};
use citum_schema::options::{IntegralNameContexts, IntegralNameRule, IntegralNameScope};
use citum_schema::reference::{InputReference, Monograph, NumberingType};
use citum_schema::{InputBibliography, Style};

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
fn test_input_reference_archive_info_and_eprint_fields() {
    let json = r#"{
        "class": "monograph",
        "type": "book",
        "title": "Test Book",
        "issued": "2023",
        "archive-info": {
            "name": "Houghton Library",
            "location": "Box 14, Folder 3",
            "url": "https://example.com/archive"
        },
        "eprint": {
            "server": "arXiv",
            "id": "2301.00001",
            "class": "cs.AI"
        }
    }"#;

    let reference: InputReference = serde_json::from_str(json).expect("reference should parse");
    if let InputReference::Monograph(m) = reference {
        let archive_info = m.archive_info.expect("archive info should exist");
        assert_eq!(
            archive_info
                .name
                .expect("archive name should exist")
                .to_string(),
            "Houghton Library"
        );
        assert_eq!(archive_info.location, Some("Box 14, Folder 3".to_string()));
        assert_eq!(
            archive_info
                .url
                .expect("archive url should exist")
                .to_string(),
            "https://example.com/archive"
        );

        let eprint = m.eprint.expect("eprint should exist");
        assert_eq!(eprint.server, "arXiv");
        assert_eq!(eprint.id, "2301.00001");
        assert_eq!(eprint.class, Some("cs.AI".to_string()));
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

#[test]
fn test_monograph_shorthand_numbering_normalizes_on_deserialize() {
    let json = r#"{
        "type": "book",
        "title": "Normalized Numbering",
        "issued": "2023",
        "volume": "12",
        "issue": "4",
        "edition": "2",
        "number": "7",
        "numbering": [
            { "type": "chapter", "value": "9" }
        ]
    }"#;

    let monograph: Monograph = serde_json::from_str(json).expect("monograph should parse");

    assert!(monograph.volume.is_none());
    assert!(monograph.issue.is_none());
    assert!(monograph.edition.is_none());
    assert!(monograph.number.is_none());
    assert_eq!(monograph.numbering.len(), 5);
    assert_eq!(monograph.numbering[0].r#type, NumberingType::Volume);
    assert_eq!(monograph.numbering[0].value, "12");
    assert_eq!(monograph.numbering[1].r#type, NumberingType::Issue);
    assert_eq!(monograph.numbering[1].value, "4");
    assert_eq!(monograph.numbering[2].r#type, NumberingType::Edition);
    assert_eq!(monograph.numbering[2].value, "2");
    assert_eq!(monograph.numbering[3].r#type, NumberingType::Number);
    assert_eq!(monograph.numbering[3].value, "7");
    assert_eq!(monograph.numbering[4].r#type, NumberingType::Chapter);
    assert_eq!(monograph.numbering[4].value, "9");
}

#[test]
fn test_monograph_numbering_only_deserialize_preserves_entries() {
    let json = r#"{
        "type": "book",
        "title": "Numbering Only",
        "issued": "2023",
        "numbering": [
            { "type": "volume", "value": "3" },
            { "type": "chapter", "value": "11" }
        ]
    }"#;

    let monograph: Monograph = serde_json::from_str(json).expect("monograph should parse");

    assert_eq!(monograph.numbering.len(), 2);
    assert_eq!(monograph.numbering[0].r#type, NumberingType::Volume);
    assert_eq!(monograph.numbering[0].value, "3");
    assert_eq!(monograph.numbering[1].r#type, NumberingType::Chapter);
    assert_eq!(monograph.numbering[1].value, "11");
}

#[test]
fn test_monograph_shorthand_overrides_conflicting_numbering_entries() {
    let json = r#"{
        "type": "book",
        "title": "Conflict",
        "issued": "2023",
        "volume": "12",
        "number": "5",
        "numbering": [
            { "type": "volume", "value": "1" },
            { "type": "number", "value": "2" },
            { "type": "supplement", "value": "A" }
        ]
    }"#;

    let monograph: Monograph = serde_json::from_str(json).expect("monograph should parse");

    assert_eq!(monograph.numbering.len(), 3);
    assert_eq!(monograph.numbering[0].r#type, NumberingType::Volume);
    assert_eq!(monograph.numbering[0].value, "12");
    assert_eq!(monograph.numbering[1].r#type, NumberingType::Number);
    assert_eq!(monograph.numbering[1].value, "5");
    assert_eq!(monograph.numbering[2].r#type, NumberingType::Supplement);
    assert_eq!(monograph.numbering[2].value, "A");
}

#[test]
fn test_monograph_report_numbering_deserializes_with_report_type() {
    let json = r#"{
        "type": "report",
        "title": "Report",
        "issued": "2023",
        "numbering": [
            { "type": "report", "value": "TR-7" }
        ]
    }"#;

    let monograph: Monograph = serde_json::from_str(json).expect("monograph should parse");

    assert_eq!(monograph.numbering.len(), 1);
    assert_eq!(monograph.numbering[0].r#type, NumberingType::Report);
    assert_eq!(monograph.numbering[0].value, "TR-7");
}

#[test]
fn test_monograph_custom_numbering_deserializes_with_custom_type() {
    let json = r#"{
        "type": "book",
        "title": "Custom Numbering",
        "issued": "2023",
        "numbering": [
            { "type": "movement", "value": "III" }
        ]
    }"#;

    let monograph: Monograph = serde_json::from_str(json).expect("custom numbering should parse");

    assert_eq!(monograph.numbering.len(), 1);
    assert_eq!(
        monograph.numbering[0].r#type,
        NumberingType::Custom("movement".to_string())
    );
    assert_eq!(monograph.numbering[0].value, "III");
}

#[test]
fn test_input_reference_round_trip_serializes_custom_numbering_type_as_plain_string() {
    let json = r#"{
        "class": "monograph",
        "type": "book",
        "title": "Custom Canonical",
        "issued": "2023",
        "numbering": [
            { "type": "Movement", "value": "II" }
        ]
    }"#;

    let reference: InputReference = serde_json::from_str(json).expect("reference should parse");
    let serialized = serde_json::to_value(&reference).expect("serialization should work");
    let numbering = serialized
        .get("numbering")
        .and_then(serde_json::Value::as_array)
        .expect("custom numbering should serialize");

    assert_eq!(numbering[0]["type"], "movement");
    assert_eq!(numbering[0]["value"], "II");
}

#[test]
fn test_input_reference_round_trip_serializes_canonical_numbering_only() {
    let json = r#"{
        "class": "monograph",
        "type": "book",
        "title": "Canonical",
        "issued": "2023",
        "volume": "6",
        "issue": "2",
        "numbering": [
            { "type": "volume", "value": "1" },
            { "type": "chapter", "value": "4" }
        ]
    }"#;

    let reference: InputReference = serde_json::from_str(json).expect("reference should parse");
    let serialized = serde_json::to_value(&reference).expect("serialization should work");

    assert!(serialized.get("volume").is_none());
    assert!(serialized.get("issue").is_none());
    let numbering = serialized
        .get("numbering")
        .and_then(serde_json::Value::as_array)
        .expect("canonical numbering should serialize");
    assert_eq!(numbering.len(), 3);
    assert_eq!(numbering[0]["type"], "volume");
    assert_eq!(numbering[0]["value"], "6");
    assert_eq!(numbering[1]["type"], "issue");
    assert_eq!(numbering[1]["value"], "2");
    assert_eq!(numbering[2]["type"], "chapter");
    assert_eq!(numbering[2]["value"], "4");
}
