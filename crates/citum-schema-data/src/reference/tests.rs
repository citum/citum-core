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
fn test_parse_csl_json_structural_author_populates_canonical_contributors() {
    let json = r#"{
        "id": "legacy-book",
        "type": "book",
        "author": [{"family": "Le Guin", "given": "Ursula"}],
        "title": "The Left Hand of Darkness",
        "issued": {"date-parts": [[1969]]}
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    assert!(reference.contributor(ContributorRole::Author).is_some());

    match reference {
        InputReference::Monograph(monograph) => assert!(
            monograph
                .contributors
                .iter()
                .any(|entry| entry.role == ContributorRole::Author),
            "legacy author should populate canonical contributors"
        ),
        other => panic!("expected monograph, got {:?}", other),
    }
}

#[test]
fn test_parse_csl_json_motion_picture_produces_audio_visual() {
    let json = r#"{
        "id": "parasite",
        "type": "motion_picture",
        "title": "Parasite",
        "director": [{"family": "Bong", "given": "Joon-ho"}],
        "issued": {"date-parts": [[2019]]}
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    match reference {
        InputReference::AudioVisual(work) => {
            assert_eq!(work.r#type, AudioVisualType::Film);
            assert!(
                work.core
                    .contributors
                    .iter()
                    .any(|entry| entry.role == ContributorRole::Director)
            );
        }
        other => panic!("expected audio-visual work, got {:?}", other),
    }
}

#[test]
fn test_parse_csl_json_broadcast_without_audio_roles_stays_serial_component() {
    let json = r#"{
        "id": "cosmos-episode",
        "type": "broadcast",
        "title": "The Universe in a Nutshell",
        "author": [{"family": "Sagan", "given": "Carl"}],
        "issued": {"date-parts": [[1980, 9, 28]]},
        "container-title": "Cosmos: A Spacetime Odyssey",
        "number": "1"
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    match reference {
        InputReference::SerialComponent(component) => {
            assert!(component.author.is_some());
            let container_title =
                component
                    .container
                    .as_ref()
                    .and_then(|relation| match relation {
                        WorkRelation::Embedded(parent) => parent.title(),
                        WorkRelation::Id(_) => None,
                    });
            assert_eq!(
                container_title,
                Some(Title::Single("Cosmos: A Spacetime Odyssey".to_string()))
            );
        }
        other => panic!("expected serial component, got {:?}", other),
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
fn unpublished_legacy_records_promote_issued_to_created() {
    let json = r#"{
        "id": "archival-letter",
        "type": "personal_communication",
        "title": "Letter to Jim Braden",
        "issued": {"date-parts": [[1973, 1, 1]]}
    }"#;

    let legacy: csl_legacy::csl_json::Reference = serde_json::from_str(json).unwrap();
    let reference: InputReference = legacy.into();

    assert_eq!(reference.created().unwrap().0, "1973-01-01");
    assert_eq!(reference.issued().unwrap().0, "1973-01-01");
    assert_eq!(reference.csl_issued_date().unwrap().0, "1973-01-01");
}

#[test]
fn created_date_backfills_csl_issued_compatibility() {
    let reference = InputReference::Monograph(Box::new(Monograph {
        id: Some("created-only".to_string()),
        r#type: MonographType::Manuscript,
        title: Some(Title::Single("Created Only".to_string())),
        created: EdtfString("1954-05-17".to_string()),
        ..Default::default()
    }));

    assert_eq!(reference.issued(), None);
    assert_eq!(reference.created().unwrap().0, "1954-05-17");
    assert_eq!(reference.csl_issued_date().unwrap().0, "1954-05-17");
}

#[test]
fn monograph_deserialization_uses_contributor_roles() {
    let yaml = r#"
class: monograph
type: personal-communication
title: Legacy personal communication
contributors:
  - role: recipient
    contributor:
      given: John
      family: Doe
  - role: interviewer
    contributor:
      given: Jane
      family: Roe
  - role: guest
    contributor:
      name: Example Guest
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).unwrap();

    assert!(matches!(
        reference.contributor(ContributorRole::Recipient),
        Some(Contributor::StructuredName(_))
    ));
    assert!(matches!(
        reference.contributor(ContributorRole::Interviewer),
        Some(Contributor::StructuredName(_))
    ));
    assert!(matches!(
        reference.contributor(ContributorRole::Guest),
        Some(Contributor::SimpleName(_))
    ));
}

#[test]
fn publisher_deserialization_accepts_legacy_string_shape() {
    let yaml = r#"
class: monograph
type: book
title: Legacy publisher
publisher: University of Chicago Press
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).unwrap();

    let publisher = reference.publisher().unwrap();
    assert_eq!(publisher.name.to_string(), "University of Chicago Press");
    assert_eq!(publisher.place, None);
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

#[test]
fn test_audio_visual_film_round_trip() {
    let yaml = r#"
class: audio-visual
type: film
title: Parasite
contributors:
  - role: director
    contributor:
      family: Bong
      given: Joon-ho
issued: "2019"
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).expect("failed to parse YAML");

    match &reference {
        InputReference::AudioVisual(av) => {
            assert_eq!(av.r#type, AudioVisualType::Film);
            assert_eq!(av.core.title, Some(Title::Single("Parasite".to_string())));
            assert_eq!(av.core.issued.0, "2019");
        }
        other => panic!("expected AudioVisual, got {:?}", other),
    }

    if let Some(Contributor::StructuredName(author)) = reference.author() {
        assert_eq!(
            author.family,
            MultilingualString::Simple("Bong".to_string())
        );
    } else {
        panic!("expected author with family name 'Bong'");
    }

    if let Some(director) = reference.contributor(ContributorRole::Director) {
        if let Contributor::StructuredName(name) = director {
            assert_eq!(name.family, MultilingualString::Simple("Bong".to_string()));
            assert_eq!(
                name.given,
                MultilingualString::Simple("Joon-ho".to_string())
            );
        } else {
            panic!("expected StructuredName contributor for director");
        }
    } else {
        panic!("expected director contributor");
    }
}

#[test]
fn test_audio_visual_episode_round_trip() {
    let yaml = r#"
class: audio-visual
type: episode
title: "A Camping We Will Go"
contributors:
  - role: director
    contributor:
      family: Rich
      given: John
issued: "1971"
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).expect("failed to parse YAML");

    match &reference {
        InputReference::AudioVisual(av) => {
            assert_eq!(av.r#type, AudioVisualType::Episode);
            assert_eq!(
                av.core.title,
                Some(Title::Single("A Camping We Will Go".to_string()))
            );
            assert_eq!(av.core.issued.0, "1971");
        }
        other => panic!("expected AudioVisual, got {:?}", other),
    }

    if let Some(director) = reference.contributor(ContributorRole::Director) {
        if let Contributor::StructuredName(name) = director {
            assert_eq!(name.family, MultilingualString::Simple("Rich".to_string()));
        } else {
            panic!("expected StructuredName contributor for director");
        }
    } else {
        panic!("expected director contributor");
    }
}

#[test]
fn test_monograph_contributor_shorthand_folding() {
    let yaml = r#"
class: monograph
type: interview
title: Thinking in Public
author:
  family: Arendt
  given: Hannah
contributors:
  - role: interviewer
    contributor:
      family: Young-Bruehl
      given: Elisabeth
issued: "1975"
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).expect("failed to parse YAML");

    match &reference {
        InputReference::Monograph(_mono) => {
            // verify it parses as monograph
        }
        other => panic!("expected Monograph, got {:?}", other),
    }

    if let Some(Contributor::StructuredName(author)) = reference.author() {
        assert_eq!(
            author.family,
            MultilingualString::Simple("Arendt".to_string())
        );
    } else {
        panic!("expected author with family name 'Arendt'");
    }

    if let Some(interviewer) = reference.contributor(ContributorRole::Interviewer) {
        if let Contributor::StructuredName(name) = interviewer {
            assert_eq!(
                name.family,
                MultilingualString::Simple("Young-Bruehl".to_string())
            );
        } else {
            panic!("expected StructuredName contributor for interviewer");
        }
    } else {
        panic!("expected interviewer contributor");
    }
}

#[test]
fn test_monograph_fold_author_shorthand() {
    // author: shorthand must be folded into contributors
    let yaml = r#"
class: monograph
type: book
title: Structure of Scientific Revolutions
author:
  family: Kuhn
  given: Thomas
issued: "1962"
"#;
    let r: InputReference = serde_yaml::from_str(yaml).unwrap();
    if let InputReference::Monograph(m) = &r {
        assert!(
            m.contributors
                .iter()
                .any(|e| e.role == ContributorRole::Author),
            "author shorthand not folded into contributors"
        );
    } else {
        panic!("expected Monograph");
    }
}

#[test]
fn test_monograph_fold_dedup() {
    // Same contributor in both author: shorthand and contributors: list must not duplicate
    let yaml = r#"
class: monograph
type: book
title: Dedup Test
author:
  family: Smith
  given: Alice
contributors:
  - role: author
    contributor:
      family: Smith
      given: Alice
issued: "2020"
"#;
    let r: InputReference = serde_yaml::from_str(yaml).unwrap();
    if let InputReference::Monograph(m) = &r {
        let author_count = m
            .contributors
            .iter()
            .filter(|e| e.role == ContributorRole::Author)
            .count();
        assert_eq!(author_count, 1, "duplicate author entry after fold");
    } else {
        panic!("expected Monograph");
    }
}

#[test]
fn test_monograph_author_accessor_reads_canonical_contributors() {
    let yaml = r#"
class: monograph
type: book
title: Contributor Canon
contributors:
  - role: author
    contributor:
      family: Le Guin
      given: Ursula
issued: "1969"
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).unwrap();

    if let Some(Contributor::StructuredName(author)) = reference.author() {
        assert_eq!(
            author.family,
            MultilingualString::Simple("Le Guin".to_string())
        );
    } else {
        panic!("expected author resolved from contributors");
    }
}

#[test]
fn test_monograph_serializes_canonical_contributors_only() {
    let yaml = r#"
class: monograph
type: book
title: Canonical Output
author:
  family: Butler
  given: Octavia
issued: "1979"
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).unwrap();
    let serialized = serde_yaml::to_string(&reference).unwrap();
    let value: serde_yaml::Value = serde_yaml::from_str(&serialized).unwrap();
    let mapping = value.as_mapping().expect("expected top-level mapping");

    assert!(!mapping.contains_key(serde_yaml::Value::String("author".to_string())));
    assert!(mapping.contains_key(serde_yaml::Value::String("contributors".to_string())));
}

#[test]
fn test_serial_editor_accessors_and_serialization_use_contributors() {
    let yaml = r#"
class: serial
type: podcast
title: Serial Contributors
editor:
  family: Gladwell
  given: Malcolm
"#;

    let reference: InputReference = serde_yaml::from_str(yaml).unwrap();

    if let Some(Contributor::StructuredName(editor)) = reference.editor() {
        assert_eq!(
            editor.family,
            MultilingualString::Simple("Gladwell".to_string())
        );
    } else {
        panic!("expected editor resolved from contributors");
    }

    if let Some(Contributor::StructuredName(editor)) =
        reference.contributor(ContributorRole::Editor)
    {
        assert_eq!(
            editor.family,
            MultilingualString::Simple("Gladwell".to_string())
        );
    } else {
        panic!("expected editor contributor on serial");
    }

    let serialized = serde_yaml::to_string(&reference).unwrap();
    let value: serde_yaml::Value = serde_yaml::from_str(&serialized).unwrap();
    let mapping = value.as_mapping().expect("expected top-level mapping");

    assert!(!mapping.contains_key(serde_yaml::Value::String("editor".to_string())));
    assert!(mapping.contains_key(serde_yaml::Value::String("contributors".to_string())));
}

#[test]
fn test_audio_visual_number_shorthand() {
    let yaml = r#"
class: audio-visual
type: recording
title: Beethoven Symphonies
contributors:
  - role: composer
    contributor:
      family: Beethoven
      given: Ludwig van
number: "PR90113"
issued: "1962"
"#;
    let r: InputReference = serde_yaml::from_str(yaml).unwrap();
    if let InputReference::AudioVisual(av) = &r {
        assert!(
            av.numbering.iter().any(|n| n.value == "PR90113"),
            "catalog number not folded into numbering"
        );
    } else {
        panic!("expected AudioVisual");
    }
}
