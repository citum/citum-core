#![allow(missing_docs, reason = "test/bench/bin crate")]

use citum_schema::reference::{
    CollectionComponent, EdtfString, MonographComponentType, SerialComponent, SerialComponentType,
    Title, WorkRelation,
};

#[test]
fn test_serial_component_with_container_id() {
    let parent_id = "journal-1".to_string();
    let component = SerialComponent {
        id: Some("article-1".to_string()),
        r#type: SerialComponentType::Article,
        title: Some(Title::Single("My Article".to_string())),
        issued: EdtfString("2023".to_string()),
        container: Some(WorkRelation::Id(parent_id.clone())),
        ..Default::default()
    };

    match component.container.unwrap() {
        WorkRelation::Id(id) => assert_eq!(id, parent_id),
        WorkRelation::Embedded(_) => panic!("Expected WorkRelation::Id"),
    }
}

#[test]
fn test_collection_component_with_container_id() {
    let parent_id = "book-1".to_string();
    let component = CollectionComponent {
        id: Some("chapter-1".to_string()),
        r#type: MonographComponentType::Chapter,
        title: Some(Title::Single("My Chapter".to_string())),
        issued: EdtfString("2023".to_string()),
        container: Some(WorkRelation::Id(parent_id.clone())),
        ..Default::default()
    };

    match component.container.unwrap() {
        WorkRelation::Id(id) => assert_eq!(id, parent_id),
        WorkRelation::Embedded(_) => panic!("Expected WorkRelation::Id"),
    }
}
