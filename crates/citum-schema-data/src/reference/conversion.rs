use crate::reference::contributor::{Contributor, ContributorList, SimpleName, StructuredName};
use crate::reference::date::EdtfString;
use crate::reference::types::{
    ArchiveInfo, Collection, CollectionComponent, CollectionType, Dataset, LegalCase, Monograph,
    MonographComponentType, MonographType, NumOrStr, Patent, Serial, SerialComponent,
    SerialComponentType, SerialType, Software, Standard, Statute, Title, Treaty,
};
use crate::reference::{Event, InputReference, Numbering, NumberingType, WorkRelation};
use std::collections::HashMap;
use url::Url;

fn short_title_from_legacy(legacy: &csl_legacy::csl_json::Reference, key: &str) -> Option<String> {
    legacy
        .extra
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn format_interviewer_note(names: &[csl_legacy::csl_json::Name]) -> Option<String> {
    if names.is_empty() {
        return None;
    }

    let formatted: Vec<String> = names
        .iter()
        .filter_map(|n| {
            if let Some(literal) = &n.literal {
                return Some(literal.clone());
            }
            let family = n.family.as_deref().unwrap_or("").trim();
            if family.is_empty() {
                return None;
            }
            let given_initial = n
                .given
                .as_deref()
                .and_then(|g| g.chars().next())
                .map(|c| format!("{c}. "));
            Some(format!("{}{}", given_initial.unwrap_or_default(), family))
        })
        .collect();

    if formatted.is_empty() {
        None
    } else {
        Some(format!("{}, Interviewer", formatted.join(", ")))
    }
}

fn archive_info_from_legacy_flat(legacy: &csl_legacy::csl_json::Reference) -> Option<ArchiveInfo> {
    if legacy.archive.is_none() && legacy.archive_location.is_none() {
        return None;
    }

    Some(ArchiveInfo {
        name: legacy.archive.clone().map(Into::into),
        location: legacy.archive_location.clone(),
        ..Default::default()
    })
}

/// Pre-extracted common fields shared by all reference conversion functions.
struct RefContext {
    id: Option<String>,
    title: Option<Title>,
    issued: EdtfString,
    url: Option<Url>,
    accessed: Option<EdtfString>,
    language: Option<String>,
    note: Option<String>,
    doi: Option<String>,
    isbn: Option<String>,
    edition: Option<String>,
    container_title_short: Option<String>,
    journal_abbreviation: Option<String>,
}

fn from_software_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Software(Box::new(Software {
        id: ctx.id,
        title: ctx.title,
        author: legacy.author.map(Contributor::from),
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| {
            Contributor::SimpleName(SimpleName {
                name: n.into(),
                location: legacy.publisher_place,
            })
        }),
        version: None,
        repository: None,
        license: None,
        platform: None,
        doi: ctx.doi,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        keywords: None,
    }))
}

#[allow(
    clippy::too_many_lines,
    reason = "Legacy CSL mapping requires extensive branching"
)]
fn from_monograph_ref(
    legacy: csl_legacy::csl_json::Reference,
    mut ctx: RefContext,
) -> InputReference {
    if (legacy.ref_type == "personal_communication" || legacy.ref_type == "personal-communication")
        && ctx.note.is_none()
    {
        ctx.note = Some("personal communication".to_string());
    } else if legacy.ref_type == "interview" && ctx.note.is_none() {
        ctx.note = legacy
            .interviewer
            .as_ref()
            .and_then(|names| format_interviewer_note(names));
    }

    let r#type = if legacy.ref_type == "report" {
        MonographType::Report
    } else if legacy.ref_type == "thesis" {
        MonographType::Thesis
    } else if legacy.ref_type == "manual" {
        MonographType::Manual
    } else if legacy.ref_type == "manuscript" {
        MonographType::Manuscript
    } else if legacy.ref_type == "webpage" {
        MonographType::Webpage
    } else if legacy.ref_type.contains("post") {
        MonographType::Post
    } else if legacy.ref_type == "interview" {
        MonographType::Interview
    } else if legacy.ref_type == "personal_communication"
        || legacy.ref_type == "personal-communication"
    {
        MonographType::PersonalCommunication
    } else {
        MonographType::Book
    };

    let archive_info = archive_info_from_legacy_flat(&legacy);

    let mut numbering = Vec::new();
    if let Some(edition) = ctx.edition {
        numbering.push(Numbering {
            r#type: NumberingType::Edition,
            value: edition,
        });
    }
    if let Some(volume) = legacy.volume {
        numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: volume.to_string(),
        });
    }
    if let Some(collection_number) = legacy.collection_number {
        numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: collection_number.to_string(),
        });
    }
    if let Some(n) = legacy.number {
        numbering.push(Numbering {
            r#type: if r#type == MonographType::Report {
                NumberingType::Report
            } else {
                NumberingType::Number
            },
            value: n,
        });
    }

    let original = legacy.original_title.map(|original_title| {
        WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
            title: Some(Title::Single(original_title)),
            ..Default::default()
        }))))
    });

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type,
        title: ctx.title,
        short_title: None,
        container: legacy.container_title.map(|t| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(t)),
                ..Default::default()
            }))))
        }),
        author: legacy.author.map(Contributor::from),
        editor: legacy.editor.map(Contributor::from),
        translator: legacy.translator.map(Contributor::from),
        recipient: legacy.recipient.map(Contributor::from),
        interviewer: legacy.interviewer.map(Contributor::from),
        guest: None,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| {
            Contributor::SimpleName(SimpleName {
                name: n.into(),
                location: legacy.publisher_place,
            })
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        isbn: ctx.isbn,
        doi: ctx.doi,
        ads_bibcode: None,
        numbering,
        genre: legacy.genre,
        medium: legacy.medium,
        archive: legacy.archive,
        archive_location: legacy.archive_location,
        archive_info,
        eprint: None,
        keywords: None,
        original,
        ..Default::default()
    }))
}

fn from_collection_component_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let parent_title = legacy.container_title.map(Title::Single);
    let mut parent_numbering = Vec::new();
    if let Some(v) = legacy.collection_number.clone().or(legacy.volume.clone()) {
        parent_numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: v.to_string(),
        });
    }

    InputReference::CollectionComponent(Box::new(CollectionComponent {
        id: ctx.id,
        r#type: if legacy.ref_type == "paper-conference" {
            MonographComponentType::Document
        } else {
            MonographComponentType::Chapter
        },
        title: ctx.title,
        author: legacy.author.map(Contributor::from),
        translator: legacy.translator.map(Contributor::from),
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(
            InputReference::Collection(Box::new(Collection {
                id: None,
                r#type: CollectionType::EditedBook,
                title: parent_title,
                short_title: ctx.container_title_short,
                container: None,
                editor: legacy.editor.map(Contributor::from),
                translator: None,
                issued: EdtfString(String::new()),
                publisher: legacy.publisher.map(|n| {
                    Contributor::SimpleName(SimpleName {
                        name: n.into(),
                        location: legacy.publisher_place,
                    })
                }),
                numbering: parent_numbering,
                ..Default::default()
            })),
        ))),
        numbering: Vec::new(),
        pages: legacy.page.map(NumOrStr::Str),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        doi: ctx.doi,
        genre: legacy.genre,
        medium: legacy.medium,
        ..Default::default()
    }))
}

/// Convert a legacy CSL edited book into the standalone Citum collection shape.
#[must_use]
pub fn input_reference_from_legacy_edited_book(
    legacy: csl_legacy::csl_json::Reference,
) -> InputReference {
    let mut numbering = Vec::new();
    if let Some(cn) = legacy.collection_number.clone() {
        numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: cn.to_string(),
        });
    }

    let csl_legacy::csl_json::Reference {
        id,
        title,
        editor,
        translator,
        issued,
        publisher,
        publisher_place,
        url,
        accessed,
        language,
        note,
        isbn,
        extra,
        ..
    } = legacy;

    InputReference::Collection(Box::new(Collection {
        id: Some(id),
        r#type: CollectionType::EditedBook,
        title: title.map(Title::Single),
        short_title: extra
            .get("shortTitle")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        container: None,
        editor: editor.map(Contributor::from),
        translator: translator.map(Contributor::from),
        issued: issued
            .map(EdtfString::from)
            .unwrap_or(EdtfString(String::new())),
        publisher: publisher.map(|name| {
            Contributor::SimpleName(SimpleName {
                name: name.into(),
                location: publisher_place.clone(),
            })
        }),
        numbering,
        url: url.as_deref().and_then(|value| Url::parse(value).ok()),
        accessed: accessed.map(EdtfString::from),
        language,
        field_languages: HashMap::new(),
        note,
        isbn,
        ..Default::default()
    }))
}

fn from_serial_component_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let mut genre = legacy.genre;
    if legacy.ref_type == "entry-encyclopedia" && genre.is_none() {
        genre = Some("entry-encyclopedia".to_string());
    }
    let serial_type = match legacy.ref_type.as_str() {
        "article-journal" => SerialType::AcademicJournal,
        "article-magazine" => SerialType::Magazine,
        "article-newspaper" => SerialType::Newspaper,
        "broadcast" | "motion_picture" => SerialType::BroadcastProgram,
        _ => SerialType::AcademicJournal,
    };
    let parent_title = legacy.container_title.map(Title::Single);

    let mut numbering = Vec::new();
    if let Some(v) = legacy.volume {
        numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: v.to_string(),
        });
    }
    if let Some(i) = legacy.issue.or_else(|| {
        if legacy.ref_type == "broadcast" || legacy.ref_type == "motion_picture" {
            legacy
                .number
                .as_ref()
                .map(|n| csl_legacy::csl_json::StringOrNumber::String(n.clone()))
        } else {
            None
        }
    }) {
        numbering.push(Numbering {
            r#type: NumberingType::Issue,
            value: i.to_string(),
        });
    }

    InputReference::SerialComponent(Box::new(SerialComponent {
        id: ctx.id,
        r#type: SerialComponentType::Article,
        title: ctx.title,
        author: legacy.author.map(Contributor::from),
        translator: legacy.translator.map(Contributor::from),
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(InputReference::Serial(
            Box::new(Serial {
                id: None,
                r#type: serial_type,
                title: parent_title,
                short_title: ctx.container_title_short.or(ctx.journal_abbreviation),
                container: None,
                editor: None,
                publisher: legacy.publisher.clone().map(|n| {
                    Contributor::SimpleName(SimpleName {
                        name: n.into(),
                        location: legacy.publisher_place.clone(),
                    })
                }),
                url: None,
                accessed: None,
                language: None,
                field_languages: HashMap::new(),
                note: None,
                issn: legacy.issn,
            }),
        )))),
        numbering,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        doi: ctx.doi,
        ads_bibcode: None,
        pages: legacy.page,
        genre,
        medium: legacy.medium,
        archive_info: None,
        eprint: None,
        keywords: None,
        reviewed: None,
        original: None,
        ..Default::default()
    }))
}

fn from_legal_case_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::LegalCase(Box::new(LegalCase {
        id: ctx.id,
        title: ctx.title,
        authority: legacy.authority.unwrap_or_default(),
        volume: legacy.volume.map(|v| v.to_string()),
        reporter: legacy.container_title,
        page: legacy.page,
        issued: ctx.issued,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        doi: ctx.doi,
        keywords: None,
    }))
}

fn from_statute_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Statute(Box::new(Statute {
        id: ctx.id,
        title: ctx.title,
        authority: legacy.authority,
        volume: legacy.volume.map(|v| v.to_string()),
        code: legacy.container_title,
        section: legacy.section,
        issued: ctx.issued,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        keywords: None,
    }))
}

fn from_treaty_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Treaty(Box::new(Treaty {
        id: ctx.id,
        title: ctx.title,
        author: legacy.author.map(Contributor::from),
        volume: legacy.volume.map(|v| v.to_string()),
        reporter: legacy.container_title,
        page: legacy.page,
        issued: ctx.issued,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        keywords: None,
    }))
}

fn from_standard_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Standard(Box::new(Standard {
        id: ctx.id,
        title: ctx.title,
        authority: legacy.authority,
        standard_number: legacy.number.unwrap_or_default(),
        issued: ctx.issued,
        status: None,
        publisher: legacy.publisher.map(|n| {
            Contributor::SimpleName(SimpleName {
                name: n.into(),
                location: legacy.publisher_place,
            })
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        keywords: None,
    }))
}

fn from_patent_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Patent(Box::new(Patent {
        id: ctx.id,
        title: ctx.title,
        author: legacy.author.map(Contributor::from),
        assignee: None,
        patent_number: legacy.number.unwrap_or_default(),
        application_number: None,
        filing_date: None,
        issued: ctx.issued,
        jurisdiction: None,
        authority: legacy.authority,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        keywords: None,
    }))
}

fn from_dataset_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Dataset(Box::new(Dataset {
        id: ctx.id,
        title: ctx.title,
        author: legacy.author.map(Contributor::from),
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| {
            Contributor::SimpleName(SimpleName {
                name: n.into(),
                location: legacy.publisher_place,
            })
        }),
        version: None,
        format: None,
        size: None,
        repository: None,
        doi: ctx.doi,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        keywords: None,
    }))
}

fn from_document_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let archive_info = archive_info_from_legacy_flat(&legacy);

    let mut numbering = Vec::new();
    if let Some(v) = legacy.volume {
        numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: v.to_string(),
        });
    }

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type: MonographType::Document,
        title: ctx.title,
        short_title: None,
        container: legacy.container_title.map(|t| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(t)),
                ..Default::default()
            }))))
        }),
        author: legacy.author.map(Contributor::from),
        editor: legacy.editor.map(Contributor::from),
        translator: legacy.translator.map(Contributor::from),
        recipient: legacy.recipient.map(Contributor::from),
        interviewer: legacy.interviewer.map(Contributor::from),
        guest: None,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| {
            Contributor::SimpleName(SimpleName {
                name: n.into(),
                location: legacy.publisher_place,
            })
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        isbn: ctx.isbn,
        doi: ctx.doi,
        ads_bibcode: None,
        numbering,
        genre: legacy.genre,
        medium: legacy.medium,
        archive: legacy.archive,
        archive_location: legacy.archive_location,
        archive_info,
        eprint: None,
        keywords: None,
        original: None,
        ..Default::default()
    }))
}

fn from_event_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Event(Box::new(Event {
        id: ctx.id,
        title: ctx.title,
        container: legacy.container_title.map(|t| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(t)),
                ..Default::default()
            }))))
        }),
        location: legacy.publisher_place.clone(),
        date: Some(ctx.issued),
        genre: legacy.genre,
        network: None,
        performer: legacy.author.map(Contributor::from),
        organizer: legacy.publisher.map(|n| {
            Contributor::SimpleName(SimpleName {
                name: n.into(),
                location: None,
            })
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
    }))
}

impl From<csl_legacy::csl_json::Reference> for InputReference {
    fn from(legacy: csl_legacy::csl_json::Reference) -> Self {
        let ctx = RefContext {
            id: Some(legacy.id.clone()),
            title: legacy.title.clone().map(Title::Single),
            issued: legacy
                .issued
                .clone()
                .map(EdtfString::from)
                .unwrap_or(EdtfString(String::new())),
            url: legacy.url.as_ref().and_then(|u| Url::parse(u).ok()),
            accessed: legacy.accessed.clone().map(EdtfString::from),
            language: legacy.language.clone(),
            note: legacy.note.clone(),
            doi: legacy.doi.clone(),
            isbn: legacy.isbn.clone(),
            edition: legacy.edition.as_ref().map(|e| e.to_string()),
            container_title_short: short_title_from_legacy(&legacy, "container-title-short"),
            journal_abbreviation: short_title_from_legacy(&legacy, "journalAbbreviation"),
        };

        match legacy.ref_type.as_str() {
            "software" => from_software_ref(legacy, ctx),
            "book"
            | "report"
            | "thesis"
            | "manual"
            | "manuscript"
            | "webpage"
            | "post"
            | "post-weblog"
            | "interview"
            | "personal_communication"
            | "personal-communication" => from_monograph_ref(legacy, ctx),
            "chapter" | "paper-conference" | "entry-dictionary" => {
                from_collection_component_ref(legacy, ctx)
            }
            "article-journal" | "article" | "article-magazine" | "article-newspaper"
            | "broadcast" | "motion_picture" | "entry-encyclopedia" => {
                from_serial_component_ref(legacy, ctx)
            }
            "speech" | "presentation" => from_event_ref(legacy, ctx),
            "legal-case" | "legal_case" => from_legal_case_ref(legacy, ctx),
            "statute" | "legislation" => from_statute_ref(legacy, ctx),
            "treaty" => from_treaty_ref(legacy, ctx),
            "standard" => from_standard_ref(legacy, ctx),
            "patent" => from_patent_ref(legacy, ctx),
            "dataset" => from_dataset_ref(legacy, ctx),
            _ => from_document_ref(legacy, ctx),
        }
    }
}

impl From<csl_legacy::csl_json::DateVariable> for EdtfString {
    fn from(date: csl_legacy::csl_json::DateVariable) -> Self {
        if let Some(literal) = date.literal {
            return EdtfString(literal);
        }
        if let Some(first) = date.date_parts.and_then(|p| p.first().cloned()) {
            let year = first
                .first()
                .map(|y| {
                    if *y < 0 {
                        format!("-{:04}", y.abs())
                    } else {
                        format!("{:04}", y)
                    }
                })
                .unwrap_or_default();
            let month = first
                .get(1)
                .map(|m| format!("-{:02}", m))
                .unwrap_or_default();
            let day = first
                .get(2)
                .map(|d| format!("-{:02}", d))
                .unwrap_or_default();
            return EdtfString(format!("{}{}{}", year, month, day));
        }
        EdtfString(String::new())
    }
}

impl From<Vec<csl_legacy::csl_json::Name>> for Contributor {
    fn from(names: Vec<csl_legacy::csl_json::Name>) -> Self {
        let contributors: Vec<Contributor> = names
            .into_iter()
            .map(|n| {
                if let Some(literal) = n.literal {
                    Contributor::SimpleName(SimpleName {
                        name: literal.into(),
                        location: None,
                    })
                } else {
                    Contributor::StructuredName(StructuredName {
                        given: n.given.unwrap_or_default().into(),
                        family: n.family.unwrap_or_default().into(),
                        suffix: n.suffix,
                        dropping_particle: n.dropping_particle,
                        non_dropping_particle: n.non_dropping_particle,
                    })
                }
            })
            .collect();
        Contributor::ContributorList(ContributorList(contributors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_report_number_maps_to_report_numbering() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "report-1".to_string(),
            ref_type: "report".to_string(),
            title: Some("Report".to_string()),
            issued: Some(csl_legacy::csl_json::DateVariable {
                date_parts: Some(vec![vec![2024]]),
                ..Default::default()
            }),
            number: Some("TR-7".to_string()),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        assert_eq!(converted.number(), None);
        assert_eq!(converted.report_number(), Some("TR-7".to_string()));
    }

    #[test]
    fn legacy_book_number_maps_to_generic_numbering() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "book-1".to_string(),
            ref_type: "book".to_string(),
            title: Some("Book".to_string()),
            issued: Some(csl_legacy::csl_json::DateVariable {
                date_parts: Some(vec![vec![2024]]),
                ..Default::default()
            }),
            number: Some("2".to_string()),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        assert_eq!(converted.number(), Some("2".to_string()));
        assert_eq!(converted.report_number(), None);
    }
}
