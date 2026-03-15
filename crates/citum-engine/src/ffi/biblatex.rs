//! Biblatex entry conversion to Citum InputReference.
//!
//! Provides functions to convert biblatex entries and contributor
//! information into Citum's InputReference and Contributor types.

use biblatex;
use citum_schema::reference::{
    InputReference,
    contributor::{Contributor, ContributorList, SimpleName, StructuredName},
    date::EdtfString,
    types::*,
};
use std::collections::HashMap;
use url::Url;

/// Build a CollectionComponent from a biblatex inbook/incollection/inproceedings entry.
#[allow(clippy::too_many_arguments)]
fn build_inbook_reference<F>(
    id: Option<String>,
    title: Option<Title>,
    author: Option<Contributor>,
    editor: Option<Contributor>,
    issued: EdtfString,
    publisher: Option<Contributor>,
    field_str: &F,
    language: Option<String>,
) -> InputReference
where
    F: Fn(&str) -> Option<String>,
{
    let parent_title = field_str("booktitle").map(Title::Single);
    InputReference::CollectionComponent(Box::new(CollectionComponent {
        id,
        r#type: MonographComponentType::Chapter,
        title,
        author,
        translator: None,
        issued,
        parent: Parent::Embedded(Collection {
            id: None,
            r#type: CollectionType::EditedBook,
            title: parent_title,
            short_title: None,
            editor,
            translator: None,
            issued: EdtfString(String::new()),
            publisher,
            collection_number: field_str("number"),
            url: None,
            accessed: None,
            language: None,
            field_languages: HashMap::new(),
            note: None,
            isbn: None,
            keywords: None,
        }),
        pages: field_str("pages").map(NumOrStr::Str),
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(EdtfString),
        language,
        field_languages: HashMap::new(),
        note: field_str("note"),
        doi: field_str("doi"),
        genre: field_str("type"),
        medium: None,
        keywords: None,
    }))
}

/// Build a SerialComponent from a biblatex article entry.
fn build_article_reference<F>(
    id: Option<String>,
    title: Option<Title>,
    author: Option<Contributor>,
    issued: EdtfString,
    field_str: &F,
    language: Option<String>,
) -> InputReference
where
    F: Fn(&str) -> Option<String>,
{
    let parent_title = field_str("journaltitle")
        .or_else(|| field_str("journal"))
        .map(Title::Single);
    InputReference::SerialComponent(Box::new(SerialComponent {
        id,
        r#type: SerialComponentType::Article,
        title,
        author,
        translator: None,
        issued,
        parent: Parent::Embedded(Serial {
            r#type: SerialType::AcademicJournal,
            title: parent_title,
            short_title: None,
            editor: None,
            publisher: None,
            issn: field_str("issn"),
        }),
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(EdtfString),
        language,
        field_languages: HashMap::new(),
        note: field_str("note"),
        doi: field_str("doi"),
        ads_bibcode: field_str("bibcode"),
        pages: field_str("pages"),
        volume: field_str("volume").map(NumOrStr::Str),
        issue: field_str("number").map(NumOrStr::Str),
        genre: field_str("type"),
        medium: None,
        keywords: None,
    }))
}

/// Convert a biblatex entry into an InputReference.
///
/// Maps biblatex entry types (book, article, inproceedings, etc.) to
/// appropriate Citum reference types. Extracts all relevant fields
/// including contributors, dates, and metadata.
pub(super) fn input_reference_from_biblatex(entry: &biblatex::Entry) -> InputReference {
    let id = Some(entry.key.clone());
    let field_str = |key: &str| {
        entry.fields.get(key).map(|f| {
            f.iter()
                .map(|c| match &c.v {
                    biblatex::Chunk::Normal(s) | biblatex::Chunk::Verbatim(s) => s.as_str(),
                    _ => "",
                })
                .collect::<String>()
        })
    };

    let title = field_str("title").map(Title::Single);
    let issued = field_str("date")
        .map(EdtfString)
        .unwrap_or(EdtfString(String::new()));
    let publisher = field_str("publisher").map(|p| {
        Contributor::SimpleName(SimpleName {
            name: p.into(),
            location: field_str("location"),
        })
    });

    let author = entry
        .author()
        .ok()
        .map(|p| contributors_from_biblatex_persons(&p));
    let editor = entry.editors().ok().map(|e| {
        let all_persons: Vec<biblatex::Person> =
            e.into_iter().flat_map(|(persons, _)| persons).collect();
        contributors_from_biblatex_persons(&all_persons)
    });

    let language = field_str("langid").or_else(|| field_str("language"));

    // Compute entry_type once to avoid repeated conversions
    let entry_type = entry.entry_type.to_string().to_lowercase();

    match entry_type.as_str() {
        "book" | "mvbook" | "collection" | "mvcollection" | "manual" => {
            let mono_type = if entry_type == "manual" {
                MonographType::Manual
            } else {
                MonographType::Book
            };
            InputReference::Monograph(Box::new(biblatex_monograph(
                id,
                mono_type,
                title,
                author,
                editor,
                issued,
                publisher,
                &field_str,
                language,
                &entry_type,
            )))
        }
        "inbook" | "incollection" | "inproceedings" => build_inbook_reference(
            id, title, author, editor, issued, publisher, &field_str, language,
        ),
        "article" => build_article_reference(id, title, author, issued, &field_str, language),
        _ => InputReference::Monograph(Box::new(biblatex_monograph(
            id,
            MonographType::Document,
            title,
            author,
            editor,
            issued,
            publisher,
            &field_str,
            language,
            &entry_type,
        ))),
    }
}

/// Build a Monograph reference with common fields from biblatex.
///
/// Extracts report_number and collection_number based on entry type,
/// and handles URL parsing.
#[allow(clippy::too_many_arguments)]
fn biblatex_monograph<F>(
    id: Option<String>,
    r#type: MonographType,
    title: Option<Title>,
    author: Option<Contributor>,
    editor: Option<Contributor>,
    issued: EdtfString,
    publisher: Option<Contributor>,
    field_str: &F,
    language: Option<String>,
    entry_type: &str,
) -> Monograph
where
    F: Fn(&str) -> Option<String>,
{
    Monograph {
        id,
        r#type,
        title,
        container_title: None,
        author,
        editor,
        translator: None,
        recipient: None,
        interviewer: None,
        issued,
        publisher,
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(EdtfString),
        language,
        field_languages: HashMap::new(),
        note: field_str("note"),
        isbn: field_str("isbn"),
        doi: field_str("doi"),
        ads_bibcode: field_str("bibcode"),
        edition: field_str("edition"),
        report_number: if entry_type == "report" {
            field_str("number")
        } else {
            None
        },
        collection_number: if entry_type != "report" {
            field_str("number")
        } else {
            None
        },
        genre: field_str("type"),
        medium: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original_date: None,
        original_title: None,
    }
}

/// Convert biblatex persons (authors/editors) to a Contributor list.
///
/// Maps biblatex Person data (given name, family name, prefix, suffix)
/// to Citum's StructuredName contributors wrapped in a ContributorList.
pub(super) fn contributors_from_biblatex_persons(persons: &[biblatex::Person]) -> Contributor {
    let contributors: Vec<Contributor> = persons
        .iter()
        .map(|p| {
            Contributor::StructuredName(StructuredName {
                given: p.given_name.clone().into(),
                family: p.name.clone().into(),
                suffix: if p.suffix.is_empty() {
                    None
                } else {
                    Some(p.suffix.clone())
                },
                dropping_particle: None,
                non_dropping_particle: if p.prefix.is_empty() {
                    None
                } else {
                    Some(p.prefix.clone())
                },
            })
        })
        .collect();
    Contributor::ContributorList(ContributorList(contributors))
}
