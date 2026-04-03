use super::*;

#[test]
fn test_parse_csl_json() {
    let json = r#"{
        "id": "kuhn1962",
        "type": "book",
        "author": [{"family": "Kuhn", "given": "Thomas S."}],
        "title": "The Structure of Scientific Revolutions",
        "issued": {"date-parts": [[1962]]},
        "publisher": "University of Chicago Press",
        "publisher-place": "Chicago"
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();
    assert_eq!(reference.id().unwrap(), "kuhn1962");
    assert_eq!(reference.ref_type(), "book");

    if let Some(Contributor::ContributorList(list)) = reference.author()
        && let Contributor::StructuredName(name) = &list.0[0]
    {
        assert_eq!(name.family, MultilingualString::Simple("Kuhn".to_string()));
    }
}

#[test]
fn test_parse_csl_json_mixed_string_date_parts() {
    let json = r#"{
        "id": "mixed-date",
        "type": "book",
        "title": "Mixed Date Parts",
        "issued": {"date-parts": [["2017", 2, 21]]},
        "publisher": "Example Press"
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    assert_eq!(reference.issued().unwrap().0, "2017-02-21");
}

#[test]
fn test_parse_csl_json_named_season() {
    let json = r#"{
        "id": "season-date",
        "type": "article-journal",
        "title": "Seasonal Issue",
        "issued": {"date-parts": [[2024]], "season": "Autumn"}
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();

    assert_eq!(legacy.issued.unwrap().season, Some(3));
}

#[test]
fn test_parse_csl_bill_record_prefers_container_title_as_title() {
    let json = r#"{
        "id": "bill-record",
        "type": "bill",
        "container-title": "Cong. Rec.",
        "volume": "147",
        "page": "19000",
        "number": "438",
        "issued": {"date-parts": [[2001]]}
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    match reference {
        InputReference::Monograph(monograph) => {
            assert_eq!(monograph.r#type, MonographType::Document);
            assert_eq!(monograph.genre.as_deref(), Some("bill-record"));
            assert_eq!(
                monograph.title,
                Some(Title::Single("Cong. Rec.".to_string()))
            );
            assert_eq!(monograph.number.as_deref(), Some("19000"));
            assert_eq!(monograph.volume.as_deref(), Some("147"));
        }
        other => panic!("expected monograph, got {:?}", other),
    }
}

#[test]
fn test_parse_csl_bill_proceeding_uses_number_as_surrogate_title() {
    let json = r#"{
        "id": "bill-proceeding",
        "type": "bill",
        "authority": "34th Cong.",
        "chapter-number": "3d Sess.",
        "number": "149",
        "issued": {"date-parts": [[1856]]}
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    match reference {
        InputReference::Monograph(monograph) => {
            assert_eq!(monograph.r#type, MonographType::Document);
            assert_eq!(monograph.genre.as_deref(), Some("bill-proceeding"));
            assert_eq!(monograph.title, Some(Title::Single("149".to_string())));
        }
        other => panic!("expected monograph, got {:?}", other),
    }
}

#[test]
fn conversion_applies_note_type_override() {
    let json = r#"{
        "id": "note-type-override",
        "type": "book",
        "note": "type: webpage"
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    assert_eq!(reference.ref_type(), "webpage");
}

#[test]
fn conversion_promotes_genre_and_preserves_free_text() {
    let json = r#"{
        "id": "note-genre",
        "type": "book",
        "note": "genre: H.R.\nReferenced via legacy note field"
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    assert_eq!(reference.genre(), Some("h.r.".to_string()));
    assert_eq!(
        reference.note(),
        Some("Referenced via legacy note field".to_string())
    );
}

#[test]
fn conversion_preserves_pre_existing_fields() {
    let json = r#"{
        "id": "note-publisher",
        "type": "book",
        "publisher": "Old Press",
        "note": "publisher: New Publisher\ntype: manual"
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    assert_eq!(reference.ref_type(), "manual");
    assert_eq!(reference.publisher_str(), Some("Old Press".to_string()));
    assert_eq!(reference.note(), None);
}
