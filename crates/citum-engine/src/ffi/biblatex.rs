/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Biblatex entry conversion to Citum `InputReference`.
//!
//! Provides functions to convert biblatex entries and contributor
//! information into Citum's `InputReference` and Contributor types.

use biblatex;
use citum_schema::reference::{
    InputReference, Numbering, NumberingType, WorkRelation,
    contributor::{Contributor, ContributorList, SimpleName, StructuredName},
    date::EdtfString,
    types::{
        Collection, CollectionComponent, CollectionType, Monograph, MonographComponentType,
        MonographType, NumOrStr, Serial, SerialComponent, SerialComponentType, SerialType, Title,
    },
};
use std::collections::HashMap;
use url::Url;

/// Common fields shared across all biblatex reference conversion helpers.
struct BibRefContext<'a> {
    id: Option<String>,
    title: Option<Title>,
    author: Option<Contributor>,
    editor: Option<Contributor>,
    issued: EdtfString,
    publisher: Option<Contributor>,
    language: Option<String>,
    field_str: &'a dyn Fn(&str) -> Option<String>,
}

/// Build a `CollectionComponent` from a biblatex inbook/incollection/inproceedings entry.
fn build_inbook_reference(ctx: BibRefContext<'_>) -> InputReference {
    let field_str = ctx.field_str;
    let parent_title = field_str("booktitle").map(Title::Single);

    let mut parent_numbering = Vec::new();
    if let Some(n) = field_str("number") {
        parent_numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: n,
        });
    }

    InputReference::CollectionComponent(Box::new(CollectionComponent {
        id: ctx.id,
        r#type: MonographComponentType::Chapter,
        title: ctx.title,
        author: ctx.author,
        translator: None,
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(
            InputReference::Collection(Box::new(Collection {
                id: None,
                r#type: CollectionType::EditedBook,
                title: parent_title,
                short_title: None,
                container: None,
                editor: ctx.editor,
                translator: None,
                issued: EdtfString(String::new()),
                publisher: ctx.publisher,
                numbering: parent_numbering,
                ..Default::default()
            })),
        ))),
        numbering: Vec::new(),
        pages: field_str("pages").map(NumOrStr::Str),
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(EdtfString),
        language: ctx.language,
        field_languages: HashMap::new(),
        note: field_str("note"),
        doi: field_str("doi"),
        genre: field_str("type"),
        ..Default::default()
    }))
}

/// Build a `SerialComponent` from a biblatex article entry.
fn build_article_reference(ctx: BibRefContext<'_>) -> InputReference {
    let field_str = ctx.field_str;
    let parent_title = field_str("journaltitle")
        .or_else(|| field_str("journal"))
        .map(Title::Single);

    let mut component_numbering = Vec::new();
    if let Some(v) = field_str("volume") {
        component_numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: v,
        });
    }
    if let Some(i) = field_str("number") {
        component_numbering.push(Numbering {
            r#type: NumberingType::Issue,
            value: i,
        });
    }

    InputReference::SerialComponent(Box::new(SerialComponent {
        id: ctx.id,
        r#type: SerialComponentType::Article,
        title: ctx.title,
        author: ctx.author,
        translator: None,
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(InputReference::Serial(
            Box::new(Serial {
                id: None,
                r#type: SerialType::AcademicJournal,
                title: parent_title,
                short_title: None,
                container: None,
                editor: None,
                publisher: None,
                url: None,
                accessed: None,
                language: None,
                field_languages: HashMap::new(),
                note: None,
                issn: field_str("issn"),
            }),
        )))),
        numbering: component_numbering,
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(EdtfString),
        language: ctx.language,
        field_languages: HashMap::new(),
        note: field_str("note"),
        doi: field_str("doi"),
        ads_bibcode: field_str("bibcode"),
        pages: field_str("pages"),
        genre: field_str("type"),
        ..Default::default()
    }))
}

/// Convert a biblatex entry into an `InputReference`.
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
    let issued = field_str("date").map_or(EdtfString(String::new()), EdtfString);
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

    let ctx = BibRefContext {
        id,
        title,
        author,
        editor,
        issued,
        publisher,
        language,
        field_str: &field_str,
    };

    match entry_type.as_str() {
        "book" | "mvbook" | "collection" | "mvcollection" | "manual" => {
            let mono_type = if entry_type == "manual" {
                MonographType::Manual
            } else {
                MonographType::Book
            };
            InputReference::Monograph(Box::new(biblatex_monograph(mono_type, &entry_type, ctx)))
        }
        "inbook" | "incollection" | "inproceedings" => build_inbook_reference(ctx),
        "article" => build_article_reference(ctx),
        _ => InputReference::Monograph(Box::new(biblatex_monograph(
            MonographType::Document,
            &entry_type,
            ctx,
        ))),
    }
}

/// Build a Monograph reference with common fields from biblatex.
///
/// Extracts `report_number` and `collection_number` based on entry type,
/// and handles URL parsing.
fn biblatex_monograph(
    r#type: MonographType,
    entry_type: &str,
    ctx: BibRefContext<'_>,
) -> Monograph {
    let field_str = ctx.field_str;

    let mut numbering = Vec::new();
    if let Some(ed) = field_str("edition") {
        numbering.push(Numbering {
            r#type: NumberingType::Edition,
            value: ed,
        });
    }
    if let Some(n) = field_str("number") {
        if entry_type == "report" {
            numbering.push(Numbering {
                r#type: NumberingType::Part,
                value: n,
            });
        } else {
            numbering.push(Numbering {
                r#type: NumberingType::Volume,
                value: n,
            });
        }
    }

    Monograph {
        id: ctx.id,
        r#type,
        title: ctx.title,
        short_title: None,
        container: None,
        author: ctx.author,
        editor: ctx.editor,
        translator: None,
        recipient: None,
        interviewer: None,
        guest: None,
        issued: ctx.issued,
        publisher: ctx.publisher,
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(EdtfString),
        language: ctx.language,
        field_languages: HashMap::new(),
        note: field_str("note"),
        isbn: field_str("isbn"),
        doi: field_str("doi"),
        ads_bibcode: field_str("bibcode"),
        numbering,
        genre: field_str("type"),
        ..Default::default()
    }
}

/// Convert biblatex persons (authors/editors) to a Contributor list.
///
/// Maps biblatex Person data (given name, family name, prefix, suffix)
/// to Citum's `StructuredName` contributors wrapped in a `ContributorList`.
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
