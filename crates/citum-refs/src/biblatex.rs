/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Biblatex entry conversion to Citum `InputReference`.
//!
//! Provides functions to convert biblatex entries and contributor
//! information into Citum's `InputReference` and Contributor types.

use biblatex as biblatex_crate;
use citum_schema::reference::{
    InputReference, LangID, Numbering, NumberingType, Publisher, RefID, RichText, WorkRelation,
    contributor::{Contributor, ContributorList, StructuredName},
    date::DateValue,
    types::{
        Collection, CollectionComponent, CollectionType, Monograph, MonographComponentType,
        MonographType, NumOrStr, Serial, SerialComponent, SerialComponentType, SerialType,
        StructuredTitle, Subtitle, Title,
    },
};
use std::collections::HashMap;
use url::Url;

/// Common fields shared across all biblatex reference conversion helpers.
struct BibRefContext<'a> {
    id: Option<RefID>,
    title: Option<Title>,
    author: Option<Contributor>,
    editor: Option<Contributor>,
    translator: Option<Contributor>,
    issued: DateValue,
    publisher: Option<Publisher>,
    language: Option<LangID>,
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
        created: DateValue::new(String::new()),
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(
            InputReference::Collection(Box::new(Collection {
                id: None,
                r#type: CollectionType::EditedBook,
                title: parent_title,
                short_title: None,
                container: None,
                editor: ctx.editor,
                translator: ctx.translator,
                created: DateValue::new(String::new()),
                issued: DateValue::new(String::new()),
                publisher: ctx.publisher,
                numbering: parent_numbering,
                isbn: field_str("isbn"),
                ..Default::default()
            })),
        ))),
        numbering: Vec::new(),
        pages: field_str("pages").map(NumOrStr::Str),
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(DateValue::new),
        language: ctx.language,
        field_languages: HashMap::new(),
        note: field_str("note").map(RichText::Plain),
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
        translator: ctx.translator,
        created: DateValue::new(String::new()),
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(InputReference::Serial(
            Box::new(Serial {
                id: None,
                r#type: SerialType::AcademicJournal,
                title: parent_title,
                short_title: None,
                container: None,
                editor: None,
                contributors: Vec::new(),
                publisher: None,
                url: None,
                accessed: None,
                language: None,
                field_languages: HashMap::new(),
                note: None,
                issn: field_str("issn"),
                unknown_fields: Default::default(),
            }),
        )))),
        numbering: component_numbering,
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(DateValue::new),
        language: ctx.language,
        field_languages: HashMap::new(),
        note: field_str("note").map(RichText::Plain),
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
pub fn input_reference_from_biblatex(entry: &biblatex_crate::Entry) -> InputReference {
    let id = Some(entry.key.clone().into());
    let field_str = |key: &str| {
        entry.fields.get(key).map(|f| {
            f.iter()
                .map(|c| match &c.v {
                    biblatex_crate::Chunk::Normal(s) | biblatex_crate::Chunk::Verbatim(s) => {
                        s.as_str()
                    }
                    _ => "",
                })
                .collect::<String>()
        })
    };

    let title = match (field_str("title"), field_str("subtitle")) {
        (Some(main), Some(sub)) => Some(Title::Structured(StructuredTitle {
            full: None,
            main,
            sub: Subtitle::String(sub),
        })),
        (Some(main), None) => Some(Title::Single(main)),
        (None, _) => None,
    };
    let issued = field_str("date").map_or(DateValue::new(String::new()), DateValue::new);
    let publisher = field_str("publisher")
        .or_else(|| field_str("institution"))
        .or_else(|| field_str("organization"))
        .or_else(|| field_str("school"))
        .map(|p| Publisher {
            name: p.into(),
            place: field_str("location").map(Into::into),
        });

    let author = entry
        .author()
        .ok()
        .map(|p| contributors_from_biblatex_persons(&p));
    let editor = entry.editors().ok().map(|e| {
        let all_persons: Vec<biblatex_crate::Person> =
            e.into_iter().flat_map(|(persons, _)| persons).collect();
        contributors_from_biblatex_persons(&all_persons)
    });
    let translator = entry
        .translator()
        .ok()
        .map(|p| contributors_from_biblatex_persons(&p));

    let language = field_str("langid")
        .or_else(|| field_str("language"))
        .map(Into::into);

    let entry_type = entry.entry_type.to_biblatex().to_string().to_lowercase();

    let ctx = BibRefContext {
        id,
        title,
        author,
        editor,
        translator,
        issued,
        publisher,
        language,
        field_str: &field_str,
    };

    match entry_type.as_str() {
        "book" | "mvbook" | "collection" | "mvcollection" | "manual" | "report" | "thesis"
        | "online" | "unpublished" | "proceedings" | "mvproceedings" => {
            let mono_type = match entry_type.as_str() {
                "manual" => MonographType::Manual,
                "report" => MonographType::Report,
                "thesis" => MonographType::Thesis,
                "online" => MonographType::Webpage,
                "unpublished" => MonographType::Manuscript,
                _ => MonographType::Book,
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
/// Maps biblatex `edition` and `number` fields into canonical `numbering`,
/// treating `report` entry numbers as `NumberingType::Report`, and handles URL parsing.
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
                r#type: NumberingType::Report,
                value: n,
            });
        } else {
            numbering.push(Numbering {
                r#type: NumberingType::Number,
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
        translator: ctx.translator,
        created: DateValue::new(String::new()),
        issued: ctx.issued,
        publisher: ctx.publisher,
        url: field_str("url").and_then(|u| Url::parse(&u).ok()),
        accessed: field_str("urldate").map(DateValue::new),
        language: ctx.language,
        field_languages: HashMap::new(),
        note: field_str("note").map(RichText::Plain),
        abstract_text: field_str("abstract").map(RichText::Plain),
        isbn: field_str("isbn"),
        doi: field_str("doi"),
        ads_bibcode: field_str("bibcode"),
        version: field_str("version"),
        keywords: field_str("keywords").map(|k| {
            k.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }),
        numbering,
        genre: field_str("type"),
        ..Default::default()
    }
}

/// Convert biblatex persons (authors/editors) to a Contributor list.
///
/// Maps biblatex Person data (given name, family name, prefix, suffix)
/// to Citum's `StructuredName` contributors wrapped in a `ContributorList`.
pub fn contributors_from_biblatex_persons(persons: &[biblatex_crate::Person]) -> Contributor {
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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn parse_single_entry(source: &str) -> biblatex_crate::Entry {
        let bibliography =
            biblatex_crate::Bibliography::parse(source).expect("biblatex should parse");
        bibliography
            .into_iter()
            .next()
            .expect("bibliography should contain one entry")
    }

    #[test]
    fn biblatex_report_number_maps_to_report_numbering() {
        let entry = parse_single_entry(
            "@report{r1,\n  title = {Report},\n  date = {2024},\n  number = {TR-7}\n}",
        );

        let converted = input_reference_from_biblatex(&entry);

        assert_eq!(converted.ref_type(), "report");
        assert_eq!(converted.number(), None);
        assert_eq!(converted.report_number(), Some("TR-7".to_string()));
    }

    #[test]
    fn biblatex_book_number_maps_to_generic_numbering() {
        let entry =
            parse_single_entry("@book{b1,\n  title = {Book},\n  date = {2024},\n  number = {2}\n}");

        let converted = input_reference_from_biblatex(&entry);

        assert_eq!(converted.number(), Some("2".to_string()));
        assert_eq!(converted.report_number(), None);
    }

    #[test]
    fn given_techreport_with_number_when_converted_then_maps_to_report_type_and_report_numbering() {
        let entry = parse_single_entry(
            "@techreport{k1,\n  title = {T},\n  date = {2024},\n  number = {TR-9}\n}",
        );

        let converted = input_reference_from_biblatex(&entry);

        assert_eq!(converted.ref_type(), "report");
        assert_eq!(converted.report_number(), Some("TR-9".to_string()));
        assert_eq!(converted.number(), None);
    }

    #[rstest]
    #[case::phdthesis_maps_to_thesis("@phdthesis{k1, title={T}, date={2024}}", "thesis")]
    #[case::mastersthesis_maps_to_thesis("@mastersthesis{k1, title={T}, date={2024}}", "thesis")]
    #[case::thesis_maps_to_thesis("@thesis{k1, title={T}, date={2024}}", "thesis")]
    #[case::online_maps_to_webpage("@online{k1, title={T}, date={2024}}", "webpage")]
    #[case::unpublished_maps_to_manuscript(
        "@unpublished{k1, title={T}, date={2024}}",
        "manuscript"
    )]
    fn given_biblatex_entry_type_when_converted_then_maps_to_expected_monograph_type(
        #[case] source: &str,
        #[case] expected_ref_type: &str,
    ) {
        let entry = parse_single_entry(source);

        let converted = input_reference_from_biblatex(&entry);

        assert_eq!(converted.ref_type(), expected_ref_type);
    }

    #[test]
    fn given_translator_field_when_converted_then_translator_is_mapped() {
        let entry = parse_single_entry(
            "@book{b2, title = {Book}, date = {2024}, translator = {Doe, Jane}}",
        );

        let converted = input_reference_from_biblatex(&entry);

        let monograph = converted.as_monograph().expect("expected a Monograph");
        assert_eq!(
            monograph.translator,
            Some(Contributor::ContributorList(ContributorList(vec![
                Contributor::StructuredName(StructuredName {
                    given: "Jane".into(),
                    family: "Doe".into(),
                    suffix: None,
                    dropping_particle: None,
                    non_dropping_particle: None,
                })
            ])))
        );
    }

    #[test]
    fn given_thesis_with_institution_and_no_publisher_when_converted_then_institution_becomes_publisher_with_location()
     {
        let entry = parse_single_entry(
            "@phdthesis{t1, title = {T}, date = {2024}, institution = {Wuhan University}, location = {Wuhan}}",
        );

        let converted = input_reference_from_biblatex(&entry);

        let monograph = converted.as_monograph().expect("expected a Monograph");
        assert_eq!(
            monograph.publisher,
            Some(Publisher {
                name: "Wuhan University".into(),
                place: Some("Wuhan".into()),
            })
        );
    }

    #[test]
    fn given_title_and_subtitle_when_converted_then_title_is_structured() {
        let entry = parse_single_entry(
            "@book{b3, title = {Main Title}, subtitle = {A Subtitle}, date = {2024}}",
        );

        let converted = input_reference_from_biblatex(&entry);

        let monograph = converted.as_monograph().expect("expected a Monograph");
        assert_eq!(
            monograph.title,
            Some(Title::Structured(StructuredTitle {
                full: None,
                main: "Main Title".to_string(),
                sub: Subtitle::String("A Subtitle".to_string()),
            }))
        );
    }

    #[test]
    fn given_incollection_with_isbn_when_converted_then_isbn_is_on_the_parent_collection() {
        let entry = parse_single_entry(
            "@incollection{c1, title = {Chapter}, booktitle = {Book}, date = {2024}, isbn = {978-0-13-468599-1}}",
        );

        let converted = input_reference_from_biblatex(&entry);

        let component = converted
            .as_collection_component()
            .expect("expected a CollectionComponent");
        let parent = match component.container.as_ref().expect("expected a container") {
            WorkRelation::Embedded(inner) => inner
                .as_collection()
                .expect("expected an embedded Collection"),
            WorkRelation::Id(_) => panic!("expected an embedded container, not an id reference"),
        };
        assert_eq!(parent.isbn, Some("978-0-13-468599-1".to_string()));
    }

    #[rstest]
    #[case::proceedings_maps_to_book("@proceedings{p1, title={T}, date={2024}}", "book")]
    #[case::mvproceedings_maps_to_book("@mvproceedings{p1, title={T}, date={2024}}", "book")]
    fn given_proceedings_entry_type_when_converted_then_maps_to_book(
        #[case] source: &str,
        #[case] expected_ref_type: &str,
    ) {
        let entry = parse_single_entry(source);

        let converted = input_reference_from_biblatex(&entry);

        assert_eq!(converted.ref_type(), expected_ref_type);
    }
}
