use crate::reference::contributor::{
    Contributor, ContributorEntry, ContributorList, ContributorRole, SimpleName, StructuredName,
};
use crate::reference::date::EdtfString;
use crate::reference::types::{
    ArchiveInfo, Collection, CollectionComponent, CollectionType, Dataset, LegalCase, Monograph,
    MonographComponentType, MonographType, NumOrStr, Patent, Publisher, Regulation, Serial,
    SerialComponent, SerialComponentType, SerialType, Software, Standard, Statute, StructuredTitle,
    Subtitle, Title, Treaty,
};
use crate::reference::{
    AudioVisualType, AudioVisualWork, Event, InputReference, Numbering, NumberingType, WorkCore,
    WorkRelation,
};
use std::collections::HashMap;
use url::Url;

/// Fold legacy named contributor fields (recipient and interviewer) into a contributors vec.
fn legacy_named_contributors(legacy: &csl_legacy::csl_json::Reference) -> Vec<ContributorEntry> {
    let mut entries = Vec::new();
    push_legacy_contributor(
        &mut entries,
        ContributorRole::Recipient,
        legacy.recipient.clone(),
    );
    push_legacy_contributor(
        &mut entries,
        ContributorRole::Interviewer,
        legacy.interviewer.clone(),
    );
    entries
}

fn push_legacy_contributor(
    entries: &mut Vec<ContributorEntry>,
    role: ContributorRole,
    src: Option<Vec<csl_legacy::csl_json::Name>>,
) {
    if let Some(names) = src {
        entries.push(ContributorEntry {
            role,
            contributor: Contributor::from(names),
        });
    }
}

fn legacy_extra_names(
    legacy: &csl_legacy::csl_json::Reference,
    key: &str,
) -> Option<Vec<csl_legacy::csl_json::Name>> {
    legacy
        .extra
        .get(key)
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
}

fn legacy_has_audio_visual_roles(legacy: &csl_legacy::csl_json::Reference) -> bool {
    legacy.director.is_some()
        || legacy_extra_names(legacy, "composer").is_some()
        || legacy_extra_names(legacy, "performer").is_some()
        || legacy_extra_names(legacy, "producer").is_some()
}

fn short_title_from_legacy(legacy: &csl_legacy::csl_json::Reference, key: &str) -> Option<String> {
    legacy
        .extra
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

/// Build a title, optionally structured if short_title is present and title contains a colon.
fn build_title(title: Option<String>, short_title: Option<String>) -> Option<Title> {
    match (title, short_title) {
        (Some(full_title), Some(short)) => {
            if let Some(colon_pos) = full_title.find(':') {
                let potential_main = full_title[..colon_pos].trim();
                // Check if short title matches pre-colon portion
                if potential_main.eq_ignore_ascii_case(short.as_str())
                    || potential_main.contains(&short)
                {
                    let post_colon = full_title[colon_pos + 1..].trim();
                    return Some(Title::Structured(StructuredTitle {
                        full: None,
                        main: short,
                        sub: Subtitle::String(post_colon.to_string()),
                    }));
                }
            }
            // Fallback: just use the full title
            Some(Title::Single(full_title))
        }
        (Some(title), None) => Some(Title::Single(title)),
        _ => None,
    }
}

fn archive_info_from_legacy_flat(legacy: &csl_legacy::csl_json::Reference) -> Option<ArchiveInfo> {
    if legacy.archive.is_none() && legacy.archive_location.is_none() {
        return None;
    }

    let collection = legacy
        .extra
        .get("archive-collection")
        .or_else(|| legacy.extra.get("archive_collection"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    Some(ArchiveInfo {
        name: legacy.archive.clone().map(Into::into),
        location: legacy.archive_location.clone(),
        collection,
        ..Default::default()
    })
}

/// Pre-extracted common fields shared by all reference conversion functions.
struct RefContext {
    id: Option<String>,
    title: Option<String>,
    short_title: Option<String>,
    created: EdtfString,
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

fn legacy_type_uses_created(ref_type: &str) -> bool {
    matches!(
        ref_type,
        "manuscript"
            | "interview"
            | "personal_communication"
            | "personal-communication"
            | "speech"
            | "presentation"
    )
}

fn from_software_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Software(Box::new(Software {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title),
        author: legacy.author.map(Contributor::from),
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place,
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
    let archive = match archive_info.as_ref() {
        Some(ai) if ai.collection.is_some() => None,
        _ => legacy.archive.clone(),
    };

    // Use report numbering only for Report type; all standard types use shorthand fields.
    let numbering = if r#type == MonographType::Report {
        legacy
            .number
            .as_ref()
            .map(|n| {
                vec![Numbering {
                    r#type: NumberingType::Report,
                    value: n.clone(),
                }]
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let author = legacy.author.clone().map(Contributor::from);
    let editor = legacy.editor.clone().map(Contributor::from);
    let translator = legacy.translator.clone().map(Contributor::from);
    let mut contributors = legacy_named_contributors(&legacy);
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Author,
        legacy.author.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Editor,
        legacy.editor.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Translator,
        legacy.translator.clone(),
    );
    let edition = ctx.edition;
    let volume = legacy
        .volume
        .map(|v| v.to_string())
        .or_else(|| legacy.collection_number.map(|cn| cn.to_string()));
    let number = if r#type == MonographType::Report {
        None
    } else {
        legacy.number
    };
    let original = legacy.original_title.map(|original_title| {
        WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
            title: Some(Title::Single(original_title)),
            ..Default::default()
        }))))
    });

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type,
        title: build_title(ctx.title, ctx.short_title.clone()),
        short_title: None,
        container: legacy.container_title.map(|t| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(t)),
                ..Default::default()
            }))))
        }),
        author,
        editor,
        translator,
        contributors,
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place,
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        isbn: ctx.isbn,
        doi: ctx.doi,
        ads_bibcode: None,
        volume,
        issue: None,
        edition,
        number,
        numbering,
        genre: legacy.genre,
        medium: legacy.medium,
        archive,
        archive_location: legacy.archive_location,
        archive_info,
        eprint: None,
        keywords: None,
        original,
    }))
}

fn from_collection_component_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let parent_title = legacy.container_title.map(Title::Single);
    let parent_volume = legacy
        .collection_number
        .clone()
        .or_else(|| legacy.volume.clone())
        .map(|v| v.to_string());

    let author = legacy.author.clone().map(Contributor::from);
    let translator = legacy.translator.clone().map(Contributor::from);
    let mut contributors = Vec::new();
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Author,
        legacy.author.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Translator,
        legacy.translator.clone(),
    );
    let container_editor = legacy.editor.clone().map(Contributor::from);
    let mut container_contributors = Vec::new();
    push_legacy_contributor(
        &mut container_contributors,
        ContributorRole::Editor,
        legacy.editor.clone(),
    );

    InputReference::CollectionComponent(Box::new(CollectionComponent {
        id: ctx.id,
        r#type: if legacy.ref_type == "paper-conference" {
            MonographComponentType::Document
        } else {
            MonographComponentType::Chapter
        },
        title: build_title(ctx.title, ctx.short_title.clone()),
        author,
        translator,
        contributors,
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(
            InputReference::Collection(Box::new(Collection {
                id: None,
                r#type: CollectionType::EditedBook,
                title: parent_title,
                short_title: ctx.container_title_short,
                container: None,
                editor: container_editor,
                translator: None,
                contributors: container_contributors,
                created: EdtfString(String::new()),
                issued: EdtfString(String::new()),
                publisher: legacy.publisher.map(|n| Publisher {
                    name: n.into(),
                    place: legacy.publisher_place,
                }),
                volume: parent_volume,
                ..Default::default()
            })),
        ))),
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

    let editor_value = editor.clone().map(Contributor::from);
    let translator_value = translator.clone().map(Contributor::from);
    let mut contributors = Vec::new();
    push_legacy_contributor(&mut contributors, ContributorRole::Editor, editor);
    push_legacy_contributor(&mut contributors, ContributorRole::Translator, translator);

    InputReference::Collection(Box::new(Collection {
        id: Some(id),
        r#type: CollectionType::EditedBook,
        title: title.map(Title::Single),
        short_title: extra
            .get("shortTitle")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        container: None,
        editor: editor_value,
        translator: translator_value,
        contributors,
        created: EdtfString(String::new()),
        issued: issued
            .map(EdtfString::from)
            .unwrap_or(EdtfString(String::new())),
        publisher: publisher.map(|name| Publisher {
            name: name.into(),
            place: publisher_place.clone(),
        }),
        numbering,
        url: url.as_deref().and_then(|value| Url::parse(value).ok()),
        accessed: accessed.map(EdtfString::from),
        language,
        field_languages: HashMap::new(),
        note,
        isbn,
        volume: None,
        issue: None,
        edition: None,
        number: None,
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

    let volume = legacy.volume.map(|v| v.to_string());
    let issue = legacy
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
        .map(|i| i.to_string());

    let author = legacy.author.clone().map(Contributor::from);
    let translator = legacy.translator.clone().map(Contributor::from);
    let mut contributors = Vec::new();
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Author,
        legacy.author.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Translator,
        legacy.translator.clone(),
    );
    let serial_editor = legacy.editor.clone().map(Contributor::from);
    let mut serial_contributors = Vec::new();
    push_legacy_contributor(
        &mut serial_contributors,
        ContributorRole::Editor,
        legacy.editor.clone(),
    );

    InputReference::SerialComponent(Box::new(SerialComponent {
        id: ctx.id,
        r#type: SerialComponentType::Article,
        title: build_title(ctx.title, ctx.short_title.clone()),
        author,
        translator,
        contributors,
        created: ctx.created,
        issued: ctx.issued,
        container: Some(WorkRelation::Embedded(Box::new(InputReference::Serial(
            Box::new(Serial {
                id: None,
                r#type: serial_type,
                title: parent_title,
                short_title: ctx.container_title_short.or(ctx.journal_abbreviation),
                container: None,
                editor: serial_editor,
                contributors: serial_contributors,
                publisher: legacy.publisher.clone().map(|n| Publisher {
                    name: n.into(),
                    place: legacy.publisher_place.clone(),
                }),
                url: None,
                accessed: None,
                language: None,
                field_languages: HashMap::new(),
                note: None,
                issn: legacy.issn,
            }),
        )))),
        volume,
        issue,
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

fn from_audio_visual_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let r#type = match legacy.ref_type.as_str() {
        "motion_picture" => AudioVisualType::Film,
        "broadcast" => AudioVisualType::Broadcast,
        _ => AudioVisualType::Broadcast,
    };
    let mut contributors = Vec::new();
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Director,
        legacy.director.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Composer,
        legacy_extra_names(&legacy, "composer"),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Performer,
        legacy_extra_names(&legacy, "performer"),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Producer,
        legacy_extra_names(&legacy, "producer"),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Author,
        legacy.author.clone(),
    );

    InputReference::AudioVisual(Box::new(AudioVisualWork {
        id: ctx.id,
        r#type,
        core: WorkCore {
            title: build_title(ctx.title, ctx.short_title.clone()),
            short_title: None,
            contributors,
            created: ctx.created,
            issued: ctx.issued,
            language: ctx.language,
            genre: legacy.genre,
        },
        container: legacy.container_title.map(|title| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(title)),
                ..Default::default()
            }))))
        }),
        numbering: legacy
            .number
            .into_iter()
            .map(|number| Numbering {
                r#type: NumberingType::Number,
                value: number,
            })
            .collect(),
        publisher: legacy.publisher.map(|name| Publisher {
            name: name.into(),
            place: legacy.publisher_place,
        }),
        medium: legacy.medium,
        platform: None,
        url: ctx.url,
        accessed: ctx.accessed,
        field_languages: HashMap::new(),
        note: ctx.note,
    }))
}

fn from_legal_case_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::LegalCase(Box::new(LegalCase {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        authority: legacy.authority,
        volume: legacy.volume.map(|v| v.to_string()),
        reporter: legacy.container_title,
        page: legacy.page,
        created: ctx.created,
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
        title: build_title(ctx.title, ctx.short_title.clone()),
        authority: legacy.authority,
        volume: legacy.volume.map(|v| v.to_string()),
        code: legacy.container_title,
        number: legacy.number,
        page: legacy.page,
        created: ctx.created,
        section: legacy.section,
        chapter_number: legacy.chapter_number,
        issued: ctx.issued,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        keywords: None,
    }))
}

fn from_regulation_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    InputReference::Regulation(Box::new(Regulation {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        authority: legacy.authority,
        volume: legacy.volume.map(|v| v.to_string()),
        code: legacy.container_title,
        created: ctx.created,
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
        title: build_title(ctx.title, ctx.short_title.clone()),
        author: legacy.author.map(Contributor::from),
        volume: legacy.volume.map(|v| v.to_string()),
        reporter: legacy.container_title,
        page: legacy.page,
        created: ctx.created,
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
        title: build_title(ctx.title, ctx.short_title.clone()),
        authority: legacy.authority,
        standard_number: legacy.number.unwrap_or_default(),
        created: ctx.created,
        issued: ctx.issued,
        status: None,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place,
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
        title: build_title(ctx.title, ctx.short_title.clone()),
        author: legacy.author.map(Contributor::from),
        assignee: None,
        patent_number: legacy.number.unwrap_or_default(),
        application_number: None,
        created: ctx.created,
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
        title: build_title(ctx.title, ctx.short_title.clone()),
        author: legacy.author.map(Contributor::from),
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place,
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

fn from_bill_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let titleless_proceeding =
        legacy.title.is_none() && (legacy.authority.is_some() || legacy.chapter_number.is_some());
    let titleless_record = legacy.title.is_none()
        && legacy.container_title.is_some()
        && legacy.volume.is_some()
        && legacy.page.is_some();

    if !(titleless_proceeding || titleless_record) {
        return from_document_ref(legacy, ctx);
    }

    let mut numbering = Vec::new();
    if let Some(chapter) = legacy.chapter_number.clone() {
        numbering.push(Numbering {
            r#type: NumberingType::Chapter,
            value: chapter,
        });
    }

    let genre = if titleless_proceeding {
        "bill-proceeding"
    } else {
        "bill-record"
    };

    let title = if titleless_record {
        build_title(ctx.title, ctx.short_title)
            .or_else(|| legacy.container_title.clone().map(Title::Single))
    } else {
        build_title(ctx.title, ctx.short_title)
            .or_else(|| legacy.number.clone().map(Title::Single))
            .or_else(|| legacy.container_title.clone().map(Title::Single))
    };

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type: MonographType::Document,
        title,
        short_title: None,
        container: None,
        author: None,
        editor: None,
        translator: None,
        contributors: Vec::new(),
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.authority.map(|name| Publisher {
            name: name.into(),
            place: None,
        }),
        volume: legacy.volume.map(|v| v.to_string()),
        number: legacy.page,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        isbn: ctx.isbn,
        doi: ctx.doi,
        ads_bibcode: None,
        numbering,
        genre: Some(genre.to_string()),
        medium: legacy.medium,
        archive: legacy.archive,
        archive_location: legacy.archive_location,
        archive_info: None,
        eprint: None,
        keywords: None,
        original: None,
        ..Default::default()
    }))
}

fn from_document_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let archive_info = archive_info_from_legacy_flat(&legacy);
    let archive = match archive_info.as_ref() {
        Some(ai) if ai.collection.is_some() => None,
        _ => legacy.archive.clone(),
    };

    let volume = legacy.volume.map(|v| v.to_string());
    let number = legacy.number.clone();
    let author = legacy.author.clone().map(Contributor::from);
    let editor = legacy.editor.clone().map(Contributor::from);
    let translator = legacy.translator.clone().map(Contributor::from);
    let mut contributors = Vec::new();
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Author,
        legacy.author.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Editor,
        legacy.editor.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Translator,
        legacy.translator.clone(),
    );

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type: MonographType::Document,
        title: build_title(ctx.title, ctx.short_title.clone()),
        short_title: None,
        container: legacy.container_title.map(|t| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(t)),
                ..Default::default()
            }))))
        }),
        author,
        editor,
        translator,
        contributors,
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place,
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note,
        isbn: ctx.isbn,
        doi: ctx.doi,
        ads_bibcode: None,
        volume,
        number,
        genre: legacy.genre,
        medium: legacy.medium,
        archive,
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
        title: build_title(ctx.title, ctx.short_title.clone()),
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
    fn from(mut legacy: csl_legacy::csl_json::Reference) -> Self {
        legacy.parse_note_field_hacks();
        let ctx = RefContext {
            id: Some(legacy.id.clone()),
            title: legacy.title.clone(),
            short_title: short_title_from_legacy(&legacy, "shortTitle")
                .or_else(|| short_title_from_legacy(&legacy, "title-short")),
            created: if legacy_type_uses_created(&legacy.ref_type) {
                legacy
                    .issued
                    .clone()
                    .map(EdtfString::from)
                    .unwrap_or(EdtfString(String::new()))
            } else {
                EdtfString(String::new())
            },
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
            | "entry-encyclopedia" => from_serial_component_ref(legacy, ctx),
            "motion_picture" => from_audio_visual_ref(legacy, ctx),
            "broadcast" => {
                if legacy_has_audio_visual_roles(&legacy) {
                    from_audio_visual_ref(legacy, ctx)
                } else {
                    from_serial_component_ref(legacy, ctx)
                }
            }
            "speech" | "presentation" => from_event_ref(legacy, ctx),
            "bill" => from_bill_ref(legacy, ctx),
            "legal-case" | "legal_case" => from_legal_case_ref(legacy, ctx),
            "statute" | "legislation" => from_statute_ref(legacy, ctx),
            "regulation" => from_regulation_ref(legacy, ctx),
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
                    let given_str = n.given.as_deref().map(str::trim).unwrap_or("");
                    if given_str.is_empty()
                        && n.dropping_particle.is_none()
                        && n.non_dropping_particle.is_none()
                    {
                        // No given name and no particles: treat family as a literal name
                        Contributor::SimpleName(SimpleName {
                            name: n.family.unwrap_or_default().into(),
                            location: None,
                        })
                    } else {
                        Contributor::StructuredName(StructuredName {
                            given: given_str.to_string().into(),
                            family: n.family.unwrap_or_default().into(),
                            suffix: n.suffix,
                            dropping_particle: n.dropping_particle,
                            non_dropping_particle: n.non_dropping_particle,
                        })
                    }
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
