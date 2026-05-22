/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Legacy CSL → Citum converters for legal references (case, statute,
//! regulation, treaty, standard, patent, bill, hearing).
//!
//! `from_bill_ref` is polymorphic: a CSL `bill` with both `title` and
//! `authority` is treated as a congressional hearing (Zotero export pattern),
//! and bills that fail the title-less proceeding / record heuristics fall
//! through to the generic document converter in [`super::scholarly`].

#[allow(
    clippy::wildcard_imports,
    reason = "submodule shares the parent's helper + type pool by design"
)]
use super::*;

pub(super) fn from_legal_case_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
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
        unknown_fields: Default::default(),
    }))
}

pub(super) fn from_statute_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
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
        unknown_fields: Default::default(),
    }))
}

pub(super) fn from_regulation_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
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
        unknown_fields: Default::default(),
    }))
}

pub(super) fn from_treaty_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
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
        unknown_fields: Default::default(),
    }))
}

pub(super) fn from_standard_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
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
        unknown_fields: Default::default(),
    }))
}

pub(super) fn from_patent_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
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
        unknown_fields: Default::default(),
    }))
}

pub(super) fn from_bill_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    // CSL bills with both title and authority are congressional hearings (Zotero export pattern)
    if legacy.title.is_some() && legacy.authority.is_some() {
        return from_hearing_ref(legacy, ctx);
    }

    let titleless_proceeding =
        legacy.title.is_none() && (legacy.authority.is_some() || legacy.chapter_number.is_some());
    let titleless_record = legacy.title.is_none()
        && legacy.container_title.is_some()
        && legacy.volume.is_some()
        && legacy.page.is_some();

    if !(titleless_proceeding || titleless_record) {
        return super::scholarly::from_document_ref(legacy, ctx);
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
        unknown_fields: Default::default(),
        original: None,
        ..Default::default()
    }))
}

pub(super) fn from_hearing_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
    let original = legacy_original_relation(&legacy);
    InputReference::Hearing(Box::new(Hearing {
        id: ctx.id,
        title: build_title(ctx.title, ctx.short_title),
        original,
        authority: legacy.authority,
        // CSL chapter-number doubles as session/congress identifier for legislative sources
        session_number: legacy.chapter_number,
        created: ctx.created,
        issued: ctx.issued,
        url: ctx.url,
        accessed: ctx.accessed,
        language: ctx.language,
        field_languages: HashMap::new(),
        note: ctx.note.map(RichText::Plain),
        keywords: None,
        unknown_fields: Default::default(),
    }))
}
