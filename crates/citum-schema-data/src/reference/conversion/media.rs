/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Legacy CSL → Citum converters for media references (software, audio-visual).

#[allow(
    clippy::wildcard_imports,
    reason = "submodule shares the parent's helper + type pool by design"
)]
use super::*;

pub(super) fn from_software_ref(
    legacy: csl_legacy::csl_json::Reference,
    ctx: RefContext,
) -> InputReference {
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
        unknown_fields: Default::default(),
    }))
}

pub(super) fn from_audio_visual_ref(
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
        unknown_fields: Default::default(),
    }))
}
