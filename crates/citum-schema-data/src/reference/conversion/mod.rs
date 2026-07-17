/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Legacy CSL-JSON → Citum reference conversion.
//!
//! The top-level [`From<csl_legacy::csl_json::Reference> for InputReference`]
//! impl dispatches by `ref_type` to a per-category converter in one of the
//! submodules:
//!
//! - `legal` — `legal-case`, `statute`, `regulation`, `treaty`, `standard`,
//!   `patent`, `bill`, `hearing`
//! - `scholarly` — `book`, `chapter`, `article-journal`, `article`,
//!   `paper-conference`, `dataset`, `event`, etc.; plus the standalone
//!   `input_reference_from_legacy_edited_book` re-exported below
//! - `media` — `software`, `motion_picture`, `song`
//!
//! Shared helpers (`legacy_*`, `relation_*`, `build_title`, …) and the
//! `RefContext` struct that bundles the fields every converter pre-extracts
//! live here in `mod.rs` so submodules can pull them in with `use super::*;`.

#[cfg(test)]
mod contract_tests;
mod legal;
mod media;
mod scholarly;

pub use scholarly::input_reference_from_legacy_edited_book;

use crate::reference::contributor::{
    Contributor, ContributorEntry, ContributorList, ContributorRole, SimpleName, StructuredName,
};
use crate::reference::date::EdtfString;
use crate::reference::types::{
    ArchiveInfo, Collection, CollectionComponent, CollectionType, Dataset, Hearing, LegalCase,
    Monograph, MonographComponentType, MonographType, NumOrStr, Patent, Publisher, Regulation,
    RichText, Serial, SerialComponent, SerialComponentType, SerialType, Software, Standard,
    Statute, StructuredTitle, Subtitle, Title, Treaty,
};
use crate::reference::{
    AudioVisualType, AudioVisualWork, Event, IdentifierName, InputReference, LangID, Numbering,
    NumberingType, RefID, WorkCore, WorkRelation,
};
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;
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
        let Contributor::ContributorList(list) = Contributor::from(names) else {
            return;
        };
        let mut unmatched = Vec::new();
        for contributor in list.0 {
            let identity = legacy_contributor_identity(&contributor);
            if let Some((entry_index, member_index)) = identity
                .as_ref()
                .and_then(|identity| find_legacy_contributor(entries, &role, identity))
                && merge_legacy_contributor_role(entries, entry_index, member_index, role.clone())
            {
                continue;
            }
            unmatched.push(contributor);
        }
        if !unmatched.is_empty() {
            let contributor = if unmatched.len() == 1 {
                let Some(contributor) = unmatched.pop() else {
                    return;
                };
                contributor
            } else {
                Contributor::ContributorList(ContributorList(unmatched))
            };
            entries.push(ContributorEntry {
                roles: role.into(),
                contributor,
                gender: None,
            });
        }
    }
}

fn find_legacy_contributor(
    entries: &[ContributorEntry],
    role: &ContributorRole,
    identity: &LegacyContributorIdentity,
) -> Option<(usize, Option<usize>)> {
    entries.iter().enumerate().find_map(|(entry_index, entry)| {
        if entry.roles.contains(role) {
            return None;
        }
        match &entry.contributor {
            Contributor::ContributorList(list) => list
                .0
                .iter()
                .position(|contributor| {
                    legacy_contributor_identity(contributor).as_ref() == Some(identity)
                })
                .map(|member_index| (entry_index, Some(member_index))),
            contributor => (legacy_contributor_identity(contributor).as_ref() == Some(identity))
                .then_some((entry_index, None)),
        }
    })
}

fn merge_legacy_contributor_role(
    entries: &mut Vec<ContributorEntry>,
    entry_index: usize,
    member_index: Option<usize>,
    role: ContributorRole,
) -> bool {
    let Some(member_index) = member_index else {
        if let Some(entry) = entries.get_mut(entry_index) {
            entry.roles.insert(role);
            return true;
        }
        return false;
    };
    if entry_index >= entries.len() {
        return false;
    }

    let ContributorEntry {
        roles,
        contributor,
        gender,
    } = entries.remove(entry_index);
    let Contributor::ContributorList(mut list) = contributor else {
        entries.insert(
            entry_index,
            ContributorEntry {
                roles,
                contributor,
                gender,
            },
        );
        return false;
    };
    if member_index >= list.0.len() {
        entries.insert(
            entry_index,
            ContributorEntry {
                roles,
                contributor: Contributor::ContributorList(list),
                gender,
            },
        );
        return false;
    }

    let mut after = list.0.split_off(member_index + 1);
    let Some(matched) = list.0.pop() else {
        return false;
    };
    let mut matched_roles = roles.clone();
    matched_roles.insert(role);
    let mut replacement = Vec::with_capacity(3);
    if !list.0.is_empty() {
        replacement.push(ContributorEntry {
            roles: roles.clone(),
            contributor: Contributor::ContributorList(list),
            gender,
        });
    }
    replacement.push(ContributorEntry {
        roles: matched_roles,
        contributor: matched,
        gender,
    });
    if !after.is_empty() {
        replacement.push(ContributorEntry {
            roles,
            contributor: Contributor::ContributorList(ContributorList(std::mem::take(&mut after))),
            gender,
        });
    }
    entries.splice(entry_index..entry_index, replacement);
    true
}

#[derive(Debug, PartialEq, Eq)]
enum LegacyContributorIdentity {
    Structured(Vec<String>),
    Literal(String),
}

fn legacy_contributor_identity(contributor: &Contributor) -> Option<LegacyContributorIdentity> {
    let name = contributor.to_names_vec().into_iter().next()?;
    if let Some(literal) = normalized_identity_part(name.literal.as_deref()) {
        return Some(LegacyContributorIdentity::Literal(literal));
    }
    let parts = [
        name.given.as_deref(),
        name.family.as_deref(),
        name.suffix.as_deref(),
        name.dropping_particle.as_deref(),
        name.non_dropping_particle.as_deref(),
    ]
    .into_iter()
    .map(|part| normalized_identity_part(part).unwrap_or_default())
    .collect::<Vec<_>>();
    parts
        .iter()
        .any(|part| !part.is_empty())
        .then_some(LegacyContributorIdentity::Structured(parts))
}

fn normalized_identity_part(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    (!trimmed.is_empty()).then(|| trimmed.nfc().collect())
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

/// Build a publisher from legacy name/place, preserving a place-only imprint.
///
/// `Publisher::name` is non-optional, so when only a place is present (no
/// publisher name) this returns a `Publisher` with an empty name rather than
/// dropping the place entirely — GB/T 7714 and other styles render
/// place-only imprints, so silently discarding `publisher-place` when
/// `publisher` is absent would lose data.
fn publisher_from_parts(name: Option<String>, place: Option<String>) -> Option<Publisher> {
    match (name, place) {
        (Some(name), place) => Some(Publisher {
            name: name.into(),
            place: place.map(Into::into),
        }),
        (None, Some(place)) => Some(Publisher {
            name: String::new().into(),
            place: Some(place.into()),
        }),
        (None, None) => None,
    }
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

    let publisher = publisher_from_parts(publisher, publisher_place);

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

fn relation_collection_title(title: Option<String>) -> Option<WorkRelation> {
    title.map(|title| {
        WorkRelation::Embedded(Box::new(InputReference::Collection(Box::new(Collection {
            title: Some(Title::Single(title)),
            ..Default::default()
        }))))
    })
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

impl From<csl_legacy::csl_json::Reference> for InputReference {
    fn from(mut legacy: csl_legacy::csl_json::Reference) -> Self {
        legacy.parse_note_field_hacks();
        let cstr = legacy_extra_str(&legacy, "CSTR");
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

        let mut reference = match legacy.ref_type.as_str() {
            "software" => media::from_software_ref(legacy, ctx),
            "book"
            | "thesis"
            | "manual"
            | "manuscript"
            | "classic"
            | "webpage"
            | "post"
            | "post-weblog"
            | "interview"
            | "personal_communication"
            | "personal-communication"
            | "musical_score"
            | "pamphlet" => scholarly::from_monograph_ref(legacy, ctx),
            "report"
                if legacy.page.is_some()
                    && (legacy.editor.is_some() || legacy.container_title.is_some()) =>
            {
                scholarly::from_collection_component_ref(legacy, ctx)
            }
            "report" => scholarly::from_monograph_ref(legacy, ctx),
            "chapter" | "paper-conference" | "entry" | "entry-dictionary"
            | "entry-encyclopedia" => scholarly::from_collection_component_ref(legacy, ctx),
            "article-journal" | "article-magazine" | "article-newspaper" | "review"
            | "review-book" => scholarly::from_serial_component_ref(legacy, ctx),
            "article" => {
                if legacy.container_title.is_none() {
                    scholarly::from_preprint_ref(legacy, ctx)
                } else {
                    scholarly::from_serial_component_ref(legacy, ctx)
                }
            }
            "motion_picture" | "song" => media::from_audio_visual_ref(legacy, ctx),
            "broadcast" => scholarly::from_serial_component_ref(legacy, ctx),
            "speech" | "presentation" | "performance" | "event" => {
                scholarly::from_event_ref(legacy, ctx)
            }
            "bill" => legal::from_bill_ref(legacy, ctx),
            "hearing" => legal::from_hearing_ref(legacy, ctx),
            "legal-case" | "legal_case" => legal::from_legal_case_ref(legacy, ctx),
            "statute" | "legislation" => legal::from_statute_ref(legacy, ctx),
            "regulation" => legal::from_regulation_ref(legacy, ctx),
            "treaty" => legal::from_treaty_ref(legacy, ctx),
            "standard" => legal::from_standard_ref(legacy, ctx),
            "patent" => legal::from_patent_ref(legacy, ctx),
            "dataset" => scholarly::from_dataset_ref(legacy, ctx),
            // `collection` is CSL 1.0.2's *archival* collection (a body of
            // manuscripts/papers held by an archive: author, archive,
            // archive-place). It routes with the other archival/document
            // shapes so those fields survive; Citum's editorial
            // `ClassExtension::Collection` (anthology/proceedings) has no
            // author or archive fields and would silently drop them. See
            // docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md.
            "document" | "map" | "figure" | "graphic" | "periodical" | "collection" => {
                scholarly::from_document_ref(legacy, ctx)
            }
            _ => {
                // Every CSL 1.0.2 type string has an explicit arm above; a
                // known type reaching this fallback means a routing arm was
                // dropped, not that the type is genuinely unmapped.
                //
                // TODO(csl26-1bdr): Layer 5 `CompatibilityWarning` plumbing
                // will surface unrecognized types as a soft-degrade warning
                // rather than silent fall-through. Until then this
                // fallback mirrors the ClassExtension::Unknown loud-fail
                // pattern in accessors.rs.
                debug_assert!(
                    !csl_legacy::csl_json::CSL_TYPES.contains(&legacy.ref_type.as_str()),
                    "unmapped CSL 1.0.2 type `{}` fell through to the document fallback; \
                     add a routing arm in conversion/mod.rs",
                    legacy.ref_type
                );
                scholarly::from_document_ref(legacy, ctx)
            }
        };
        if let Some(cstr) = cstr
            && let Ok(name) = IdentifierName::new("cstr")
        {
            reference.insert_identifier(name, cstr);
        }
        reference
    }
}

impl From<csl_legacy::csl_json::DateVariable> for EdtfString {
    fn from(date: csl_legacy::csl_json::DateVariable) -> Self {
        if let Some(literal) = date.literal {
            return EdtfString(normalize_csl_circa_literal(&literal).unwrap_or(literal));
        }
        if let Some(parts) = date.date_parts {
            let mut rendered = parts.iter().map(|part| render_date_part(part));
            if let Some(first) = rendered.next() {
                if let Some(second) = rendered.next() {
                    return EdtfString(apply_csl_circa_marker(
                        format!("{first}/{second}"),
                        date.circa,
                    ));
                }
                return EdtfString(apply_csl_circa_marker(first, date.circa));
            }
        }
        if let Some(raw) = date.raw {
            return EdtfString(normalize_csl_circa_literal(&raw).unwrap_or(raw));
        }
        EdtfString(String::new())
    }
}

/// Convert CSL's common circa spellings into their equivalent EDTF form.
fn normalize_csl_circa_literal(value: &str) -> Option<String> {
    let year = value.strip_prefix('c')?.trim();
    (year.len() == 4 && year.bytes().all(|byte| byte.is_ascii_digit())).then(|| format!("{year}~"))
}

/// Apply CSL's structured circa flag to an EDTF date that lacks a qualifier.
fn apply_csl_circa_marker(value: String, circa: Option<bool>) -> String {
    if circa == Some(true) && !value.ends_with('~') && !value.ends_with('%') {
        format!("{value}~")
    } else {
        value
    }
}

fn render_date_part(part: &[i32]) -> String {
    let year = part
        .first()
        .map(|y| {
            if *y < 0 {
                format!("-{:04}", y.abs())
            } else {
                format!("{:04}", y)
            }
        })
        .unwrap_or_default();
    let month = part
        .get(1)
        .map(|m| format!("-{:02}", m))
        .unwrap_or_default();
    let day = part
        .get(2)
        .map(|d| format!("-{:02}", d))
        .unwrap_or_default();
    format!("{year}{month}{day}")
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
                        short_name: None,
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
                            short_name: None,
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
    use crate::reference::ClassExtension;

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
    fn legacy_note_type_classic_maps_to_classic_reference() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "classic-1".to_string(),
            ref_type: "book".to_string(),
            title: Some("De civitate Dei".to_string()),
            issued: Some(legacy_year(1931)),
            note: Some("type: classic".to_string()),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        assert_eq!(converted.ref_type(), "classic");
        assert!(matches!(converted.extension(), ClassExtension::Classic(_)));
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

        let ClassExtension::Monograph(monograph) = converted.extension() else {
            panic!("expected monograph");
        };
        let Some(WorkRelation::Embedded(original)) = monograph.original.as_ref() else {
            panic!("expected embedded original relation");
        };
        let ClassExtension::Monograph(original_monograph) = original.extension() else {
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

        let ClassExtension::SerialComponent(component) = converted.extension() else {
            panic!("expected serial component");
        };
        assert!(
            component
                .numbering
                .iter()
                .any(|entry| entry.r#type == NumberingType::Supplement && entry.value == "S1")
        );
        let Some(WorkRelation::Embedded(reviewed)) = component.reviewed.as_ref() else {
            panic!("expected reviewed relation");
        };
        let ClassExtension::Monograph(reviewed_work) = reviewed.extension() else {
            panic!("expected reviewed monograph relation");
        };
        assert_eq!(
            reviewed_work.title,
            Some(Title::Single("Reviewed Book".to_string()))
        );
        assert!(reviewed_work.author.is_some());
    }

    #[test]
    fn legacy_serial_component_exposes_section_and_review_event_metadata() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "newspaper-review".to_string(),
            ref_type: "article-newspaper".to_string(),
            title: Some("Young Concert Artists Is Back".to_string()),
            container_title: Some("New York Times".to_string()),
            section: Some("Arts".to_string()),
            issued: Some(csl_legacy::csl_json::DateVariable::full(2021, 11, 12)),
            extra: HashMap::from([
                ("reviewed-genre".to_string(), json!("recital")),
                (
                    "reviewed-author".to_string(),
                    json!([{"family":"Zhu Wang (piano)","given":""}]),
                ),
                ("event-title".to_string(), json!("Zankel Hall")),
                ("event-place".to_string(), json!("New York")),
            ]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        assert_eq!(converted.section(), Some("Arts".to_string()));
        assert!(
            converted
                .contributor(ContributorRole::Unknown("reviewed-author".to_string()))
                .is_some(),
            "reviewed author should be renderable from the top-level article component"
        );

        let ClassExtension::SerialComponent(component) = converted.extension() else {
            panic!("expected serial component");
        };
        let Some(WorkRelation::Embedded(event)) = component.event.as_ref() else {
            panic!("expected review event relation");
        };
        let ClassExtension::Event(event) = event.extension() else {
            panic!("expected event relation");
        };
        assert_eq!(event.title, Some(Title::Single("Zankel Hall".to_string())));
        assert_eq!(event.location, Some("New York".to_string()));
    }

    #[test]
    fn legacy_broadcast_preserves_writer_cast_network_and_duration() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "broadcast-rich".to_string(),
            ref_type: "broadcast".to_string(),
            title: Some("Her Sister's Shadow".to_string()),
            container_title: Some("The Brady Bunch".to_string()),
            number: Some("season 3, episode 10".to_string()),
            publisher: Some("ABC".to_string()),
            dimensions: Some("26m".to_string()),
            issued: Some(csl_legacy::csl_json::DateVariable::full(1971, 11, 19)),
            extra: HashMap::from([(
                "script-writer".to_string(),
                json!([{"family":"Schwartz","given":"Sherwood"}]),
            )]),
            contributor: Some(vec![csl_legacy::csl_json::Name {
                family: Some("Reed".to_string()),
                given: Some("Robert".to_string()),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        assert_eq!(converted.ref_type(), "broadcast");
        let ClassExtension::SerialComponent(component) = converted.extension() else {
            panic!("expected serial component");
        };
        assert_eq!(component.issue.as_deref(), Some("season 3, episode 10"));
        assert_eq!(component.duration.as_deref(), Some("26m"));
        assert!(
            component
                .contributors
                .iter()
                .any(|entry| entry.roles.contains(&ContributorRole::Writer))
        );
        assert!(
            component
                .contributors
                .iter()
                .any(|entry| entry.roles.contains(&ContributorRole::Performer))
        );
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

        let ClassExtension::Event(event) = converted.extension() else {
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

        let ClassExtension::Event(event) = converted.extension() else {
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

        let ClassExtension::SerialComponent(work) = converted.extension() else {
            panic!("expected serial component");
        };
        assert!(
            work.contributors
                .iter()
                .any(|entry| entry.roles.contains(&ContributorRole::Producer))
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

        let ClassExtension::Monograph(monograph) = converted.extension() else {
            panic!("expected monograph");
        };

        let composer_count = monograph
            .contributors
            .iter()
            .filter(|entry| entry.roles.contains(&ContributorRole::Composer))
            .count();
        let producer_count = monograph
            .contributors
            .iter()
            .filter(|entry| entry.roles.contains(&ContributorRole::Producer))
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

        let ClassExtension::Monograph(monograph) = converted.extension() else {
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

        let ClassExtension::CollectionComponent(component) = converted.extension() else {
            panic!("expected collection component");
        };
        assert_eq!(
            component.title,
            Some(Title::Single("Actual Chapter".to_string()))
        );
    }

    #[test]
    fn legacy_publisher_place_without_name_preserves_place() {
        let legacy = csl_legacy::csl_json::Reference {
            id: "book-5".to_string(),
            ref_type: "book".to_string(),
            title: Some("A history of Chinese mathematics".to_string()),
            publisher_place: Some("Cambridge, Eng".to_string()),
            issued: Some(legacy_year(1959)),
            ..Default::default()
        };

        let converted = InputReference::from(legacy);

        assert_eq!(
            converted.publisher(),
            Some(Publisher {
                name: String::new().into(),
                place: Some("Cambridge, Eng".to_string().into()),
            })
        );
        assert_eq!(
            converted.publisher_place(),
            Some("Cambridge, Eng".to_string())
        );
    }
}
