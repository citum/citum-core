/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! A reference is a bibliographic item, such as a book, article, or web page.
//! It is the basic unit of bibliographic data.

pub mod contributor;
#[cfg(feature = "legacy-convert")]
pub mod conversion;
pub mod date;
pub mod types;

#[cfg(all(test, feature = "legacy-convert"))]
mod tests;

pub use self::contributor::{Contributor, ContributorList, FlatName, SimpleName, StructuredName};
pub use self::date::EdtfString;
use self::types::common::HasNumbering;
pub use self::types::common::{
    FieldLanguageMap, LangID, MultilingualString, NumOrStr, Numbering, NumberingType, RefID, Title,
};
pub use self::types::legal::{Brief, Hearing, LegalCase, Regulation, Statute, Treaty};
pub use self::types::specialized::{Classic, Dataset, Event, Patent, Software, Standard};
pub use self::types::structural::{
    Collection, CollectionComponent, CollectionType, Monograph, MonographComponentType,
    MonographType, Serial, SerialComponent, SerialComponentType, SerialType,
};

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
use url::Url;

/// A relation to another bibliographic entity.
///
/// Untagged in serde to allow either an inline object or a string ID reference.
/// Used for both hierarchical (`container`) and associative (`original`, `reviewed`, `series`) links.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum WorkRelation {
    /// The target work is referenced by its ID (resolved at render time).
    Id(RefID),
    /// The target work is embedded inline.
    Embedded(Box<InputReference>),
}

/// The Reference model.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(tag = "class", rename_all = "kebab-case")]
pub enum InputReference {
    /// A monograph, such as a book or a report, is a monolithic work published or produced as a complete entity.
    Monograph(Box<Monograph>),
    /// A component of a larger Monograph, such as a chapter in a book.
    /// The parent monograph is referenced by its ID.
    CollectionComponent(Box<CollectionComponent>),
    /// A component of a larger serial publication; for example a journal or newspaper article.
    /// The parent serial is referenced by its ID.
    SerialComponent(Box<SerialComponent>),
    /// A collection of works, such as an anthology or proceedings.
    Collection(Box<Collection>),
    /// A serial publication (journal, magazine, etc.).
    Serial(Box<Serial>),
    /// A legal case (court decision).
    LegalCase(Box<LegalCase>),
    /// A statute or legislative act.
    Statute(Box<Statute>),
    /// An international treaty or agreement.
    Treaty(Box<Treaty>),
    /// A legislative or administrative hearing.
    Hearing(Box<Hearing>),
    /// An administrative regulation.
    Regulation(Box<Regulation>),
    /// A legal brief or filing.
    Brief(Box<Brief>),
    /// A classic work with standard citation forms.
    Classic(Box<Classic>),
    /// A patent.
    Patent(Box<Patent>),
    /// A research dataset.
    Dataset(Box<Dataset>),
    /// A technical standard or specification.
    Standard(Box<Standard>),
    /// Software or source code.
    Software(Box<Software>),
    /// An event such as a conference, performance, or broadcast.
    Event(Box<Event>),
}

impl InputReference {
    fn numbered(&self) -> Option<&dyn HasNumbering> {
        match self {
            InputReference::Monograph(reference) => Some(reference.as_ref()),
            InputReference::Collection(reference) => Some(reference.as_ref()),
            InputReference::CollectionComponent(reference) => Some(reference.as_ref()),
            InputReference::SerialComponent(reference) => Some(reference.as_ref()),
            InputReference::Classic(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Internal helper to find a numbering by type.
    fn find_numbering(&self, numbering_type: NumberingType) -> Option<String> {
        self.numbered()
            .and_then(|reference| reference.find_numbering(numbering_type))
    }

    /// Return the reference ID.
    pub fn id(&self) -> Option<RefID> {
        match self {
            InputReference::Monograph(r) => r.id.clone(),
            InputReference::CollectionComponent(r) => r.id.clone(),
            InputReference::SerialComponent(r) => r.id.clone(),
            InputReference::Collection(r) => r.id.clone(),
            InputReference::Serial(r) => r.id.clone(),
            InputReference::LegalCase(r) => r.id.clone(),
            InputReference::Statute(r) => r.id.clone(),
            InputReference::Treaty(r) => r.id.clone(),
            InputReference::Hearing(r) => r.id.clone(),
            InputReference::Regulation(r) => r.id.clone(),
            InputReference::Brief(r) => r.id.clone(),
            InputReference::Classic(r) => r.id.clone(),
            InputReference::Patent(r) => r.id.clone(),
            InputReference::Dataset(r) => r.id.clone(),
            InputReference::Standard(r) => r.id.clone(),
            InputReference::Software(r) => r.id.clone(),
            InputReference::Event(r) => r.id.clone(),
        }
    }

    /// Return the author.
    pub fn author(&self) -> Option<Contributor> {
        match self {
            InputReference::Monograph(r) => r.author.clone(),
            InputReference::CollectionComponent(r) => r.author.clone(),
            InputReference::SerialComponent(r) => r.author.clone(),
            InputReference::Treaty(r) => r.author.clone(),
            InputReference::Brief(r) => r.author.clone(),
            InputReference::Classic(r) => r.author.clone(),
            InputReference::Patent(r) => r.author.clone(),
            InputReference::Dataset(r) => r.author.clone(),
            InputReference::Software(r) => r.author.clone(),
            InputReference::Event(r) => r.performer.clone().or(r.organizer.clone()),
            _ => None,
        }
    }

    pub fn editor(&self) -> Option<Contributor> {
        match self {
            InputReference::Monograph(r) => r.editor.clone(),
            InputReference::Collection(r) => r.editor.clone(),
            InputReference::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.editor(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::Serial(r) => r.editor.clone(),
            InputReference::Classic(r) => r.editor.clone(),
            _ => None,
        }
    }

    /// Return the translator.
    pub fn translator(&self) -> Option<Contributor> {
        match self {
            InputReference::Monograph(r) => r.translator.clone(),
            InputReference::CollectionComponent(r) => r.translator.clone(),
            InputReference::SerialComponent(r) => r.translator.clone(),
            InputReference::Collection(r) => r.translator.clone(),
            InputReference::Classic(r) => r.translator.clone(),
            _ => None,
        }
    }

    /// Return the recipient.
    pub fn recipient(&self) -> Option<Contributor> {
        match self {
            InputReference::Monograph(r) => r.recipient.clone(),
            _ => None,
        }
    }

    /// Return the interviewer.
    pub fn interviewer(&self) -> Option<Contributor> {
        match self {
            InputReference::Monograph(r) => r.interviewer.clone(),
            _ => None,
        }
    }

    /// Return the guest.
    pub fn guest(&self) -> Option<Contributor> {
        match self {
            InputReference::Monograph(r) => r.guest.clone(),
            _ => None,
        }
    }

    /// Return the publisher.
    pub fn publisher(&self) -> Option<Contributor> {
        match self {
            InputReference::Monograph(r) => r.publisher.clone(),
            InputReference::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::Collection(r) => r.publisher.clone(),
            InputReference::Serial(r) => r.publisher.clone(),
            InputReference::Classic(r) => r.publisher.clone(),
            InputReference::Dataset(r) => r.publisher.clone(),
            InputReference::Standard(r) => r.publisher.clone(),
            InputReference::Software(r) => r.publisher.clone(),
            _ => None,
        }
    }

    /// Return the title.
    pub fn title(&self) -> Option<Title> {
        match self {
            InputReference::Monograph(r) => match (&r.title, &r.short_title) {
                (Some(Title::Single(long)), Some(short)) => {
                    Some(Title::Shorthand(short.clone(), long.clone()))
                }
                _ => r.title.clone(),
            },
            InputReference::CollectionComponent(r) => r.title.clone(),
            InputReference::SerialComponent(r) => r.title.clone(),
            InputReference::Collection(r) => match (&r.title, &r.short_title) {
                (Some(Title::Single(long)), Some(short)) => {
                    Some(Title::Shorthand(short.clone(), long.clone()))
                }
                _ => r.title.clone(),
            },
            InputReference::Serial(r) => match (&r.title, &r.short_title) {
                (Some(Title::Single(long)), Some(short)) => {
                    Some(Title::Shorthand(short.clone(), long.clone()))
                }
                _ => r.title.clone(),
            },
            InputReference::LegalCase(r) => r.title.clone(),
            InputReference::Statute(r) => r.title.clone(),
            InputReference::Treaty(r) => r.title.clone(),
            InputReference::Hearing(r) => r.title.clone(),
            InputReference::Regulation(r) => r.title.clone(),
            InputReference::Brief(r) => r.title.clone(),
            InputReference::Classic(r) => r.title.clone(),
            InputReference::Patent(r) => r.title.clone(),
            InputReference::Dataset(r) => r.title.clone(),
            InputReference::Standard(r) => r.title.clone(),
            InputReference::Software(r) => r.title.clone(),
            InputReference::Event(r) => r.title.clone(),
        }
    }

    /// Return the issued date.
    pub fn issued(&self) -> Option<EdtfString> {
        match self {
            InputReference::Monograph(r) => Some(r.issued.clone()),
            InputReference::CollectionComponent(r) => Some(r.issued.clone()),
            InputReference::SerialComponent(r) => Some(r.issued.clone()),
            InputReference::Collection(r) => Some(r.issued.clone()),
            InputReference::Serial(_) => None,
            InputReference::LegalCase(r) => Some(r.issued.clone()),
            InputReference::Statute(r) => Some(r.issued.clone()),
            InputReference::Treaty(r) => Some(r.issued.clone()),
            InputReference::Hearing(r) => Some(r.issued.clone()),
            InputReference::Regulation(r) => Some(r.issued.clone()),
            InputReference::Brief(r) => Some(r.issued.clone()),
            InputReference::Classic(r) => Some(r.issued.clone()),
            InputReference::Patent(r) => Some(r.issued.clone()),
            InputReference::Dataset(r) => Some(r.issued.clone()),
            InputReference::Standard(r) => Some(r.issued.clone()),
            InputReference::Software(r) => Some(r.issued.clone()),
            InputReference::Event(r) => r.date.clone(),
        }
    }

    /// Return the DOI.
    pub fn doi(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.doi.clone(),
            InputReference::CollectionComponent(r) => r.doi.clone(),
            InputReference::SerialComponent(r) => r.doi.clone(),
            InputReference::LegalCase(r) => r.doi.clone(),
            InputReference::Dataset(r) => r.doi.clone(),
            InputReference::Software(r) => r.doi.clone(),
            _ => None,
        }
    }

    /// Return the ADS bibcode.
    pub fn ads_bibcode(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.ads_bibcode.clone(),
            InputReference::SerialComponent(r) => r.ads_bibcode.clone(),
            _ => None,
        }
    }

    /// Return the note.
    pub fn note(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.note.clone(),
            InputReference::CollectionComponent(r) => r.note.clone(),
            InputReference::SerialComponent(r) => r.note.clone(),
            InputReference::Collection(r) => r.note.clone(),
            InputReference::Serial(r) => r.note.clone(),
            InputReference::LegalCase(r) => r.note.clone(),
            InputReference::Statute(r) => r.note.clone(),
            InputReference::Treaty(r) => r.note.clone(),
            InputReference::Standard(r) => r.note.clone(),
            InputReference::Event(r) => r.note.clone(),
            _ => None,
        }
    }

    /// Return the URL.
    pub fn url(&self) -> Option<Url> {
        match self {
            InputReference::Monograph(r) => r.url.clone(),
            InputReference::CollectionComponent(r) => r.url.clone(),
            InputReference::SerialComponent(r) => r.url.clone(),
            InputReference::Collection(r) => r.url.clone(),
            InputReference::Serial(r) => r.url.clone(),
            InputReference::LegalCase(r) => r.url.clone(),
            InputReference::Statute(r) => r.url.clone(),
            InputReference::Treaty(r) => r.url.clone(),
            InputReference::Hearing(r) => r.url.clone(),
            InputReference::Regulation(r) => r.url.clone(),
            InputReference::Brief(r) => r.url.clone(),
            InputReference::Classic(r) => r.url.clone(),
            InputReference::Patent(r) => r.url.clone(),
            InputReference::Dataset(r) => r.url.clone(),
            InputReference::Standard(r) => r.url.clone(),
            InputReference::Software(r) => r.url.clone(),
            InputReference::Event(r) => r.url.clone(),
        }
    }

    /// Return the publisher place.
    pub fn publisher_place(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => {
                r.publisher.as_ref().and_then(|c| c.location()).or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_place(),
                        _ => None,
                    })
                })
            }
            InputReference::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                _ => None,
            }),
            InputReference::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                _ => None,
            }),
            InputReference::Collection(r) => {
                r.publisher.as_ref().and_then(|c| c.location()).or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_place(),
                        _ => None,
                    })
                })
            }
            InputReference::Serial(_) => None,
            InputReference::Classic(r) => r.publisher.as_ref().and_then(|c| c.location()),
            InputReference::Dataset(r) => r.publisher.as_ref().and_then(|c| c.location()),
            InputReference::Standard(r) => r.publisher.as_ref().and_then(|c| c.location()),
            InputReference::Software(r) => r.publisher.as_ref().and_then(|c| c.location()),
            InputReference::Event(r) => r.location.clone(),
            _ => None,
        }
    }

    /// Return the publisher as a string.
    pub fn publisher_str(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => {
                r.publisher.as_ref().and_then(|c| c.name()).or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_str(),
                        _ => None,
                    })
                })
            }
            InputReference::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str(),
                _ => None,
            }),
            InputReference::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str(),
                _ => None,
            }),
            InputReference::Collection(r) => {
                r.publisher.as_ref().and_then(|c| c.name()).or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_str(),
                        _ => None,
                    })
                })
            }
            InputReference::Serial(r) => r.publisher.as_ref().and_then(|c| c.name()),
            InputReference::Classic(r) => r.publisher.as_ref().and_then(|c| c.name()),
            InputReference::Dataset(r) => r.publisher.as_ref().and_then(|c| c.name()),
            InputReference::Standard(r) => r.publisher.as_ref().and_then(|c| c.name()),
            InputReference::Software(r) => r.publisher.as_ref().and_then(|c| c.name()),
            InputReference::Event(r) => r.network.clone(),
            _ => None,
        }
    }

    /// Normalize genre/medium values to canonical kebab-case (defensive fallback for legacy producers).
    ///
    /// Converts to ASCII lowercase and replaces whitespace/underscores with dashes.
    fn normalize_genre_medium(s: &str) -> String {
        let lower = s.to_ascii_lowercase();
        lower
            .split(|c: char| c.is_whitespace() || c == '_')
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Return the genre/type as string, normalized to canonical kebab-case.
    pub fn genre(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => {
                r.genre.as_ref().map(|g| Self::normalize_genre_medium(g))
            }
            InputReference::CollectionComponent(r) => {
                r.genre.as_ref().map(|g| Self::normalize_genre_medium(g))
            }
            InputReference::SerialComponent(r) => {
                r.genre.as_ref().map(|g| Self::normalize_genre_medium(g))
            }
            InputReference::Event(r) => r.genre.as_ref().map(|g| Self::normalize_genre_medium(g)),
            _ => None,
        }
    }

    /// Return the archive or repository name.
    pub fn archive(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.archive.clone(),
            _ => None,
        }
    }

    /// Return the archive shelfmark or repository location.
    pub fn archive_location(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r
                .archive_info
                .as_ref()
                .and_then(|info| info.location.clone())
                .or_else(|| r.archive_location.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.location.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.location.clone())
            }
            _ => None,
        }
    }

    /// Return the archive name from structured ArchiveInfo.
    pub fn archive_name(&self) -> Option<MultilingualString> {
        match self {
            InputReference::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.name.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.name.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.name.clone())
            }
            _ => None,
        }
    }

    /// Return the archive geographic place from structured ArchiveInfo.
    pub fn archive_place(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.place.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.place.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.place.clone())
            }
            _ => None,
        }
    }

    /// Return the archive collection name from structured ArchiveInfo.
    pub fn archive_collection(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => {
                r.archive_info.as_ref().and_then(|i| i.collection.clone())
            }
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.collection.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.collection.clone())
            }
            _ => None,
        }
    }

    /// Return the archive collection identifier from structured ArchiveInfo.
    pub fn archive_collection_id(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.collection_id.clone()),
            InputReference::CollectionComponent(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.collection_id.clone()),
            InputReference::SerialComponent(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.collection_id.clone()),
            _ => None,
        }
    }

    /// Return the archive series from structured ArchiveInfo.
    pub fn archive_series(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.series.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.series.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.series.clone())
            }
            _ => None,
        }
    }

    /// Return the archive box number from structured ArchiveInfo.
    pub fn archive_box(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.r#box.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.r#box.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.r#box.clone())
            }
            _ => None,
        }
    }

    /// Return the archive folder from structured ArchiveInfo.
    pub fn archive_folder(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.folder.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.folder.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.folder.clone())
            }
            _ => None,
        }
    }

    /// Return the archive item identifier from structured ArchiveInfo.
    pub fn archive_item(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.item.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.item.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.item.clone())
            }
            _ => None,
        }
    }

    /// Return the archive URL from structured ArchiveInfo.
    pub fn archive_url(&self) -> Option<Url> {
        match self {
            InputReference::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.url.clone()),
            InputReference::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.url.clone())
            }
            InputReference::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.url.clone())
            }
            _ => None,
        }
    }

    /// Return the eprint identifier.
    pub fn eprint_id(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.eprint.as_ref().map(|e| e.id.clone()),
            InputReference::CollectionComponent(r) => r.eprint.as_ref().map(|e| e.id.clone()),
            InputReference::SerialComponent(r) => r.eprint.as_ref().map(|e| e.id.clone()),
            _ => None,
        }
    }

    /// Return the eprint server name.
    pub fn eprint_server(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.eprint.as_ref().map(|e| e.server.clone()),
            InputReference::CollectionComponent(r) => r.eprint.as_ref().map(|e| e.server.clone()),
            InputReference::SerialComponent(r) => r.eprint.as_ref().map(|e| e.server.clone()),
            _ => None,
        }
    }

    /// Return the eprint subject class.
    pub fn eprint_class(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.eprint.as_ref().and_then(|e| e.class.clone()),
            InputReference::CollectionComponent(r) => {
                r.eprint.as_ref().and_then(|e| e.class.clone())
            }
            InputReference::SerialComponent(r) => r.eprint.as_ref().and_then(|e| e.class.clone()),
            _ => None,
        }
    }

    /// Return the medium, normalized to canonical kebab-case.
    pub fn medium(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => {
                r.medium.as_ref().map(|m| Self::normalize_genre_medium(m))
            }
            InputReference::CollectionComponent(r) => {
                r.medium.as_ref().map(|m| Self::normalize_genre_medium(m))
            }
            InputReference::SerialComponent(r) => {
                r.medium.as_ref().map(|m| Self::normalize_genre_medium(m))
            }
            _ => None,
        }
    }

    /// Return the version.
    pub fn version(&self) -> Option<String> {
        match self {
            InputReference::Dataset(r) => r.version.clone(),
            InputReference::Software(r) => r.version.clone(),
            _ => None,
        }
    }

    /// Return the abstract.
    pub fn abstract_text(&self) -> Option<String> {
        None
    }

    /// Return the container-style title for parent works, reporters, or codes.
    pub fn container_title(&self) -> Option<Title> {
        match self {
            InputReference::Monograph(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            InputReference::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            InputReference::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            InputReference::Serial(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            InputReference::LegalCase(r) => r.reporter.clone().map(Title::Single),
            InputReference::Treaty(r) => r.reporter.clone().map(Title::Single),
            InputReference::Event(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the volume.
    pub fn volume(&self) -> Option<NumOrStr> {
        match self {
            InputReference::Monograph(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            InputReference::Collection(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            InputReference::CollectionComponent(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            InputReference::SerialComponent(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            InputReference::Classic(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            InputReference::LegalCase(r) => r.volume.clone().map(NumOrStr::Str),
            InputReference::Statute(r) => r.volume.clone().map(NumOrStr::Str),
            InputReference::Treaty(r) => r.volume.clone().map(NumOrStr::Str),
            InputReference::Regulation(r) => r.volume.clone().map(NumOrStr::Str),
            _ => self
                .find_numbering(NumberingType::Volume)
                .map(NumOrStr::Str),
        }
    }

    /// Return the collection number (series number).
    pub fn collection_number(&self) -> Option<String> {
        match self {
            InputReference::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.collection_number(),
                WorkRelation::Id(_) => None,
            }),
            _ => self.find_numbering(NumberingType::Volume),
        }
    }

    /// Return the issue.
    pub fn issue(&self) -> Option<NumOrStr> {
        match self {
            InputReference::Monograph(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            InputReference::Collection(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            InputReference::CollectionComponent(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            InputReference::SerialComponent(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            InputReference::Classic(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            _ => self.find_numbering(NumberingType::Issue).map(NumOrStr::Str),
        }
    }

    /// Return the pages.
    pub fn pages(&self) -> Option<NumOrStr> {
        match self {
            InputReference::CollectionComponent(r) => r.pages.clone(),
            InputReference::SerialComponent(r) => r.pages.clone().map(NumOrStr::Str),
            InputReference::LegalCase(r) => r.page.clone().map(NumOrStr::Str),
            InputReference::Treaty(r) => r.page.clone().map(NumOrStr::Str),
            _ => None,
        }
    }

    /// Return the authority (court, legislative body, standards org, etc.).
    pub fn authority(&self) -> Option<String> {
        match self {
            InputReference::LegalCase(r) => Some(r.authority.clone()),
            InputReference::Statute(r) => r.authority.clone(),
            InputReference::Hearing(r) => r.authority.clone(),
            InputReference::Regulation(r) => r.authority.clone(),
            InputReference::Brief(r) => r.authority.clone(),
            InputReference::Patent(r) => r.authority.clone(),
            InputReference::Standard(r) => r.authority.clone(),
            _ => None,
        }
    }

    /// Return the reporter (legal reporter series).
    pub fn reporter(&self) -> Option<String> {
        match self {
            InputReference::LegalCase(r) => r.reporter.clone(),
            InputReference::Treaty(r) => r.reporter.clone(),
            _ => None,
        }
    }

    /// Return the code (legal code abbreviation).
    pub fn code(&self) -> Option<String> {
        match self {
            InputReference::Statute(r) => r.code.clone(),
            InputReference::Regulation(r) => r.code.clone(),
            _ => None,
        }
    }

    /// Return the section (legal section number).
    pub fn section(&self) -> Option<String> {
        match self {
            InputReference::Statute(r) => r.section.clone(),
            InputReference::Regulation(r) => r.section.clone(),
            InputReference::Classic(_) => self.find_numbering(NumberingType::Section),
            _ => None,
        }
    }

    /// Return the number (docket number, session number, etc.).
    pub fn number(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Part)),
            InputReference::Collection(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Part)),
            InputReference::CollectionComponent(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Part)),
            InputReference::SerialComponent(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Part)),
            InputReference::Classic(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Part)),
            InputReference::Hearing(r) => r.session_number.clone(),
            InputReference::Brief(r) => r.docket_number.clone(),
            InputReference::Patent(r) => Some(r.patent_number.clone()),
            InputReference::Standard(r) => Some(r.standard_number.clone()),
            _ => self.find_numbering(NumberingType::Part),
        }
    }

    /// Return the edition.
    pub fn edition(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            InputReference::Collection(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            InputReference::CollectionComponent(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            InputReference::SerialComponent(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            InputReference::Classic(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            _ => self.find_numbering(NumberingType::Edition),
        }
    }

    /// Return the accessed date.
    pub fn accessed(&self) -> Option<EdtfString> {
        match self {
            InputReference::Monograph(r) => r.accessed.clone(),
            InputReference::CollectionComponent(r) => r.accessed.clone(),
            InputReference::SerialComponent(r) => r.accessed.clone(),
            InputReference::Collection(r) => r.accessed.clone(),
            InputReference::Serial(r) => r.accessed.clone(),
            InputReference::LegalCase(r) => r.accessed.clone(),
            InputReference::Statute(r) => r.accessed.clone(),
            InputReference::Treaty(r) => r.accessed.clone(),
            InputReference::Hearing(r) => r.accessed.clone(),
            InputReference::Regulation(r) => r.accessed.clone(),
            InputReference::Brief(r) => r.accessed.clone(),
            InputReference::Classic(r) => r.accessed.clone(),
            InputReference::Patent(r) => r.accessed.clone(),
            InputReference::Dataset(r) => r.accessed.clone(),
            InputReference::Standard(r) => r.accessed.clone(),
            InputReference::Software(r) => r.accessed.clone(),
            InputReference::Event(r) => r.accessed.clone(),
        }
    }

    /// Return the original publication date.
    pub fn original_date(&self) -> Option<EdtfString> {
        match self {
            InputReference::Monograph(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.issued(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::CollectionComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.issued(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::SerialComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.issued(),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the original title.
    pub fn original_title(&self) -> Option<Title> {
        match self {
            InputReference::Monograph(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::CollectionComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::SerialComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the ISBN.
    pub fn isbn(&self) -> Option<String> {
        match self {
            InputReference::Monograph(r) => r.isbn.clone(),
            _ => None,
        }
    }

    /// Return the ISSN.
    pub fn issn(&self) -> Option<String> {
        match self {
            InputReference::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(s) => s.issn(),
                WorkRelation::Id(_) => None,
            }),
            InputReference::Serial(r) => r.issn.clone(),
            _ => None,
        }
    }

    /// Return the Keywords.
    pub fn keywords(&self) -> Option<Vec<String>> {
        match self {
            InputReference::Monograph(r) => r.keywords.clone(),
            InputReference::CollectionComponent(r) => r.keywords.clone(),
            InputReference::SerialComponent(r) => r.keywords.clone(),
            InputReference::Collection(r) => r.keywords.clone(),
            InputReference::Serial(_) => None,
            InputReference::LegalCase(r) => r.keywords.clone(),
            InputReference::Statute(r) => r.keywords.clone(),
            InputReference::Treaty(r) => r.keywords.clone(),
            InputReference::Hearing(r) => r.keywords.clone(),
            InputReference::Regulation(r) => r.keywords.clone(),
            InputReference::Brief(r) => r.keywords.clone(),
            InputReference::Classic(r) => r.keywords.clone(),
            InputReference::Patent(r) => r.keywords.clone(),
            InputReference::Dataset(r) => r.keywords.clone(),
            InputReference::Standard(r) => r.keywords.clone(),
            InputReference::Software(r) => r.keywords.clone(),
            InputReference::Event(_) => None,
        }
    }

    /// Return the language.
    pub fn language(&self) -> Option<LangID> {
        match self {
            InputReference::Monograph(r) => r.language.clone(),
            InputReference::CollectionComponent(r) => r.language.clone(),
            InputReference::SerialComponent(r) => r.language.clone(),
            InputReference::Collection(r) => r.language.clone(),
            InputReference::Serial(r) => r.language.clone(),
            InputReference::LegalCase(r) => r.language.clone(),
            InputReference::Statute(r) => r.language.clone(),
            InputReference::Treaty(r) => r.language.clone(),
            InputReference::Hearing(r) => r.language.clone(),
            InputReference::Regulation(r) => r.language.clone(),
            InputReference::Brief(r) => r.language.clone(),
            InputReference::Classic(r) => r.language.clone(),
            InputReference::Patent(r) => r.language.clone(),
            InputReference::Dataset(r) => r.language.clone(),
            InputReference::Standard(r) => r.language.clone(),
            InputReference::Software(r) => r.language.clone(),
            InputReference::Event(r) => r.language.clone(),
        }
    }

    /// Return field-level language overrides.
    pub fn field_languages(&self) -> &FieldLanguageMap {
        match self {
            InputReference::Monograph(r) => &r.field_languages,
            InputReference::CollectionComponent(r) => &r.field_languages,
            InputReference::SerialComponent(r) => &r.field_languages,
            InputReference::Collection(r) => &r.field_languages,
            InputReference::Serial(r) => &r.field_languages,
            InputReference::LegalCase(r) => &r.field_languages,
            InputReference::Statute(r) => &r.field_languages,
            InputReference::Treaty(r) => &r.field_languages,
            InputReference::Hearing(r) => &r.field_languages,
            InputReference::Regulation(r) => &r.field_languages,
            InputReference::Brief(r) => &r.field_languages,
            InputReference::Classic(r) => &r.field_languages,
            InputReference::Patent(r) => &r.field_languages,
            InputReference::Dataset(r) => &r.field_languages,
            InputReference::Standard(r) => &r.field_languages,
            InputReference::Software(r) => &r.field_languages,
            InputReference::Event(r) => &r.field_languages,
        }
    }

    /// Set the reference ID.
    pub fn set_id(&mut self, id: String) {
        match self {
            InputReference::Monograph(monograph) => monograph.id = Some(id),
            InputReference::CollectionComponent(component) => component.id = Some(id),
            InputReference::SerialComponent(component) => component.id = Some(id),
            InputReference::Collection(collection) => collection.id = Some(id),
            InputReference::Serial(serial) => serial.id = Some(id),
            InputReference::LegalCase(r) => r.id = Some(id),
            InputReference::Statute(r) => r.id = Some(id),
            InputReference::Treaty(r) => r.id = Some(id),
            InputReference::Hearing(r) => r.id = Some(id),
            InputReference::Regulation(r) => r.id = Some(id),
            InputReference::Brief(r) => r.id = Some(id),
            InputReference::Classic(r) => r.id = Some(id),
            InputReference::Patent(r) => r.id = Some(id),
            InputReference::Dataset(r) => r.id = Some(id),
            InputReference::Standard(r) => r.id = Some(id),
            InputReference::Software(r) => r.id = Some(id),
            InputReference::Event(r) => r.id = Some(id),
        }
    }

    /// Return the reference type as a string (CSL-compatible).
    #[allow(
        clippy::too_many_lines,
        reason = "Enum dispatch for reference types requires extensive branching"
    )]
    pub fn ref_type(&self) -> String {
        match self {
            InputReference::Monograph(r) => match r.r#type {
                MonographType::Book => {
                    if r.medium
                        .as_deref()
                        .is_some_and(|m| m.to_ascii_lowercase().contains("interview"))
                    {
                        "interview".to_string()
                    } else {
                        "book".to_string()
                    }
                }
                MonographType::Manual => "manual".to_string(),
                MonographType::Report => "report".to_string(),
                MonographType::Thesis => "thesis".to_string(),
                MonographType::Webpage => "webpage".to_string(),
                MonographType::Post => "post".to_string(),
                MonographType::Interview => "interview".to_string(),
                MonographType::Manuscript => "manuscript".to_string(),
                MonographType::Preprint => "preprint".to_string(),
                MonographType::PersonalCommunication => "personal-communication".to_string(),
                MonographType::Document => {
                    if r.medium
                        .as_deref()
                        .is_some_and(|m| m.to_ascii_lowercase().contains("interview"))
                    {
                        "interview".to_string()
                    } else {
                        "document".to_string()
                    }
                }
            },
            InputReference::CollectionComponent(r) => match r.r#type {
                MonographComponentType::Chapter => "chapter".to_string(),
                MonographComponentType::Document => "paper-conference".to_string(),
            },
            InputReference::SerialComponent(r) => {
                let container_type = r.container.as_ref().and_then(|c| match c {
                    WorkRelation::Embedded(p) => Some(p.ref_type()),
                    _ => None,
                });

                match container_type.as_deref() {
                    Some("article-journal") => {
                        if r.genre.as_deref() == Some("entry-encyclopedia") {
                            "entry-encyclopedia".to_string()
                        } else {
                            "article-journal".to_string()
                        }
                    }
                    Some("article-magazine") => "article-magazine".to_string(),
                    Some("article-newspaper") => "article-newspaper".to_string(),
                    Some("broadcast") => {
                        if r.genre
                            .as_deref()
                            .is_some_and(|g| g.to_ascii_lowercase().contains("film"))
                        {
                            "motion-picture".to_string()
                        } else {
                            "broadcast".to_string()
                        }
                    }
                    _ => "article-journal".to_string(),
                }
            }
            InputReference::Collection(r) => match r.r#type {
                CollectionType::EditedBook => "book".to_string(),
                _ => "collection".to_string(),
            },
            InputReference::Serial(r) => match r.r#type {
                SerialType::AcademicJournal => "article-journal".to_string(),
                SerialType::Magazine => "article-magazine".to_string(),
                SerialType::Newspaper => "article-newspaper".to_string(),
                SerialType::BroadcastProgram => "broadcast".to_string(),
                _ => "serial".to_string(),
            },
            InputReference::LegalCase(_) => "legal-case".to_string(),
            InputReference::Statute(_) => "statute".to_string(),
            InputReference::Treaty(_) => "treaty".to_string(),
            InputReference::Hearing(_) => "hearing".to_string(),
            InputReference::Regulation(_) => "regulation".to_string(),
            InputReference::Brief(_) => "brief".to_string(),
            InputReference::Classic(_) => "classic".to_string(),
            InputReference::Patent(_) => "patent".to_string(),
            InputReference::Dataset(_) => "dataset".to_string(),
            InputReference::Standard(_) => "standard".to_string(),
            InputReference::Software(_) => "software".to_string(),
            InputReference::Event(r) => {
                match r
                    .genre
                    .as_deref()
                    .map(|g| g.to_ascii_lowercase())
                    .as_deref()
                {
                    Some(g) if g.contains("conference") || g.contains("paper") => {
                        "paper-conference".to_string()
                    }
                    Some(g) if g.contains("broadcast") => "broadcast".to_string(),
                    Some(g) if g.contains("talk") || g.contains("speech") => "speech".to_string(),
                    _ => "event".to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod normalize_tests {
    use super::InputReference;

    fn norm(s: &str) -> String {
        InputReference::normalize_genre_medium(s)
    }

    #[test]
    fn test_normalize_genre_medium() {
        assert_eq!(norm("Technical report"), "technical-report");
        assert_eq!(norm("PhD thesis"), "phd-thesis");
        assert_eq!(norm("Short film"), "short-film");
        assert_eq!(norm("video-interview"), "video-interview");
        assert_eq!(norm("film"), "film");
        assert_eq!(norm("Annual report"), "annual-report");
    }
}

#[cfg(test)]
mod numbering_tests {
    use super::{InputReference, NumOrStr};

    fn parse_reference(json: &str) -> InputReference {
        serde_json::from_str(json).expect("reference should parse")
    }

    #[test]
    fn shorthand_numbering_accessors_cover_all_numbered_reference_variants() {
        let cases = [
            (
                "monograph",
                r#"{
                    "class": "monograph",
                    "type": "book",
                    "title": "Book",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "collection",
                r#"{
                    "class": "collection",
                    "type": "anthology",
                    "title": "Collection",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "collection-component",
                r#"{
                    "class": "collection-component",
                    "type": "chapter",
                    "title": "Chapter",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "serial-component",
                r#"{
                    "class": "serial-component",
                    "type": "article",
                    "title": "Article",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "classic",
                r#"{
                    "class": "classic",
                    "title": "Classic",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
        ];

        for (label, json) in cases {
            let reference = parse_reference(json);

            assert_eq!(
                reference.volume(),
                Some(NumOrStr::Str("1".to_string())),
                "{label} should resolve volume"
            );
            assert_eq!(
                reference.issue(),
                Some(NumOrStr::Str("2".to_string())),
                "{label} should resolve issue"
            );
            assert_eq!(
                reference.edition(),
                Some("3".to_string()),
                "{label} should resolve edition"
            );
            assert_eq!(
                reference.number(),
                Some("4".to_string()),
                "{label} should resolve number"
            );
        }
    }

    #[test]
    fn collection_component_collection_number_bubbles_from_embedded_container() {
        let reference = parse_reference(
            r#"{
                "class": "collection-component",
                "type": "chapter",
                "title": "Chapter",
                "issued": "2024",
                "container": {
                    "class": "collection",
                    "type": "edited-book",
                    "title": "Series",
                    "issued": "2024",
                    "volume": "7"
                }
            }"#,
        );

        assert_eq!(reference.collection_number(), Some("7".to_string()));
    }
}
