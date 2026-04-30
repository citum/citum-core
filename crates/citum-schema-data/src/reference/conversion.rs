use crate::reference::contributor::{
    Contributor, ContributorEntry, ContributorList, ContributorRole, SimpleName, StructuredName,
};
use crate::reference::date::EdtfString;
use crate::reference::types::{
    ArchiveInfo, Collection, CollectionComponent, CollectionType, Dataset, LegalCase, Monograph,
    MonographComponentType, MonographType, NumOrStr, Patent, Publisher, Regulation, RichText,
    Serial, SerialComponent, SerialComponentType, SerialType, Software, Standard, Statute,
    StructuredTitle, Subtitle, Title, Treaty,
};
use crate::reference::{
    AudioVisualType, AudioVisualWork, Event, InputReference, LangID, Numbering, NumberingType,
    RefID, WorkCore, WorkRelation,
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
            gender: None,
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

fn legacy_extra_str(legacy: &csl_legacy::csl_json::Reference, key: &str) -> Option<String> {
    legacy
        .extra
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn legacy_extra_date(legacy: &csl_legacy::csl_json::Reference, key: &str) -> Option<EdtfString> {
    legacy
        .extra
        .get(key)
        .and_then(|value| {
            serde_json::from_value::<csl_legacy::csl_json::DateVariable>(value.clone()).ok()
        })
        .map(EdtfString::from)
        .or_else(|| {
            legacy
                .extra
                .get(key)
                .and_then(serde_json::Value::as_str)
                .map(|value| EdtfString(value.to_string()))
        })
}

fn legacy_extra_contributor(
    legacy: &csl_legacy::csl_json::Reference,
    key: &str,
) -> Option<Contributor> {
    legacy_extra_names(legacy, key).map(Contributor::from)
}

fn relation_monograph(
    title: Option<Title>,
    author: Option<Contributor>,
    issued: Option<EdtfString>,
    genre: Option<String>,
    publisher: Option<String>,
    publisher_place: Option<String>,
) -> Option<WorkRelation> {
    if title.is_none()
        && author.is_none()
        && issued.is_none()
        && genre.is_none()
        && publisher.is_none()
        && publisher_place.is_none()
    {
        return None;
    }

    let publisher = match (publisher, publisher_place) {
        (Some(name), place) => Some(Publisher {
            name: name.into(),
            place: place.map(Into::into),
        }),
        (None, Some(place)) => Some(Publisher {
            name: String::new().into(),
            place: Some(place.into()),
        }),
        (None, None) => None,
    };

    Some(WorkRelation::Embedded(Box::new(InputReference::Monograph(
        Box::new(Monograph {
            title,
            author,
            issued: issued.unwrap_or_default(),
            genre,
            publisher,
            ..Default::default()
        }),
    ))))
}

fn legacy_original_relation(legacy: &csl_legacy::csl_json::Reference) -> Option<WorkRelation> {
    relation_monograph(
        legacy.original_title.clone().map(Title::Single),
        legacy_extra_contributor(legacy, "original-author"),
        legacy_extra_date(legacy, "original-date"),
        None,
        legacy_extra_str(legacy, "original-publisher"),
        legacy_extra_str(legacy, "original-publisher-place"),
    )
}

fn relation_event(
    title: Option<String>,
    location: Option<String>,
    date: Option<EdtfString>,
) -> Option<WorkRelation> {
    if title.is_none() && location.is_none() && date.is_none() {
        return None;
    }
    Some(WorkRelation::Embedded(Box::new(InputReference::Event(
        Box::new(Event {
            title: title.map(Title::Single),
            location,
            date,
            ..Default::default()
        }),
    ))))
}

fn short_title_from_legacy(legacy: &csl_legacy::csl_json::Reference, key: &str) -> Option<String> {
    legacy_extra_str(legacy, key)
}

fn normalize_broadcast_issue(
    ref_type: &str,
    medium: Option<&str>,
    number: &str,
) -> csl_legacy::csl_json::StringOrNumber {
    let normalized = if matches!(ref_type, "broadcast" | "motion_picture")
        && medium
            .map(|value| value.to_ascii_lowercase().contains("podcast"))
            .unwrap_or(false)
        && number.chars().all(|ch| ch.is_ascii_digit())
    {
        format!("No. {number}")
    } else {
        number.to_string()
    };

    csl_legacy::csl_json::StringOrNumber::String(normalized)
}

/// Build a title, optionally structured if short_title is present and title contains a colon.
fn build_title(title: Option<String>, short_title: Option<String>) -> Option<Title> {
    match (title, short_title) {
        (Some(full_title), Some(short)) => {
            if let Some(colon_pos) = full_title.find(':') {
                #[allow(
                    clippy::string_slice,
                    reason = "colon_pos is found via find(':'), which is a 1-byte ASCII boundary"
                )]
                let potential_main = full_title[..colon_pos].trim();
                // Check if short title matches pre-colon portion
                if potential_main.eq_ignore_ascii_case(short.as_str())
                    || potential_main.contains(&short)
                {
                    #[allow(
                        clippy::string_slice,
                        reason = "colon_pos + 1 is a valid boundary after ':' (1-byte ASCII)"
                    )]
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
    let place = legacy
        .extra
        .get("archive-place")
        .or_else(|| legacy.extra.get("archive_place"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    Some(ArchiveInfo {
        name: legacy.archive.clone().map(Into::into),
        location: legacy.archive_location.clone(),
        place: place.map(Into::into),
        collection,
        ..Default::default()
    })
}

/// Pre-extracted common fields shared by all reference conversion functions.
struct RefContext {
    id: Option<RefID>,
    title: Option<String>,
    short_title: Option<String>,
    created: EdtfString,
    issued: EdtfString,
    url: Option<Url>,
    accessed: Option<EdtfString>,
    language: Option<LangID>,
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
    let original = legacy_original_relation(&legacy);
    InputReference::Software(Box::new(Software {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title),
        original,
        author: legacy.author.map(Contributor::from),
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place.map(Into::into),
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
        note: ctx.note.map(RichText::Plain),
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
    let mut numbering = if r#type == MonographType::Report {
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
    let original_author = legacy_extra_contributor(&legacy, "original-author");
    let original_date = legacy_extra_date(&legacy, "original-date");
    let original_publisher = legacy_extra_str(&legacy, "original-publisher");
    let original_publisher_place = legacy_extra_str(&legacy, "original-publisher-place");
    let volume_title = legacy_extra_str(&legacy, "volume-title");
    let part_title = legacy_extra_str(&legacy, "part-title");
    let part_number = legacy_extra_str(&legacy, "part-number");
    let status = legacy_extra_str(&legacy, "status");
    let available_date = legacy_extra_date(&legacy, "available-date");
    let references = legacy_extra_str(&legacy, "references");
    let scale = legacy_extra_str(&legacy, "scale");
    let dimensions = legacy_extra_str(&legacy, "dimensions");
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
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Host,
        legacy_extra_names(&legacy, "host"),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Narrator,
        legacy_extra_names(&legacy, "narrator"),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Compiler,
        legacy_extra_names(&legacy, "compiler"),
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
        legacy_extra_names(&legacy, "producer")
            .or_else(|| legacy_extra_names(&legacy, "executive-producer")),
    );
    let edition = ctx.edition;
    let volume = legacy
        .volume
        .map(|v| v.to_string())
        .or_else(|| legacy.collection_number.clone().map(|cn| cn.to_string()));
    let number = if r#type == MonographType::Report {
        None
    } else {
        legacy.number.clone()
    };
    let original = relation_monograph(
        legacy.original_title.clone().map(Title::Single),
        original_author,
        original_date,
        None,
        original_publisher,
        original_publisher_place,
    );

    let title = if r#type == MonographType::Webpage {
        ctx.title.clone().map(|base_title| {
            let mut combined = base_title;
            if let Some(part_num) = part_number.as_ref() {
                combined.push_str(": Pt. ");
                combined.push_str(part_num);
            }
            if let Some(part) = part_title.as_ref() {
                if part_number.is_none() {
                    combined.push(':');
                    combined.push(' ');
                } else {
                    combined.push('.');
                    combined.push(' ');
                }
                combined.push_str(part);
            }
            combined
        })
    } else {
        part_title.or(ctx.title)
    };

    // Batch 1: volume-title enriches container; part-number adds a Part numbering entry.
    let container = {
        let base_title = legacy.container_title.clone().map(Title::Single);
        let effective_title = volume_title.map(Title::Single).or(base_title);
        effective_title.map(|t| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(t),
                ..Default::default()
            }))))
        })
    };
    if let Some(part_num) = part_number {
        numbering.push(Numbering {
            r#type: NumberingType::Part,
            value: part_num,
        });
    }
    if let Some(chapter_num) = legacy.chapter_number.clone()
        && numbering.iter().all(|n| n.r#type != NumberingType::Chapter)
    {
        numbering.push(Numbering {
            r#type: NumberingType::Chapter,
            value: chapter_num,
        });
    }

    // Map CSL `dimensions` to size (freeform) or duration (ISO 8601 / HH:MM pattern).
    let (size, duration) = match dimensions {
        Some(ref d)
            if d.starts_with('P')
                || d.starts_with("PT")
                || d.chars().next().is_some_and(|c| c.is_ascii_digit()) && d.contains(':') =>
        {
            (None, dimensions)
        }
        Some(_) => (dimensions, None),
        None => (None, None),
    };

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type,
        title: build_title(title, ctx.short_title.clone()),
        short_title: None,
        container,
        author,
        editor,
        translator,
        contributors,
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place.map(Into::into),
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
        abstract_text: None,
        isbn: ctx.isbn,
        doi: ctx.doi,
        ads_bibcode: None,
        volume,
        issue: None,
        edition,
        number,
        part_number: None,
        supplement_number: None,
        printing_number: legacy.printing_number,
        numbering,
        genre: legacy.genre,
        medium: legacy.medium,
        archive,
        archive_location: legacy.archive_location,
        archive_info,
        eprint: None,
        keywords: None,
        original,
        status,
        available_date,
        size,
        duration,
        references,
        scale,
    }))
}

#[allow(
    clippy::too_many_lines,
    reason = "Legacy CSL collection-component mapping requires extensive field wiring"
)]
fn from_collection_component_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let (genre, status) = collection_component_metadata(&legacy);
    let part_title = legacy_extra_str(&legacy, "part-title");
    let part_number = legacy_extra_str(&legacy, "part-number");
    let original_author = legacy_extra_contributor(&legacy, "original-author");
    let original_date = legacy_extra_date(&legacy, "original-date");
    let original_publisher = legacy_extra_str(&legacy, "original-publisher");
    let original_publisher_place = legacy_extra_str(&legacy, "original-publisher-place");
    let parent_title = legacy.container_title.clone().map(Title::Single);
    let parent_volume = legacy
        .collection_number
        .clone()
        .or_else(|| legacy.volume.clone())
        .map(|v| v.to_string());
    let parent_edition = ctx.edition.clone();
    let container_author = legacy_extra_names(&legacy, "container-author").map(Contributor::from);
    let has_named_parent = parent_title.is_some() || container_author.is_some();

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
    if !has_named_parent {
        push_legacy_contributor(
            &mut contributors,
            ContributorRole::Editor,
            legacy.editor.clone(),
        );
    }
    if let Some(container_author) = container_author.clone() {
        contributors.push(ContributorEntry {
            role: ContributorRole::Custom("container-author".to_string()),
            contributor: container_author,
            gender: None,
        });
    }
    let container_editor = has_named_parent
        .then(|| legacy.editor.clone().map(Contributor::from))
        .flatten();
    let mut container_contributors = Vec::new();
    if has_named_parent {
        push_legacy_contributor(
            &mut container_contributors,
            ContributorRole::Editor,
            legacy.editor.clone(),
        );
    }

    let container = if legacy.ref_type == "report" {
        Some(WorkRelation::Embedded(Box::new(InputReference::Monograph(
            Box::new(Monograph {
                r#type: MonographType::Report,
                title: parent_title,
                editor: container_editor,
                contributors: container_contributors,
                publisher: legacy.publisher.map(|n| Publisher {
                    name: n.into(),
                    place: legacy.publisher_place.map(Into::into),
                }),
                edition: parent_edition.clone(),
                numbering: legacy
                    .number
                    .as_ref()
                    .map(|number| {
                        vec![Numbering {
                            r#type: NumberingType::Report,
                            value: number.clone(),
                        }]
                    })
                    .unwrap_or_default(),
                genre: legacy.genre.clone(),
                ..Default::default()
            }),
        ))))
    } else {
        let event_relation = if legacy.ref_type == "paper-conference" {
            let event_title = legacy_extra_str(&legacy, "event-title");
            let event_place = legacy_extra_str(&legacy, "event-place")
                .or_else(|| legacy_extra_str(&legacy, "event-location"));
            let event_date = legacy_extra_date(&legacy, "event-date");
            relation_event(event_title, event_place, event_date)
        } else {
            None
        };

        Some(WorkRelation::Embedded(Box::new(
            InputReference::Collection(Box::new(Collection {
                id: None,
                r#type: if legacy.ref_type == "paper-conference" {
                    CollectionType::Proceedings
                } else {
                    CollectionType::EditedBook
                },
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
                    place: legacy.publisher_place.map(Into::into),
                }),
                edition: parent_edition,
                numbering: parent_volume
                    .clone()
                    .map(|volume| {
                        vec![Numbering {
                            r#type: NumberingType::Volume,
                            value: volume,
                        }]
                    })
                    .unwrap_or_default(),
                event: event_relation,
                ..Default::default()
            })),
        )))
    };

    InputReference::CollectionComponent(Box::new(CollectionComponent {
        id: ctx.id,
        r#type: if legacy.ref_type == "paper-conference" {
            MonographComponentType::Document
        } else {
            MonographComponentType::Chapter
        },
        title: build_title(part_title.or(ctx.title), ctx.short_title.clone()),
        author,
        translator,
        contributors,
        issued: ctx.issued,
        container,
        edition: ctx.edition,
        pages: legacy.page.map(NumOrStr::Str),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
        doi: ctx.doi,
        genre,
        medium: legacy.medium,
        status,
        numbering: {
            let mut numbering = Vec::new();
            if let Some(part_number) = part_number {
                numbering.push(Numbering {
                    r#type: NumberingType::Part,
                    value: part_number,
                });
            }
            if let Some(chapter_number) = legacy.chapter_number.clone() {
                numbering.push(Numbering {
                    r#type: NumberingType::Chapter,
                    value: chapter_number,
                });
            }
            if legacy.ref_type == "report"
                && let Some(report_number) = legacy.number.clone()
            {
                numbering.push(Numbering {
                    r#type: NumberingType::Report,
                    value: report_number,
                });
            }
            numbering
        },
        original: relation_monograph(
            legacy.original_title.clone().map(Title::Single),
            original_author,
            original_date,
            None,
            original_publisher,
            original_publisher_place,
        ),
        ..Default::default()
    }))
}

fn collection_component_metadata(
    legacy: &csl_legacy::csl_json::Reference,
) -> (Option<String>, Option<String>) {
    let mut genre = legacy.genre.clone();
    if genre.is_none() {
        genre = match legacy.ref_type.as_str() {
            "entry-dictionary" => Some("entry-dictionary".to_string()),
            "entry-encyclopedia" => Some("entry-encyclopedia".to_string()),
            _ => None,
        };
    }
    let status = legacy_extra_str(legacy, "status");
    (genre, status)
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
        note: raw_note,
        isbn,
        extra,
        ..
    } = legacy;
    let note = raw_note.map(RichText::Plain);

    let editor_value = editor.clone().map(Contributor::from);
    let translator_value = translator.clone().map(Contributor::from);
    let mut contributors = Vec::new();
    push_legacy_contributor(&mut contributors, ContributorRole::Editor, editor);
    push_legacy_contributor(&mut contributors, ContributorRole::Translator, translator);

    InputReference::Collection(Box::new(Collection {
        id: Some(id.into()),
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
            place: publisher_place.clone().map(Into::into),
        }),
        numbering,
        url: url.as_deref().and_then(|value| Url::parse(value).ok()),
        accessed: accessed.map(EdtfString::from),
        language: language.map(Into::into),
        field_languages: HashMap::new(),
        note,
        isbn,
        event: None,
        volume: None,
        issue: None,
        edition: None,
        number: None,
        part_number: None,
        supplement_number: None,
        printing_number: None,
        keywords: None,
    }))
}

#[allow(
    clippy::too_many_lines,
    reason = "Legacy CSL mapping requires extensive field wiring"
)]
fn from_serial_component_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let mut genre = legacy.genre.clone();
    if legacy.ref_type == "entry-encyclopedia" && genre.is_none() {
        genre = Some("entry-encyclopedia".to_string());
    }
    let host_names = legacy_extra_names(&legacy, "host");
    let narrator_names = legacy_extra_names(&legacy, "narrator");
    let producer_names = legacy_extra_names(&legacy, "producer")
        .or_else(|| legacy_extra_names(&legacy, "executive-producer"));
    let container_author = legacy_extra_names(&legacy, "container-author");
    let reviewed_author = legacy_extra_names(&legacy, "reviewed-author");
    let reviewed_title = legacy_extra_str(&legacy, "reviewed-title").map(Title::Single);
    let reviewed_genre = legacy_extra_str(&legacy, "reviewed-genre");
    let supplement_number = legacy_extra_str(&legacy, "supplement-number");
    let status = legacy_extra_str(&legacy, "status");
    let available_date = legacy_extra_date(&legacy, "available-date");
    let original_author = legacy_extra_contributor(&legacy, "original-author");
    let original_date = legacy_extra_date(&legacy, "original-date");
    let original_publisher = legacy_extra_str(&legacy, "original-publisher");
    let original_publisher_place = legacy_extra_str(&legacy, "original-publisher-place");
    let serial_type = match legacy.ref_type.as_str() {
        "article-journal" => SerialType::AcademicJournal,
        "article-magazine" => SerialType::Magazine,
        "article-newspaper" => SerialType::Newspaper,
        "broadcast" | "motion_picture" => SerialType::BroadcastProgram,
        _ => SerialType::AcademicJournal,
    };
    let parent_title = legacy.container_title.clone().map(Title::Single);

    let volume = legacy.volume.map(|v| v.to_string());
    let issue = legacy
        .issue
        .clone()
        .or_else(|| {
            if legacy.ref_type == "broadcast" || legacy.ref_type == "motion_picture" {
                legacy.number.as_ref().map(|n| {
                    normalize_broadcast_issue(&legacy.ref_type, legacy.medium.as_deref(), n)
                })
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
    push_legacy_contributor(&mut contributors, ContributorRole::Host, host_names);
    push_legacy_contributor(&mut contributors, ContributorRole::Narrator, narrator_names);
    push_legacy_contributor(&mut contributors, ContributorRole::Producer, producer_names);
    let serial_editor = legacy.editor.clone().map(Contributor::from);
    let mut serial_contributors = Vec::new();
    push_legacy_contributor(
        &mut serial_contributors,
        ContributorRole::Editor,
        legacy.editor.clone(),
    );
    let reviewed = relation_monograph(
        reviewed_title,
        reviewed_author
            .clone()
            .map(Contributor::from)
            .or_else(|| container_author.clone().map(Contributor::from)),
        None,
        reviewed_genre,
        None,
        None,
    );
    if reviewed.is_none() {
        push_legacy_contributor(
            &mut serial_contributors,
            ContributorRole::Author,
            container_author,
        );
    }

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
                    place: legacy.publisher_place.clone().map(Into::into),
                }),
                url: None,
                accessed: None,
                language: None,
                field_languages: HashMap::new(),
                note: None,
                issn: legacy.issn.clone(),
            }),
        )))),
        volume,
        issue,
        numbering: {
            let mut numbering = Vec::new();
            if let Some(supplement_number) = supplement_number {
                numbering.push(Numbering {
                    r#type: NumberingType::Supplement,
                    value: supplement_number,
                });
            }
            numbering
        },
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
        doi: ctx.doi,
        ads_bibcode: None,
        pages: legacy.page,
        genre,
        medium: legacy.medium,
        archive_info: None,
        eprint: None,
        keywords: None,
        section: legacy.section,
        status,
        available_date,
        reviewed,
        original: relation_monograph(
            legacy.original_title.clone().map(Title::Single),
            original_author,
            original_date,
            None,
            original_publisher,
            original_publisher_place,
        ),
        ..Default::default()
    }))
}

fn from_audio_visual_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let original = legacy_original_relation(&legacy);
    let r#type = match legacy.ref_type.as_str() {
        "motion_picture" => AudioVisualType::Film,
        "song" => AudioVisualType::Recording,
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
    // For recordings (song type), `author` is the performer; store it as Performer
    // so that Composer takes precedence as the primary author in APA style.
    let author_role = if legacy.ref_type == "song" {
        ContributorRole::Performer
    } else {
        ContributorRole::Author
    };
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Performer,
        legacy_extra_names(&legacy, "performer"),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Producer,
        legacy_extra_names(&legacy, "producer")
            .or_else(|| legacy_extra_names(&legacy, "executive-producer")),
    );
    push_legacy_contributor(&mut contributors, author_role, legacy.author.clone());
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Translator,
        legacy.translator.clone(),
    );

    let mut numbering: Vec<Numbering> = legacy
        .number
        .into_iter()
        .map(|number| Numbering {
            r#type: NumberingType::Number,
            value: number,
        })
        .collect();
    if let Some(vol) = legacy.volume.map(|v| v.to_string()) {
        numbering.push(Numbering {
            r#type: NumberingType::Volume,
            value: vol,
        });
    }
    if let Some(chapter) = legacy.chapter_number.clone() {
        numbering.push(Numbering {
            r#type: NumberingType::Chapter,
            value: chapter,
        });
    }

    InputReference::AudioVisual(Box::new(AudioVisualWork {
        id: ctx.id,
        r#type,
        core: WorkCore {
            title: build_title(ctx.title, ctx.short_title.clone()),
            short_title: None,
            contributors,
            created: ctx.created,
            issued: ctx.issued,
            original,
            language: ctx.language,
            genre: legacy.genre,
        },
        container: legacy.container_title.map(|title| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(title)),
                ..Default::default()
            }))))
        }),
        numbering,
        publisher: legacy.publisher.map(|name| Publisher {
            name: name.into(),
            place: legacy.publisher_place.map(Into::into),
        }),
        medium: legacy.medium,
        platform: None,
        url: ctx.url,
        accessed: ctx.accessed,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
    }))
}

fn from_legal_case_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let original = legacy_original_relation(&legacy);
    InputReference::LegalCase(Box::new(LegalCase {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        original,
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
        note: ctx.note.map(RichText::Plain),
        doi: ctx.doi,
        keywords: None,
    }))
}

fn from_statute_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let original = legacy_original_relation(&legacy);
    InputReference::Statute(Box::new(Statute {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        original,
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
        note: ctx.note.map(RichText::Plain),
        keywords: None,
    }))
}

fn from_regulation_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let original = legacy_original_relation(&legacy);
    InputReference::Regulation(Box::new(Regulation {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        original,
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
        note: ctx.note.map(RichText::Plain),
        keywords: None,
    }))
}

fn from_treaty_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let original = legacy_original_relation(&legacy);
    InputReference::Treaty(Box::new(Treaty {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        original,
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
        note: ctx.note.map(RichText::Plain),
        keywords: None,
    }))
}

fn from_standard_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let original = legacy_original_relation(&legacy);
    InputReference::Standard(Box::new(Standard {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        original,
        authority: legacy.authority,
        standard_number: legacy.number.unwrap_or_default(),
        created: ctx.created,
        issued: ctx.issued,
        status: None,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place.map(Into::into),
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
        keywords: None,
    }))
}

fn from_patent_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let original = legacy_original_relation(&legacy);
    InputReference::Patent(Box::new(Patent {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        author: legacy.author.map(Contributor::from),
        assignee: None,
        original,
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
        note: ctx.note.map(RichText::Plain),
        keywords: None,
    }))
}

fn from_dataset_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let original = legacy_original_relation(&legacy);
    let version = legacy_extra_str(&legacy, "version");
    let synthesized_title = ctx.title.clone().or_else(|| {
        legacy
            .genre
            .as_ref()
            .map(|genre| format!("[{genre}]"))
            .map(|title| {
                version
                    .as_ref()
                    .map(|version| format!("{title} (Version {version})"))
                    .unwrap_or(title)
            })
    });

    InputReference::Dataset(Box::new(Dataset {
        id: ctx.id,
        title: build_title(synthesized_title, ctx.short_title.clone()),
        author: legacy.author.map(Contributor::from),
        original,
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|n| Publisher {
            name: n.into(),
            place: legacy.publisher_place.map(Into::into),
        }),
        version,
        format: legacy.medium,
        size: None,
        repository: None,
        doi: ctx.doi,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
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
        note: ctx.note.map(RichText::Plain),
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
    let original = legacy_original_relation(&legacy);

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
            place: legacy.publisher_place.map(Into::into),
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
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
        original,
        ..Default::default()
    }))
}

fn from_preprint_ref(legacy: csl_legacy::csl_json::Reference, ctx: RefContext) -> InputReference {
    let archive_info = archive_info_from_legacy_flat(&legacy);
    let archive = match archive_info.as_ref() {
        Some(ai) if ai.collection.is_some() => None,
        _ => legacy.archive.clone(),
    };

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

    let numbering = legacy
        .number
        .as_ref()
        .map(|number| {
            vec![Numbering {
                r#type: NumberingType::Report,
                value: number.clone(),
            }]
        })
        .unwrap_or_default();

    InputReference::Monograph(Box::new(Monograph {
        id: ctx.id,
        r#type: MonographType::Preprint,
        title: build_title(ctx.title, ctx.short_title.clone()),
        short_title: None,
        container: None,
        author,
        editor,
        translator,
        contributors,
        created: ctx.created,
        issued: ctx.issued,
        publisher: legacy.publisher.map(|name| Publisher {
            name: name.into(),
            place: legacy.publisher_place.map(Into::into),
        }),
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
        doi: ctx.doi,
        isbn: ctx.isbn,
        numbering,
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
    let original = legacy_original_relation(&legacy);
    let producer_names = legacy_extra_names(&legacy, "producer")
        .or_else(|| legacy_extra_names(&legacy, "executive-producer"));
    let host_names = legacy_extra_names(&legacy, "host");
    let chair_names = legacy_extra_names(&legacy, "chair");
    let available_date = legacy_extra_date(&legacy, "available-date");
    let event_title = legacy_extra_str(&legacy, "event-title");
    let event_place = legacy_extra_str(&legacy, "event-place");
    let event_date = legacy_extra_date(&legacy, "event-date");
    let mut contributors = Vec::new();
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Performer,
        legacy.author.clone(),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Narrator,
        legacy_extra_names(&legacy, "narrator"),
    );
    push_legacy_contributor(
        &mut contributors,
        ContributorRole::Custom("chair".to_string()),
        chair_names,
    );
    push_legacy_contributor(&mut contributors, ContributorRole::Producer, producer_names);
    push_legacy_contributor(&mut contributors, ContributorRole::Host, host_names);
    if let Some(organizer_name) = legacy.publisher.clone() {
        contributors.push(ContributorEntry {
            role: ContributorRole::Custom("organizer".to_string()),
            contributor: Contributor::SimpleName(SimpleName {
                name: organizer_name.into(),
                location: None,
            }),
            gender: None,
        });
    }
    InputReference::Event(Box::new(Event {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title.clone()),
        original,
        container: legacy.container_title.clone().map(|t| {
            WorkRelation::Embedded(Box::new(InputReference::Monograph(Box::new(Monograph {
                title: Some(Title::Single(t)),
                ..Default::default()
            }))))
        }),
        series: event_title.map(|title| {
            WorkRelation::Embedded(Box::new(InputReference::Collection(Box::new(Collection {
                title: Some(Title::Single(title)),
                ..Default::default()
            }))))
        }),
        location: event_place.or(legacy.publisher_place.clone()),
        date: event_date.or_else(|| (!ctx.issued.is_empty()).then_some(ctx.issued)),
        available_date,
        genre: legacy.genre,
        network: None,
        contributors,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
    }))
}

impl From<csl_legacy::csl_json::Reference> for InputReference {
    fn from(mut legacy: csl_legacy::csl_json::Reference) -> Self {
        legacy.parse_note_field_hacks();
        let ctx = RefContext {
            id: Some(legacy.id.clone().into()),
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
            language: legacy.language.clone().map(Into::into),
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
            | "thesis"
            | "manual"
            | "manuscript"
            | "webpage"
            | "post"
            | "post-weblog"
            | "interview"
            | "personal_communication"
            | "personal-communication" => from_monograph_ref(legacy, ctx),
            "report"
                if legacy.page.is_some()
                    && (legacy.editor.is_some() || legacy.container_title.is_some()) =>
            {
                from_collection_component_ref(legacy, ctx)
            }
            "report" => from_monograph_ref(legacy, ctx),
            "chapter" | "paper-conference" | "entry-dictionary" | "entry-encyclopedia" => {
                from_collection_component_ref(legacy, ctx)
            }
            "article-journal" | "article-magazine" | "article-newspaper" => {
                from_serial_component_ref(legacy, ctx)
            }
            "article" => {
                if legacy.container_title.is_none() {
                    from_preprint_ref(legacy, ctx)
                } else {
                    from_serial_component_ref(legacy, ctx)
                }
            }
            "motion_picture" | "song" => from_audio_visual_ref(legacy, ctx),
            "broadcast" => from_serial_component_ref(legacy, ctx),
            "speech" | "presentation" | "event" => from_event_ref(legacy, ctx),
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
    use serde_json::json;

    fn legacy_year(year: i32) -> csl_legacy::csl_json::DateVariable {
        csl_legacy::csl_json::DateVariable {
            date_parts: Some(vec![vec![year]]),
            ..Default::default()
        }
    }

    #[test]
    fn legacy_report_number_maps_to_report_numbering() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "report-1".to_string(),
            ref_type: "report".to_string(),
            title: Some("Report".to_string()),
            issued: Some(legacy_year(2024)),
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
            issued: Some(legacy_year(2024)),
            number: Some("2".to_string()),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        assert_eq!(converted.number(), Some("2".to_string()));
        assert_eq!(converted.report_number(), None);
    }

    #[test]
    fn legacy_monograph_original_relation_uses_original_author_and_date() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "book-2".to_string(),
            ref_type: "book".to_string(),
            title: Some("Translated Book".to_string()),
            issued: Some(legacy_year(2024)),
            original_title: Some("Original Book".to_string()),
            extra: HashMap::from([
                (
                    "original-author".to_string(),
                    json!([{"family":"Woolf","given":"Virginia"}]),
                ),
                ("original-date".to_string(), json!({"date-parts":[[1925]]})),
            ]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::Monograph(monograph) = converted else {
            panic!("expected monograph");
        };
        let Some(WorkRelation::Embedded(original)) = monograph.original else {
            panic!("expected embedded original relation");
        };
        let InputReference::Monograph(original_monograph) = *original else {
            panic!("expected original monograph relation");
        };

        assert_eq!(
            original_monograph.title,
            Some(Title::Single("Original Book".to_string()))
        );
        assert_eq!(original_monograph.issued, EdtfString("1925".to_string()));
        assert!(original_monograph.author.is_some());
    }

    #[test]
    fn legacy_serial_component_maps_reviewed_relation_and_supplement_number() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "article-1".to_string(),
            ref_type: "article-journal".to_string(),
            title: Some("Review Essay".to_string()),
            container_title: Some("Journal".to_string()),
            issued: Some(legacy_year(2024)),
            extra: HashMap::from([
                ("reviewed-title".to_string(), json!("Reviewed Book")),
                (
                    "reviewed-author".to_string(),
                    json!([{"family":"Morrison","given":"Toni"}]),
                ),
                ("supplement-number".to_string(), json!("S1")),
            ]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::SerialComponent(component) = converted else {
            panic!("expected serial component");
        };
        assert!(
            component
                .numbering
                .iter()
                .any(|entry| entry.r#type == NumberingType::Supplement && entry.value == "S1")
        );
        let Some(WorkRelation::Embedded(reviewed)) = component.reviewed else {
            panic!("expected reviewed relation");
        };
        let InputReference::Monograph(reviewed_work) = *reviewed else {
            panic!("expected reviewed monograph relation");
        };
        assert_eq!(
            reviewed_work.title,
            Some(Title::Single("Reviewed Book".to_string()))
        );
        assert!(reviewed_work.author.is_some());
    }

    #[test]
    fn legacy_event_prefers_extra_event_fields() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "event-1".to_string(),
            ref_type: "speech".to_string(),
            title: Some("Fallback Title".to_string()),
            issued: Some(legacy_year(2024)),
            extra: HashMap::from([
                ("event-title".to_string(), json!("Actual Event")),
                ("event-place".to_string(), json!("Chicago")),
                (
                    "event-date".to_string(),
                    json!({"date-parts":[[2023, 5, 6]]}),
                ),
            ]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::Event(event) = converted else {
            panic!("expected event");
        };
        assert_eq!(
            event.title,
            Some(Title::Single("Fallback Title".to_string()))
        );
        assert_eq!(
            event.series.as_ref().and_then(|relation| match relation {
                WorkRelation::Embedded(parent) => parent.title(),
                WorkRelation::Id(_) => None,
            }),
            Some(Title::Single("Actual Event".to_string()))
        );
        assert_eq!(event.location, Some("Chicago".to_string()));
        assert_eq!(event.date, Some(EdtfString("2023-05-06".to_string())));
    }

    #[test]
    fn legacy_event_omits_empty_fallback_date() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "event-2".to_string(),
            ref_type: "speech".to_string(),
            title: Some("Fallback Title".to_string()),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::Event(event) = converted else {
            panic!("expected event");
        };
        assert_eq!(event.date, None);
    }

    #[test]
    fn legacy_broadcast_maps_executive_producer_to_producer() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "broadcast-1".to_string(),
            ref_type: "broadcast".to_string(),
            title: Some("Episode".to_string()),
            issued: Some(legacy_year(2024)),
            extra: HashMap::from([(
                "executive-producer".to_string(),
                json!([{"family":"Rhimes","given":"Shonda"}]),
            )]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::SerialComponent(work) = converted else {
            panic!("expected serial component");
        };
        assert!(
            work.contributors
                .iter()
                .any(|entry| entry.role == ContributorRole::Producer)
        );
    }

    #[test]
    fn legacy_monograph_dedupes_extra_role_pushes() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "book-3".to_string(),
            ref_type: "book".to_string(),
            title: Some("Role Dedup".to_string()),
            issued: Some(legacy_year(2024)),
            extra: HashMap::from([
                (
                    "composer".to_string(),
                    json!([{"family":"Glass","given":"Philip"}]),
                ),
                (
                    "producer".to_string(),
                    json!([{"family":"Jones","given":"Quincy"}]),
                ),
                (
                    "executive-producer".to_string(),
                    json!([{"family":"Jones","given":"Quincy"}]),
                ),
            ]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::Monograph(monograph) = converted else {
            panic!("expected monograph");
        };

        let composer_count = monograph
            .contributors
            .iter()
            .filter(|entry| entry.role == ContributorRole::Composer)
            .count();
        let producer_count = monograph
            .contributors
            .iter()
            .filter(|entry| entry.role == ContributorRole::Producer)
            .count();

        assert_eq!(
            composer_count, 1,
            "duplicate composer entry after conversion"
        );
        assert_eq!(
            producer_count, 1,
            "duplicate producer entry after conversion"
        );
    }

    #[test]
    fn legacy_monograph_prefers_part_title_over_title() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "book-4".to_string(),
            ref_type: "book".to_string(),
            title: Some("Container Work".to_string()),
            extra: HashMap::from([("part-title".to_string(), json!("Actual Part"))]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::Monograph(monograph) = converted else {
            panic!("expected monograph");
        };
        assert_eq!(
            monograph.title,
            Some(Title::Single("Actual Part".to_string()))
        );
    }

    #[test]
    fn legacy_collection_component_prefers_part_title_over_title() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "chapter-1".to_string(),
            ref_type: "chapter".to_string(),
            title: Some("Collected Volume".to_string()),
            extra: HashMap::from([("part-title".to_string(), json!("Actual Chapter"))]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        let InputReference::CollectionComponent(component) = converted else {
            panic!("expected collection component");
        };
        assert_eq!(
            component.title,
            Some(Title::Single("Actual Chapter".to_string()))
        );
    }
}
