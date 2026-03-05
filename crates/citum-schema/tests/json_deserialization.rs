use citum_schema::InputBibliography;
use citum_schema::reference::{InputReference, Monograph};

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
