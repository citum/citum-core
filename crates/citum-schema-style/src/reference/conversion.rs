use crate::reference::InputReference;
use crate::reference::contributor::{Contributor, ContributorList, SimpleName, StructuredName};
use crate::reference::date::EdtfString;
use crate::reference::types::{
    Collection, CollectionComponent, CollectionType, Dataset, LegalCase, Monograph,
    MonographComponentType, MonographType, NumOrStr, Parent, Patent, Serial, SerialComponent,
    SerialComponentType, SerialType, Software, Standard, Statute, Title, Treaty,
};
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

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type,
        title: ctx.title,
        container_title: legacy.container_title.clone().map(Title::Single),
        author: legacy.author.map(Contributor::from),
        editor: legacy.editor.map(Contributor::from),
        translator: legacy.translator.map(Contributor::from),
        recipient: legacy.recipient.map(Contributor::from),
        interviewer: legacy.interviewer.map(Contributor::from),
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
        edition: ctx.edition,
        report_number: legacy.number,
        collection_number: legacy.collection_number.map(|v| v.to_string()),
        genre: legacy.genre,
        medium: legacy.medium,
        archive: legacy.archive,
        archive_location: legacy.archive_location,
        keywords: None,
        original_date: None,
        original_title: legacy.original_title.map(Title::Single),
    }))
}

fn from_collection_component_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let parent_title = legacy.container_title.map(Title::Single);
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
        parent: Parent::Embedded(Collection {
            id: None,
            r#type: CollectionType::EditedBook,
            title: parent_title,
            short_title: ctx.container_title_short,
            editor: legacy.editor.map(Contributor::from),
            translator: None,
            issued: EdtfString(String::new()),
            publisher: legacy.publisher.map(|n| {
                Contributor::SimpleName(SimpleName {
                    name: n.into(),
                    location: legacy.publisher_place,
                })
            }),
            collection_number: legacy.collection_number.map(|v| v.to_string()).or(legacy
                .volume
                .as_ref()
                .map(|v| match v {
                    csl_legacy::csl_json::StringOrNumber::String(s) => s.clone(),
                    csl_legacy::csl_json::StringOrNumber::Number(n) => n.to_string(),
                })),
            url: None,
            accessed: None,
            language: None,
            field_languages: HashMap::new(),
            note: None,
            isbn: None,
            keywords: None,
        }),
        pages: legacy.page.map(NumOrStr::Str),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        doi: ctx.doi,
        genre: legacy.genre,
        medium: legacy.medium,
        keywords: None,
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
    InputReference::SerialComponent(Box::new(SerialComponent {
        id: ctx.id,
        r#type: SerialComponentType::Article,
        title: ctx.title,
        author: legacy.author.map(Contributor::from),
        translator: legacy.translator.map(Contributor::from),
        issued: ctx.issued,
        parent: Parent::Embedded(Serial {
            r#type: serial_type,
            title: parent_title,
            short_title: ctx.container_title_short.or(ctx.journal_abbreviation),
            editor: None,
            publisher: legacy.publisher.clone().map(|n| {
                Contributor::SimpleName(SimpleName {
                    name: n.into(),
                    location: legacy.publisher_place.clone(),
                })
            }),
            issn: legacy.issn,
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        doi: ctx.doi,
        ads_bibcode: None,
        pages: legacy.page,
        volume: legacy.volume.map(|v| match v {
            csl_legacy::csl_json::StringOrNumber::String(s) => NumOrStr::Str(s),
            csl_legacy::csl_json::StringOrNumber::Number(n) => NumOrStr::Number(n),
        }),
        issue: legacy
            .issue
            .or_else(|| {
                if legacy.ref_type == "broadcast" || legacy.ref_type == "motion_picture" {
                    legacy
                        .number
                        .as_ref()
                        .map(|n| csl_legacy::csl_json::StringOrNumber::String(n.clone()))
                } else {
                    None
                }
            })
            .map(|v| match v {
                csl_legacy::csl_json::StringOrNumber::String(s) => NumOrStr::Str(s),
                csl_legacy::csl_json::StringOrNumber::Number(n) => NumOrStr::Number(n),
            }),
        genre,
        medium: legacy.medium,
        keywords: None,
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
    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type: MonographType::Document,
        title: ctx.title,
        container_title: legacy.container_title.clone().map(Title::Single),
        author: legacy.author.map(Contributor::from),
        editor: legacy.editor.map(Contributor::from),
        translator: legacy.translator.map(Contributor::from),
        recipient: legacy.recipient.map(Contributor::from),
        interviewer: legacy.interviewer.map(Contributor::from),
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
        edition: ctx.edition,
        report_number: legacy.number,
        collection_number: legacy.collection_number.map(|v| v.to_string()),
        genre: legacy.genre,
        medium: legacy.medium,
        archive: legacy.archive,
        archive_location: legacy.archive_location,
        keywords: None,
        original_date: None,
        original_title: legacy.original_title.map(Title::Single),
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
        if let Some(parts) = date.date_parts
            && let Some(first) = parts.first()
        {
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
