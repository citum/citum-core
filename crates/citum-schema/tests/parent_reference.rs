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
#![allow(missing_docs, reason = "test/bench/bin crate")]

use citum_schema::reference::{
    CollectionComponent, EdtfString, MonographComponentType, SerialComponent, SerialComponentType,
    Title, WorkRelation,
};

#[test]
fn test_serial_component_with_container_id() {
    let parent_id = "journal-1".to_string();
    let component = SerialComponent {
        id: Some("article-1".into()),
        r#type: SerialComponentType::Article,
        title: Some(Title::Single("My Article".to_string())),
        issued: EdtfString("2023".to_string()),
        container: Some(WorkRelation::Id(parent_id.clone().into())),
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
        id: Some("chapter-1".into()),
        r#type: MonographComponentType::Chapter,
        title: Some(Title::Single("My Chapter".to_string())),
        issued: EdtfString("2023".to_string()),
        container: Some(WorkRelation::Id(parent_id.clone().into())),
        ..Default::default()
    };

    match component.container.unwrap() {
        WorkRelation::Id(id) => assert_eq!(id, parent_id),
        WorkRelation::Embedded(_) => panic!("Expected WorkRelation::Id"),
    }
}
